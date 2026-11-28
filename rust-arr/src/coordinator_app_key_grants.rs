//! Opaque relay for owner-signed, device-encrypted app-key grants.
use super::*;
use axum::body::Body;

const MAX_GRANT_BYTES: usize = 64 * 1024;
const MAX_ENVELOPE_BYTES: usize = 16 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExistingGrantAction {
    Insert,
    Retry,
    Replace,
    RejectStale,
    RejectConflict,
}

fn existing_grant_action(
    current_epoch: Option<u64>,
    current_recipient: Option<&str>,
    current_bundle: Option<&[u8]>,
    submitted_epoch: u64,
    submitted_recipient: &str,
    submitted_bundle: &[u8],
) -> ExistingGrantAction {
    let Some(current_epoch) = current_epoch else {
        return ExistingGrantAction::Insert;
    };
    if submitted_epoch < current_epoch {
        ExistingGrantAction::RejectStale
    } else if submitted_epoch > current_epoch {
        ExistingGrantAction::Replace
    } else if current_recipient == Some(submitted_recipient)
        && current_bundle == Some(submitted_bundle)
    {
        ExistingGrantAction::Retry
    } else {
        ExistingGrantAction::RejectConflict
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Grant {
    algorithm: String,
    root_public_key: String,
    root_fingerprint: String,
    signed_payload: GrantPayload,
    signature: String,
    encrypted_identity: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GrantPayload {
    domain: String,
    schema_version: u32,
    app_instance_id: String,
    app_type: String,
    app_recipient: String,
    device_id: String,
    device_recipient: String,
    key_epoch: u64,
    encrypted_identity_sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GrantRequest {
    app_instance_id: String,
    recipient: String,
}

type GrantError = (StatusCode, &'static str);
const DATABASE_ERROR: GrantError = (
    StatusCode::SERVICE_UNAVAILABLE,
    "App-key grant storage unavailable",
);

pub(super) async fn submit(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, GrantError> {
    let owner = crate::auth::authenticated_user_id(&auth, &headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Sign in with your passkey"))?;
    if headers.get(header::ORIGIN).and_then(|v| v.to_str().ok()) != Some(state.web_origin.as_str())
    {
        return Err((
            StatusCode::FORBIDDEN,
            "Use the authenticated coordinator origin",
        ));
    }
    let bytes = to_bytes(body, MAX_GRANT_BYTES).await.map_err(|_| {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            "App-key grant exceeds 64 KiB",
        )
    })?;
    let grant: Grant = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid app-key grant"))?;
    let payload = &grant.signed_payload;
    let app_id = Uuid::parse_str(&payload.app_instance_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid app instance ID"))?;
    if app_id.to_string() != payload.app_instance_id
        || grant.algorithm != crate::device_signing::ALGORITHM
        || payload.domain != "rawrarr-app-key-grant-v1"
        || payload.schema_version != 1
        || payload.key_epoch == 0
        || payload.key_epoch > i64::MAX as u64
        || payload.app_type.is_empty()
        || payload.app_type.len() > 64
        || !payload
            .app_type
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || !valid_node_id(&payload.device_id)
        || !crate::secure_config::is_post_quantum_recipient(&payload.app_recipient)
        || !crate::secure_config::is_post_quantum_recipient(&payload.device_recipient)
        || payload.encrypted_identity_sha256.len() != 64
        || !payload
            .encrypted_identity_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid app-key grant routing metadata",
        ));
    }

    let root = sqlx::query("SELECT public_key, fingerprint FROM rawrarr_instance_signing_root WHERE singleton=TRUE AND owner_user_id=$1")
        .bind(owner).fetch_optional(&state.pool).await.map_err(|_| DATABASE_ERROR)?
        .ok_or((StatusCode::FORBIDDEN, "No instance signing root is provisioned for this account"))?;
    let root_key: Vec<u8> = root.try_get("public_key").map_err(|_| DATABASE_ERROR)?;
    let root_fingerprint: String = root.try_get("fingerprint").map_err(|_| DATABASE_ERROR)?;
    let submitted_root = BASE64
        .decode(&grant.root_public_key)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid root signing key"))?;
    let signature = BASE64
        .decode(&grant.signature)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid grant signature"))?;
    let ciphertext = BASE64
        .decode(&grant.encrypted_identity)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid encrypted identity"))?;
    if grant.root_fingerprint != root_fingerprint
        || submitted_root != root_key
        || crate::device_signing::fingerprint(&submitted_root) != root_fingerprint
        || ciphertext.is_empty()
        || ciphertext.len() > MAX_ENVELOPE_BYTES
        || !ciphertext.starts_with(b"age-encryption.org/v1\n")
        || format!("{:x}", Sha256::digest(&ciphertext)) != payload.encrypted_identity_sha256
    {
        return Err((
            StatusCode::BAD_REQUEST,
            "Grant root or encrypted envelope does not match",
        ));
    }
    let signed = serde_json::to_value(&grant.signed_payload)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid signed grant payload"))?;
    let canonical = crate::device_signing::canonical_json(&signed)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid signed grant payload"))?;
    if !crate::device_signing::verify(&submitted_root, &signature, &canonical) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Grant signature verification failed",
        ));
    }
    let owned = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_nodes WHERE id=$1 AND user_id=$2 AND role='metadata' AND token_sha256 IS NOT NULL)")
        .bind(&payload.device_id).bind(owner).fetch_one(&state.pool).await.map_err(|_| DATABASE_ERROR)?;
    if !owned {
        return Err((
            StatusCode::FORBIDDEN,
            "Grant target is not an approved fossil owned by this account",
        ));
    }

    let mut tx = state.pool.begin().await.map_err(|_| DATABASE_ERROR)?;
    let lock_key = format!("{}:{}", app_id, payload.device_id);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(lock_key)
        .execute(&mut *tx)
        .await
        .map_err(|_| DATABASE_ERROR)?;
    let existing = sqlx::query("SELECT key_epoch, device_recipient, bundle FROM rawrarr_app_key_grants WHERE app_instance_id=$1 AND device_id=$2 FOR UPDATE")
        .bind(app_id).bind(&payload.device_id).fetch_optional(&mut *tx).await.map_err(|_| DATABASE_ERROR)?;
    if let Some(row) = existing {
        let epoch: i64 = row.try_get("key_epoch").map_err(|_| DATABASE_ERROR)?;
        let recipient: String = row
            .try_get("device_recipient")
            .map_err(|_| DATABASE_ERROR)?;
        let prior: Vec<u8> = row.try_get("bundle").map_err(|_| DATABASE_ERROR)?;
        match existing_grant_action(
            Some(epoch as u64),
            Some(&recipient),
            Some(&prior),
            payload.key_epoch,
            &payload.device_recipient,
            bytes.as_ref(),
        ) {
            ExistingGrantAction::RejectStale => {
                return Err((
                    StatusCode::CONFLICT,
                    "A newer app-key epoch is already stored",
                ));
            }
            ExistingGrantAction::Retry => {
                tx.commit().await.map_err(|_| DATABASE_ERROR)?;
                return Ok(StatusCode::NO_CONTENT.into_response());
            }
            ExistingGrantAction::RejectConflict => {
                return Err((
                    StatusCode::CONFLICT,
                    "A different grant already exists for this app-key epoch",
                ));
            }
            ExistingGrantAction::Replace => {
                sqlx::query("UPDATE rawrarr_app_key_grants SET key_epoch=$3, device_recipient=$4, bundle=$5, created_at=now() WHERE app_instance_id=$1 AND device_id=$2")
                    .bind(app_id).bind(&payload.device_id).bind(payload.key_epoch as i64)
                    .bind(&payload.device_recipient).bind(bytes.as_ref()).execute(&mut *tx).await.map_err(|_| DATABASE_ERROR)?;
            }
            ExistingGrantAction::Insert => unreachable!("existing row has an epoch"),
        }
    } else {
        sqlx::query("INSERT INTO rawrarr_app_key_grants(app_instance_id,device_id,device_recipient,key_epoch,bundle) VALUES ($1,$2,$3,$4,$5)")
            .bind(app_id).bind(&payload.device_id).bind(&payload.device_recipient)
            .bind(payload.key_epoch as i64).bind(bytes.as_ref()).execute(&mut *tx).await.map_err(|_| DATABASE_ERROR)?;
    }
    tx.commit().await.map_err(|_| DATABASE_ERROR)?;
    Ok(StatusCode::CREATED.into_response())
}

