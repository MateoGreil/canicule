use axum::http::{HeaderValue, header};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const REFRESH_INTERVAL: Duration = Duration::from_secs(12 * 3_600);
const FAILURE_COOLDOWN: Duration = Duration::from_secs(30);
const USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 Chrome/130.0 Safari/537.36";

#[derive(Debug, Clone)]
pub struct CloudFrontCookies {
    pub key_pair_id: String,
    pub policy: String,
    pub signature: String,
}

impl CloudFrontCookies {
    pub fn cookie_header(&self) -> String {
        format!(
            "CloudFront-Key-Pair-Id={}; CloudFront-Policy={}; CloudFront-Signature={}",
            self.key_pair_id, self.policy, self.signature
        )
    }
}

pub fn parse_set_cookies(headers: &[HeaderValue]) -> Option<CloudFrontCookies> {
    let mut key_pair_id = None;
    let mut policy = None;
    let mut signature = None;
    for header in headers {
        let (name, rest) = header.to_str().ok()?.split_once('=')?;
        let value = rest.split(';').next()?.to_owned();
        match name {
            "CloudFront-Key-Pair-Id" => key_pair_id = Some(value),
            "CloudFront-Policy" => policy = Some(value),
            "CloudFront-Signature" => signature = Some(value),
            _ => {}
        }
    }
    Some(CloudFrontCookies {
        key_pair_id: key_pair_id?,
        policy: policy?,
        signature: signature?,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum CookieError {
    #[error("CloudFront cookie refresh failed")]
    RefreshFailed,
}

#[derive(Default)]
struct CookieState {
    cookies: Option<(CloudFrontCookies, Instant)>,
    retry_after: Option<Instant>,
}

pub struct CookieManager {
    session_cookie: String,
    refresh_url: String,
    client: reqwest::Client,
    cooldown: Duration,
    state: Mutex<CookieState>,
}

impl CookieManager {
    pub fn new(session_cookie: String, refresh_url: String, client: reqwest::Client) -> Self {
        Self::with_cooldown(session_cookie, refresh_url, client, FAILURE_COOLDOWN)
    }

    pub fn with_cooldown(
        session_cookie: String,
        refresh_url: String,
        client: reqwest::Client,
        cooldown: Duration,
    ) -> Self {
        Self {
            session_cookie,
            refresh_url,
            client,
            cooldown,
            state: Mutex::new(CookieState::default()),
        }
    }

    pub async fn get_header(&self) -> Result<String, CookieError> {
        let mut state = self.state.lock().await;
        if state
            .retry_after
            .is_some_and(|retry_after| Instant::now() < retry_after)
        {
            return Err(CookieError::RefreshFailed);
        }
        if let Some((cookies, fetched_at)) = state.cookies.as_ref()
            && fetched_at.elapsed() < REFRESH_INTERVAL
        {
            return Ok(cookies.cookie_header());
        }
        self.refresh_locked(&mut state).await
    }

    pub async fn refresh_after(&self, used_header: &str) -> Result<String, CookieError> {
        let mut state = self.state.lock().await;
        if state
            .retry_after
            .is_some_and(|retry_after| Instant::now() < retry_after)
        {
            return Err(CookieError::RefreshFailed);
        }
        if let Some((cookies, _)) = state.cookies.as_ref() {
            let current = cookies.cookie_header();
            if current != used_header {
                return Ok(current);
            }
        }
        self.refresh_locked(&mut state).await
    }

    async fn refresh_locked(&self, state: &mut CookieState) -> Result<String, CookieError> {
        match self.fetch_cookies().await {
            Ok(cookies) => {
                let result = cookies.cookie_header();
                state.cookies = Some((cookies, Instant::now()));
                state.retry_after = None;
                Ok(result)
            }
            Err(err) => {
                state.cookies = None;
                state.retry_after = Some(Instant::now() + self.cooldown);
                Err(err)
            }
        }
    }

    async fn fetch_cookies(&self) -> Result<CloudFrontCookies, CookieError> {
        let response = self
            .client
            .get(&self.refresh_url)
            .header(
                header::COOKIE,
                format!("_strava4_session={}", self.session_cookie),
            )
            .header(header::USER_AGENT, USER_AGENT)
            .send()
            .await
            .map_err(|_| CookieError::RefreshFailed)?;
        if !response.status().is_success() {
            return Err(CookieError::RefreshFailed);
        }
        let headers: Vec<HeaderValue> = response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .cloned()
            .collect();
        parse_set_cookies(&headers).ok_or(CookieError::RefreshFailed)
    }
}
