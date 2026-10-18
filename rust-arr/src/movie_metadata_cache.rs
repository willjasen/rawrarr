//! Unlocked-vault-local metadata memoization; ciphertext changes invalidate entries.
use super::VaultError;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs, path::{Path, PathBuf}, sync::Mutex};

#[derive(Default)]
pub(super) struct MetadataCache(Mutex<HashMap<PathBuf, ([u8; 32], Value)>>);

impl MetadataCache {
    pub(super) fn load(
        &self, path: &Path, decrypt: impl FnOnce() -> Result<Option<Value>, VaultError>,
    ) -> Result<Option<Value>, VaultError> {
        let ciphertext = fs::read(path).map_err(|_| VaultError::StorageFailed)?;
        let digest: [u8; 32] = Sha256::digest(&ciphertext).into();
        if let Some((saved_digest, value)) = self.0.lock().map_err(|_| VaultError::StorageFailed)?.get(path) {
            if *saved_digest == digest { return Ok(Some(value.clone())); }
        }
        let value = decrypt()?;
        // A synchronized replacement during decryption must not poison the cache.
        if let Some(value) = &value {
            if fs::read(path).ok().as_deref() == Some(ciphertext.as_slice()) {
                let mut entries = self.0.lock().map_err(|_| VaultError::StorageFailed)?;
                if entries.len() >= 4096 { entries.clear(); }
                entries.insert(path.to_owned(), (digest, value.clone()));
            }
        }
        Ok(value)
    }
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_metadata_cache.rs"]
mod tests;