#[cfg(test)]
mod tests {
    use super::{ExistingGrantAction, existing_grant_action};

    #[test]
    fn grants_are_retry_safe_conflict_on_same_epoch_and_replace_only_with_newer_epoch() {
        assert_eq!(
            existing_grant_action(None, None, None, 1, "age1pq1target", b"cipher"),
            ExistingGrantAction::Insert
        );
        assert_eq!(
            existing_grant_action(
                Some(3),
                Some("age1pq1target"),
                Some(b"cipher"),
                3,
                "age1pq1target",
                b"cipher"
            ),
            ExistingGrantAction::Retry
        );
        assert_eq!(
            existing_grant_action(
                Some(3),
                Some("age1pq1target"),
                Some(b"cipher"),
                2,
                "age1pq1target",
                b"older"
            ),
            ExistingGrantAction::RejectStale
        );
        assert_eq!(
            existing_grant_action(
                Some(3),
                Some("age1pq1target"),
                Some(b"cipher"),
                3,
                "age1pq1other",
                b"cipher"
            ),
            ExistingGrantAction::RejectConflict
        );
        assert_eq!(
            existing_grant_action(
                Some(3),
                Some("age1pq1target"),
                Some(b"cipher"),
                4,
                "age1pq1new",
                b"new"
            ),
            ExistingGrantAction::Replace
        );
    }
}

