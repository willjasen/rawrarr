//! Passkey-backed web authentication and PostgreSQL persistence.

use axum::{
    Json,
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{
    PgPool, Row,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use std::env;
use url::Url;
use uuid::Uuid;
use webauthn_rs::prelude::*;
use zeroize::Zeroizing;

const SESSION_COOKIE: &str = "rawrarr_session";
const SESSION_SECONDS: i64 = 8 * 60 * 60;
const CHALLENGE_SECONDS: i64 = 5 * 60;

#[derive(Clone)]
pub(super) struct AuthState {
    pub pool: PgPool,
    pub webauthn: Webauthn,
    secure_cookie: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AuthStatus {
    user_exists: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ChallengeResponse<T> {
    challenge_id: Uuid,
    options: T,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AuthComplete {
    next: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RegistrationStart {
    username: String,
    display_name: String,
}

#[derive(Deserialize)]
pub(super) struct AuthenticationStart {
    username: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RegistrationFinish {
    challenge_id: Uuid,
    credential: RegisterPublicKeyCredential,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AuthenticationFinish {
    challenge_id: Uuid,
    credential: PublicKeyCredential,
}

#[derive(Debug, Serialize)]
pub(super) struct AuthError {
    error: &'static str,
}

impl AuthState {
    pub(super) async fn connect() -> Result<Self, Box<dyn std::error::Error>> {
        let pool = database_pool().await?;
        sqlx::migrate!("./migrations").run(&pool).await?;

        let rp_id = env::var("RAWARR_WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_owned());
        let origin = env::var("RAWARR_WEBAUTHN_ORIGIN")
            .unwrap_or_else(|_| "http://localhost:8080".to_owned());
        let origin = Url::parse(&origin)?;
        let secure_cookie = origin.scheme() == "https";
        let webauthn = WebauthnBuilder::new(&rp_id, &origin)?
            .rp_name("rawrarr")
            .build()?;
        Ok(Self {
            pool,
            webauthn,
            secure_cookie,
        })
    }
}

async fn database_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = if let Ok(database_url) = env::var("RAWARR_DATABASE_URL") {
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?
    } else {
        let password = if let Ok(path) = env::var("RAWARR_PG_PASSWORD_FILE") {
            std::fs::read_to_string(path)?.trim().to_owned()
        } else {
            env::var("RAWARR_PG_PASSWORD")?
        };
        let options = PgConnectOptions::new()
            .host(&env::var("RAWARR_PG_HOST").unwrap_or_else(|_| "127.0.0.1".to_owned()))
            .port(
                env::var("RAWARR_PG_PORT")
                    .ok()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(5432),
            )
            .username(&env::var("RAWARR_PG_USER").unwrap_or_else(|_| "rawrarr".to_owned()))
            .database(&env::var("RAWARR_PG_DATABASE").unwrap_or_else(|_| "rawrarr".to_owned()))
            .password(&password);
        PgPoolOptions::new()
            .max_connections(10)
            .connect_with(options)
            .await?
    };
    Ok(pool)
}

fn normalize_username(input: &str) -> Result<String, (StatusCode, Json<AuthError>)> {
    let username = input.trim().to_ascii_lowercase();
    if !(3..=32).contains(&username.len())
        || !username.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        })
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthError {
                error: "Choose a username of 3 to 32 letters, numbers, dots, dashes, or underscores",
            }),
        ));
    }
    Ok(username)
}

fn invalid_auth() -> (StatusCode, Json<AuthError>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(AuthError {
            error: "Passkey verification failed; try again",
        }),
    )
}

pub(super) async fn status(
    State(auth): State<AuthState>,
) -> Result<Json<AuthStatus>, (StatusCode, Json<AuthError>)> {
    let _ = sqlx::query("DELETE FROM rawrarr_auth_challenges WHERE expires_at <= now()")
        .execute(&auth.pool)
        .await;
    let _ = sqlx::query("DELETE FROM rawrarr_sessions WHERE expires_at <= now()")
        .execute(&auth.pool)
        .await;
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_users)")
        .fetch_one(&auth.pool)
        .await
        .map_err(|_| invalid_auth())?;
    Ok(Json(AuthStatus {
        user_exists: exists,
    }))
}

