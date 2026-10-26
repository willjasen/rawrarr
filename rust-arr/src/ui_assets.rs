//! Runtime UI bundle loader with embedded first-boot fallback.
use std::{env, path::PathBuf};

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
