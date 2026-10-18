//! Management client for Rawrarr's Syncthing peers.
//!
//! Syncthing itself performs the encrypted peer protocol. Rawrarr uses its
//! authenticated REST API to introduce peers and configure shared folders.

use reqwest::{Client as HttpClient, Url, header::HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{env, path::Path};
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone)]
pub struct Client {
    base: Url,
    key: HeaderValue,
    http: HttpClient,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub my_id: String,
    pub version: String,
}

#[derive(Debug)]
pub enum Error {
    InvalidConfiguration,
    Request,
    Rejected,
}

impl Client {
    pub fn from_env() -> Result<Option<Self>, Error> {
        let Some(url) = env::var("RAWARR_SYNCTHING_API_URL")
            .ok()
            .filter(|v| !v.trim().is_empty())
        else {
            return Ok(None);
        };
        let key = match env::var("RAWARR_SYNCTHING_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                let path = env::var_os("RAWARR_SYNCTHING_API_KEY_FILE")
                    .ok_or(Error::InvalidConfiguration)?;
                std::fs::read_to_string(Path::new(&path))
                    .map_err(|_| Error::InvalidConfiguration)?
            }
        };
        Self::new(&url, key.trim()).map(Some)
    }

    pub fn new(url: &str, key: &str) -> Result<Self, Error> {
        let mut base = Url::parse(url).map_err(|_| Error::InvalidConfiguration)?;
        if !matches!(base.scheme(), "http" | "https")
            || base.host_str().is_none()
            || !base.username().is_empty()
            || base.password().is_some()
            || base.query().is_some()
            || base.fragment().is_some()
            || key.trim().is_empty()
        {
            return Err(Error::InvalidConfiguration);
        }
        if !base.path().ends_with('/') {
            base.set_path(&format!("{}/", base.path()));
        }
        let mut key = Zeroizing::new(key.trim().to_owned());
        let mut header = HeaderValue::from_str(&key).map_err(|_| Error::InvalidConfiguration)?;
        header.set_sensitive(true);
        key.zeroize();
        let http = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| Error::InvalidConfiguration)?;
        Ok(Self {
            base,
            key: header,
            http,
        })
    }

    async fn get(&self, path: &str) -> Result<Value, Error> {
        let url = self
            .base
            .join(path)
            .map_err(|_| Error::InvalidConfiguration)?;
        let response = self
            .http
            .get(url)
            .header("X-API-Key", self.key.clone())
            .send()
            .await
            .map_err(|_| Error::Request)?;
        if !response.status().is_success() {
            return Err(Error::Rejected);
        }
        response.json().await.map_err(|_| Error::Rejected)
    }

    async fn post(&self, path: &str, body: &Value) -> Result<(), Error> {
        let url = self
            .base
            .join(path)
            .map_err(|_| Error::InvalidConfiguration)?;
        let response = self
            .http
            .post(url)
            .header("X-API-Key", self.key.clone())
            .json(body)
            .send()
            .await
            .map_err(|_| Error::Request)?;
        if !response.status().is_success() {
            return Err(Error::Rejected);
        }
        Ok(())
    }

    pub async fn status(&self) -> Result<Status, Error> {
        let value = self.get("rest/system/status").await?;
        serde_json::from_value(
            json!({ "myId": value.get("myID"), "version": value.get("version") }),
        )
        .map_err(|_| Error::Rejected)
    }

    async fn add_device(&self, id: &str, name: &str, address: &str) -> Result<(), Error> {
        let mut device = self.get("rest/config/defaults/device").await?;
        device["deviceID"] = json!(id);
        device["name"] = json!(name);
        device["addresses"] = json!([address]);
        self.post("rest/config/devices", &device).await
    }

    async fn add_folder(
        &self,
        id: &str,
        label: &str,
        path: &str,
        peer_id: &str,
        folder_type: &str,
    ) -> Result<(), Error> {
        let mut folder = self.get("rest/config/defaults/folder").await?;
        folder["id"] = json!(id);
        folder["label"] = json!(label);
        folder["path"] = json!(path);
        folder["type"] = json!(folder_type);
        folder["paused"] = json!(false);
        folder["devices"] = json!([{ "deviceID": peer_id, "introducedBy": "" }]);
        self.post("rest/config/folders", &folder).await
    }

    pub async fn share_with(
        &self,
        remote: &Client,
        request: &ShareRequest,
    ) -> Result<(Status, Status), Error> {
        validate_share(request)?;
        let local_status = self.status().await?;
        let remote_status = remote.status().await?;
        if local_status.my_id == remote_status.my_id {
            return Err(Error::InvalidConfiguration);
        }

        self.add_device(
            &remote_status.my_id,
            &request.remote_name,
            &request.remote_address,
        )
        .await?;
        remote
            .add_device(
                &local_status.my_id,
                &request.local_name,
                &request.local_address,
            )
            .await?;
        self.add_folder(
            &request.folder_id,
            &request.label,
            &request.local_path,
            &remote_status.my_id,
            "sendonly",
        )
        .await?;
        remote
            .add_folder(
                &request.folder_id,
                &request.label,
                &request.remote_path,
                &local_status.my_id,
                "receiveonly",
            )
            .await?;
        Ok((local_status, remote_status))
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareRequest {
    pub remote_api_url: String,
    pub remote_api_key: String,
    pub folder_id: String,
    pub label: String,
    pub local_path: String,
    pub remote_path: String,
    pub local_name: String,
    pub remote_name: String,
    pub local_address: String,
    pub remote_address: String,
}

impl Drop for ShareRequest {
    fn drop(&mut self) {
        self.remote_api_key.zeroize();
    }
}

fn validate_share(request: &ShareRequest) -> Result<(), Error> {
    for value in [
        &request.folder_id,
        &request.label,
        &request.local_name,
        &request.remote_name,
    ] {
        if value.trim().is_empty() || value.len() > 128 {
            return Err(Error::InvalidConfiguration);
        }
    }
    for value in [&request.local_path, &request.remote_path] {
        if !Path::new(value).is_absolute() || value.contains('\0') {
            return Err(Error::InvalidConfiguration);
        }
    }
    for value in [&request.local_address, &request.remote_address] {
        if !(value.starts_with("tcp://") || value.starts_with("quic://") || value == "dynamic")
            || value.contains('@')
            || value.contains('#')
            || value.contains('?')
        {
            return Err(Error::InvalidConfiguration);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_url_credentials_and_non_http_schemes() {
        assert!(Client::new("https://user:pass@example.test", "key").is_err());
        assert!(Client::new("file:///tmp/config", "key").is_err());
        assert!(Client::new("https://example.test", "key").is_ok());
    }

    #[test]
    fn validates_paths_and_peer_addresses() {
        let request = ShareRequest {
            remote_api_url: "https://example.test".into(),
            remote_api_key: "secret".into(),
            folder_id: "metadata".into(),
            label: "Metadata".into(),
            local_path: "/data/metadata".into(),
            remote_path: "/media/metadata".into(),
            local_name: "web".into(),
            remote_name: "storage".into(),
            local_address: "tcp://web:22000".into(),
            remote_address: "tcp://storage:22000".into(),
        };
        assert!(validate_share(&request).is_ok());
        let mut bad = request;
        bad.local_path = "relative".into();
        assert!(validate_share(&bad).is_err());
    }
}