pub(super) async fn begin_registration(
    State(auth): State<AuthState>,
    Json(request): Json<RegistrationStart>,
) -> Result<Json<ChallengeResponse<CreationChallengeResponse>>, (StatusCode, Json<AuthError>)> {
    let username = normalize_username(&request.username)?;
    let display_name = request.display_name.trim();
    if display_name.is_empty() || display_name.chars().count() > 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AuthError {
                error: "Enter a display name up to 64 characters",
            }),
        ));
    }
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_users)")
        .fetch_one(&auth.pool)
        .await
        .map_err(|_| invalid_auth())?;
    if exists {
        return Err((
            StatusCode::CONFLICT,
            Json(AuthError {
                error: "First-user setup is already complete; ask an existing user for an invitation",
            }),
        ));
    }
    let user_id = Uuid::new_v4();
    let (options, registration) = auth
        .webauthn
        .start_passkey_registration(user_id, &username, display_name, None)
        .map_err(|_| invalid_auth())?;
    let state = serde_json::to_value(registration).map_err(|_| invalid_auth())?;
    let challenge_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO rawrarr_auth_challenges (id, flow, user_id, username, display_name, state, expires_at) VALUES ($1, 'registration', $2, $3, $4, $5, now() + ($6 * interval '1 second'))",
    )
    .bind(challenge_id)
    .bind(user_id)
    .bind(&username)
    .bind(display_name)
    .bind(state)
    .bind(CHALLENGE_SECONDS)
    .execute(&auth.pool)
    .await
    .map_err(|_| invalid_auth())?;
    Ok(Json(ChallengeResponse {
        challenge_id,
        options,
    }))
}

pub(super) async fn finish_registration(
    State(auth): State<AuthState>,
    Json(request): Json<RegistrationFinish>,
) -> Result<Response, (StatusCode, Json<AuthError>)> {
    let mut transaction = auth.pool.begin().await.map_err(|_| invalid_auth())?;
    sqlx::query("SELECT pg_advisory_xact_lock(8257193821)")
        .execute(&mut *transaction)
        .await
        .map_err(|_| invalid_auth())?;
    let challenge = sqlx::query(
        "SELECT user_id, username, display_name, state FROM rawrarr_auth_challenges WHERE id = $1 AND flow = 'registration' AND expires_at > now() FOR UPDATE",
    )
    .bind(request.challenge_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| invalid_auth())?
    .ok_or_else(invalid_auth)?;
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_users)")
        .fetch_one(&mut *transaction)
        .await
        .map_err(|_| invalid_auth())?;
    if exists {
        return Err((
            StatusCode::CONFLICT,
            Json(AuthError {
                error: "First-user setup has already been completed",
            }),
        ));
    }
    let user_id: Uuid = challenge.try_get("user_id").map_err(|_| invalid_auth())?;
    let username: String = challenge.try_get("username").map_err(|_| invalid_auth())?;
    let display_name: String = challenge
        .try_get("display_name")
        .map_err(|_| invalid_auth())?;
    let state: serde_json::Value = challenge.try_get("state").map_err(|_| invalid_auth())?;
    let state: PasskeyRegistration = serde_json::from_value(state).map_err(|_| invalid_auth())?;
    let passkey = auth
        .webauthn
        .finish_passkey_registration(&request.credential, &state)
        .map_err(|_| invalid_auth())?;
    let credential_id = passkey.cred_id().as_ref();
    let passkey_json = serde_json::to_value(&passkey).map_err(|_| invalid_auth())?;

    sqlx::query("INSERT INTO rawrarr_users (id, username, display_name) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(username)
        .bind(display_name)
        .execute(&mut *transaction)
        .await
        .map_err(|_| invalid_auth())?;
    sqlx::query(
        "INSERT INTO rawrarr_passkeys (credential_id, user_id, passkey) VALUES ($1, $2, $3)",
    )
    .bind(credential_id)
    .bind(user_id)
    .bind(passkey_json)
    .execute(&mut *transaction)
    .await
    .map_err(|_| invalid_auth())?;
    sqlx::query("DELETE FROM rawrarr_auth_challenges WHERE id = $1")
        .bind(request.challenge_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| invalid_auth())?;
    let token = issue_session(&mut transaction, user_id).await?;
    transaction.commit().await.map_err(|_| invalid_auth())?;
    Ok(session_response(token, auth.secure_cookie))
}

