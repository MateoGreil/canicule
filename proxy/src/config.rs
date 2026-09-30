use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub session_cookie: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub cache_ttl_days: u64,
    pub cache_max_entries: u64,
    pub upstream_timeout_secs: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            port: read_var("PORT", 8080)?,
            session_cookie: std::env::var("STRAVA_SESSION_COOKIE")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            cache_dir: std::env::var("CACHE_DIR")
                .ok()
                .filter(|value| !value.trim().is_empty())
                .map(PathBuf::from),
            cache_ttl_days: read_var("CACHE_TTL_DAYS", 14)?,
            cache_max_entries: read_var("CACHE_MAX_ENTRIES", 10_000)?,
            upstream_timeout_secs: read_var("UPSTREAM_TIMEOUT_SECS", 10)?,
        })
    }

    pub fn authenticated(&self) -> bool {
        self.session_cookie.is_some()
    }

    pub fn max_zoom(&self) -> u8 {
        if self.authenticated() { 15 } else { 11 }
    }
}

fn read_var<T: FromStr>(name: &str, default: T) -> Result<T, String> {
    match std::env::var(name) {
        Ok(value) => value
            .parse()
            .map_err(|_| format!("invalid {name}: {value:?}")),
        Err(std::env::VarError::NotPresent) => Ok(default),
        Err(err) => Err(format!("invalid {name}: {err}")),
    }
}
