use canicule::app::{AppState, STRAVA_REFRESH_URL, UpstreamEndpoints, router};
use canicule::cache::TileCache;
use canicule::config::Config;
use canicule::cookies::CookieManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::from_env().unwrap_or_else(|err| {
        eprintln!("configuration error: {err}");
        std::process::exit(1);
    });
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(config.upstream_timeout_secs))
        .build()
        .expect("failed to build HTTP client");
    let cookies = config.session_cookie.as_ref().map(|session| {
        Arc::new(CookieManager::new(
            session.clone(),
            STRAVA_REFRESH_URL.into(),
            http.clone(),
        ))
    });
    let state = AppState {
        cache: Arc::new(TileCache::new(
            config.cache_max_entries,
            config.cache_ttl_days,
            config.cache_dir.as_deref(),
        )),
        config: config.clone(),
        cookies,
        http,
        upstream: UpstreamEndpoints::strava(),
        semaphore: Arc::new(Semaphore::new(8)),
        in_flight: moka::future::Cache::builder()
            .max_capacity(10_000)
            .time_to_live(Duration::from_secs(300))
            .build(),
    };
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port))
        .await
        .expect("failed to bind HTTP listener");
    tracing::info!(port = config.port, "listening");
    axum::serve(listener, router(state))
        .await
        .expect("HTTP server failed");
}
