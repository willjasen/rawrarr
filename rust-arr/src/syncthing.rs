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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerHealth {
    pub device_id: String,
    pub name: String,
    pub connected: bool,
    pub folder_state: Option<String>,
    pub completion_percent: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedHealth {
    pub folder_id: String,
    pub state: String,
    pub completion_percent: Option<f64>,
    pub local_device_id: String,
    pub local_device_name: String,
    pub peers: Vec<PeerHealth>,
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
        let key = match env::var("RAWARR_SYNCTHING_API_KEY")
            .ok()
            .filter(|key| !key.trim().is_empty()) {
            Some(key) => key,
            None => {
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
            eprintln!("Syncthing API request {path} returned HTTP {}", response.status());
            return Err(Error::Rejected);
        }
        response.json().await.map_err(|_| {
            eprintln!("Syncthing API request {path} returned invalid JSON");
            Error::Rejected
        })
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
        let Some(my_id) = value.get("myID").and_then(Value::as_str) else {
            eprintln!("Syncthing status response omitted its device ID");
            return Err(Error::Rejected);
        };
        let version_response = self.get("rest/system/version").await?;
        let Some(version) = version_response.get("version").and_then(Value::as_str) else {
            eprintln!("Syncthing status response omitted its version");
            return Err(Error::Rejected);
        };
        Ok(Status { my_id: my_id.to_owned(), version: version.to_owned() })
    }

    /// Read local folder state and peer connectivity for one shared folder.
    pub async fn shared_health(&self, folder_id: &str) -> Result<SharedHealth, Error> {
        let local_status = self.status().await?;
        let folders = self.get("rest/config/folders").await?;
        let folder = folders.as_array().and_then(|all| all.iter().find(|f| f.get("id").and_then(Value::as_str) == Some(folder_id)))
            .ok_or_else(|| { eprintln!("Syncthing folder {folder_id} was not found in its config response"); Error::Rejected })?;
        let state_value = self.get(&format!("rest/db/status?folder={}", folder_id)).await?;
        let state = state_value["state"].as_str().unwrap_or("unknown").to_owned();
        let completion = self.get(&format!("rest/db/completion?folder={}", folder_id)).await.ok();
        let completion_percent = completion.as_ref().and_then(|v| v["completion"].as_f64());
        let connections = self.get("rest/system/connections").await?;
        let devices = self.get("rest/config/devices").await?;
        let local_device_name = devices.as_array().and_then(|all| all.iter().find(|d| d.get("deviceID").and_then(Value::as_str) == Some(local_status.my_id.as_str())))
            .and_then(|d| d["name"].as_str()).unwrap_or("This device").to_owned();
        let folder_devices = folder["devices"].as_array().cloned().unwrap_or_default();
        let mut peers = Vec::new();
        for fd in folder_devices {
            let Some(id) = fd["deviceID"].as_str() else { continue };
            if id == local_status.my_id { continue; }
            let Some(configured) = devices.as_array().and_then(|all| all.iter().find(|d| d.get("deviceID").and_then(Value::as_str) == Some(id))) else { continue };
            let connection = &connections["connections"][id];
            let query = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("folder", folder_id).append_pair("device", id).finish();
            // Peer completion is supplementary. Syncthing may omit it while
            // the device is disconnected or still indexing; keep reporting
            // folder state and connectivity in that case.
            let peer_completion = self.get(&format!("rest/db/completion?{query}")).await.ok()
                .and_then(|value| value["completion"].as_f64());
            peers.push(PeerHealth {
                device_id: id.to_owned(),
                name: configured["name"].as_str().unwrap_or(id).to_owned(),
                connected: connection["connected"].as_bool().unwrap_or(false),
                folder_state: connection["paused"].as_bool().map(|paused| if paused { "paused".into() } else { "connected".into() }),
                completion_percent: peer_completion,
            });
        }
        Ok(SharedHealth { folder_id: folder_id.to_owned(), state, completion_percent, local_device_id: local_status.my_id, local_device_name, peers })
    }

    pub async fn shared_health_all(&self) -> Result<Vec<SharedHealth>, Error> {
        let folders = self.get("rest/config/folders").await?;
        let Some(folders) = folders.as_array() else {
            eprintln!("Syncthing folder config response was not a list");
            return Err(Error::Rejected);
        };
        let ids = folders.iter()
            .filter_map(|folder| folder["id"].as_str().map(str::to_owned)).collect::<Vec<_>>();
        let mut result = Vec::new();
        for id in ids {
            match self.shared_health(&id).await {
                Ok(health) => result.push(health),
                Err(error) => {
                    eprintln!("Syncthing shared-folder health query failed: {error:?}");
                    return Err(error);
                }
            }
        }
        Ok(result)
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
    use axum::{Json, Router, http::StatusCode, routing::get};
    use serde_json::json;

    #[tokio::test]
    async fn peer_completion_failure_does_not_hide_folder_health() {
        let app = Router::new()
            .route("/rest/system/status", get(|| async { Json(json!({"myID":"LOCAL"})) }))
            .route("/rest/system/version", get(|| async { Json(json!({"version":"test"})) }))
            .route("/rest/config/folders", get(|| async { Json(json!([{
                "id":"shared", "devices":[{"deviceID":"LOCAL"},{"deviceID":"PEER"}]
            }])) }))
            .route("/rest/db/status", get(|| async { Json(json!({"state":"idle"})) }))
            .route("/rest/db/completion", get(|uri: axum::http::Uri| async move {
                if uri.query().unwrap_or_default().contains("device=PEER") {
                    (StatusCode::NOT_FOUND, Json(json!({"error":"peer completion unavailable"})))
                } else {
                    (StatusCode::OK, Json(json!({"completion":100.0})))
                }
            }))
            .route("/rest/system/connections", get(|| async { Json(json!({
                "connections":{"PEER":{"connected":true,"paused":false}}
            })) }))
            .route("/rest/config/devices", get(|| async { Json(json!([
                {"deviceID":"LOCAL","name":"Local"}, {"deviceID":"PEER","name":"Peer"}
            ])) }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
        let client = Client::new(&format!("http://{address}"), "fixture").unwrap();

        let reports = client.shared_health_all().await.unwrap();
        assert_eq!(reports.len(), 1);
        assert_eq!(reports[0].state, "idle");
        assert_eq!(reports[0].completion_percent, Some(100.0));
        assert_eq!(reports[0].peers.len(), 1);
        assert!(reports[0].peers[0].connected);
        assert_eq!(reports[0].peers[0].completion_percent, None);
        server.abort();
    }

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
