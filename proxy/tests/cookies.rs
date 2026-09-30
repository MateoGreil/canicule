use axum::http::HeaderValue;
use canicule::cookies::{CookieError, CookieManager, parse_set_cookies};
use std::sync::Arc;
use std::time::Duration;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn cookie_response() -> ResponseTemplate {
    ResponseTemplate::new(200)
        .append_header("Set-Cookie", "CloudFront-Key-Pair-Id=K; Path=/")
        .append_header("Set-Cookie", "CloudFront-Policy=P; Path=/")
        .append_header("Set-Cookie", "CloudFront-Signature=S; Path=/")
}

#[test]
fn parses_the_three_cloudfront_cookies() {
    let headers = [
        HeaderValue::from_static("CloudFront-Key-Pair-Id=K; Path=/"),
        HeaderValue::from_static("CloudFront-Policy=P==; Path=/"),
        HeaderValue::from_static("CloudFront-Signature=S; Path=/"),
    ];
    let cookies = parse_set_cookies(&headers).unwrap();
    assert_eq!(cookies.key_pair_id, "K");
    assert_eq!(cookies.policy, "P==");
    assert_eq!(cookies.signature, "S");
    assert_eq!(
        cookies.cookie_header(),
        "CloudFront-Key-Pair-Id=K; CloudFront-Policy=P==; CloudFront-Signature=S"
    );
    assert!(parse_set_cookies(&headers[..2]).is_none());
}

#[tokio::test]
async fn fetches_cookies_once_and_reuses_them() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(cookie_response())
        .expect(1)
        .mount(&server)
        .await;
    let manager = CookieManager::new("session123".into(), server.uri(), reqwest::Client::new());
    let first = manager.get_header().await.unwrap();
    let second = manager.get_header().await.unwrap();
    assert_eq!(
        first,
        "CloudFront-Key-Pair-Id=K; CloudFront-Policy=P; CloudFront-Signature=S"
    );
    assert_eq!(first, second);
    server.verify().await;
}

#[tokio::test]
async fn concurrent_requests_share_one_refresh() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(cookie_response().set_delay(Duration::from_millis(100)))
        .expect(1)
        .mount(&server)
        .await;
    let manager = Arc::new(CookieManager::new(
        "session123".into(),
        server.uri(),
        reqwest::Client::new(),
    ));
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let manager = Arc::clone(&manager);
        tasks.push(tokio::spawn(
            async move { manager.get_header().await.unwrap() },
        ));
    }
    for task in tasks {
        assert!(task.await.unwrap().contains("CloudFront-Signature=S"));
    }
    server.verify().await;
}

#[tokio::test]
async fn concurrent_failed_refreshes_share_one_attempt() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403).set_delay(Duration::from_millis(100)))
        .expect(1)
        .mount(&server)
        .await;
    let manager = Arc::new(CookieManager::new(
        "invalid".into(),
        server.uri(),
        reqwest::Client::new(),
    ));
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let manager = Arc::clone(&manager);
        tasks.push(tokio::spawn(async move { manager.get_header().await }));
    }
    for task in tasks {
        assert!(matches!(
            task.await.unwrap(),
            Err(CookieError::RefreshFailed)
        ));
    }
    server.verify().await;
}

#[tokio::test]
async fn failed_refresh_does_not_poison_the_state() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(403))
        .expect(2)
        .mount(&server)
        .await;
    let manager = CookieManager::with_cooldown(
        "invalid".into(),
        server.uri(),
        reqwest::Client::new(),
        Duration::from_millis(50),
    );
    assert!(matches!(
        manager.get_header().await,
        Err(CookieError::RefreshFailed)
    ));
    assert!(matches!(
        manager.get_header().await,
        Err(CookieError::RefreshFailed)
    ));
    tokio::time::sleep(Duration::from_millis(65)).await;
    assert!(matches!(
        manager.get_header().await,
        Err(CookieError::RefreshFailed)
    ));
    server.verify().await;
}

#[tokio::test]
async fn forced_refresh_after_skips_when_another_request_already_refreshed() {
    let server = MockServer::start().await;
    let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let calls_for_mock = Arc::clone(&calls);
    Mock::given(method("GET"))
        .respond_with(move |_: &wiremock::Request| {
            let n = calls_for_mock.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            ResponseTemplate::new(200)
                .append_header("Set-Cookie", "CloudFront-Key-Pair-Id=K; Path=/")
                .append_header("Set-Cookie", "CloudFront-Policy=P; Path=/")
                .append_header("Set-Cookie", format!("CloudFront-Signature=S{n}; Path=/"))
        })
        .expect(2)
        .mount(&server)
        .await;
    let manager = CookieManager::new("session".into(), server.uri(), reqwest::Client::new());
    let old = manager.get_header().await.unwrap();
    let refreshed = manager.refresh_after(&old).await.unwrap();
    assert!(refreshed.contains("CloudFront-Signature=S1"));
    assert_eq!(manager.refresh_after(&old).await.unwrap(), refreshed);
    server.verify().await;
}
