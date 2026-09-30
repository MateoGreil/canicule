use std::sync::{Mutex, MutexGuard};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn lock_env() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

fn clear_vars() {
    for key in [
        "PORT",
        "STRAVA_SESSION_COOKIE",
        "CACHE_DIR",
        "CACHE_TTL_DAYS",
        "CACHE_MAX_ENTRIES",
        "UPSTREAM_TIMEOUT_SECS",
    ] {
        unsafe { std::env::remove_var(key) };
    }
}

#[test]
fn defaults_are_applied() {
    let _guard = lock_env();
    clear_vars();
    let cfg = canicule::config::Config::from_env().unwrap();
    assert_eq!(cfg.port, 8080);
    assert_eq!(cfg.cache_ttl_days, 14);
    assert_eq!(cfg.cache_max_entries, 10_000);
    assert_eq!(cfg.upstream_timeout_secs, 10);
    assert!(cfg.cache_dir.is_none());
    assert!(cfg.session_cookie.is_none());
    assert!(!cfg.authenticated());
    assert_eq!(cfg.max_zoom(), 11);
}

#[test]
fn session_cookie_enables_authenticated_mode() {
    let _guard = lock_env();
    clear_vars();
    unsafe { std::env::set_var("STRAVA_SESSION_COOKIE", "abc123") };
    let cfg = canicule::config::Config::from_env().unwrap();
    assert_eq!(cfg.session_cookie.as_deref(), Some("abc123"));
    assert!(cfg.authenticated());
    assert_eq!(cfg.max_zoom(), 15);
}

#[test]
fn blank_session_cookie_is_ignored() {
    let _guard = lock_env();
    clear_vars();
    unsafe { std::env::set_var("STRAVA_SESSION_COOKIE", "   ") };
    let cfg = canicule::config::Config::from_env().unwrap();
    assert!(!cfg.authenticated());
}

#[test]
fn overrides_are_parsed() {
    let _guard = lock_env();
    clear_vars();
    unsafe {
        std::env::set_var("PORT", "9000");
        std::env::set_var("CACHE_DIR", "/tmp/canicule-cache");
        std::env::set_var("CACHE_TTL_DAYS", "7");
        std::env::set_var("CACHE_MAX_ENTRIES", "5");
        std::env::set_var("UPSTREAM_TIMEOUT_SECS", "3");
    }
    let cfg = canicule::config::Config::from_env().unwrap();
    assert_eq!(cfg.port, 9000);
    assert_eq!(cfg.cache_dir, Some("/tmp/canicule-cache".into()));
    assert_eq!(cfg.cache_ttl_days, 7);
    assert_eq!(cfg.cache_max_entries, 5);
    assert_eq!(cfg.upstream_timeout_secs, 3);
}

#[test]
fn invalid_port_is_rejected() {
    let _guard = lock_env();
    clear_vars();
    unsafe { std::env::set_var("PORT", "abc") };
    let err = canicule::config::Config::from_env().unwrap_err();
    assert!(
        err.contains("PORT"),
        "error should mention PORT, got: {err}"
    );
}

#[test]
fn invalid_ttl_is_rejected() {
    let _guard = lock_env();
    clear_vars();
    unsafe { std::env::set_var("CACHE_TTL_DAYS", "-1") };
    assert!(canicule::config::Config::from_env().is_err());
}
