use axum::body::Bytes;
use canicule::app::{AppState, UpstreamEndpoints, router};
use canicule::cache::TileCache;
use canicule::config::Config;
use canicule::cookies::CookieManager;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::tempdir;
use tokio::sync::Semaphore;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const TILE: &[u8] = include_bytes!("fixtures/1x1.png");
const TILE_PATH: &str = "/api/tiles/all/hot/6/32/21.png";

fn state(server: &MockServer, authenticated: bool, ttl_days: u64, dir: Option<&Path>) -> AppState {
    let config = Config {
        port: 8080,
        session_cookie: authenticated.then(|| "session123".to_owned()),
        cache_dir: dir.map(Path::to_path_buf),
        cache_ttl_days: ttl_days,
        cache_max_entries: 100,
        upstream_timeout_secs: 10,
    };
    let http = reqwest::Client::new();
    AppState {
        config,
        cache: Arc::new(TileCache::new(100, ttl_days, dir)),
        cookies: authenticated.then(|| {
            Arc::new(CookieManager::new(
                "session123".into(),
                format!("{}/refresh", server.uri()),
                http.clone(),
            ))
        }),
        http,
        upstream: UpstreamEndpoints {
            anonymous: std::array::from_fn(|_| format!("{}/tiles", server.uri())),
            authenticated: format!("{}/identified/globalheat", server.uri()),
        },
        semaphore: Arc::new(Semaphore::new(8)),
        in_flight: moka::future::Cache::builder()
            .max_capacity(100)
            .time_to_live(std::time::Duration::from_secs(300))
            .build(),
    }
}

async fn serve(state: AppState) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router(state)).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn serves_a_tile_and_caches_the_second_request() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(TILE))
        .expect(1)
        .mount(&upstream)
        .await;
    let url = serve(state(&upstream, false, 14, None)).await;
    let client = reqwest::Client::new();
    let first = client
        .get(format!("{url}{TILE_PATH}"))
        .send()
        .await
        .unwrap();
    assert_eq!(first.status(), 200);
    assert_eq!(first.headers()["content-type"], "image/png");
    assert_eq!(first.headers()["cache-control"], "public, max-age=1209600");
    assert_eq!(first.headers()["x-cache"], "miss");
    assert_eq!(first.bytes().await.unwrap(), Bytes::from_static(TILE));
    let second = client
        .get(format!("{url}{TILE_PATH}"))
        .send()
        .await
        .unwrap();
    assert_eq!(second.headers()["x-cache"], "hit");
    upstream.verify().await;
}

#[tokio::test]
async fn upstream_html_response_is_not_cached_as_a_png() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<html>sign in</html>"))
        .expect(2)
        .mount(&upstream)
        .await;
    let url = serve(state(&upstream, false, 14, None)).await;
    for _ in 0..2 {
        assert_eq!(
            reqwest::get(format!("{url}{TILE_PATH}"))
                .await
                .unwrap()
                .status(),
            502
        );
    }
    upstream.verify().await;
}

#[tokio::test]
async fn caches_empty_ocean_tiles() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1)
        .mount(&upstream)
        .await;
    let url = serve(state(&upstream, false, 14, None)).await;
    let client = reqwest::Client::new();
    for _ in 0..2 {
        assert_eq!(
            client
                .get(format!("{url}{TILE_PATH}"))
                .send()
                .await
                .unwrap()
                .status(),
            404
        );
    }
    upstream.verify().await;
}

#[tokio::test]
async fn reauthenticates_after_an_upstream_403() {
    let upstream = MockServer::start().await;
    let refresh_count = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&refresh_count);
    Mock::given(path("/refresh"))
        .respond_with(move |_: &Request| {
            let n = count.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200)
                .append_header("Set-Cookie", "CloudFront-Key-Pair-Id=K; Path=/")
                .append_header("Set-Cookie", "CloudFront-Policy=P; Path=/")
                .append_header("Set-Cookie", format!("CloudFront-Signature=S{n}; Path=/"))
        })
        .expect(2)
        .mount(&upstream)
        .await;
    Mock::given(path("/identified/globalheat/all/hot/6/32/21.png"))
        .respond_with(|request: &Request| {
            if request
                .headers
                .get("cookie")
                .is_some_and(|value| value.to_str().unwrap().contains("CloudFront-Signature=S1"))
            {
                ResponseTemplate::new(200).set_body_bytes(TILE)
            } else {
                ResponseTemplate::new(403)
            }
        })
        .expect(2)
        .mount(&upstream)
        .await;
    let url = serve(state(&upstream, true, 14, None)).await;
    let response = reqwest::get(format!("{url}{TILE_PATH}")).await.unwrap();
    assert_eq!(response.status(), 200);
    upstream.verify().await;
}

