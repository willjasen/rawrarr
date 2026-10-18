//! Subscriber authentication gateway. Only fixed authentication endpoints leave this node.
use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::{
    env, fs, io::ErrorKind, os::unix::fs::OpenOptionsExt, path::PathBuf, sync::Arc, time::Duration,
};
use tokio::time::sleep;
use uuid::Uuid;
use zeroize::Zeroizing;

#[derive(Clone)]
pub struct NodeAuth {
    client: reqwest::Client,
    base: reqwest::Url,
    token: Arc<Zeroizing<String>>,
}

impl NodeAuth {
    pub async fn from_env(
        role: crate::node_role::NodeRole,
        node_id: &str,
        display_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let base = reqwest::Url::parse(&env::var("RAWARR_COORDINATOR_URL")?)?;
        validate_url(
            &base,
            env::var("RAWARR_COORDINATOR_LOCAL_HTTP").as_deref() == Ok("true"),
        )?;
        let token_path = PathBuf::from(env::var("RAWARR_NODE_TOKEN_FILE")?);
        let enrollment_enabled =
            env::var("RAWARR_NODE_ENROLLMENT_ENABLED").as_deref() == Ok("true");
        let token = match fs::read_to_string(&token_path) {
            Ok(token) => Zeroizing::new(token.trim().to_owned()),
            Err(error) if error.kind() == ErrorKind::NotFound && enrollment_enabled => {
                create_node_token(&token_path)?
            }
            Err(error) => return Err(error.into()),
        };
        if token.len() != 64 || !token.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err("node token must contain 64 hexadecimal characters".into());
        }
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::none());
        if let Ok(proxy_url) = env::var("RAWARR_COORDINATOR_SOCKS5_URL") {
            builder = builder.proxy(reqwest::Proxy::all(&proxy_url)?);
        }
        let auth = Self {
            client: builder.build()?,
            base,
            token: Arc::new(token),
        };
        match auth.register(role).await {
            Ok(()) => {
                auth.start_heartbeat(role);
                return Ok(auth);
            }
            Err("coordinator rejected node credential or role") if enrollment_enabled => {}
            Err(error) => return Err(error.into()),
        }
        let origin =
            env::var("RAWARR_WEBAUTHN_ORIGIN").unwrap_or_else(|_| "http://localhost:7297".into());
        let rp_id = env::var("RAWARR_WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".into());
        // Keep this payload deliberately limited to public node metadata. In
        // particular, age identities and shared app data stay on this node.
        let payload = serde_json::json!({"nodeId":node_id,"role":role,"displayName":display_name,"origin":origin,"rpId":rp_id});
        let request = auth
            .client
            .post(auth.base.join("/api/nodes/enroll")?)
            .bearer_auth(auth.token.as_str())
            .json(&payload)
            .send()
            .await?;
        if request.status() != StatusCode::CREATED {
            return Err(
                format!("node enrollment request failed (HTTP {})", request.status()).into(),
            );
        }
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RequestCreated {
            request_id: Uuid,
        }
        let request_id = request.json::<RequestCreated>().await?.request_id;
        println!(
            "Rawrarr {role:?} node {node_id} is waiting for approval in an already signed-in Rawrarr node."
        );
        for _ in 0..300 {
            let body = serde_json::to_vec(&serde_json::json!({"requestId":request_id}))?;
            let response = auth
                .send(
                    reqwest::Method::POST,
                    "/api/nodes/enroll/status",
                    &HeaderMap::new(),
                    body,
                )
                .await
                .map_err(|_| "could not check node enrollment status")?;
            if response.status() == StatusCode::NOT_FOUND {
                return Err(
                    "node enrollment request expired; restart the node to request approval".into(),
                );
            }
            if !response.status().is_success() {
                return Err("could not check node enrollment status".into());
            }
            let bytes = to_bytes(response.into_body(), 64 * 1024).await?;
            let status: EnrollmentStatus = serde_json::from_slice(&bytes)?;
            if status.status == "approved" {
                auth.register(role)
                    .await
                    .map_err(|_| "approved node could not register with coordinator")?;
                auth.start_heartbeat(role);
                return Ok(auth);
            }
            sleep(Duration::from_secs(3)).await;
        }
        Err("node enrollment approval timed out".into())
    }

    async fn send(
        &self,
        method: reqwest::Method,
        path: &str,
        headers: &HeaderMap,
        body: Vec<u8>,
    ) -> Result<Response, ()> {
        if !allowed_path(path) {
            return Err(());
        }
        let mut request = self
            .client
            .request(method, self.base.join(coordinator_path(path)).map_err(|_| ())?)
            .bearer_auth(self.token.as_str())
            .header(header::CONTENT_TYPE, "application/json");
        // Forward only Rawrarr's opaque session, never arbitrary cookies or proxy headers.
        if let Some(cookie) = crate::auth::cookie_token(headers) {
            request = request.header(header::COOKIE, format!("rawrarr_session={cookie}"));
        }
        let mut upstream = request.body(body).send().await.map_err(|_| ())?;
        let status = upstream.status();
        let mut response = Response::builder()
            .status(status)
            .header(header::CACHE_CONTROL, "no-store");
        for name in [header::CONTENT_TYPE, header::SET_COOKIE] {
            if let Some(value) = upstream.headers().get(&name) {
                response = response.header(name, value);
            }
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = upstream.chunk().await.map_err(|_| ())? {
            if bytes.len() + chunk.len() > 256 * 1024 {
                return Err(());
            }
            bytes.extend_from_slice(&chunk);
        }
        response.body(Body::from(bytes)).map_err(|_| ())
    }

    pub async fn authenticated(&self, headers: &HeaderMap) -> bool {
        let Some(token) = direct_session_token(headers).or_else(|| crate::auth::cookie_token(headers)) else {
            return false;
        };
        let Ok(cookie) = HeaderValue::from_str(&format!("rawrarr_session={token}")) else {
            return false;
        };
        let mut session_headers = HeaderMap::new();
        session_headers.insert(header::COOKIE, cookie);
        self.send(reqwest::Method::GET, "/api/auth/session", &session_headers, vec![])
            .await
            .is_ok_and(|response| response.status() == StatusCode::NO_CONTENT)
    }

    pub async fn logout(&self, headers: &HeaderMap) -> Response {
        self.send(reqwest::Method::POST, "/api/auth/logout", headers, vec![])
            .await
            .unwrap_or_else(|_| unavailable())
    }

    pub async fn register(&self, role: crate::node_role::NodeRole) -> Result<(), &'static str> {
        let response = self
            .send(
                reqwest::Method::POST,
                "/api/nodes/register",
                &HeaderMap::new(),
                serde_json::to_vec(&serde_json::json!({"role": role}))
                    .map_err(|_| "invalid node role")?,
            )
            .await
            .map_err(|_| "coordinator unavailable; node registration failed")?;
        if response.status() != StatusCode::NO_CONTENT {
            return Err("coordinator rejected node credential or role");
        }
        Ok(())
    }

    fn start_heartbeat(&self, role: crate::node_role::NodeRole) {
        let auth = self.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                let _ = auth.register(role).await;
            }
        });
    }

    async fn forward_enrollment(
        &self,
        path: &str,
        token: &str,
        body: Vec<u8>,
    ) -> Result<Response, ()> {
        if !matches!(path, "/api/nodes/enroll" | "/api/nodes/enroll/status")
            || token.len() != 64
            || !token.bytes().all(|byte| byte.is_ascii_hexdigit())
            || body.len() > 64 * 1024
        {
            return Err(());
        }
        let upstream = self
            .client
            .post(self.base.join(path).map_err(|_| ())?)
            .bearer_auth(token)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await
            .map_err(|_| ())?;
        let status = upstream.status();
        let content_type = upstream.headers().get(header::CONTENT_TYPE).cloned();
        let bytes = upstream.bytes().await.map_err(|_| ())?;
        if bytes.len() > 64 * 1024 {
            return Err(());
        }
        let mut response = Response::builder()
            .status(status)
            .header(header::CACHE_CONTROL, "no-store");
        if let Some(content_type) = content_type {
            response = response.header(header::CONTENT_TYPE, content_type);
        }
        response.body(Body::from(bytes)).map_err(|_| ())
    }
}

