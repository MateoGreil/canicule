use crate::cache::{TileCache, valid_png};
use crate::config::Config;
use crate::cookies::CookieManager;
use crate::spa;
use crate::tiles::{Color, Sport, TileError, TileRequest};
use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

pub const STRAVA_REFRESH_URL: &str = "https://www.strava.com/maps/global-heatmap?sport=All&style=dark&terrain=false&labels=true&poi=true";
const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/130.0 Safari/537.36";

#[derive(Clone)]
pub struct UpstreamEndpoints {
    pub anonymous: [String; 3],
    pub authenticated: String,
}

impl UpstreamEndpoints {
    pub fn strava() -> Self {
        Self {
            anonymous: [
                "https://heatmap-external-a.strava.com/tiles".into(),
                "https://heatmap-external-b.strava.com/tiles".into(),
                "https://heatmap-external-c.strava.com/tiles".into(),
            ],
            authenticated: "https://content-a.strava.com/identified/globalheat".into(),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub cache: Arc<TileCache>,
    pub cookies: Option<Arc<CookieManager>>,
    pub http: reqwest::Client,
    pub upstream: UpstreamEndpoints,
    pub semaphore: Arc<Semaphore>,
    pub in_flight: moka::future::Cache<String, Arc<Mutex<()>>>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/healthz", get(|| async { "ok" }))
        .route("/api/config", get(get_config))
        .route(
            "/api/tiles/{sport}/{color}/{z}/{x}/{filename}",
            get(get_tile),
        )
        .route("/", get(spa::index))
        .route("/{*path}", get(spa::static_file))
        .with_state(state)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublicConfig {
    authenticated: bool,
    max_zoom: u8,
    sports: [&'static str; 5],
    colors: [&'static str; 6],
}

async fn get_config(State(state): State<AppState>) -> Json<PublicConfig> {
    Json(PublicConfig {
        authenticated: state.config.authenticated(),
        max_zoom: state.config.max_zoom(),
        sports: [
            Sport::All,
            Sport::Ride,
            Sport::Run,
            Sport::Water,
            Sport::Winter,
        ]
        .map(|s| s.as_slug()),
        colors: [
            Color::Hot,
            Color::MobileBlue,
            Color::Gray,
            Color::BlueRed,
            Color::Purple,
            Color::Orange,
        ]
        .map(|c| c.as_slug()),
    })
}

async fn get_tile(
    State(state): State<AppState>,
    Path((sport, color, z, x, filename)): Path<(String, String, u8, u64, String)>,
) -> Response {
    let Some(y) = filename
        .strip_suffix(".png")
        .and_then(|value| value.parse::<u64>().ok())
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if !state.config.authenticated() && (12..=15).contains(&z) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let tile = match TileRequest::parse(&sport, &color, z, x, y, state.config.max_zoom()) {
        Ok(tile) => tile,
        Err(TileError::ZoomTooHigh) if !state.config.authenticated() && z <= 15 => {
            return StatusCode::NOT_FOUND.into_response();
        }
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let key = format!(
        "{}/{}/{}/{}/{}",
        tile.sport.as_slug(),
        tile.color.as_slug(),
        z,
        x,
        y
    );
    let ttl_secs = state.config.cache_ttl_days.saturating_mul(86_400);
    if state.cache.is_negative(&key).await {
        return StatusCode::NOT_FOUND.into_response();
    }
    if let Some(tile) = state.cache.get(&key).await {
        let remaining = tile.remaining_ttl().as_secs();
        return png_response(tile.body, "hit", remaining);
    }
    let lock = state
        .in_flight
        .get_with(key.clone(), async { Arc::new(Mutex::new(())) })
        .await;
    let _tile_guard = lock.lock().await;
    if state.cache.is_negative(&key).await {
        return StatusCode::NOT_FOUND.into_response();
    }
    if let Some(tile) = state.cache.get(&key).await {
        let remaining = tile.remaining_ttl().as_secs();
        return png_response(tile.body, "hit", remaining);
    }
    let Ok(_permit) = state.semaphore.acquire().await else {
        return unavailable(&state.cache, &key).await;
    };
    let cookie_header = if let Some(manager) = &state.cookies {
        match manager.get_header().await {
            Ok(value) => Some(value),
            Err(err) => {
                tracing::warn!(%err, "could not refresh Strava cookies");
                return unavailable(&state.cache, &key).await;
            }
        }
    } else {
        None
    };
    let url = if cookie_header.is_some() {
        format!(
            "{}/{}/{}/{}/{}/{}.png?v=19",
            state.upstream.authenticated,
            tile.sport.as_slug(),
            tile.color.as_slug(),
            z,
            x,
            y
        )
    } else {
        let index = ((x % 3) + (y % 3)) as usize % 3;
        format!(
            "{}/{}/{}/{}/{}/{}.png",
            state.upstream.anonymous[index],
            tile.sport.as_slug(),
            tile.color.as_slug(),
            z,
            x,
            y
        )
    };
    let mut response = fetch_tile(&state.http, &url, cookie_header.as_deref()).await;
    if matches!(&response, Ok(res) if res.status() == StatusCode::FORBIDDEN)
        && let (Some(manager), Some(used_header)) = (&state.cookies, &cookie_header)
    {
        match manager.refresh_after(used_header).await {
            Ok(new_header) => response = fetch_tile(&state.http, &url, Some(&new_header)).await,
            Err(err) => {
                tracing::warn!(%err, "Strava cookie recovery failed");
                return unavailable(&state.cache, &key).await;
            }
        }
    }
    match response {
        Ok(res) if res.status().is_success() => match res.bytes().await {
            Ok(body) if valid_png(&body) => {
                state.cache.put(&key, body.clone()).await;
                png_response(body, "miss", ttl_secs)
            }
            Ok(_) => {
                tracing::warn!("Strava returned an invalid PNG tile");
                unavailable(&state.cache, &key).await
            }
            Err(err) => {
                tracing::warn!(%err, "could not read Strava tile");
                unavailable(&state.cache, &key).await
            }
        },
        Ok(res) if res.status() == StatusCode::NOT_FOUND => {
            state.cache.put_negative(&key).await;
            (
                StatusCode::NOT_FOUND,
                [(header::CACHE_CONTROL, "public, max-age=3600")],
            )
                .into_response()
        }
        Ok(res) => {
            tracing::warn!(status = %res.status(), "Strava tile request failed");
            unavailable(&state.cache, &key).await
        }
        Err(err) => {
            tracing::warn!(%err, "Strava tile request failed");
            unavailable(&state.cache, &key).await
        }
    }
}

async fn fetch_tile(
    client: &reqwest::Client,
    url: &str,
    cookie_header: Option<&str>,
) -> Result<reqwest::Response, reqwest::Error> {
    let mut request = client.get(url).header(header::USER_AGENT, USER_AGENT);
    if let Some(cookies) = cookie_header {
        request = request.header(header::COOKIE, cookies);
    }
    request.send().await
}

async fn unavailable(cache: &TileCache, key: &str) -> Response {
    if let Some(body) = cache.get_stale(key).await {
        return png_response(body, "stale", 0);
    }
    StatusCode::BAD_GATEWAY.into_response()
}

fn png_response(body: Bytes, cache_status: &'static str, ttl_secs: u64) -> Response {
    let mut response = body.into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    headers.insert("x-cache", HeaderValue::from_static(cache_status));
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_str(&format!("public, max-age={ttl_secs}")).expect("valid cache TTL"),
    );
    response
}