#[tokio::test]
async fn fails_cleanly_when_cookie_refresh_fails() {
    let upstream = MockServer::start().await;
    Mock::given(path("/refresh"))
        .respond_with(ResponseTemplate::new(403))
        .expect(1)
        .mount(&upstream)
        .await;
    let url = serve(state(&upstream, true, 14, None)).await;
    assert_eq!(
        reqwest::get(format!("{url}{TILE_PATH}"))
            .await
            .unwrap()
            .status(),
        502
    );
    upstream.verify().await;
}

#[tokio::test]
async fn rejects_invalid_tile_paths_before_contacting_strava() {
    let upstream = MockServer::start().await;
    let url = serve(state(&upstream, false, 14, None)).await;
    let client = reqwest::Client::new();
    for (route, expected) in [
        ("/api/tiles/all/hot/12/32/21.png", 404),
        ("/api/tiles/all/hot/42/32/21.png", 400),
        ("/api/tiles/all/hot/2/5/1.png", 400),
        ("/api/tiles/swim/hot/6/32/21.png", 400),
    ] {
        let result = client.get(format!("{url}{route}")).send().await.unwrap();
        assert_eq!(result.status().as_u16(), expected, "{route}");
    }
    assert!(upstream.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn simultaneous_requests_for_one_tile_share_one_upstream_fetch() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_millis(100))
                .set_body_bytes(TILE),
        )
        .expect(1)
        .mount(&upstream)
        .await;
    let url = serve(state(&upstream, false, 14, None)).await;
    let client = reqwest::Client::new();
    let mut tasks = Vec::new();
    for _ in 0..10 {
        let client = client.clone();
        let route = format!("{url}{TILE_PATH}");
        tasks.push(tokio::spawn(async move {
            client.get(route).send().await.unwrap().status()
        }));
    }
    for task in tasks {
        assert_eq!(task.await.unwrap(), 200);
    }
    upstream.verify().await;
}

#[tokio::test]
async fn disk_hit_browser_max_age_is_only_the_remaining_freshness() {
    let upstream = MockServer::start().await;
    let dir = tempdir().unwrap();
    let seed = TileCache::new(100, 1, Some(dir.path()));
    seed.put("all/hot/6/32/21", Bytes::from_static(TILE)).await;
    drop(seed);
    let path = dir.path().join("all/hot/6/32/21.png");
    std::fs::File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(86_370))
        .unwrap();
    let url = serve(state(&upstream, false, 1, Some(dir.path()))).await;
    let response = reqwest::get(format!("{url}{TILE_PATH}")).await.unwrap();
    assert_eq!(response.status(), 200);
    let header = response.headers()["cache-control"].to_str().unwrap();
    let remaining: u64 = header
        .strip_prefix("public, max-age=")
        .unwrap()
        .parse()
        .unwrap();
    assert!(
        remaining <= 30,
        "cache-control wrongly renewed freshness: {header}"
    );
    assert!(upstream.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn does_not_serve_a_truncated_png_when_upstream_fails() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&upstream)
        .await;
    let dir = tempdir().unwrap();
    let path = dir.path().join("all/hot/6/32/21.png");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, &TILE[..TILE.len() - 10]).unwrap();
    let url = serve(state(&upstream, false, 0, Some(dir.path()))).await;
    let response = reqwest::get(format!("{url}{TILE_PATH}")).await.unwrap();
    assert_eq!(response.status(), 502);
}

#[tokio::test]
async fn serves_an_expired_disk_tile_when_upstream_fails() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&upstream)
        .await;
    let dir = tempdir().unwrap();
    let state = state(&upstream, false, 0, Some(dir.path()));
    state
        .cache
        .put("all/hot/6/32/21", Bytes::from_static(TILE))
        .await;
    let url = serve(state).await;
    let response = reqwest::get(format!("{url}{TILE_PATH}")).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers()["x-cache"], "stale");
    assert_eq!(response.bytes().await.unwrap(), Bytes::from_static(TILE));
    upstream.verify().await;
}