#[derive(serde::Deserialize)]
struct EnrollmentStatus {
    status: String,
}

fn create_node_token(
    path: &std::path::Path,
) -> Result<Zeroizing<String>, Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes)?;
    let token = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    use std::io::Write;
    writeln!(file, "{token}")?;
    file.sync_all()?;
    Ok(Zeroizing::new(token))
}

fn unavailable() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "Coordinator unavailable; try again shortly",
    )
        .into_response()
}

pub async fn proxy(
    State(state): State<std::sync::Arc<crate::AppState>>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let Ok(body) = to_bytes(body, 64 * 1024).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };
    state
        .auth
        .send(
            parts.method,
            parts.uri.path(),
            &parts.headers,
            body.to_vec(),
        )
        .await
        .unwrap_or_else(|_| unavailable())
}

/// Preserve a new node's bearer token across the metadata-to-coordinator hop.
/// Only the public, metadata-only enrollment endpoints can use this path.
pub async fn enrollment_proxy(
    State(state): State<std::sync::Arc<crate::AppState>>,
    request: Request,
) -> Response {
    let path = request.uri().path().to_owned();
    let authorization = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::to_owned);
    let Ok(body) = to_bytes(request.into_body(), 64 * 1024).await else {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    };
    let Some(token) = authorization else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    state
        .auth
        .forward_enrollment(&path, &token, body.to_vec())
        .await
        .unwrap_or_else(|_| unavailable())
}

