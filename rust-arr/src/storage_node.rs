//! Storage-node media scanning and age-encrypted sidecar publication.

use crate::{devices, library, secure_config};
use std::{error::Error, fs, time::Duration};

const DEFAULT_SCAN_INTERVAL_SECONDS: u64 = 6 * 60 * 60;
const MIN_SCAN_INTERVAL_SECONDS: u64 = 60;
const MAX_SCAN_INTERVAL_SECONDS: u64 = 7 * 24 * 60 * 60;

/// Read the unattended scan interval. Zero is an explicit opt-out.
pub fn scan_interval_from_env() -> Result<Option<Duration>, &'static str> {
    let configured = std::env::var("RAWARR_STORAGE_SCAN_INTERVAL_SECONDS").ok();
    parse_scan_interval(configured.as_deref())
}

fn parse_scan_interval(value: Option<&str>) -> Result<Option<Duration>, &'static str> {
    let seconds = value
        .map(str::parse::<u64>)
        .transpose()
        .map_err(|_| "RAWARR_STORAGE_SCAN_INTERVAL_SECONDS must be an integer")?
        .unwrap_or(DEFAULT_SCAN_INTERVAL_SECONDS);
    if seconds == 0 {
        return Ok(None);
    }
    if !(MIN_SCAN_INTERVAL_SECONDS..=MAX_SCAN_INTERVAL_SECONDS).contains(&seconds) {
        return Err("RAWARR_STORAGE_SCAN_INTERVAL_SECONDS must be 0 or between 60 and 604800");
    }
    Ok(Some(Duration::from_secs(seconds)))
}

/// Run an initial scan after enrollment, then wait the configured interval
/// between scans. Work is moved off the async executor because walking media
/// trees and encrypting sidecars are blocking operations.
pub fn start_periodic_scan(interval: Duration, logger: Option<crate::shared_logs::SharedLogger>) {
    tokio::spawn(async move {
        loop {
            match tokio::task::spawn_blocking(|| scan_once().map_err(|error| error.to_string()))
                .await
            {
                Ok(Ok(summary)) => {
                    if let Some(logger) = &logger {
                        logger.scan_completed(
                            summary.items.len(),
                            summary.sidecars_created,
                            summary.sidecar_errors,
                            summary.issues,
                            summary.truncated,
                        );
                    }
                    println!(
                        "Storage scan complete: {} media files; {} encrypted sidecars saved; {} sidecar errors; {} scan issues; truncated={}",
                        summary.items.len(),
                        summary.sidecars_created,
                        summary.sidecar_errors,
                        summary.issues,
                        summary.truncated
                    );
                    if summary.sidecar_errors > 0 || summary.issues > 0 || summary.truncated {
                        eprintln!(
                            "storage scan was incomplete; it will retry after the configured interval"
                        );
                    }
                }
                Ok(Err(_)) | Err(_) => {
                    eprintln!("storage scan failed; it will retry after the configured interval");
                }
            }
            tokio::time::sleep(interval).await;
        }
    });
}

pub fn scan_once() -> Result<library::ScanSummary, Box<dyn Error>> {
    let root = library::LibraryRoot::from_env()?.ok_or("RAWARR_LIBRARY_PATH is required")?;
    let metadata_dir = secure_config::AgeVault::metadata_dir();
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
                    &metadata_dir,
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

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_storage_node.rs"]
mod tests;
