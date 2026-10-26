//! Ciphertext-only local config mirror and deterministic Merkle primitives.
//!
//! Peer transport is deliberately kept separate from the Valkey cache: Valkey
//! is disposable and contains only the exact age ciphertext bytes.
use redis::Commands;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{env, fs, path::{Path, PathBuf}, time::Duration};

const MAX_CONFIG_BYTES: usize = 2 * 1024 * 1024;
const AGE_MAGIC: &[u8] = b"age-encryption.org/v1\n";

#[derive(Clone)]
pub struct CiphertextMirror { client: Option<redis::Client>, root_fingerprint: String }

impl CiphertextMirror {
    pub fn from_env() -> Self {
        let client = env::var("RAWARR_CONFIG_CACHE_URL").ok().and_then(|url| redis::Client::open(url).ok());
        let root_fingerprint = env::var("RAWARR_INSTANCE_ROOT_FINGERPRINT").unwrap_or_default().trim().to_ascii_lowercase();
        Self { client, root_fingerprint }
    }

    pub fn mirror(&self, bytes: &[u8]) {
        if !valid_ciphertext(bytes) || self.root_fingerprint.len() != 64 { return; }
        let Some(client) = &self.client else { return; };
        let digest = hex(&Sha256::digest(bytes));
        let key = format!("rawrarr:config:v1:{}:{digest}", self.root_fingerprint);
        if let Ok(mut connection) = client.get_connection_with_timeout(Duration::from_millis(150)) {
            let _: redis::RedisResult<()> = connection.set(&key, bytes);
            let _: redis::RedisResult<()> = connection.set(format!("rawrarr:config:current:{}", self.root_fingerprint), digest);
        }
    }

    /// Read the active file and repopulate Valkey. The disk file remains authoritative.
    pub fn rebuild_from_file(&self, path: &Path) -> std::io::Result<Option<String>> {
        match fs::read(path) {
            Ok(bytes) if valid_ciphertext(&bytes) => {
                let digest = hex(&Sha256::digest(&bytes));
                self.mirror(&bytes);
                Ok(Some(digest))
            }
            Ok(_) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid age ciphertext")),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => self.restore_from_valkey(path),
            Err(error) => Err(error),
        }
    }

    /// Rehydrate the active ciphertext file from this node's local Valkey mirror.
    /// Writes through a private staging file and never decrypts age data.
    pub fn restore_from_valkey(&self, path: &Path) -> std::io::Result<Option<String>> {
        let Some(client) = &self.client else { return Ok(None); };
        if self.root_fingerprint.len() != 64 { return Ok(None); }
        let mut connection = match client.get_connection_with_timeout(Duration::from_millis(150)) {
            Ok(connection) => connection,
            Err(_) => return Ok(None),
        };
        let index_key = format!("rawrarr:config:current:{}", self.root_fingerprint);
        let digest: Option<String> = connection.get(&index_key).unwrap_or(None);
        let Some(digest) = digest.filter(|value| value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())) else { return Ok(None); };
        let object_key = format!("rawrarr:config:v1:{}:{digest}", self.root_fingerprint);
        let bytes: Option<Vec<u8>> = connection.get(&object_key).unwrap_or(None);
        let Some(bytes) = bytes else { return Ok(None); };
        if !valid_ciphertext(&bytes) || hex(&Sha256::digest(&bytes)) != digest.to_ascii_lowercase() {
            return Ok(None);
        }
        let parent = path.parent().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "config path has no parent"))?;
        fs::create_dir_all(parent)?;
        let staging = parent.join(format!(".config.{}.tmp", uuid::Uuid::new_v4().simple()));
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let write_result = (|| {
            let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staging)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&staging, path)?;
            fs::File::open(parent)?.sync_all()?;
            Ok::<(), std::io::Error>(())
        })();
        if write_result.is_err() { let _ = fs::remove_file(&staging); }
        write_result?;
        Ok(Some(digest))
    }
}

pub fn valid_ciphertext(bytes: &[u8]) -> bool {
    bytes.len() > AGE_MAGIC.len() && bytes.len() <= MAX_CONFIG_BYTES && bytes.starts_with(AGE_MAGIC)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    pub schema_version: u8,
    pub root_fingerprint: String,
    pub device_id: String,
    pub counter: u64,
    pub operation: String,
    pub ciphertext_digest: Option<String>,
    pub signature: String,
}

impl Revision {
    pub fn revision_id(&self) -> String {
        let payload = serde_json::json!({"schemaVersion":self.schema_version,"rootFingerprint":self.root_fingerprint,"deviceId":self.device_id,"counter":self.counter,"operation":self.operation,"ciphertextDigest":self.ciphertext_digest});
        let canonical = crate::device_signing::canonical_json(&payload).unwrap_or_default();
        let mut hash = Sha256::new(); hash.update(b"rawrarr-config-revision-v1\0"); hash.update(canonical); hash.update(self.signature.as_bytes()); hex(&hash.finalize())
    }
}

pub fn merkle_root(revisions: &[Revision]) -> String {
    if revisions.is_empty() { return hex(&Sha256::digest(b"rawrarr-config-merkle-empty-v1\0")); }
    let mut leaves = revisions.iter().map(|revision| {
        let leaf = serde_json::json!({"revisionId":revision.revision_id(),"deviceId":revision.device_id,"counter":revision.counter,"operation":revision.operation,"ciphertextDigest":revision.ciphertext_digest});
        let canonical = crate::device_signing::canonical_json(&leaf).unwrap_or_default();
        let mut hash = Sha256::new(); hash.update([0]); hash.update(canonical); hash.finalize().to_vec()
    }).collect::<Vec<_>>();
    leaves.sort();
    while leaves.len() > 1 {
        let mut next = Vec::with_capacity((leaves.len()+1)/2);
        for pair in leaves.chunks(2) {
            if pair.len() == 1 { next.push(pair[0].clone()); }
            else { let mut hash=Sha256::new(); hash.update([1]); hash.update(&pair[0]); hash.update(&pair[1]); next.push(hash.finalize().to_vec()); }
        }
        leaves = next;
    }
    hex(&leaves[0])
}

pub fn winner<'a>(revisions: &'a [Revision]) -> Option<&'a Revision> {
    revisions.iter().max_by_key(|revision| (revision.counter, revision.device_id.as_str(), revision.ciphertext_digest.as_deref().unwrap_or("")))
}

pub fn revision_dir() -> PathBuf {
    env::var_os("RAWARR_DATA_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("data")).join("config-revisions")
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8;16]=b"0123456789abcdef"; let mut out=String::with_capacity(bytes.len()*2);
    for byte in bytes { out.push(DIGITS[(byte>>4) as usize] as char); out.push(DIGITS[(byte&15) as usize] as char); } out
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_config_repair.rs"]
mod tests;
