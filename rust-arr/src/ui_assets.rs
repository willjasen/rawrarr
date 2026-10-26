//! Runtime UI bundle loader with embedded first-boot fallback.
use std::{env, path::PathBuf};

#[derive(serde::Serialize)]
pub struct SignatureStatus {
    pub status: &'static str,
    pub version: Option<String>,
}

pub async fn signature_status() -> SignatureStatus {
    let current = env::var_os("RAWARR_UI_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/ui/current"));
    let verified = tokio::fs::read_to_string(current.join("bundle.signature-status")).await
        .map(|value| value.trim() == "verified").unwrap_or(false);
    let version = tokio::fs::read_to_string(current.join("bundle.version")).await.ok().map(|value| value.trim().to_owned());
    let update_status_path = current.parent().unwrap_or(&current).join("update-status.json");
    let update_status = tokio::fs::read_to_string(update_status_path).await
        .ok().and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok())
        .and_then(|value| value.get("status").and_then(serde_json::Value::as_str).map(str::to_owned));
    let status = if update_status.as_deref() == Some("rejected") { "rejected" }
        else if verified && update_status.as_deref() == Some("signed") { "signed" }
        else { "unverified" };
    SignatureStatus { status, version }
}

pub async fn text(path: &'static str, fallback: &'static str) -> String {
    match bundle_path(path) {
        Some(path) => tokio::fs::read_to_string(path).await.unwrap_or_else(|_| fallback.to_owned()),
        None => fallback.to_owned(),
    }
}

pub async fn bytes(path: &'static str, fallback: &'static [u8]) -> Vec<u8> {
    match bundle_path(path) {
        Some(path) => tokio::fs::read(path).await.unwrap_or_else(|_| fallback.to_vec()),
        None => fallback.to_vec(),
    }
}

fn bundle_path(relative: &str) -> Option<PathBuf> {
    // Callers use compile-time paths; reject traversal anyway so this remains
    // safe if a future caller accepts input.
    if relative.is_empty() || relative.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        return None;
    }
    let root = env::var_os("RAWARR_UI_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/ui/current"));
    Some(root.join(relative))
}
