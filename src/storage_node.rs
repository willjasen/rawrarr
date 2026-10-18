//! One-shot scanner mode for a media-holding node without the web service.

use crate::{devices, library, secure_config};
use std::{error::Error, fs};

pub fn scan_once() -> Result<library::ScanSummary, Box<dyn Error>> {
    let root = library::LibraryRoot::from_env()?.ok_or("RAWARR_LIBRARY_PATH is required")?;
    let shared_dir = secure_config::AgeVault::shared_dir();
    let recipient = fs::read_to_string(secure_config::AgeVault::public_recipient_path())?;
    let recipient = recipient.trim();
    if !secure_config::is_post_quantum_recipient(recipient) {
        return Err("shared age-recipient is missing or invalid".into());
    }

    let device_id = devices::load_or_create_device_id()?;
    let mut summary = root.scan();
    for file in &summary.items {
        let sidecar = library::MediaSidecar {
            schema_version: 1,
            device_id: device_id.clone(),
            library_name: summary.library_name.clone(),
            scanned_at_unix_seconds: summary.scanned_at_unix_seconds,
            file: file.clone(),
        };
        match serde_json::to_vec(&sidecar)
            .map_err(|_| secure_config::VaultError::InvalidConfig)
            .and_then(|json| {
                secure_config::AgeVault::store_library_sidecar_with_recipient(
                    &shared_dir,
                    recipient,
                    &device_id,
                    &summary.library_name,
                    &file.path,
                    &json,
                )
            }) {
            Ok(()) => summary.sidecars_created += 1,
            Err(_) => summary.sidecar_errors += 1,
        }
    }
    Ok(summary)
}
