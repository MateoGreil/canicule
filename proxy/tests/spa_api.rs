use canicule::app::{AppState, UpstreamEndpoints, router};
use canicule::cache::TileCache;
use canicule::config::Config;
use std::sync::Arc;
use tokio::sync::Semaphore;

fn state(authenticated: bool) -> AppState {
    AppState {
        config: Config {
            port: 8080,
            session_cookie: authenticated.then(|| "session".into()),
            cache_dir: None,
            cache_ttl_days: 14,
            cache_max_entries: 100,
            upstream_timeout_secs: 10,
        },
        cache: Arc::new(TileCache::new(100, 14, None)),
        cookies: None,
        http: reqwest::Client::new(),
        upstream: UpstreamEndpoints::strava(),
        semaphore: Arc::new(Semaphore::new(8)),
        in_flight: moka::future::Cache::new(100),
    }
}

async fn serve(authenticated: bool) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router(state(authenticated)))
            .await
            .unwrap()
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn anonymous_config_lists_supported_sports_and_colors() {
    let url = serve(false).await;
    let response = reqwest::get(format!("{url}/api/config")).await.unwrap();
    assert_eq!(response.status(), 200);
    let json: serde_json::Value = response.json().await.unwrap();
    assert_eq!(json["authenticated"], false);
    assert_eq!(json["maxZoom"], 11);
    assert_eq!(
        json["sports"],
        serde_json::json!(["all", "ride", "run", "water", "winter"])
    );
    assert_eq!(
        json["colors"],
        serde_json::json!(["hot", "mobileblue", "gray", "bluered", "purple", "orange"])
    );
}

#[tokio::test]
async fn authenticated_config_exposes_high_zoom() {
    let url = serve(true).await;
    let json: serde_json::Value = reqwest::get(format!("{url}/api/config"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(json["authenticated"], true);
    assert_eq!(json["maxZoom"], 15);
}

#[tokio::test]
async fn root_serves_embedded_html() {
    let url = serve(false).await;
    let response = reqwest::get(&url).await.unwrap();
    assert_eq!(response.status(), 200);
    assert!(
        response.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/html")
    );
    assert!(response.text().await.unwrap().contains("<h1>canicule</h1>"));
}

#[tokio::test]
async fn unknown_asset_is_not_mistaken_for_index() {
    let url = serve(false).await;
    let response = reqwest::get(format!("{url}/assets/missing.js"))
        .await
        .unwrap();
    assert_eq!(response.status(), 404);
}
