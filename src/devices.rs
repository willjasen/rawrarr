//! Device and scanner identity types exposed by rawrarr's web API.

use serde::Serialize;
use std::env;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub role: DeviceRole,
    pub connection: DeviceConnection,
    pub status: DeviceStatus,
    pub library_name: Option<String>,
    pub last_seen_unix_seconds: Option<u64>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceRole {
    Scanner,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceConnection {
    Local,
    Remote,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Ready,
    NeedsConfiguration,
}

impl DeviceInfo {
    pub fn local_scanner(library_name: Option<&str>) -> Self {
        let name = env::var("RAWARR_DEVICE_NAME")
            .ok()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "This device".to_owned());
        Self {
            id: "local-scanner".to_owned(),
            name,
            role: DeviceRole::Scanner,
            connection: DeviceConnection::Local,
            status: if library_name.is_some() {
                DeviceStatus::Ready
            } else {
                DeviceStatus::NeedsConfiguration
            },
            library_name: library_name.map(str::to_owned),
            last_seen_unix_seconds: None,
        }
    }
}
