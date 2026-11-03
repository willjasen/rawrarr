//! Explicit environment ownership, separate from instance signing trust.
use super::*;

type OwnerError = (StatusCode, &'static str);
const DATABASE_ERROR: OwnerError = (
    StatusCode::SERVICE_UNAVAILABLE,
    "Environment owner database unavailable",
);

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ProvisionRequest {
    node_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OwnerInfo {
    owner_user_id: Uuid,
    age_recipient: String,
    source_node_id: String,
}

pub(super) async fn get(
    State(state): State<CoordinatorState>,
    headers: HeaderMap,
) -> Result<Response, OwnerError> {
    let user = crate::auth::authenticated_user_id(&state.coordinator_auth, &headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Sign in with your passkey"))?;
    let row = sqlx::query("SELECT owner_user_id, age_recipient, source_node_id FROM rawrarr_environment_owner WHERE singleton=TRUE AND owner_user_id=$1")
        .bind(user).fetch_optional(&state.pool).await.map_err(|_| DATABASE_ERROR)?
        .ok_or((StatusCode::NOT_FOUND, "Environment owner recipient is not provisioned for this account"))?;
    owner_response(OwnerInfo {
        owner_user_id: row.try_get("owner_user_id").map_err(|_| DATABASE_ERROR)?,
        age_recipient: row.try_get("age_recipient").map_err(|_| DATABASE_ERROR)?,
        source_node_id: row.try_get("source_node_id").map_err(|_| DATABASE_ERROR)?,
    })
}

pub(super) async fn provision(
    State(state): State<CoordinatorState>,
    headers: HeaderMap,
    Json(request): Json<ProvisionRequest>,
) -> Result<Response, OwnerError> {
    let user = crate::auth::authenticated_user_id(&state.coordinator_auth, &headers)
        .await
        .ok_or((StatusCode::UNAUTHORIZED, "Sign in with your passkey"))?;
    // This browser action must originate from the coordinator, not another site.
    if headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        != Some(state.web_origin.as_str())
    {
        return Err((
            StatusCode::FORBIDDEN,
            "Use the authenticated coordinator origin",
        ));
    }
    if !valid_node_id(&request.node_id) {
        return Err((StatusCode::BAD_REQUEST, "Invalid fossil ID"));
    }
    let owned = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM rawrarr_nodes WHERE id=$1 AND user_id=$2 AND role='metadata')",
    )
    .bind(&request.node_id)
    .bind(user)
    .fetch_one(&state.pool)
    .await
    .map_err(|_| DATABASE_ERROR)?;
    if !owned {
        return Err((
            StatusCode::FORBIDDEN,
            "Select a fossil owned by this account",
        ));
    }
    let target = state.relay_targets.get(&request.node_id).ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "No coordinator relay route is configured for this fossil",
    ))?;
    let url = target
        .join("api/metadata/export/owner-recipient")
        .map_err(|_| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Invalid configured fossil route",
            )
        })?;
    let (session, _) = delegated_node_session(&state, &request.node_id, user, &headers)
        .await
        .map_err(|status| (status, "Could not authenticate to the selected fossil"))?;
    let mut upstream = state
        .relay_client
        .get(url)
        .header(header::COOKIE, format!("rawrarr_session={session}"))
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .map_err(|_| {
            (
                StatusCode::BAD_GATEWAY,
                "The selected fossil is unavailable",
            )
        })?;
    match upstream.status().as_u16() {
        200 => {}
        404 => {
            return Err((
                StatusCode::PRECONDITION_REQUIRED,
                "Upgrade the selected fossil to support owner public recipients",
            ));
        }
        423 => {
            return Err((
                StatusCode::LOCKED,
                "Unlock the selected fossil with the existing owner age identity",
            ));
        }
        _ => {
            return Err((
                StatusCode::BAD_GATEWAY,
                "The selected fossil could not supply its owner public recipient",
            ));
        }
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = upstream.chunk().await.map_err(|_| {
        (
            StatusCode::BAD_GATEWAY,
            "Could not read the fossil public recipient",
        )
    })? {
        if bytes.len() + chunk.len() > 4096 {
            return Err((StatusCode::BAD_GATEWAY, "Oversized fossil public recipient"));
        }
        bytes.extend_from_slice(&chunk);
    }
    let recipient = String::from_utf8(bytes)
        .map_err(|_| (StatusCode::BAD_GATEWAY, "Invalid fossil public recipient"))?;
    if !crate::secure_config::is_post_quantum_recipient(&recipient) {
        return Err((
            StatusCode::BAD_GATEWAY,
            "Invalid fossil post-quantum public recipient",
        ));
    }
    let checked = recipient.clone();
    tokio::task::spawn_blocking(move || {
        std::process::Command::new("age")
            .args(["--encrypt", "--recipient", &checked])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
    })
    .await
    .map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Owner recipient validation failed",
        )
    })?
    .map_err(|_| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "The age validator is unavailable",
        )
    })?
    .success()
    .then_some(())
    .ok_or((
        StatusCode::BAD_GATEWAY,
        "The fossil supplied an invalid age recipient",
    ))?;
    let info = pin(&state.pool, user, &request.node_id, &recipient).await?;
    owner_response(info)
}