pub(super) async fn retrieve(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    Extension(role): Extension<NodeRole>,
    body: Body,
) -> Result<Response, GrantError> {
    if role != NodeRole::Metadata {
        return Err((
            StatusCode::FORBIDDEN,
            "Only an approved fossil can retrieve app-key grants",
        ));
    }
    let request = to_bytes(body, 8192).await.map_err(|_| {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            "App-key grant request exceeds 8 KiB",
        )
    })?;
    let request: GrantRequest = serde_json::from_slice(&request)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid app-key grant request"))?;
    let app_id = Uuid::parse_str(&request.app_instance_id)
        .ok()
        .filter(|id| id.to_string() == request.app_instance_id)
        .ok_or((StatusCode::BAD_REQUEST, "Invalid app instance ID"))?;
    if !crate::secure_config::is_post_quantum_recipient(&request.recipient) {
        return Err((StatusCode::BAD_REQUEST, "Invalid device age recipient"));
    }
    // Recheck persistent approval so deletion/revocation takes effect even
    // while another coordinator worker still has a cached token entry.
    let approved = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_nodes WHERE id=$1 AND role='metadata' AND token_sha256 IS NOT NULL)")
        .bind(&auth.node_id).fetch_one(&state.pool).await.map_err(|_| DATABASE_ERROR)?;
    if !approved {
        return Err((StatusCode::FORBIDDEN, "Fossil has been revoked"));
    }
    let row = sqlx::query("SELECT bundle FROM rawrarr_app_key_grants WHERE app_instance_id=$1 AND device_id=$2 AND device_recipient=$3")
        .bind(app_id).bind(&auth.node_id).bind(&request.recipient).fetch_optional(&state.pool).await.map_err(|_| DATABASE_ERROR)?
        .ok_or((StatusCode::NOT_FOUND, "No matching app-key grant is available"))?;
    let bundle: Vec<u8> = row.try_get("bundle").map_err(|_| DATABASE_ERROR)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/json"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        Body::from(bundle),
    )
        .into_response())
}
