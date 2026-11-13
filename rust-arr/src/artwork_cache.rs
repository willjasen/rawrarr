//! Optional Redis cache for encrypted artwork ciphertext.
//!
//! The shared metadata file remains authoritative. Each read hashes the current
//! file before lookup, so a synchronized replacement cannot hit an old entry.
use sha2::{Digest, Sha256};
use redis::Commands;
use std::{env, fs, io::Read, path::Path, time::{Duration, UNIX_EPOCH}};
use std::os::unix::fs::MetadataExt;

#[derive(Clone)]
pub struct ArtworkCache {
    client: Option<redis::Client>,
    ttl_seconds: u64,
}

impl ArtworkCache {
    pub fn disabled() -> Self { Self { client: None, ttl_seconds: 0 } }

    pub fn from_env() -> Self {
        let enabled = env::var("RAWARR_ARTWORK_CACHE_ENABLED")
            .is_ok_and(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"));
        let url = env::var("RAWARR_ARTWORK_CACHE_URL").ok();
        let ttl_seconds = env::var("RAWARR_ARTWORK_CACHE_TTL_SECONDS")
            .ok().and_then(|value| value.parse::<u64>().ok()).unwrap_or(86_400);
        Self::from_options(enabled, url.as_deref(), ttl_seconds)
    }

    fn from_options(enabled: bool, url: Option<&str>, ttl_seconds: u64) -> Self {
        if !enabled {
            return Self { client: None, ttl_seconds: 0 };
        }
        let ttl_seconds = ttl_seconds.clamp(60, 604_800);
        let client = url.and_then(|url| redis::Client::open(url).ok());
        if client.is_none() {
            eprintln!("rawrarr artwork cache enabled but Redis URL is missing or invalid; using shared-file reads");
        }
        Self { client, ttl_seconds }
    }

    pub fn is_configured(&self) -> bool { self.client.is_some() }

    pub fn ping(&self) -> bool {
        self.client.as_ref().is_some_and(|client| {
            client.get_connection_with_timeout(Duration::from_millis(150))
                .and_then(|mut connection| redis::cmd("PING").query::<String>(&mut connection))
                .is_ok()
        })
    }

    /// Returns verified ciphertext. Redis is best-effort; all failures fall back to disk.
    pub fn read(&self, path: &Path) -> std::io::Result<Vec<u8>> {
        self.read_bounded(path, u64::MAX)
    }

    pub fn read_bounded(&self, path: &Path, max_bytes: u64) -> std::io::Result<Vec<u8>> {
        let (file, metadata) = crate::safe_fs::open_regular_file(path)?;
        if metadata.len() > max_bytes {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "artwork ciphertext exceeds the permitted size"));
        }
        let Some(client) = &self.client else { return read_bounded(file, max_bytes); };
        // Syncthing installs changed files by replacement; inode, size, and nanosecond
        // mtime form a cheap generation key, so hits can be served without reading disk.
        let modified = metadata.modified()?.duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let path_digest = Sha256::digest(path.as_os_str().to_string_lossy().as_bytes());
        let key = format!("rawrarr:artwork:v1:{}:{}:{}:{}", hex(&path_digest), metadata.ino(), metadata.len(), modified);
        let mut connection = match client.get_connection_with_timeout(Duration::from_millis(150)) {
            Ok(connection) => connection,
            Err(_) => return read_bounded(file, max_bytes),
        };
        let cached: Option<Vec<u8>> = connection.get(&key).ok().flatten();
        if let Some(cached) = cached {
            if cached.len() as u64 > max_bytes {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "cached artwork ciphertext exceeds the permitted size"));
            }
            return Ok(cached);
        }
        let current = read_bounded(file, max_bytes)?;
        let _: redis::RedisResult<()> = connection.set_ex(&key, &current, self.ttl_seconds);
        Ok(current)
    }
}

fn read_bounded(file: fs::File, max_bytes: u64) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    file.take(max_bytes.saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max_bytes {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "artwork ciphertext exceeds the permitted size"));
    }
    Ok(bytes)
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes { output.push(HEX[(byte >> 4) as usize] as char); output.push(HEX[(byte & 15) as usize] as char); }
    output
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_artwork_cache.rs"]
mod tests;