fn owner_response(info: OwnerInfo) -> Result<Response, OwnerError> {
    let mut response = Json(info).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    Ok(response)
}

async fn pin(
    pool: &PgPool,
    user: Uuid,
    node: &str,
    recipient: &str,
) -> Result<OwnerInfo, OwnerError> {
    let mut tx = pool.begin().await.map_err(|_| DATABASE_ERROR)?;
    // Serialize first claims and freeze account/root associations while committing.
    sqlx::query("LOCK TABLE rawrarr_users, rawrarr_instance_signing_root, rawrarr_environment_owner IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut *tx).await.map_err(|_| DATABASE_ERROR)?;
    let owned = sqlx::query(
        "SELECT id FROM rawrarr_nodes WHERE id=$1 AND user_id=$2 AND role='metadata' FOR SHARE",
    )
    .bind(node)
    .bind(user)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| DATABASE_ERROR)?;
    if owned.is_none() {
        return Err((
            StatusCode::FORBIDDEN,
            "The fossil no longer belongs to this account",
        ));
    }
    let root_owner = sqlx::query_scalar::<_, Uuid>(
        "SELECT owner_user_id FROM rawrarr_instance_signing_root WHERE singleton=TRUE",
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| DATABASE_ERROR)?;
    if root_owner.is_some_and(|owner| owner != user) {
        return Err((
            StatusCode::FORBIDDEN,
            "This account does not own the established environment",
        ));
    }
    let existing = sqlx::query("SELECT owner_user_id, age_recipient, source_node_id FROM rawrarr_environment_owner WHERE singleton=TRUE")
        .fetch_optional(&mut *tx).await.map_err(|_| DATABASE_ERROR)?;
    let source_node_id = if let Some(row) = existing {
        let owner: Uuid = row.try_get("owner_user_id").map_err(|_| DATABASE_ERROR)?;
        let pinned: String = row.try_get("age_recipient").map_err(|_| DATABASE_ERROR)?;
        if owner != user || pinned != recipient {
            return Err((
                StatusCode::CONFLICT,
                "Environment owner or age recipient conflicts with the existing pin; no change was made",
            ));
        }
        row.try_get("source_node_id").map_err(|_| DATABASE_ERROR)?
    } else {
        let accounts: i64 = sqlx::query_scalar("SELECT count(*) FROM rawrarr_users")
            .fetch_one(&mut *tx)
            .await
            .map_err(|_| DATABASE_ERROR)?;
        if accounts != 1 {
            return Err((
                StatusCode::CONFLICT,
                "Environment ownership is ambiguous; no owner was inferred",
            ));
        }
        sqlx::query("INSERT INTO rawrarr_environment_owner(singleton, owner_user_id, age_recipient, source_node_id) VALUES(TRUE,$1,$2,$3)")
            .bind(user).bind(recipient).bind(node).execute(&mut *tx).await.map_err(|_| DATABASE_ERROR)?;
        node.to_owned()
    };
    tx.commit().await.map_err(|_| DATABASE_ERROR)?;
    Ok(OwnerInfo {
        owner_user_id: user,
        age_recipient: recipient.to_owned(),
        source_node_id,
    })
}

#[cfg(test)]
#[path = "../../scripts/tests/test_rawrarr_environment_owner.rs"]
mod tests;
