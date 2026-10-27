//! Central passkey authentication and owner-approved node enrollment.
#[path = "coordinator_ai.rs"]
mod ai;
use crate::{auth::AuthState, node_role::NodeRole};
use axum::{
    Extension, Json, Router,
    body::{Body, to_bytes},
    extract::{Path, Request, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, delete, get, post},
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Row};
use std::{collections::{HashMap, HashSet}, env, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApprovedNode {
    id: String,
    role: NodeRole,
    token_sha256: String,
    rp_id: String,
    origin: String,
}

fn parse_approved_nodes(registry: &str) -> Result<Vec<ApprovedNode>, serde_json::Error> {
    serde_json::from_str(registry)
}

#[derive(Clone)]
struct ApprovedAuth {
    digest: String,
    auth: AuthState,
    role: NodeRole,
    signing_public_key: Option<Vec<u8>>,
}

#[derive(Clone)]
struct CoordinatorState {
    pool: PgPool,
    approved: Arc<RwLock<Vec<ApprovedAuth>>>,
    webauthn: webauthn_rs::prelude::Webauthn,
    web_origin: String,
    coordinator_auth: AuthState,
    relay_targets: HashMap<String, reqwest::Url>,
    direct_targets: HashMap<String, reqwest::Url>,
    relay_client: reqwest::Client,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EnrollmentRequest {
    node_id: String,
    role: NodeRole,
    display_name: String,
    rp_id: String,
    origin: String,
    signing_public_key_hex: String,
    key_proof_signature_hex: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EnrollmentStatusRequest {
    request_id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EnrollmentStatus {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    node_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PendingNode {
    request_id: Uuid,
    node_id: String,
    requested_node_id: String,
    role: NodeRole,
    display_name: String,
    owner_display_name: String,
    origin: String,
    requested_at: String,
    signing_public_key_hex: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ApproveRequest {
    request_id: Uuid,
    root_public_key: String,
    root_fingerprint: String,
    signature: String,
    signed_payload: serde_json::Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstanceSigningRoot {
    algorithm: &'static str,
    public_key: String,
    fingerprint: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SigningCertificate {
    algorithm: &'static str,
    root_public_key: String,
    root_fingerprint: String,
    node_id: String,
    role: String,
    signing_public_key_hex: String,
    signing_fingerprint: String,
    signed_payload: serde_json::Value,
    signature: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DirectSessionRequest {
    node_id: String,
}

pub async fn router() -> Result<Router, Box<dyn std::error::Error>> {
    let registry = std::fs::read_to_string(std::env::var("RAWARR_APPROVED_NODES_FILE")?)?;
    let nodes = parse_approved_nodes(&registry).map_err(|_| "invalid approved-node registry")?;
    let pool = crate::auth::database_pool().await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    let web_rp_id = std::env::var("RAWARR_COORDINATOR_WEB_RP_ID")
        .unwrap_or_else(|_| "localhost".to_owned());
    let web_origin_text = std::env::var("RAWARR_COORDINATOR_WEB_ORIGIN")
        .unwrap_or_else(|_| "http://localhost:7298".to_owned());
    let web_origin = validate_origin(&web_rp_id, &web_origin_text)?;
    let webauthn = webauthn_rs::prelude::WebauthnBuilder::new(&web_rp_id, &web_origin)?
        .rp_name("rawrarr")
        .build()?;
    let coordinator_auth = AuthState {
        pool: pool.clone(),
        webauthn: webauthn.clone(),
        secure_cookie: web_origin.scheme() == "https",
        node_id: "coordinator".to_owned(),
        next_path: "/",
    };
    let mut ids = HashSet::new();
    let mut digests = HashSet::new();
    for node in nodes {
        if node.role == NodeRole::Coordinator
            || !valid_node_id(&node.id)
            || !valid_digest(&node.token_sha256)
            || !ids.insert(node.id.clone())
            || !digests.insert(node.token_sha256.to_lowercase())
        {
            return Err("invalid or duplicate approved node".into());
        }
        let origin = validate_origin(&node.rp_id, &node.origin)?;
        let revoked = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_revoked_nodes WHERE node_id=$1 AND token_sha256=$2)")
            .bind(&node.id).bind(node.token_sha256.to_lowercase()).fetch_one(&pool).await?;
        if revoked {
            continue;
        }
        sqlx::query("INSERT INTO rawrarr_nodes(id, role, token_sha256, origin, rp_id) VALUES ($1, $2, $3, $4, $5) ON CONFLICT(id) DO UPDATE SET token_sha256 = COALESCE(rawrarr_nodes.token_sha256, EXCLUDED.token_sha256), origin = COALESCE(rawrarr_nodes.origin, EXCLUDED.origin), rp_id = COALESCE(rawrarr_nodes.rp_id, EXCLUDED.rp_id) WHERE rawrarr_nodes.role = EXCLUDED.role")
            .bind(&node.id).bind(role_name(node.role)).bind(node.token_sha256.to_lowercase()).bind(node.origin).bind(node.rp_id)
            .execute(&pool).await?;
        let _ = origin;
    }
    let rows = sqlx::query("SELECT id, role, token_sha256, origin, rp_id, signing_public_key FROM rawrarr_nodes WHERE token_sha256 IS NOT NULL")
        .fetch_all(&pool).await?;
    let mut approved = Vec::new();
    for row in rows {
        let id: String = row.try_get("id")?;
        let role = NodeRole::parse(&row.try_get::<String, _>("role")?)?;
        let digest: String = row.try_get("token_sha256")?;
        let origin_text: String = row.try_get("origin")?;
        let rp_id: String = row.try_get("rp_id")?;
        let signing_public_key: Option<Vec<u8>> = row.try_get("signing_public_key")?;
        let origin = validate_origin(&rp_id, &origin_text)?;
        let webauthn = webauthn_rs::prelude::WebauthnBuilder::new(&rp_id, &origin)?
            .rp_name("rawrarr")
            .build()?;
        approved.push(ApprovedAuth {
            digest,
            role,
            signing_public_key,
            auth: AuthState {
                pool: pool.clone(),
                webauthn,
                secure_cookie: origin.scheme() == "https",
                node_id: id,
                next_path: "/setup/age",
            },
        });
    }
    let targets_text = env::var("RAWARR_NODE_RELAY_TARGETS")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| r#"{"macbook-metadata":"http://fossil:8080"}"#.to_owned());
    let configured_targets: HashMap<String, String> = serde_json::from_str(&targets_text)
        .map_err(|_| "RAWARR_NODE_RELAY_TARGETS must be a JSON object of node IDs to base URLs")?;
    let mut relay_targets = HashMap::new();
    for (node_id, target) in configured_targets {
        if !valid_node_id(&node_id) {
            return Err("invalid node ID in relay targets".into());
        }
        let url = reqwest::Url::parse(&target)?;
        if !matches!(url.scheme(), "https" | "http")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("invalid node relay target URL".into());
        }
        relay_targets.insert(node_id, url);
    }
    let mut relay_client_builder = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::none());
    if let Ok(proxy_url) = env::var("RAWARR_NODE_RELAY_SOCKS5_URL") {
        let proxy = reqwest::Proxy::all(&proxy_url)?
            .no_proxy(reqwest::NoProxy::from_string("fossil"));
        relay_client_builder = relay_client_builder.proxy(proxy);
    }
    let relay_client = relay_client_builder.build()?;
    let direct_targets_text = env::var("RAWARR_NODE_DIRECT_TARGETS")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "{}".to_owned());
    let configured_direct_targets: HashMap<String, String> = serde_json::from_str(&direct_targets_text)
        .map_err(|_| "RAWARR_NODE_DIRECT_TARGETS must be a JSON object of node IDs to same-origin paths")?;
    let mut direct_targets = HashMap::new();
    for (node_id, target) in configured_direct_targets {
        if !valid_node_id(&node_id) {
            return Err("invalid node ID in direct targets".into());
        }
        let url = validate_direct_target(&web_origin.origin().ascii_serialization(), &target)?;
        direct_targets.insert(node_id, url);
    }
    let state = CoordinatorState {
        pool,
        approved: Arc::new(RwLock::new(approved)),
        webauthn,
        web_origin: web_origin_text.trim_end_matches('/').to_owned(),
        coordinator_auth,
        relay_targets,
        direct_targets,
        relay_client,
    };
    let public = Router::new()
        .route("/", get(coordinator_home))
        .route("/auth", get(coordinator_auth_page))
        .route("/health", get(coordinator_health))
        .route("/assets/rawrarr-logo.png", get(coordinator_rawrarr_logo))
        .route("/assets/rawrarr-logo.jxl", get(coordinator_rawrarr_logo_jxl))
        .route("/assets/coordinator-workspace.mjs", get(coordinator_workspace_script))
        .route("/assets/coordinator-availability.mjs", get(coordinator_availability_script))
        .route("/assets/coordinator-node-avatar.mjs", get(coordinator_node_avatar_script))
        .route("/assets/coordinator-ai.mjs", get(coordinator_ai_script))
        .route("/assets/coordinator-device-routes.mjs", get(coordinator_device_routes_script))
        .route("/assets/profile-color.mjs", get(coordinator_profile_color_script))
        .route("/assets/metadata-export.js", get(coordinator_metadata_export_js))
        .route("/assets/subscriber-onboarding.js", get(coordinator_subscriber_onboarding_script))
        .route("/assets/device-signing-client.js", get(coordinator_device_signing_script))
        .route("/nodes/{node_id}", any(node_proxy_root))
        .route("/nodes/{node_id}/", any(node_proxy_root))
        .route("/nodes/{node_id}/{*path}", any(node_proxy))
        .route("/passkeys/add", get(coordinator_passkey_page))
        .route(
            "/assets/passkey-client.mjs",
            get(coordinator_passkey_client),
        )
        .route(
            "/assets/coordinator-passkey.mjs",
            get(coordinator_passkey_script),
        )
        .route(
            "/api/passkeys/coordinator/register/start",
            post(start_coordinator_passkey),
        )
        .route(
            "/api/passkeys/coordinator/register/finish",
            post(finish_coordinator_passkey),
        )
        .route("/api/nodes/enroll", post(request_enrollment))
        .route("/api/nodes/enroll/status", post(enrollment_status))
        .route("/api/nodes/signing-root", get(instance_signing_root))
        .with_state(state.clone());
    let coordinator_browser = Router::new()
        .route("/setup/node", get(coordinator_subscriber_onboarding_page))
        .route("/api/auth/status", get(crate::auth::status))
        .route(
            "/api/auth/register/start",
            post(crate::auth::begin_registration),
        )
        .route(
            "/api/auth/register/finish",
            post(crate::auth::finish_registration),
        )
        .route(
            "/api/auth/login/start",
            post(crate::auth::begin_authentication),
        )
        .route(
            "/api/auth/login/finish",
            post(crate::auth::finish_authentication),
        )
        .route("/api/auth/session", get(session))
        .route("/api/auth/logout", post(logout))
        .route("/api/workspace/nodes", get(workspace_nodes))
        .route("/api/workspace/nodes/{node_id}", delete(delete_workspace_node))
        .route("/api/workspace/profile", get(workspace_profile))
        .route("/api/workspace/profile-picture/{seed}/jxl", get(workspace_profile_picture_jxl))
        .route("/api/workspace/profile-picture/{seed}/png", get(workspace_profile_picture_png))
        .route("/api/workspace/device-picture/{seed}/jxl", get(workspace_device_picture_jxl))
        .route("/api/workspace/device-picture/{seed}/png", get(workspace_device_picture_png))
        .route("/api/workspace/direct-session", post(direct_session))
        .route("/api/workspace/pending", get(pending_nodes))
        .route("/api/workspace/approve", post(approve_node))
        .route("/api/workspace/reject", post(reject_node))
        .route("/api/ai/status", get(ai::status))
        .route("/api/ai/requests", get(ai::list_requests).post(ai::submit))
        .route("/api/ai/transcribe", post(ai::transcribe))
        .layer(middleware::from_fn_with_state(
            state.coordinator_auth.clone(),
            coordinator_auth_context,
        ))
        .with_state(state.clone());
    let authenticated = Router::new()
        .route("/api/nodes/auth/status", get(crate::auth::status))
        .route(
            "/api/nodes/auth/register/start",
            post(crate::auth::begin_registration),
        )
        .route(
            "/api/nodes/auth/register/finish",
            post(crate::auth::finish_registration),
        )
        .route(
            "/api/nodes/auth/login/start",
            post(crate::auth::begin_authentication),
        )
        .route(
            "/api/nodes/auth/login/finish",
            post(crate::auth::finish_authentication),
        )
        .route("/api/nodes/auth/session", get(session))
        .route("/api/nodes/auth/logout", post(logout))
        .route("/api/nodes/register", post(register))
        .route("/api/nodes/signing-certificate", get(node_signing_certificate))
        .route("/api/nodes/pending", get(pending_nodes))
        .route("/api/nodes/approve", post(approve_node))
        .route(
            "/api/auth/passkeys/coordinator/link",
            post(create_coordinator_passkey_link),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            authorize_node,
        ))
        .with_state(state);
    Ok(public.merge(coordinator_browser).merge(authenticated))
}

async fn coordinator_auth_context(
    State(auth): State<AuthState>,
    mut request: Request,
    next: Next,
) -> Response {
    request.extensions_mut().insert(auth);
    next.run(request).await
}

async fn authorize_node(
    State(state): State<CoordinatorState>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let Some(token) = token.filter(|v| valid_digest(v)) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let digest = format!("{:x}", Sha256::digest(token.as_bytes()));
    let node = state
        .approved
        .read()
        .await
        .iter()
        .find(|node| node.digest == digest)
        .cloned();
    let Some(node) = node else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    if let Some(public_key) = node.signing_public_key.as_deref() {
        let (parts, body) = request.into_parts();
        let Ok(body) = to_bytes(body, 256 * 1024).await else {
            return StatusCode::PAYLOAD_TOO_LARGE.into_response();
        };
        let value = |name: &str| parts.headers.get(name).and_then(|header| header.to_str().ok());
        let Some(signed_node_id) = value("x-rawarr-node-id") else { return StatusCode::UNAUTHORIZED.into_response(); };
        let Some(timestamp_text) = value("x-rawarr-timestamp") else { return StatusCode::UNAUTHORIZED.into_response(); };
        let Some(nonce) = value("x-rawarr-nonce") else { return StatusCode::UNAUTHORIZED.into_response(); };
        let Some(signature_hex) = value("x-rawarr-signature") else { return StatusCode::UNAUTHORIZED.into_response(); };
        let Ok(timestamp) = timestamp_text.parse::<i64>() else { return StatusCode::UNAUTHORIZED.into_response(); };
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64;
        if signed_node_id != node.auth.node_id || now.abs_diff(timestamp) > 60 {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        let Ok(nonce_id) = Uuid::parse_str(nonce) else { return StatusCode::UNAUTHORIZED.into_response(); };
        let Some(signature) = decode_hex(signature_hex) else { return StatusCode::UNAUTHORIZED.into_response(); };
        let body_hash = Sha256::digest(&body).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let signed_payload = serde_json::json!({
            "domain": "rawrarr-coordination-request-v1",
            "nodeId": signed_node_id,
            "method": parts.method.as_str(),
            "path": parts.uri.path(),
            "bodyHash": body_hash,
            "timestamp": timestamp,
            "nonce": nonce,
        });
        let Ok(signed_bytes) = crate::device_signing::canonical_json(&signed_payload) else {
            return StatusCode::BAD_REQUEST.into_response();
        };
        if !crate::device_signing::verify(public_key, &signature, &signed_bytes) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        let inserted = sqlx::query_scalar::<_, Uuid>("INSERT INTO rawrarr_node_request_nonces(node_id, nonce, expires_at) VALUES ($1,$2,now() + interval '2 minutes') ON CONFLICT(node_id, nonce) DO NOTHING RETURNING nonce")
            .bind(&node.auth.node_id).bind(nonce_id).fetch_optional(&node.auth.pool).await;
        if !matches!(inserted, Ok(Some(_))) { return StatusCode::UNAUTHORIZED.into_response(); }
        let _ = sqlx::query("DELETE FROM rawrarr_node_request_nonces WHERE expires_at < now()")
            .execute(&node.auth.pool).await;
        request = Request::from_parts(parts, Body::from(body));
    }
    request.extensions_mut().insert(node.auth);
    request.extensions_mut().insert(node.role);
    next.run(request).await
}

async fn request_enrollment(
    State(state): State<CoordinatorState>,
    headers: HeaderMap,
    Json(request): Json<EnrollmentRequest>,
) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    if request.role == NodeRole::Coordinator
        || !valid_node_id(&request.node_id)
        || request.display_name.trim().is_empty()
        || request.display_name.chars().count() > 64
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if validate_origin(&request.rp_id, &request.origin).is_err() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let Some(signing_public_key) = decode_hex(&request.signing_public_key_hex) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(key_proof_signature) = decode_hex(&request.key_proof_signature_hex) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let proof_payload = serde_json::json!({
        "domain": "rawrarr-node-enrollment-proof-v1",
        "nodeId": request.node_id,
        "role": request.role,
        "publicKeyHex": request.signing_public_key_hex,
    });
    let Ok(proof_message) = crate::device_signing::canonical_json(&proof_payload) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if signing_public_key.len() != 1952 || key_proof_signature.len() != 3309
        || !crate::device_signing::verify(&signing_public_key, &key_proof_signature, &proof_message)
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let digest = format!("{:x}", Sha256::digest(token.as_bytes()));
    let request_id = Uuid::new_v4();
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    // A new node does not choose its user. The coordinator assigns enrollment
    // to the sole account on this instance; multi-account instances must use
    // an explicit invitation flow before this assumption can be relaxed.
    let owners = match sqlx::query("SELECT id FROM rawrarr_users ORDER BY created_at")
        .fetch_all(&mut *tx)
        .await
    {
        Ok(rows) if rows.len() == 1 => rows,
        Ok(_) => return StatusCode::CONFLICT.into_response(),
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let owner_user_id: Uuid = match owners[0].try_get("id") {
        Ok(id) => id,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let approved =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_nodes WHERE id = $1 AND (token_sha256 IS DISTINCT FROM $2 OR role <> $3))")
            .bind(&request.node_id)
            .bind(&digest)
            .bind(role_name(request.role))
            .fetch_one(&mut *tx)
            .await
            .unwrap_or(true);
    if approved {
        return StatusCode::CONFLICT.into_response();
    }
    let _ = sqlx::query(
        "DELETE FROM rawrarr_node_enrollments WHERE expires_at <= now() AND approved_at IS NULL",
    )
    .execute(&mut *tx)
    .await;
    let count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM rawrarr_node_enrollments WHERE approved_at IS NULL AND expires_at > now()")
        .fetch_one(&mut *tx).await.unwrap_or(50);
    if count >= 50 {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    }
    let row = sqlx::query("INSERT INTO rawrarr_node_enrollments(request_id, node_id, role, display_name, token_sha256, origin, rp_id, expires_at, user_id, signing_public_key, key_proof_signature) VALUES ($1,$2,$3,$4,$5,$6,$7,now() + interval '15 minutes',$8,$9,$10) ON CONFLICT(node_id) DO UPDATE SET request_id=EXCLUDED.request_id, role=EXCLUDED.role, display_name=EXCLUDED.display_name, token_sha256=EXCLUDED.token_sha256, origin=EXCLUDED.origin, rp_id=EXCLUDED.rp_id, requested_at=now(), expires_at=EXCLUDED.expires_at, user_id=EXCLUDED.user_id, signing_public_key=EXCLUDED.signing_public_key, key_proof_signature=EXCLUDED.key_proof_signature WHERE rawrarr_node_enrollments.approved_at IS NULL RETURNING request_id")
        .bind(request_id).bind(&request.node_id).bind(role_name(request.role)).bind(request.display_name.trim())
        .bind(digest).bind(&request.origin).bind(&request.rp_id).bind(owner_user_id).bind(&signing_public_key).bind(&key_proof_signature)
        .fetch_optional(&mut *tx).await;
    match row {
        Ok(Some(row)) => match tx.commit().await { Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({"requestId": row.try_get::<Uuid, _>("request_id").unwrap_or(request_id)}))).into_response(), Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response() },
        Ok(None) => StatusCode::CONFLICT.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn instance_signing_root(State(state): State<CoordinatorState>) -> Response {
    let row = sqlx::query("SELECT public_key, fingerprint FROM rawrarr_instance_signing_root WHERE singleton=TRUE")
        .fetch_optional(&state.pool).await;
    match row {
        Ok(Some(row)) => {
            let Ok(public_key) = row.try_get::<Vec<u8>, _>("public_key") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            let Ok(fingerprint) = row.try_get::<String, _>("fingerprint") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            Json(InstanceSigningRoot { algorithm: "ML-DSA-65", public_key: BASE64.encode(public_key), fingerprint }).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn enrollment_status(
    State(state): State<CoordinatorState>,
    headers: HeaderMap,
    Json(request): Json<EnrollmentStatusRequest>,
) -> Response {
    let Some(token) = bearer_token(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let digest = format!("{:x}", Sha256::digest(token.as_bytes()));
    let row = sqlx::query("SELECT e.approved_at IS NOT NULL AS approved, n.id AS node_id FROM rawrarr_node_enrollments e LEFT JOIN rawrarr_nodes n ON n.token_sha256=e.token_sha256 WHERE e.request_id=$1 AND e.token_sha256=$2 AND e.expires_at > now()")
        .bind(request.request_id).bind(digest).fetch_optional(&state.pool).await;
    match row {
        Ok(Some(row)) => {
            let approved: bool = row.try_get("approved").unwrap_or(false);
            let node_id: Option<String> = row.try_get("node_id").ok();
            Json(EnrollmentStatus {
                status: if approved { "approved" } else { "pending" },
                node_id: if approved { node_id } else { None },
            })
            .into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn pending_nodes(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
) -> Response {
    let Some(owner_user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let rows = match sqlx::query("SELECT e.request_id, COALESCE(n.id,e.node_id) AS node_id, e.node_id AS requested_node_id, e.role, e.display_name, e.origin, e.requested_at::text AS requested_at, e.signing_public_key, u.display_name AS owner_display_name FROM rawrarr_node_enrollments e JOIN rawrarr_users u ON u.id=e.user_id LEFT JOIN rawrarr_nodes n ON n.token_sha256=e.token_sha256 WHERE e.user_id=$1 AND e.approved_at IS NULL AND e.expires_at > now() ORDER BY e.requested_at")
        .bind(owner_user_id).fetch_all(&state.pool).await { Ok(rows) => rows, Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response() };
    let mut pending = Vec::with_capacity(rows.len());
    for row in rows {
        let role_text: String = row.try_get("role").unwrap_or_default();
        let Ok(role) = NodeRole::parse(&role_text) else {
            continue;
        };
        pending.push(PendingNode {
            request_id: row.try_get("request_id").unwrap_or_else(|_| Uuid::nil()),
            node_id: row.try_get("node_id").unwrap_or_default(),
            requested_node_id: row.try_get("requested_node_id").unwrap_or_default(),
            role,
            display_name: row.try_get("display_name").unwrap_or_default(),
            owner_display_name: row.try_get("owner_display_name").unwrap_or_default(),
            origin: row.try_get("origin").unwrap_or_default(),
            requested_at: row.try_get("requested_at").unwrap_or_default(),
            signing_public_key_hex: row.try_get::<Vec<u8>, _>("signing_public_key").map(|bytes| encode_hex(&bytes)).unwrap_or_default(),
        });
    }
    Json(pending).into_response()
}

async fn approve_node(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    Json(request): Json<ApproveRequest>,
) -> Response {
    let Some(owner_user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let row = sqlx::query("SELECT node_id, role, display_name, token_sha256, origin, rp_id, user_id, signing_public_key FROM rawrarr_node_enrollments WHERE request_id=$1 AND user_id=$2 AND approved_at IS NULL AND expires_at > now() FOR UPDATE")
        .bind(request.request_id).bind(owner_user_id).fetch_optional(&mut *tx).await;
    let Ok(Some(row)) = row else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let node_id: String = match row.try_get("node_id") {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let role_text: String = match row.try_get("role") {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let Ok(role) = NodeRole::parse(&role_text) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let token_sha256: String = match row.try_get("token_sha256") {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let display_name: String = match row.try_get("display_name") {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let origin_text: String = match row.try_get("origin") {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let rp_id: String = match row.try_get("rp_id") {
        Ok(v) => v,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let origin = match validate_origin(&rp_id, &origin_text) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let row_owner_user_id: Uuid = match row.try_get("user_id") {
        Ok(value) if value == owner_user_id => value,
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let signing_public_key: Vec<u8> = match row.try_get::<Vec<u8>, _>("signing_public_key") {
        Ok(value) if value.len() == 1952 => value,
        _ => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let canonical_node_id = match sqlx::query_scalar::<_, String>("SELECT id FROM rawrarr_nodes WHERE token_sha256=$1")
        .bind(&token_sha256).fetch_optional(&mut *tx).await {
        Ok(Some(id)) => id,
        Ok(None) => node_id.clone(),
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let expected_payload = serde_json::json!({
        "domain": "rawrarr-device-approval-v1",
        "requestId": request.request_id.to_string(),
        "nodeId": canonical_node_id,
        "role": role_name(role),
        "displayName": display_name,
        "publicKeyHex": encode_hex(&signing_public_key),
    });
    if request.signed_payload != expected_payload {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let Ok(root_public_key) = BASE64.decode(&request.root_public_key) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Ok(root_signature) = BASE64.decode(&request.signature) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if root_public_key.len() != 1952
        || crate::device_signing::fingerprint(&root_public_key) != request.root_fingerprint
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let Ok(approval_bytes) = crate::device_signing::canonical_json(&request.signed_payload) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if !crate::device_signing::verify(&root_public_key, &root_signature, &approval_bytes) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let pinned_root = sqlx::query("SELECT public_key, fingerprint, owner_user_id FROM rawrarr_instance_signing_root WHERE singleton=TRUE FOR UPDATE")
        .fetch_optional(&mut *tx).await;
    let pinned_root = match pinned_root {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if let Some(pinned_root) = pinned_root {
        let stored_key: Vec<u8> = match pinned_root.try_get("public_key") {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let stored_fingerprint: String = match pinned_root.try_get("fingerprint") {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let stored_owner: Uuid = match pinned_root.try_get("owner_user_id") {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        if stored_key != root_public_key || stored_fingerprint != request.root_fingerprint || stored_owner != owner_user_id {
            return StatusCode::FORBIDDEN.into_response();
        }
    } else if sqlx::query("INSERT INTO rawrarr_instance_signing_root(singleton, algorithm, public_key, fingerprint, owner_user_id) VALUES (TRUE, 'ML-DSA-65', $1, $2, $3) ON CONFLICT(singleton) DO NOTHING")
        .bind(&root_public_key).bind(&request.root_fingerprint).bind(owner_user_id)
        .execute(&mut *tx).await.is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let root_check = sqlx::query("SELECT public_key, fingerprint, owner_user_id FROM rawrarr_instance_signing_root WHERE singleton=TRUE FOR UPDATE")
        .fetch_optional(&mut *tx).await;
    let Ok(Some(root_check)) = root_check else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
    let checked_key: Vec<u8> = match root_check.try_get("public_key") { Ok(value) => value, Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response() };
    let checked_fingerprint: String = match root_check.try_get("fingerprint") { Ok(value) => value, Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response() };
    let checked_owner: Uuid = match root_check.try_get("owner_user_id") { Ok(value) => value, Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response() };
    if checked_key != root_public_key || checked_fingerprint != request.root_fingerprint || checked_owner != owner_user_id {
        return StatusCode::FORBIDDEN.into_response();
    }
    // A subscriber may retain a token from an earlier node ID while its
    // Compose configuration is regenerated. In that case the credential is
    // already loaded for its original identity; approving this request should
    // refresh that identity's public name rather than collide with the unique
    // token index and leave the button appearing inert.
    let existing = sqlx::query("SELECT id, role, user_id, signing_public_key FROM rawrarr_nodes WHERE token_sha256=$1")
        .bind(&token_sha256)
        .fetch_optional(&mut *tx)
        .await;
    let existing = match existing {
        Ok(existing) => existing,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    if let Some(existing) = existing {
        let existing_role: String = match existing.try_get("role") {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let existing_user_id: Uuid = match existing.try_get("user_id") {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        if existing_role != role_name(role) || existing_user_id != owner_user_id {
            return StatusCode::CONFLICT.into_response();
        }
        let existing_node_id: String = match existing.try_get("id") {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let existing_public_key: Option<Vec<u8>> = match existing.try_get("signing_public_key") {
            Ok(value) => value,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        if existing_public_key.as_ref().is_some_and(|key| key != &signing_public_key) {
            return StatusCode::CONFLICT.into_response();
        }
        if sqlx::query("UPDATE rawrarr_nodes SET signing_public_key=$2, signing_fingerprint=$3, signing_approval_payload=$4, signing_approval_signature=$5 WHERE id=$1 AND signing_public_key IS NULL")
            .bind(&existing_node_id).bind(&signing_public_key).bind(crate::device_signing::node_fingerprint(&signing_public_key))
            .bind(&request.signed_payload).bind(&root_signature)
            .execute(&mut *tx).await.is_err()
        {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        if sqlx::query("UPDATE rawrarr_node_enrollments SET display_name=$2 WHERE node_id=$1 AND user_id=$3 AND approved_at IS NOT NULL")
            .bind(existing_node_id)
            .bind(display_name)
            .bind(owner_user_id)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        if sqlx::query("UPDATE rawrarr_node_enrollments SET approved_at=now() WHERE request_id=$1")
            .bind(request.request_id)
            .execute(&mut *tx)
            .await
            .is_err()
        {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        return match tx.commit().await {
            Ok(()) => {
                for entry in state.approved.write().await.iter_mut().filter(|entry| entry.digest == token_sha256) {
                    entry.signing_public_key = Some(signing_public_key.clone());
                }
                StatusCode::NO_CONTENT.into_response()
            }
            Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        };
    }
    let inserted = sqlx::query("INSERT INTO rawrarr_nodes(id, role, token_sha256, origin, rp_id, user_id, signing_public_key, signing_fingerprint, signing_approval_payload, signing_approval_signature) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10) ON CONFLICT(id) DO NOTHING")
        .bind(&node_id).bind(role_name(role)).bind(&token_sha256).bind(&origin_text).bind(&rp_id).bind(row_owner_user_id)
        .bind(&signing_public_key).bind(crate::device_signing::node_fingerprint(&signing_public_key))
        .bind(&request.signed_payload).bind(&root_signature).execute(&mut *tx).await;
    if !matches!(inserted, Ok(ref result) if result.rows_affected() == 1) {
        return StatusCode::CONFLICT.into_response();
    }
    if sqlx::query("UPDATE rawrarr_node_enrollments SET approved_at=now() WHERE request_id=$1")
        .bind(request.request_id)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if tx.commit().await.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let webauthn = match webauthn_rs::prelude::WebauthnBuilder::new(&rp_id, &origin)
        .and_then(|builder| builder.rp_name("rawrarr").build())
    {
        Ok(webauthn) => webauthn,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    state.approved.write().await.push(ApprovedAuth {
        digest: token_sha256,
        role,
        signing_public_key: Some(signing_public_key),
        auth: AuthState {
            pool: state.pool.clone(),
            webauthn,
            secure_cookie: origin.scheme() == "https",
            node_id,
            next_path: "/setup/age",
        },
    });
    StatusCode::NO_CONTENT.into_response()
}

async fn reject_node(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    Json(request): Json<ApproveRequest>,
) -> StatusCode {
    let Some(owner_user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED;
    };
    match sqlx::query("DELETE FROM rawrarr_node_enrollments WHERE request_id=$1 AND user_id=$2 AND approved_at IS NULL")
        .bind(request.request_id).bind(owner_user_id).execute(&state.pool).await
    {
        Ok(result) if result.rows_affected() == 1 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

async fn delete_workspace_node(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    Path(node_id): Path<String>,
) -> StatusCode {
    let Some(owner_user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED;
    };
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE,
    };
    let node = match sqlx::query("DELETE FROM rawrarr_nodes WHERE id=$1 AND user_id=$2 RETURNING token_sha256")
        .bind(&node_id).bind(owner_user_id).fetch_optional(&mut *tx).await
    {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::NOT_FOUND,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE,
    };
    let token_sha256: Option<String> = match node.try_get("token_sha256") {
        Ok(value) => value,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if let Some(token_sha256) = token_sha256 {
        if sqlx::query("INSERT INTO rawrarr_revoked_nodes(node_id, token_sha256) VALUES ($1,$2) ON CONFLICT(node_id) DO UPDATE SET token_sha256=EXCLUDED.token_sha256, revoked_at=now()")
            .bind(&node_id).bind(token_sha256).execute(&mut *tx).await.is_err()
        {
            return StatusCode::SERVICE_UNAVAILABLE;
        }
    }
    if sqlx::query("DELETE FROM rawrarr_sessions WHERE node_id=$1")
        .bind(&node_id).execute(&mut *tx).await.is_err()
        || sqlx::query("DELETE FROM rawrarr_node_enrollments WHERE node_id=$1 AND user_id=$2")
            .bind(&node_id).bind(owner_user_id).execute(&mut *tx).await.is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    if tx.commit().await.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    state.approved.write().await.retain(|entry| entry.auth.node_id != node_id);
    StatusCode::NO_CONTENT
}

async fn session(Extension(auth): Extension<AuthState>, headers: HeaderMap) -> StatusCode {
    if crate::auth::is_authenticated(&auth, &headers).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::UNAUTHORIZED
    }
}
async fn logout(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
) -> Response {
    if auth.node_id == "coordinator" {
        if let Some(user_id) = crate::auth::authenticated_user_id(&auth, &headers).await {
            let _ = sqlx::query("DELETE FROM rawrarr_sessions WHERE user_id=$1 AND node_id <> 'coordinator'")
                .bind(user_id)
                .execute(&state.pool)
                .await;
        }
    }
    let mut response = crate::auth::end_session(&auth, &headers).await;
    if auth.node_id == "coordinator" {
        for target in state.direct_targets.values() {
            let cookie = format!(
                "rawrarr_direct_session=; Path={}; HttpOnly; Secure; SameSite=Strict; Max-Age=0",
                target.path()
            );
            if let Ok(value) = HeaderValue::from_str(&cookie) {
                response.headers_mut().append(header::SET_COOKIE, value);
            }
        }
    }
    response
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Registration {
    role: NodeRole,
    #[serde(default)]
    display_name: Option<String>,
}
async fn register(
    Extension(auth): Extension<AuthState>,
    Extension(role): Extension<NodeRole>,
    headers: HeaderMap,
    Json(request): Json<Registration>,
) -> StatusCode {
    if request.role != role {
        return StatusCode::FORBIDDEN;
    }
    let display_name = request.display_name.map(|name| name.trim().to_owned());
    if display_name.as_ref().is_some_and(|name| {
        name.is_empty() || name.chars().count() > 64 || name.chars().any(char::is_control)
    }) {
        return StatusCode::BAD_REQUEST;
    }
    let has_signing_key = sqlx::query_scalar::<_, bool>("SELECT signing_public_key IS NOT NULL FROM rawrarr_nodes WHERE id=$1")
        .bind(&auth.node_id).fetch_optional(&auth.pool).await;
    match has_signing_key {
        Ok(Some(false)) if headers.contains_key("x-rawarr-signature") => return StatusCode::PRECONDITION_REQUIRED,
        Ok(Some(_)) => {},
        _ => return StatusCode::SERVICE_UNAVAILABLE,
    }
    if let Some(display_name) = display_name {
        if sqlx::query("UPDATE rawrarr_node_enrollments SET display_name=$2 WHERE node_id=$1 AND approved_at IS NOT NULL")
            .bind(&auth.node_id)
            .bind(display_name)
            .execute(&auth.pool)
            .await
            .is_err()
        {
            return StatusCode::SERVICE_UNAVAILABLE;
        }
    }
    match sqlx::query("UPDATE rawrarr_nodes SET last_seen=now() WHERE id=$1 AND role=$2")
        .bind(&auth.node_id)
        .bind(role_name(role))
        .execute(&auth.pool)
        .await
    {
        Ok(result) if result.rows_affected() == 1 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::FORBIDDEN,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

async fn node_signing_certificate(Extension(auth): Extension<AuthState>) -> Response {
    let row = sqlx::query("SELECT n.id, n.role, n.signing_public_key, n.signing_fingerprint, n.signing_approval_payload, n.signing_approval_signature, r.public_key AS root_public_key, r.fingerprint AS root_fingerprint FROM rawrarr_nodes n JOIN rawrarr_instance_signing_root r ON r.singleton=TRUE WHERE n.id=$1 AND n.signing_public_key IS NOT NULL")
        .bind(&auth.node_id).fetch_optional(&auth.pool).await;
    match row {
        Ok(Some(row)) => {
            let Ok(node_id) = row.try_get::<String, _>("id") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            let Ok(role) = row.try_get::<String, _>("role") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            let Ok(public_key) = row.try_get::<Vec<u8>, _>("signing_public_key") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            let Ok(fingerprint) = row.try_get::<String, _>("signing_fingerprint") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            let Ok(signed_payload) = row.try_get::<serde_json::Value, _>("signing_approval_payload") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            let Ok(signature) = row.try_get::<Vec<u8>, _>("signing_approval_signature") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            let Ok(root_public_key) = row.try_get::<Vec<u8>, _>("root_public_key") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            let Ok(root_fingerprint) = row.try_get::<String, _>("root_fingerprint") else { return StatusCode::SERVICE_UNAVAILABLE.into_response(); };
            Json(SigningCertificate {
                algorithm: "ML-DSA-65",
                root_public_key: BASE64.encode(root_public_key),
                root_fingerprint,
                node_id,
                role,
                signing_public_key_hex: encode_hex(&public_key),
                signing_fingerprint: fingerprint,
                signed_payload,
                signature: BASE64.encode(signature),
            }).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| valid_digest(token))
}
fn valid_node_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn decode_hex(value: &str) -> Option<Vec<u8>> {
    if value.len() % 2 != 0 { return None; }
    value.as_bytes().chunks_exact(2).map(|pair| {
        let digit = |byte: u8| match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        };
        Some(digit(pair[0])? * 16 + digit(pair[1])?)
    }).collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 15) as usize] as char);
    }
    result
}

fn validate_direct_target(origin: &str, target: &str) -> Result<reqwest::Url, &'static str> {
    let url = reqwest::Url::parse(target).map_err(|_| "invalid direct route URL")?;
    let safe_path = url.path().starts_with('/')
        && url.path().ends_with('/')
        && url.path().split('/').skip(1).all(|component| {
            component
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        });
    let tailnet_origin = url.scheme() == "https"
        && url.host_str().is_some_and(|host| {
            host.ends_with(".risk-mermaid.ts.net")
                && (host == "rawrarr.risk-mermaid.ts.net"
                    || host.starts_with("rawrarr-"))
        })
        && url.port().is_none();
    if url.scheme() != "https"
        || (url.origin().ascii_serialization() != origin && !tailnet_origin)
        || target.contains("..")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !safe_path
    {
        return Err("direct targets must be same-origin or tailnet HTTPS paths ending in /");
    }
    Ok(url)
}
fn role_name(role: NodeRole) -> &'static str {
    match role {
        NodeRole::Storage => "storage",
        NodeRole::Metadata => "metadata",
        NodeRole::Coordinator => "coordinator",
    }
}
fn validate_origin(rp_id: &str, value: &str) -> Result<url::Url, &'static str> {
    let origin = url::Url::parse(value).map_err(|_| "invalid node origin")?;
    let host = origin.host_str().ok_or("invalid node origin")?;
    if (origin.scheme() != "https" && !(origin.scheme() == "http" && host == "localhost"))
        || origin.username() != ""
        || origin.password().is_some()
        || origin.query().is_some()
        || origin.fragment().is_some()
        || origin.path() != "/"
        || !(host == rp_id || host.ends_with(&format!(".{rp_id}")))
    {
        return Err(
            "node origin and RP ID must be a valid HTTPS origin (localhost development excepted)",
        );
    }
    Ok(origin)
}

#[cfg(test)]
mod approved_node_registry_tests {
    use super::parse_approved_nodes;

    #[test]
    fn empty_registry_is_valid_before_the_owner_adds_a_node() {
        assert!(parse_approved_nodes("[]").unwrap().is_empty());
    }

    #[test]
    fn malformed_registry_is_still_rejected() {
        assert!(parse_approved_nodes("{}").is_err());
    }
}

#[cfg(test)]
mod direct_route_tests {
    use super::{storage_scan_relay_allowed, validate_direct_target};
    use axum::http::Method;

    #[test]
    fn direct_routes_must_be_safe_paths_on_the_coordinator_https_origin() {
        let origin = "https://rawrarr.risk-mermaid.ts.net";
        assert!(validate_direct_target(origin, "https://rawrarr.risk-mermaid.ts.net/direct-api/").is_ok());
        assert!(validate_direct_target(origin, "https://rawrarr-mini.risk-mermaid.ts.net/direct-api/").is_ok());
        for target in [
            "http://rawrarr.risk-mermaid.ts.net/direct-api/",
            "https://other.risk-mermaid.ts.net/direct-api/",
            "https://not-tailnet.example/direct-api/",
            "https://rawrarr.risk-mermaid.ts.net/direct-api?x=1",
            "https://rawrarr.risk-mermaid.ts.net/direct-api/../api/",
            "https://rawrarr.risk-mermaid.ts.net/direct-api%2f/",
        ] {
            assert!(validate_direct_target(origin, target).is_err(), "accepted {target}");
        }
    }

    #[test]
    fn only_owner_authenticated_storage_scan_requests_can_use_the_relay() {
        let body = br#"{"kind":"movie","maxFiles":3}"#;
        assert!(storage_scan_relay_allowed(
            "storage",
            "api/library/scan/missing",
            &Method::POST,
            body
        ));
        assert!(storage_scan_relay_allowed(
            "storage",
            "api/library/scan/status",
            &Method::GET,
            b""
        ));
        assert!(storage_scan_relay_allowed(
            "storage",
            "api/library/scan/cancel",
            &Method::POST,
            b"{}"
        ));
        assert!(storage_scan_relay_allowed(
            "storage",
            "api/library/scan/schedule",
            &Method::GET,
            b""
        ));
        assert!(storage_scan_relay_allowed(
            "storage",
            "api/library/scan/schedule",
            &Method::PUT,
            br#"{"frequency":"daily","time":"04:00","timeZone":"America/New_York","weekday":null}"#
        ));
        assert!(!storage_scan_relay_allowed(
            "metadata",
            "api/library/scan/missing",
            &Method::POST,
            body
        ));
        assert!(!storage_scan_relay_allowed(
            "storage",
            "api/library/scan/missing",
            &Method::GET,
            body
        ));
        assert!(!storage_scan_relay_allowed(
            "storage",
            "api/library/scan/missing",
            &Method::POST,
            br#"{"unknown":true}"#
        ));
    }
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_node_enrollment.rs"]
mod tests;

async fn coordinator_health() -> axum::response::Html<String> {
    axum::response::Html(crate::ui_assets::text("coordinator_health.html", include_str!("coordinator_health.html")).await)
}

async fn coordinator_home(
    State(state): State<CoordinatorState>,
    headers: HeaderMap,
) -> Response {
    let mut response = if crate::auth::is_authenticated(&state.coordinator_auth, &headers).await {
        axum::response::Html(crate::ui_assets::text("coordinator_workspace.html", include_str!("coordinator_workspace.html")).await).into_response()
    } else {
        axum::response::Html(crate::ui_assets::text("landing.html", include_str!("landing.html")).await).into_response()
    };
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn coordinator_auth_page(
    State(state): State<CoordinatorState>,
    headers: HeaderMap,
) -> Response {
    if crate::auth::is_authenticated(&state.coordinator_auth, &headers).await {
        axum::response::Redirect::to("/").into_response()
    } else {
        crate::auth::auth_html().await.into_response()
    }
}

async fn coordinator_subscriber_onboarding_page(
    State(state): State<CoordinatorState>,
    headers: HeaderMap,
) -> Response {
    if !crate::auth::is_authenticated(&state.coordinator_auth, &headers).await {
        return axum::response::Redirect::to("/auth").into_response();
    }
    let mut response = axum::response::Html(crate::ui_assets::text("subscriber_onboarding.html", include_str!("subscriber_onboarding.html")).await)
        .into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn coordinator_subscriber_onboarding_script() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::text("assets/subscriber-onboarding.js", include_str!("assets/subscriber-onboarding.js")).await));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/javascript; charset=utf-8"),
    );
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn coordinator_device_signing_script() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::text(
        "assets/device-signing-client.js",
        include_str!("assets/device-signing-client.js"),
    ).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceNode {
    node_id: String,
    display_name: String,
    role: NodeRole,
    last_seen: String,
    online: bool,
    relay_available: bool,
    direct_url: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceProfile {
    account_id: Uuid,
    display_name: String,
    trial_started_at: Option<String>,
    trial_ends_at: Option<String>,
}

async fn workspace_profile(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
) -> Response {
    let Some(owner) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    match sqlx::query("SELECT id, to_char(trial_started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS trial_started_at, to_char((trial_started_at + interval '3 days') AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS trial_ends_at FROM rawrarr_users WHERE id=$1")
        .bind(owner).fetch_optional(&state.pool).await {
        Ok(Some(row)) => {
            let user_id: Uuid = match row.try_get("id") {
                Ok(user_id) => user_id,
                Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
            };
            let trial_enabled = std::env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public");
            let trial_started_at: Option<String> = match row.try_get("trial_started_at") {
                Ok(value) => value,
                Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
            };
            let trial_ends_at: Option<String> = match row.try_get("trial_ends_at") {
                Ok(value) => value,
                Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
            };
            axum::Json(WorkspaceProfile {
                account_id: user_id,
                display_name: crate::auth::generated_account_name(user_id),
                trial_started_at: trial_enabled.then_some(trial_started_at).flatten(),
                trial_ends_at: trial_enabled.then_some(trial_ends_at).flatten(),
            }).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn workspace_profile_picture_jxl(
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    Path(seed): Path<String>,
) -> Response {
    if crate::auth::authenticated_user_id(&auth, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !crate::coordinator_avatar::valid_seed(&seed) { return StatusCode::BAD_REQUEST.into_response(); }
    let encoded = match tokio::task::spawn_blocking(move || crate::coordinator_avatar::render_jxl(&seed)).await {
        Ok(Ok(bytes)) => bytes,
        _ => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    workspace_profile_picture_response(encoded, "image/jxl")
}

async fn workspace_profile_picture_png(
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    Path(seed): Path<String>,
) -> Response {
    if crate::auth::authenticated_user_id(&auth, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !crate::coordinator_avatar::valid_seed(&seed) { return StatusCode::BAD_REQUEST.into_response(); }
    let encoded = match tokio::task::spawn_blocking(move || crate::coordinator_avatar::render_png(&seed)).await {
        Ok(Ok(bytes)) => bytes,
        _ => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    workspace_profile_picture_response(encoded, "image/png")
}

async fn workspace_device_picture_jxl(
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    Path(seed): Path<String>,
) -> Response {
    if crate::auth::authenticated_user_id(&auth, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !crate::coordinator_avatar::valid_seed(&seed) { return StatusCode::BAD_REQUEST.into_response(); }
    let encoded = match tokio::task::spawn_blocking(move || crate::coordinator_avatar::render_fossil_jxl(&seed)).await {
        Ok(Ok(bytes)) => bytes,
        _ => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    workspace_profile_picture_response(encoded, "image/jxl")
}

async fn workspace_device_picture_png(
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    Path(seed): Path<String>,
) -> Response {
    if crate::auth::authenticated_user_id(&auth, &headers).await.is_none() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !crate::coordinator_avatar::valid_seed(&seed) { return StatusCode::BAD_REQUEST.into_response(); }
    let encoded = match tokio::task::spawn_blocking(move || crate::coordinator_avatar::render_fossil_png(&seed)).await {
        Ok(Ok(bytes)) => bytes,
        _ => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    workspace_profile_picture_response(encoded, "image/png")
}

fn workspace_profile_picture_response(encoded: Vec<u8>, content_type: &'static str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "private, max-age=31536000, immutable")
        .header(header::X_CONTENT_TYPE_OPTIONS, "nosniff")
        .body(Body::from(encoded))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

async fn workspace_nodes(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
) -> Response {
    let Some(owner) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let rows = match sqlx::query("SELECT n.id, n.role, n.last_seen::text AS last_seen, n.last_seen > now() - interval '90 seconds' AS online, n.origin, COALESCE(e.display_name, n.id) AS display_name FROM rawrarr_nodes n LEFT JOIN rawrarr_node_enrollments e ON e.node_id=n.id AND e.user_id=n.user_id WHERE n.user_id=$1 ORDER BY CASE n.role WHEN 'metadata' THEN 0 WHEN 'storage' THEN 1 ELSE 2 END, n.id")
        .bind(owner).fetch_all(&state.pool).await {
            Ok(rows) => rows,
            Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
        };
    let nodes = rows.into_iter().filter_map(|row| {
        let node_id: String = row.try_get("id").ok()?;
        let role = NodeRole::parse(&row.try_get::<String, _>("role").ok()?).ok()?;
        let origin: String = row.try_get("origin").unwrap_or_default();
        let direct_url = state.direct_targets.get(&node_id).map(url::Url::to_string).or_else(|| {
            let mut parsed = url::Url::parse(&origin).ok()?;
            if parsed.host_str() != Some("localhost") || parsed.port() != Some(7297) {
                return None;
            }
            parsed.set_port(Some(7298)).ok()?;
            Some(parsed.to_string())
        });
        Some(WorkspaceNode {
            display_name: row.try_get("display_name").unwrap_or_else(|_| node_id.clone()),
            relay_available: state.relay_targets.contains_key(&node_id),
            direct_url,
            node_id,
            role,
            last_seen: row.try_get("last_seen").unwrap_or_default(),
            online: row.try_get("online").unwrap_or(false),
        })
    }).collect::<Vec<_>>();
    Json(nodes).into_response()
}

const DIRECT_SESSION_SECONDS: i64 = 300;

async fn direct_session(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    Json(request): Json<DirectSessionRequest>,
) -> Response {
    if headers.get(header::ORIGIN).and_then(|value| value.to_str().ok()) != Some(state.web_origin.as_str()) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some(owner_user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some(target) = state.direct_targets.get(&request.node_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let role = sqlx::query_scalar::<_, String>("SELECT role FROM rawrarr_nodes WHERE id=$1 AND user_id=$2")
        .bind(&request.node_id)
        .bind(owner_user_id)
        .fetch_optional(&state.pool)
        .await;
    match role {
        Ok(Some(role)) if role == "metadata" || role == "storage" => {}
        Ok(Some(_)) => return StatusCode::FORBIDDEN.into_response(),
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
    let mut bytes = [0u8; 32];
    if getrandom::fill(&mut bytes).is_err() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let token = bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let token_hash = Sha256::digest(token.as_bytes()).to_vec();
    if sqlx::query("DELETE FROM rawrarr_sessions WHERE expires_at <= now()")
        .execute(&state.pool).await.is_err()
        || sqlx::query("INSERT INTO rawrarr_sessions(token_hash,user_id,expires_at,node_id) VALUES ($1,$2,now()+($3 * interval '1 second'),$4)")
            .bind(token_hash).bind(owner_user_id).bind(DIRECT_SESSION_SECONDS).bind(&request.node_id)
            .execute(&state.pool).await.is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let cookie = format!("rawrarr_direct_session={token}; Path={}; HttpOnly; Secure; SameSite=Strict; Max-Age={DIRECT_SESSION_SECONDS}", target.path());
    let mut response = Json(serde_json::json!({ "token": token })).into_response();
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    if target.origin().ascii_serialization() == state.web_origin {
        if let Ok(value) = HeaderValue::from_str(&cookie) {
            response.headers_mut().insert(header::SET_COOKIE, value);
        } else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    response
}

async fn coordinator_workspace_script() -> Response {
    let mut response = axum::response::Response::new(Body::from(crate::ui_assets::text("coordinator-workspace.mjs", include_str!("coordinator-workspace.mjs")).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn coordinator_availability_script() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::text("coordinator-availability.mjs", include_str!("coordinator-availability.mjs")).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn coordinator_node_avatar_script() -> Response {
    let mut response = Response::new(Body::from(include_str!("coordinator-node-avatar.mjs")));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
async fn coordinator_ai_script() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::text("coordinator-ai.mjs", include_str!("coordinator-ai.mjs")).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn coordinator_device_routes_script() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::text("coordinator-device-routes.mjs", include_str!("coordinator-device-routes.mjs")).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn coordinator_profile_color_script() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::text("profile-color.mjs", include_str!("profile-color.mjs")).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn coordinator_metadata_export_js() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::bytes("assets/metadata-export.js", include_bytes!("assets/metadata-export.js")).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("text/javascript; charset=utf-8"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("private, no-cache"));
    response
}

async fn coordinator_rawrarr_logo() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::bytes("assets/rawrarr-logo.png", include_bytes!("assets/rawrarr-logo.png")).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));
    response
}

async fn coordinator_rawrarr_logo_jxl() -> Response {
    let mut response = Response::new(Body::from(crate::ui_assets::bytes("assets/rawrarr-logo.jxl", include_bytes!("assets/rawrarr-logo.jxl")).await));
    response.headers_mut().insert(header::CONTENT_TYPE, HeaderValue::from_static("image/jxl"));
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600"));
    response
}

async fn node_proxy_root(
    State(state): State<CoordinatorState>,
    Path(node_id): Path<String>,
    request: Request,
) -> Response {
    proxy_node_request(state, node_id, String::new(), request).await
}

async fn node_proxy(
    State(state): State<CoordinatorState>,
    Path((node_id, path)): Path<(String, String)>,
    request: Request,
) -> Response {
    proxy_node_request(state, node_id, path, request).await
}

const RELAY_BODY_LIMIT: usize = 8 * 1024 * 1024;
const NODE_SESSION_SECONDS: i64 = 8 * 60 * 60;

async fn proxy_node_request(
    state: CoordinatorState,
    node_id: String,
    path: String,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let Some(owner_user_id) = crate::auth::authenticated_user_id(
        &state.coordinator_auth,
        &parts.headers,
    ).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    if path.split('/').any(|part| part == ".." || part == ".") || path.contains('\\') {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let role = match sqlx::query_scalar::<_, String>("SELECT role FROM rawrarr_nodes WHERE id=$1 AND user_id=$2")
        .bind(&node_id).bind(owner_user_id).fetch_optional(&state.pool).await {
            Ok(Some(role)) => role,
            Ok(None) => return StatusCode::NOT_FOUND.into_response(),
            Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
        };
    if !matches!(role.as_str(), "metadata" | "storage") {
        return StatusCode::FORBIDDEN.into_response();
    }
    let Some(target) = state.relay_targets.get(&node_id) else {
        return (StatusCode::SERVICE_UNAVAILABLE, "No coordinator relay route is configured for this node").into_response();
    };
    let (node_session, set_cookie) = match delegated_node_session(
        &state, &node_id, owner_user_id, &parts.headers,
    ).await {
        Ok(result) => result,
        Err(status) => return status.into_response(),
    };
    let request_body = match to_bytes(body, RELAY_BODY_LIMIT).await {
        Ok(body) => body,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let method_allowed = if storage_scan_relay_allowed(
        &role,
        &path,
        &parts.method,
        &request_body,
    ) {
        true
    } else if path == "api/metadata/session/library" {
        encrypted_session_request(&path, &parts.method, &request_body)
    } else if path == "api/metadata/export/config" && parts.method == Method::PUT {
        request_body.starts_with(b"age-encryption.org/v1\n") && request_body.len() <= RELAY_BODY_LIMIT
    } else if path == "api/metadata/storage" {
        parts.method == Method::GET && request_body.is_empty()
    } else {
        parts.method == Method::GET
            && (ciphertext_export_path(&path) || path == "api/syncthing/shared-health" || path == "api/config-peer/status")
            && request_body.is_empty()
    };
    if !method_allowed {
        return StatusCode::NOT_FOUND.into_response();
    }
    let mut upstream_url = match target.join(if path.is_empty() { "" } else { path.trim_start_matches('/') }) {
        Ok(url) => url,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };
    if let Some(query) = parts.uri.query() {
        upstream_url.set_query(Some(query));
    }
    let method = parts.method.clone();
    if !matches!(method, Method::GET | Method::HEAD | Method::POST | Method::PUT | Method::PATCH | Method::DELETE | Method::OPTIONS) {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let mut outgoing = state.relay_client.request(method, upstream_url)
        .header(header::COOKIE, format!("rawrarr_session={node_session}"));
    for name in [header::ACCEPT, header::CONTENT_TYPE, header::ACCEPT_LANGUAGE, header::CACHE_CONTROL, header::IF_NONE_MATCH, header::IF_MODIFIED_SINCE, header::RANGE] {
        if let Some(value) = parts.headers.get(&name) {
            outgoing = outgoing.header(name, value);
        }
    }
    let upstream = match outgoing.body(request_body).send().await {
        Ok(response) => response,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };
    let status = StatusCode::from_u16(upstream.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let content_type = upstream.headers().get(header::CONTENT_TYPE).cloned();
    let location = upstream.headers().get(header::LOCATION).and_then(|value| value.to_str().ok()).map(str::to_owned);
    let is_rewritable = content_type.as_ref().and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("text/html") || value.starts_with("text/javascript") || value.starts_with("application/javascript") || value.starts_with("text/css"));
    let mut response = Response::builder().status(status);
    for name in [header::CONTENT_TYPE, header::CACHE_CONTROL, header::ETAG, header::LAST_MODIFIED, header::CONTENT_RANGE, header::ACCEPT_RANGES, header::VARY] {
        if let Some(value) = upstream.headers().get(&name) {
            response = response.header(name, value);
        }
    }
    if let Some(location) = location {
        let prefix = format!("/nodes/{node_id}/");
        let rewritten = if location.starts_with('/') && !location.starts_with("//") {
            format!("{prefix}{}", location.trim_start_matches('/'))
        } else { location };
        if let Ok(value) = HeaderValue::from_str(&rewritten) {
            response = response.header(header::LOCATION, value);
        }
    }
    if let Some(cookie) = set_cookie {
        if let Ok(value) = HeaderValue::from_str(&cookie) {
            response = response.header(header::SET_COOKIE, value);
        }
    }
    let body = if is_rewritable {
        let bytes = match upstream.bytes().await {
            Ok(bytes) if bytes.len() <= RELAY_BODY_LIMIT => bytes,
            _ => return StatusCode::BAD_GATEWAY.into_response(),
        };
        let text = String::from_utf8_lossy(&bytes);
        let prefix = format!("/nodes/{node_id}/");
        let rewritten = rewrite_node_asset_paths(&text, &prefix);
        Body::from(rewritten)
    } else {
        Body::from_stream(upstream.bytes_stream().map(|chunk| chunk.map_err(std::io::Error::other)))
    };
    response.body(body).unwrap_or_else(|_| StatusCode::BAD_GATEWAY.into_response())
}

fn storage_scan_relay_allowed(
    role: &str,
    path: &str,
    method: &Method,
    body: &[u8],
) -> bool {
    if role != "storage" {
        return false;
    }
    match path {
        "api/library/scan/missing" => {
            *method == Method::POST
                && body.len() <= 1024
                && serde_json::from_slice::<crate::storage_node::ScanOptions>(body)
                    .is_ok_and(|options| options.validate().is_ok())
        }
        "api/library/scan/status" => *method == Method::GET && body.is_empty(),
        "api/library/scan/cancel" => *method == Method::POST && body == b"{}",
        "api/library/scan/schedule" if *method == Method::GET => body.is_empty(),
        "api/library/scan/schedule" if *method == Method::PUT => {
            body.len() <= 1024
                && serde_json::from_slice::<crate::storage_node::ScanSchedule>(body)
                    .is_ok_and(|schedule| schedule.validate().is_ok())
        }
        _ => false,
    }
}

fn ciphertext_export_path(path: &str) -> bool {
    matches!(path, "api/metadata/export/mini" | "api/metadata/export/config" | "api/metadata/export/recipient" | "api/metadata/export/node-recipient" | "api/metadata/export/group-user-envelope")
        || path.strip_prefix("api/metadata/export/ciphertext/").is_some_and(|tail| {
            let mut parts = tail.split('/');
            let kind = parts.next().unwrap_or_default();
            let filename = parts.next().unwrap_or_default();
            parts.next().is_none() && matches!(kind, "text" | "artwork")
                && crate::valid_export_filename(kind, filename)
        })
}

fn encrypted_session_request(path: &str, method: &Method, body: &[u8]) -> bool {
    path == "api/metadata/session/library"
        && *method == Method::POST
        && body.len() == 65
        && body.first() == Some(&4)
        && p256::PublicKey::from_sec1_bytes(body).is_ok()
}

async fn delegated_node_session(
    state: &CoordinatorState,
    node_id: &str,
    owner_user_id: Uuid,
    headers: &HeaderMap,
) -> Result<(String, Option<String>), StatusCode> {
    if let Some(token) = cookie_value(headers, "rawrarr_node_session") {
        let hash = Sha256::digest(token.as_bytes()).to_vec();
        let valid = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_sessions WHERE token_hash=$1 AND user_id=$2 AND node_id=$3 AND expires_at > now())")
            .bind(hash).bind(owner_user_id).bind(node_id).fetch_one(&state.pool).await;
        if matches!(valid, Ok(true)) {
            return Ok((token.to_owned(), None));
        }
    }
    let token = Uuid::new_v4().simple().to_string();
    let hash = Sha256::digest(token.as_bytes()).to_vec();
    let inserted = sqlx::query("INSERT INTO rawrarr_sessions(token_hash,user_id,expires_at,node_id) VALUES ($1,$2,now()+($3 * interval '1 second'),$4)")
        .bind(hash).bind(owner_user_id).bind(NODE_SESSION_SECONDS).bind(node_id).execute(&state.pool).await;
    if inserted.is_err() {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let cookie = format!("rawrarr_node_session={token}; Path=/nodes/{node_id}/; HttpOnly; Secure; SameSite=Lax; Max-Age={NODE_SESSION_SECONDS}");
    Ok((token, Some(cookie)) )
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(header::COOKIE)?.to_str().ok()?.split(';').filter_map(|part| part.trim().split_once('=')).find_map(|(key, value)| (key == name).then_some(value))
}

fn rewrite_node_asset_paths(source: &str, prefix: &str) -> String {
    let mut text = source.to_owned();
    for path in ["/api/", "/assets/"] {
        for quote in ['\'', '"', '`'] {
            let needle = format!("{quote}{path}");
            let replacement = format!("{quote}{prefix}{}", path.trim_start_matches('/'));
            text = text.replace(&needle, &replacement);
        }
    }
    text
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CoordinatorPasskeyLink {
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CoordinatorPasskeyToken {
    token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CoordinatorPasskeyChallenge<T> {
    challenge_id: Uuid,
    options: T,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CoordinatorPasskeyFinish {
    challenge_id: Uuid,
    token: String,
    credential: webauthn_rs::prelude::RegisterPublicKeyCredential,
}

async fn create_coordinator_passkey_link(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
) -> Response {
    let Some(user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let mut bytes = [0u8; 32];
    if getrandom::fill(&mut bytes).is_err() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let token = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let token_sha256 = format!("{:x}", Sha256::digest(token.as_bytes()));
    let cleanup = sqlx::query(
        "DELETE FROM rawrarr_passkey_link_tokens WHERE expires_at <= now() OR (consumed_at IS NOT NULL AND consumed_at < now() - interval '10 minutes')",
    )
    .execute(&state.pool)
    .await;
    if cleanup.is_err()
        || sqlx::query("INSERT INTO rawrarr_passkey_link_tokens(token_sha256, user_id, source_node_id, expires_at) VALUES ($1,$2,$3,now() + interval '5 minutes')")
            .bind(token_sha256)
            .bind(user_id)
            .bind(&auth.node_id)
            .execute(&state.pool)
            .await
            .is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    Json(CoordinatorPasskeyLink {
        url: format!("{}/passkeys/add#{token}", state.web_origin),
    })
    .into_response()
}

async fn coordinator_passkey_page() -> Response {
    let mut response = axum::response::Html(crate::ui_assets::text("coordinator_passkey.html", include_str!("coordinator_passkey.html")).await)
        .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; script-src 'self'; style-src 'unsafe-inline'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"),
    );
    response
}

async fn coordinator_passkey_client() -> Response {
    let mut response = axum::response::Response::new(
        axum::body::Body::from(crate::ui_assets::text("passkey-client.mjs", include_str!("passkey-client.mjs")).await),
    );
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/javascript; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    response
}

async fn coordinator_passkey_script() -> Response {
    let mut response = axum::response::Response::new(axum::body::Body::from(crate::ui_assets::text("coordinator-passkey.mjs", include_str!("coordinator-passkey.mjs")).await));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/javascript; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    );
    response
}

async fn start_coordinator_passkey(
    State(state): State<CoordinatorState>,
    Json(request): Json<CoordinatorPasskeyToken>,
) -> Response {
    if !valid_digest(&request.token) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let token_sha256 = format!("{:x}", Sha256::digest(request.token.as_bytes()));
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let row = sqlx::query("SELECT l.user_id,u.username,u.display_name,l.source_node_id FROM rawrarr_passkey_link_tokens l JOIN rawrarr_users u ON u.id=l.user_id WHERE l.token_sha256=$1 AND l.expires_at>now() AND l.consumed_at IS NULL FOR UPDATE OF l")
        .bind(&token_sha256).fetch_optional(&mut *tx).await;
    let row = match row {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::GONE.into_response(),
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let user_id: Uuid = match row.try_get("user_id") {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let username: String = match row.try_get("username") {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let display_name: String = match row.try_get("display_name") {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let source_node_id: String = match row.try_get("source_node_id") {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let (options, registration) = match state
        .webauthn
        .start_passkey_registration(user_id, &username, &display_name, None)
    {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let registration = match serde_json::to_value(registration) {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let challenge_id = Uuid::new_v4();
    if sqlx::query("INSERT INTO rawrarr_auth_challenges(id,flow,user_id,username,display_name,state,expires_at,node_id,link_token_sha256) VALUES ($1,'add_passkey',$2,$3,$4,$5,now()+interval '5 minutes',$6,$7)")
        .bind(challenge_id).bind(user_id).bind(username).bind(display_name).bind(registration).bind(source_node_id).bind(&token_sha256)
        .execute(&mut *tx).await.is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if sqlx::query("UPDATE rawrarr_passkey_link_tokens SET consumed_at=now() WHERE token_sha256=$1 AND consumed_at IS NULL")
        .bind(&token_sha256).execute(&mut *tx).await.is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if tx.commit().await.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    Json(CoordinatorPasskeyChallenge {
        challenge_id,
        options,
    })
    .into_response()
}

async fn finish_coordinator_passkey(
    State(state): State<CoordinatorState>,
    Json(request): Json<CoordinatorPasskeyFinish>,
) -> Response {
    if !valid_digest(&request.token) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let token_sha256 = format!("{:x}", Sha256::digest(request.token.as_bytes()));
    let mut tx = match state.pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let row = sqlx::query("SELECT c.user_id,c.state FROM rawrarr_auth_challenges c JOIN rawrarr_passkey_link_tokens l ON l.token_sha256=c.link_token_sha256 WHERE c.id=$1 AND c.flow='add_passkey' AND c.link_token_sha256=$2 AND c.expires_at>now() AND l.expires_at>now() AND l.consumed_at IS NOT NULL FOR UPDATE OF c,l")
        .bind(request.challenge_id).bind(&token_sha256).fetch_optional(&mut *tx).await;
    let row = match row {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::GONE.into_response(),
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let user_id: Uuid = match row.try_get("user_id") {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let state_json: serde_json::Value = match row.try_get("state") {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let registration: webauthn_rs::prelude::PasskeyRegistration =
        match serde_json::from_value(state_json) {
            Ok(value) => value,
            Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
        };
    let passkey = match state
        .webauthn
        .finish_passkey_registration(&request.credential, &registration)
    {
        Ok(value) => value,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };
    let credential_id = passkey.cred_id().as_ref();
    let passkey_json = match serde_json::to_value(&passkey) {
        Ok(value) => value,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE.into_response(),
    };
    let inserted = sqlx::query("INSERT INTO rawrarr_passkeys(credential_id,user_id,passkey) VALUES ($1,$2,$3) ON CONFLICT(credential_id) DO NOTHING")
        .bind(credential_id).bind(user_id).bind(passkey_json).execute(&mut *tx).await;
    if !matches!(inserted, Ok(result) if result.rows_affected() == 1) {
        return StatusCode::CONFLICT.into_response();
    }
    if sqlx::query("DELETE FROM rawrarr_auth_challenges WHERE id=$1")
        .bind(request.challenge_id)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if sqlx::query("DELETE FROM rawrarr_passkey_link_tokens WHERE token_sha256=$1")
        .bind(&token_sha256)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    if tx.commit().await.is_err() {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    Json(serde_json::json!({"ok": true})).into_response()
}