pub async fn require_session(
    State(auth): State<NodeAuth>,
    request: Request,
    next: Next,
) -> Response {
    if auth.authenticated(request.headers()).await {
        return next.run(request).await;
    }
    if request.uri().path() == "/" {
        Redirect::to("/auth").into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            "Sign in with your passkey to continue",
        )
            .into_response()
    }
}

fn allowed_path(path: &str) -> bool {
    matches!(
        path,
        "/api/auth/status"
            | "/api/auth/register/start"
            | "/api/auth/register/finish"
            | "/api/auth/login/start"
            | "/api/auth/login/finish"
            | "/api/auth/passkeys/coordinator/link"
            | "/api/auth/session"
            | "/api/auth/logout"
            | "/api/nodes/register"
            | "/api/nodes/enroll"
            | "/api/nodes/enroll/status"
            | "/api/nodes/pending"
            | "/api/nodes/approve"
    )
}

fn coordinator_path(path: &str) -> &str {
    match path {
        "/api/auth/status" => "/api/nodes/auth/status",
        "/api/auth/register/start" => "/api/nodes/auth/register/start",
        "/api/auth/register/finish" => "/api/nodes/auth/register/finish",
        "/api/auth/login/start" => "/api/nodes/auth/login/start",
        "/api/auth/login/finish" => "/api/nodes/auth/login/finish",
        "/api/auth/session" => "/api/nodes/auth/session",
        "/api/auth/logout" => "/api/nodes/auth/logout",
        _ => path,
    }
}

fn direct_session_token(headers: &HeaderMap) -> Option<&str> {
    if let Some(token) = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
    {
        return Some(token);
    }
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(name, value)| {
            (name == "rawrarr_direct_session"
                && value.len() == 64
                && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
                .then_some(value)
        })
}

#[cfg(test)]
mod direct_session_cookie_tests {
    use super::direct_session_token;
    use axum::http::{HeaderMap, HeaderValue, header};

    #[test]
    fn direct_session_cookie_is_selected_and_must_be_64_hex_characters() {
        let token = "a".repeat(64);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&format!("rawrarr_session=coordinator; rawrarr_direct_session={token}"))
                .expect("valid cookie header"),
        );
        assert_eq!(direct_session_token(&headers), Some(token.as_str()));
        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("rawrarr_direct_session=not-a-session"),
        );
        assert_eq!(direct_session_token(&headers), None);
    }

    #[test]
    fn direct_session_bearer_is_selected_and_must_be_64_hex_characters() {
        let token = "b".repeat(64);
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {token}")).expect("valid authorization"),
        );
        assert_eq!(direct_session_token(&headers), Some(token.as_str()));
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer invalid"));
        assert_eq!(direct_session_token(&headers), None);
    }
}

fn validate_url(url: &reqwest::Url, local_http: bool) -> Result<(), &'static str> {
    let local = matches!(
        url.host_str(),
        Some(
            "coordinator"
                | "rawrarr-mini-tailscale"
                | "localhost"
                | "127.0.0.1"
                | "[::1]"
        )
    );
    if (url.scheme() != "https" && !(local_http && local && url.scheme() == "http"))
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(
            "coordinator URL must be an HTTPS origin; local HTTP is restricted to the explicit local-development option",
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_node_auth.rs"]
mod tests;
