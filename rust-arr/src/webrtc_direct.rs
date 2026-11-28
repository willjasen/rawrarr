//! Bounded WebRTC workspace protocol and local transport qualification.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const WORKSPACE_PROTOCOL_VERSION: u8 = 1;
pub const MAX_WORKSPACE_FRAME_BYTES: usize = 48 * 1024;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum WorkspaceMethod {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceRequest {
    pub version: u8,
    pub request_id: Uuid,
    pub session_token: String,
    pub method: WorkspaceMethod,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

impl WorkspaceRequest {
    pub fn decode(frame: &[u8]) -> Result<Self, &'static str> {
        if frame.len() > MAX_WORKSPACE_FRAME_BYTES {
            return Err("WebRTC request frame exceeds the size limit");
        }
        let request: Self =
            serde_json::from_slice(frame).map_err(|_| "WebRTC request frame is invalid")?;
        request.validate()?;
        Ok(request)
    }

    pub fn encode(&self) -> Result<Vec<u8>, &'static str> {
        self.validate()?;
        let frame = serde_json::to_vec(self).map_err(|_| "WebRTC request frame is invalid")?;
        if frame.len() > MAX_WORKSPACE_FRAME_BYTES {
            return Err("WebRTC request frame exceeds the size limit");
        }
        Ok(frame)
    }

    fn validate(&self) -> Result<(), &'static str> {
        if self.version != WORKSPACE_PROTOCOL_VERSION {
            return Err("unsupported WebRTC workspace protocol version");
        }
        if self.session_token.len() != 64
            || !self
                .session_token
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("invalid node session token");
        }
        if self.path.len() > 256
            || !self.path.starts_with("/api/")
            || self
                .path
                .chars()
                .any(|character| matches!(character, '?' | '#' | '\\'))
            || self.path.split('/').any(|part| part == "." || part == "..")
        {
            return Err("invalid workspace operation path");
        }
        if !workspace_operation_allowed(self.method, self.path.as_str()) {
            return Err("workspace operation is not allowed over WebRTC");
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkspaceResponse {
    pub version: u8,
    pub request_id: Uuid,
    pub status: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Value>,
}

impl WorkspaceResponse {
    pub fn encode(&self) -> Result<Vec<u8>, &'static str> {
        if self.version != WORKSPACE_PROTOCOL_VERSION || !(100..=599).contains(&self.status) {
            return Err("invalid WebRTC workspace response");
        }
        let frame = serde_json::to_vec(self).map_err(|_| "invalid WebRTC workspace response")?;
        if frame.len() > MAX_WORKSPACE_FRAME_BYTES {
            return Err("WebRTC response frame exceeds the size limit");
        }
        Ok(frame)
    }
}

fn workspace_operation_allowed(method: WorkspaceMethod, path: &str) -> bool {
    match method {
        WorkspaceMethod::Get => matches!(
            path,
            "/api/settings/age"
                | "/api/settings/radarr"
                | "/api/settings/sonarr"
                | "/api/settings/lidarr"
                | "/api/settings/prowlarr"
                | "/api/prowlarr/indexers"
                | "/api/sonarr/series"
                | "/api/lidarr/library"
                | "/api/metadata/artwork/backfill"
                | "/api/metadata/sonarr/backfill"
                | "/api/radarr/live-library"
        ),
        WorkspaceMethod::Post => matches!(
            path,
            "/api/settings/age"
                | "/api/settings/radarr"
                | "/api/settings/sonarr"
                | "/api/settings/lidarr"
                | "/api/settings/prowlarr"
                | "/api/prowlarr/search"
                | "/api/metadata/artwork/rescan"
                | "/api/metadata/sonarr/rescan"
        ),
        WorkspaceMethod::Put => matches!(
            path,
            "/api/settings/age/group-identity"
                | "/api/settings/radarr"
                | "/api/settings/sonarr"
                | "/api/settings/lidarr"
                | "/api/settings/prowlarr"
        ),
        WorkspaceMethod::Delete => matches!(
            path,
            "/api/settings/radarr"
                | "/api/settings/sonarr"
                | "/api/settings/lidarr"
                | "/api/settings/prowlarr"
        ),
    }
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_webrtc_direct.rs"]
mod tests;

/// Bind a browser challenge and the negotiated SDP to the approved node key.
/// Callers must verify the certificate against their independently pinned root.
pub(crate) fn node_proof_payload(
    session_id: Uuid,
    node_id: &str,
    challenge: &str,
    offer_sdp: &str,
    answer_sdp: &str,
) -> Result<serde_json::Value, &'static str> {
    use sha2::{Digest, Sha256};
    if challenge.len() != 64 || !challenge.bytes().all(|byte| byte.is_ascii_hexdigit())
        || node_id.is_empty() || node_id.len() > 128
        || offer_sdp.len() > MAX_WORKSPACE_FRAME_BYTES
        || answer_sdp.len() > MAX_WORKSPACE_FRAME_BYTES
    {
        return Err("invalid WebRTC proof context");
    }
    Ok(serde_json::json!({
        "domain": "rawrarr-webrtc-node-proof-v1",
        "version": WORKSPACE_PROTOCOL_VERSION,
        "sessionId": session_id,
        "nodeId": node_id,
        "challenge": challenge,
        "offerSha256": format!("{:x}", Sha256::digest(offer_sdp.as_bytes())),
        "answerSha256": format!("{:x}", Sha256::digest(answer_sdp.as_bytes())),
    }))
}
