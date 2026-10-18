//! Device and scanner identity types exposed by rawrarr's web API.

use serde::Serialize;
use std::{env, fs, io, path::PathBuf};
use uuid::Uuid;

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
    pub fn local_scanner(library_name: Option<&str>, device_id: &str) -> Self {
        let name = env::var("RAWARR_DEVICE_NAME")
            .ok()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| "This device".to_owned());
        Self {
            id: device_id.to_owned(),
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

/// Load an explicitly configured device ID or persist one in node-local data.
pub fn load_or_create_device_id() -> Result<String, io::Error> {
    if let Ok(device_id) = env::var("RAWARR_DEVICE_ID") {
        if valid_device_id(&device_id) {
            return Ok(device_id);
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "RAWARR_DEVICE_ID must be 1-128 ASCII letters, digits, hyphens, or underscores",
        ));
    }

    let data_dir = env::var_os("RAWARR_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"));
    fs::create_dir_all(&data_dir)?;
    let id_path = data_dir.join("device-id");
    match fs::read_to_string(&id_path) {
        Ok(device_id) => {
            let device_id = device_id.trim();
            if valid_device_id(device_id) {
                return Ok(device_id.to_owned());
            }
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "saved Rawrarr device ID is invalid",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    let device_id = Uuid::new_v4().to_string();
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    match options.open(&id_path) {
        Ok(mut file) => {
            use io::Write;
            writeln!(file, "{device_id}")?;
            file.sync_all()?;
            Ok(device_id)
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            let existing = fs::read_to_string(id_path)?;
            let existing = existing.trim();
            if valid_device_id(existing) {
                Ok(existing.to_owned())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "saved Rawrarr device ID is invalid",
                ))
            }
        }
        Err(error) => Err(error),
    }
}

fn valid_device_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}
