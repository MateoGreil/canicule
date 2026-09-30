use axum::body::Bytes;
use moka::future::Cache;
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Clone)]
pub struct CachedTile {
    pub body: Bytes,
    expires_at: SystemTime,
}

impl CachedTile {
    pub fn remaining_ttl(&self) -> Duration {
        self.expires_at
            .duration_since(SystemTime::now())
            .unwrap_or_default()
    }
}

pub struct TileCache {
    memory: Cache<String, CachedTile>,
    negative: Cache<String, ()>,
    dir: Option<PathBuf>,
    ttl: Duration,
}

impl TileCache {
    pub fn new(max_entries: u64, ttl_days: u64, dir: Option<&Path>) -> Self {
        let ttl = Duration::from_secs(ttl_days.saturating_mul(86_400));
        Self {
            memory: Cache::builder().max_capacity(max_entries).build(),
            negative: Cache::builder()
                .max_capacity(max_entries)
                .time_to_live(Duration::from_secs(3_600))
                .build(),
            dir: dir.map(Path::to_path_buf),
            ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<CachedTile> {
        if self.ttl.is_zero() {
            return None;
        }
        if let Some(tile) = self.memory.get(key).await {
            if !tile.remaining_ttl().is_zero() {
                return Some(tile);
            }
            self.memory.invalidate(key).await;
        }
        let (body, modified) = self.read_disk(key).await?;
        let expires_at = modified.checked_add(self.ttl)?;
        if SystemTime::now() >= expires_at {
            return None;
        }
        let tile = CachedTile { body, expires_at };
        self.memory.insert(key.to_owned(), tile.clone()).await;
        Some(tile)
    }

    pub async fn put(&self, key: &str, body: Bytes) {
        if !valid_png(&body) {
            return;
        }
        if !self.ttl.is_zero()
            && let Some(expires_at) = SystemTime::now().checked_add(self.ttl)
        {
            self.memory
                .insert(
                    key.to_owned(),
                    CachedTile {
                        body: body.clone(),
                        expires_at,
                    },
                )
                .await;
        }
        if let Some(path) = self.disk_path(key)
            && let Err(err) = write_disk(path, body).await
        {
            tracing::warn!(%err, "failed to write tile cache file");
        }
    }

    pub async fn get_stale(&self, key: &str) -> Option<Bytes> {
        self.read_disk(key).await.map(|(body, _)| body)
    }

    pub async fn put_negative(&self, key: &str) {
        self.negative.insert(key.to_owned(), ()).await;
    }

    pub async fn is_negative(&self, key: &str) -> bool {
        self.negative.contains_key(key)
    }

    fn disk_path(&self, key: &str) -> Option<PathBuf> {
        self.dir.as_ref().map(|dir| dir.join(format!("{key}.png")))
    }

    async fn read_disk(&self, key: &str) -> Option<(Bytes, SystemTime)> {
        let path = self.disk_path(key)?;
        let body = tokio::fs::read(&path).await.ok()?;
        if !valid_png(&body) {
            return None;
        }
        let modified = tokio::fs::metadata(&path).await.ok()?.modified().ok()?;
        Some((Bytes::from(body), modified))
    }
}

pub(crate) fn valid_png(body: &[u8]) -> bool {
    let Ok(mut reader) = png::Decoder::new(Cursor::new(body)).read_info() else {
        return false;
    };
    let mut pixels = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut pixels).is_ok()
}

async fn write_disk(path: PathBuf, body: Bytes) -> std::io::Result<()> {
    tokio::task::spawn_blocking(move || {
        let parent = path
            .parent()
            .ok_or_else(|| std::io::Error::other("no cache parent directory"))?;
        std::fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(&body)?;
        file.persist(&path).map_err(|err| err.error)?;
        Ok(())
    })
    .await
    .map_err(std::io::Error::other)?
}