pub(super) async fn begin_authentication(
    State(auth): State<AuthState>,
    Json(request): Json<AuthenticationStart>,
) -> Result<Json<ChallengeResponse<RequestChallengeResponse>>, (StatusCode, Json<AuthError>)> {
    let username = normalize_username(&request.username)?;
    let user = sqlx::query("SELECT id FROM rawrarr_users WHERE username = $1")
        .bind(&username)
        .fetch_optional(&auth.pool)
        .await
        .map_err(|_| invalid_auth())?
        .ok_or_else(invalid_auth)?;
    let user_id: Uuid = user.try_get("id").map_err(|_| invalid_auth())?;
    let rows = sqlx::query("SELECT passkey FROM rawrarr_passkeys WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&auth.pool)
        .await
        .map_err(|_| invalid_auth())?;
    let passkeys = rows
        .into_iter()
        .map(|row| {
            row.try_get::<serde_json::Value, _>("passkey")
                .map_err(|_| invalid_auth())
                .and_then(|value| serde_json::from_value(value).map_err(|_| invalid_auth()))
        })
        .collect::<Result<Vec<Passkey>, _>>()?;
    let (options, authentication) = auth
        .webauthn
        .start_passkey_authentication(&passkeys)
        .map_err(|_| invalid_auth())?;
    let state = serde_json::to_value(authentication).map_err(|_| invalid_auth())?;
    let challenge_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO rawrarr_auth_challenges (id, flow, user_id, username, state, expires_at) VALUES ($1, 'authentication', $2, $3, $4, now() + ($5 * interval '1 second'))",
    )
    .bind(challenge_id)
    .bind(user_id)
    .bind(username)
    .bind(state)
    .bind(CHALLENGE_SECONDS)
    .execute(&auth.pool)
    .await
    .map_err(|_| invalid_auth())?;
    Ok(Json(ChallengeResponse {
        challenge_id,
        options,
    }))
}

