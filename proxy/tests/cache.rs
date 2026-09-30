use axum::body::Bytes;
use canicule::cache::TileCache;
use std::fs;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;

const KEY: &str = "all/hot/6/32/21";
const TILE: &[u8] = include_bytes!("fixtures/1x1.png");

#[tokio::test]
async fn fresh_tile_is_served_from_memory() {
    let cache = TileCache::new(100, 14, None);
    cache.put(KEY, Bytes::from_static(TILE)).await;
    assert_eq!(
        cache.get(KEY).await.map(|tile| tile.body),
        Some(Bytes::from_static(TILE))
    );
    assert!(cache.get("ride/hot/6/32/21").await.is_none());
}

#[tokio::test]
async fn disk_tile_survives_cache_recreation() {
    let dir = tempdir().unwrap();
    let cache = TileCache::new(100, 14, Some(dir.path()));
    cache.put(KEY, Bytes::from_static(TILE)).await;
    drop(cache);
    let cache = TileCache::new(100, 14, Some(dir.path()));
    assert_eq!(
        cache.get(KEY).await.map(|tile| tile.body),
        Some(Bytes::from_static(TILE))
    );
}

#[tokio::test]
async fn expired_disk_tile_is_stale_but_not_fresh() {
    let dir = tempdir().unwrap();
    let cache = TileCache::new(100, 0, Some(dir.path()));
    cache.put(KEY, Bytes::from_static(TILE)).await;
    assert!(cache.get(KEY).await.is_none());
    assert_eq!(cache.get_stale(KEY).await, Some(Bytes::from_static(TILE)));
}

#[tokio::test]
async fn corrupt_disk_tile_is_never_served() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("all/hot/6/32/21.png");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, b"junk").unwrap();
    let cache = TileCache::new(100, 14, Some(dir.path()));
    assert!(cache.get(KEY).await.is_none());
    assert!(cache.get_stale(KEY).await.is_none());
}

#[tokio::test]
async fn signed_but_truncated_png_is_never_served() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("all/hot/6/32/21.png");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, &TILE[..TILE.len() - 10]).unwrap();
    let cache = TileCache::new(100, 14, Some(dir.path()));
    assert!(cache.get(KEY).await.is_none());
    assert!(cache.get_stale(KEY).await.is_none());
}

#[tokio::test]
async fn disk_tile_promoted_to_memory_keeps_its_original_expiry() {
    let dir = tempdir().unwrap();
    let seed = TileCache::new(100, 1, Some(dir.path()));
    seed.put(KEY, Bytes::from_static(TILE)).await;
    drop(seed);
    let path = dir.path().join("all/hot/6/32/21.png");
    fs::File::options()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(SystemTime::now() - Duration::from_secs(86_399))
        .unwrap();
    let cache = TileCache::new(100, 1, Some(dir.path()));
    assert!(cache.get(KEY).await.is_some());
    tokio::time::sleep(Duration::from_millis(1_200)).await;
    assert!(cache.get(KEY).await.is_none());
    assert!(cache.get_stale(KEY).await.is_some());
}

#[tokio::test]
async fn negative_cache_remembers_empty_tile() {
    let cache = TileCache::new(100, 14, None);
    assert!(!cache.is_negative(KEY).await);
    cache.put_negative(KEY).await;
    assert!(cache.is_negative(KEY).await);
    assert!(!cache.is_negative("ride/hot/6/32/21").await);
}