pub(super) async fn finish_authentication(
    State(auth): State<AuthState>,
    Json(request): Json<AuthenticationFinish>,
) -> Result<Response, (StatusCode, Json<AuthError>)> {
    let mut transaction = auth.pool.begin().await.map_err(|_| invalid_auth())?;
    let challenge = sqlx::query(
        "SELECT user_id, state FROM rawrarr_auth_challenges WHERE id = $1 AND flow = 'authentication' AND expires_at > now() FOR UPDATE",
    )
    .bind(request.challenge_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(|_| invalid_auth())?
    .ok_or_else(invalid_auth)?;
    let user_id: Uuid = challenge.try_get("user_id").map_err(|_| invalid_auth())?;
    let state: serde_json::Value = challenge.try_get("state").map_err(|_| invalid_auth())?;
    let state: PasskeyAuthentication = serde_json::from_value(state).map_err(|_| invalid_auth())?;
    let result = auth
        .webauthn
        .finish_passkey_authentication(&request.credential, &state)
        .map_err(|_| invalid_auth())?;
    let rows =
        sqlx::query("SELECT credential_id, passkey FROM rawrarr_passkeys WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(&mut *transaction)
            .await
            .map_err(|_| invalid_auth())?;
    let mut credential_updated = false;
    for row in rows {
        let credential_id: Vec<u8> = row.try_get("credential_id").map_err(|_| invalid_auth())?;
        let value: serde_json::Value = row.try_get("passkey").map_err(|_| invalid_auth())?;
        let mut passkey: Passkey = serde_json::from_value(value).map_err(|_| invalid_auth())?;
        if passkey.update_credential(&result).is_some() {
            let passkey_json = serde_json::to_value(&passkey).map_err(|_| invalid_auth())?;
            sqlx::query("UPDATE rawrarr_passkeys SET passkey = $1 WHERE credential_id = $2")
                .bind(passkey_json)
                .bind(credential_id)
                .execute(&mut *transaction)
                .await
                .map_err(|_| invalid_auth())?;
            credential_updated = true;
        }
    }
    if !credential_updated {
        return Err(invalid_auth());
    }
    sqlx::query("DELETE FROM rawrarr_auth_challenges WHERE id = $1")
        .bind(request.challenge_id)
        .execute(&mut *transaction)
        .await
        .map_err(|_| invalid_auth())?;
    let token = issue_session(&mut transaction, user_id).await?;
    transaction.commit().await.map_err(|_| invalid_auth())?;
    Ok(session_response(token, auth.secure_cookie))
}

async fn issue_session(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: Uuid,
) -> Result<Zeroizing<String>, (StatusCode, Json<AuthError>)> {
    let token = Zeroizing::new(Uuid::new_v4().simple().to_string());
    let token_hash = Sha256::digest(token.as_bytes()).to_vec();
    sqlx::query(
        "INSERT INTO rawrarr_sessions (token_hash, user_id, expires_at) VALUES ($1, $2, now() + ($3 * interval '1 second'))",
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(SESSION_SECONDS)
    .execute(&mut **transaction)
    .await
    .map_err(|_| invalid_auth())?;
    Ok(token)
}

fn session_response(token: Zeroizing<String>, secure: bool) -> Response<Body> {
    let mut cookie = format!(
        "{SESSION_COOKIE}={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={SESSION_SECONDS}",
        token.as_str()
    );
    if secure {
        cookie.push_str("; Secure");
    }
    let mut response = Json(AuthComplete { next: "/setup/age" }).into_response();
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

fn cookie_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find_map(|(name, value)| (name == SESSION_COOKIE).then_some(value))
}

fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

pub(super) async fn require_session(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Response {
    let authenticated = if let Some(hash) = cookie_token(request.headers()).map(token_hash) {
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM rawrarr_sessions WHERE token_hash = $1 AND expires_at > now())",
        )
        .bind(hash)
        .fetch_one(&pool)
        .await
        .unwrap_or(false)
    } else {
        false
    };
    if authenticated {
        return next.run(request).await;
    }
    if request.uri().path() == "/" {
        Redirect::to("/auth").into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "Sign in with your passkey to continue",
            }),
        )
            .into_response()
    }
}

pub(super) async fn end_session(auth: &AuthState, headers: &HeaderMap) -> Response<Body> {
    if let Some(token) = cookie_token(headers) {
        let _ = sqlx::query("DELETE FROM rawrarr_sessions WHERE token_hash = $1")
            .bind(token_hash(token))
            .execute(&auth.pool)
            .await;
    }
    let mut response = Json(serde_json::json!({ "ok": true })).into_response();
    let secure = if auth.secure_cookie { "; Secure" } else { "" };
    let cookie = format!("{SESSION_COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{secure}");
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
    response
}

pub(super) async fn is_authenticated(pool: &PgPool, headers: &HeaderMap) -> bool {
    let Some(token) = cookie_token(headers) else {
        return false;
    };
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM rawrarr_sessions WHERE token_hash = $1 AND expires_at > now())",
    )
    .bind(token_hash(token))
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

pub(super) fn auth_html() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        axum::response::Html(include_str!("auth.html")),
    )
}

pub(super) fn age_setup_html() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        axum::response::Html(include_str!("age_setup.html")),
    )
}

#[cfg(test)]
mod tests {
    use super::normalize_username;

    #[test]
    fn usernames_are_normalized_and_restricted() {
        assert_eq!(normalize_username(" Owner_1 ").unwrap(), "owner_1");
        assert!(normalize_username("ab").is_err());
        assert!(normalize_username("contains space").is_err());
        assert!(normalize_username("contains/slash").is_err());
    }
}
