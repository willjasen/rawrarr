//! Coordinator-only ENS wallet authentication and Ethereum subscription payments.
//!
//! This module handles public addresses and public-chain transaction metadata only.
//! It never receives or persists wallet private keys or seed phrases.

use axum::{
    Extension, Json, Router,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha3::{Digest, Keccak256};
use sqlx::{PgPool, Row};
use std::{env, time::Duration};
use url::Url;
use uuid::Uuid;

const MAINNET_CHAIN_ID: &str = "0x1";
const DEFAULT_RPC_URL: &str = "https://ethereum-rpc.publicnode.com";
const ENS_UNIVERSAL_RESOLVER: &str = "0xeeeeeeeE14d718C2b47D9923Deab1335E144EeEe";
const SIWE_TTL_SECONDS: i64 = 300;
const MONTH_SECONDS: i64 = 30 * 24 * 60 * 60;
const YEAR_SECONDS: i64 = 365 * 24 * 60 * 60;
const FREE_FOSSIL_NODE_LIMIT: i64 = 1;

#[derive(Clone)]
pub(super) struct CoordinatorCryptoState {
    pool: PgPool,
    auth: crate::auth::AuthState,
    origin: String,
    rpc_url: Url,
    treasury: Option<String>,
    monthly_price_wei: Option<u128>,
    annual_price_wei: Option<u128>,
    client: reqwest::Client,
}

impl CoordinatorCryptoState {
    pub(super) fn new(pool: PgPool, origin: String, auth: crate::auth::AuthState) -> Result<Self, Box<dyn std::error::Error>> {
        let rpc_text = env::var("RAWARR_ETH_RPC_URL").unwrap_or_else(|_| DEFAULT_RPC_URL.to_owned());
        let rpc_url = Url::parse(&rpc_text)?;
        let loopback_http = rpc_url.scheme() == "http"
            && rpc_url.host_str().is_some_and(|host| matches!(host, "localhost" | "127.0.0.1" | "::1"));
        if (rpc_url.scheme() != "https" && !loopback_http)
            || !rpc_url.username().is_empty()
            || rpc_url.password().is_some()
            || rpc_url.query().is_some()
            || rpc_url.fragment().is_some()
        {
            return Err("RAWARR_ETH_RPC_URL must be an HTTPS endpoint without embedded credentials (HTTP loopback is allowed for development)".into());
        }
        let treasury = env::var("RAWARR_ETH_TREASURY_ADDRESS")
            .ok()
            .map(|value| normalize_address(&value))
            .transpose()?;
        let monthly_price_wei = env::var("RAWARR_ETH_MONTHLY_PRICE_WEI")
            .ok()
            .map(|value| parse_positive_u128(&value))
            .transpose()?;
        let annual_price_wei = env::var("RAWARR_ETH_ANNUAL_PRICE_WEI")
            .ok()
            .map(|value| parse_positive_u128(&value))
            .transpose()?;
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()?;
        Ok(Self {
            pool,
            auth,
            origin,
            rpc_url,
            treasury,
            monthly_price_wei,
            annual_price_wei,
            client,
        })
    }

    fn plan_price(&self, plan: &str) -> Option<(u128, i64)> {
        match plan {
            "monthly" => self.monthly_price_wei.map(|amount| (amount, MONTH_SECONDS)),
            "annual" => self.annual_price_wei.map(|amount| (amount, YEAR_SECONDS)),
            _ => None,
        }
    }

    fn subscriptions_enabled(&self) -> bool {
        self.treasury.is_some()
            && (self.monthly_price_wei.is_some() || self.annual_price_wei.is_some())
    }

    async fn rpc(&self, method: &str, params: Value) -> Result<Value, RpcError> {
        let response = self.client
            .post(self.rpc_url.clone())
            .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
            .send()
            .await
            .map_err(|_| RpcError::Unavailable)?;
        if !response.status().is_success() {
            return Err(RpcError::Unavailable);
        }
        let body: Value = response.json().await.map_err(|_| RpcError::Unavailable)?;
        if body.get("error").is_some() {
            return Err(RpcError::Unavailable);
        }
        body.get("result").cloned().ok_or(RpcError::Unavailable)
    }

    async fn ensure_mainnet(&self) -> Result<(), RpcError> {
        match self.rpc("eth_chainId", json!([])).await?.as_str() {
            Some(MAINNET_CHAIN_ID) => Ok(()),
            _ => Err(RpcError::WrongNetwork),
        }
    }

    async fn resolve_ens_address(&self, ens_name: &str) -> Result<String, RpcError> {
        self.ensure_mainnet().await?;
        let node = namehash(ens_name);
        let mut resolver_call = Vec::from([0x3b, 0x3b, 0x57, 0xde]); // addr(bytes32)
        resolver_call.extend_from_slice(&node);
        let calldata = encode_universal_resolve(ens_name, &resolver_call);
        let result = self.rpc(
            "eth_call",
            json!([{"to": ENS_UNIVERSAL_RESOLVER, "data": format!("0x{}", encode_hex(&calldata))}, "latest"]),
        ).await?;
        let encoded = result.as_str().ok_or(RpcError::NameUnresolved)?;
        let raw = decode_hex(encoded).ok_or(RpcError::NameUnresolved)?;
        decode_address_return(&raw).ok_or(RpcError::NameUnresolved)
    }

    async fn verify_subscription_tx(
        &self,
        tx_hash: &str,
        expected_from: &str,
        expected_amount: u128,
    ) -> Result<TxVerification, RpcError> {
        self.ensure_mainnet().await?;
        let tx_hash = normalize_tx_hash(tx_hash).map_err(|_| RpcError::InvalidTransaction)?;
        let transaction = self.rpc("eth_getTransactionByHash", json!([tx_hash])).await?;
        if transaction.is_null() {
            return Ok(TxVerification::Pending);
        }
        let tx_from = transaction.get("from").and_then(Value::as_str)
            .and_then(|value| normalize_address(value).ok()).ok_or(RpcError::InvalidTransaction)?;
        let tx_to = transaction.get("to").and_then(Value::as_str)
            .and_then(|value| normalize_address(value).ok()).ok_or(RpcError::InvalidTransaction)?;
        let tx_value = transaction.get("value").and_then(Value::as_str)
            .and_then(parse_hex_u128).ok_or(RpcError::InvalidTransaction)?;
        let input = transaction.get("input").and_then(Value::as_str).unwrap_or_default();
        let chain_id = transaction.get("chainId").and_then(Value::as_str);
        if tx_from != expected_from || Some(tx_to.as_str()) != self.treasury.as_deref()
            || tx_value != expected_amount || !matches!(input, "0x" | "0x0")
            || chain_id.is_some_and(|id| !id.eq_ignore_ascii_case(MAINNET_CHAIN_ID))
        {
            return Err(RpcError::InvalidTransaction);
        }
        let receipt = self.rpc("eth_getTransactionReceipt", json!([tx_hash])).await?;
        if receipt.is_null() {
            return Ok(TxVerification::Pending);
        }
        if receipt.get("status").and_then(Value::as_str) != Some("0x1") {
            return Err(RpcError::FailedTransaction);
        }
        let tx_block = transaction.get("blockNumber").and_then(Value::as_str)
            .and_then(parse_hex_u128).ok_or(RpcError::InvalidTransaction)?;
        let tx_block_hash = transaction.get("blockHash").and_then(Value::as_str)
            .ok_or(RpcError::InvalidTransaction)?;
        if receipt.get("blockNumber").and_then(Value::as_str).and_then(parse_hex_u128) != Some(tx_block)
            || !receipt.get("blockHash").and_then(Value::as_str).is_some_and(|value| value.eq_ignore_ascii_case(tx_block_hash))
        {
            return Ok(TxVerification::Pending);
        }
        let finalized = self.rpc("eth_getBlockByNumber", json!(["finalized", false])).await?;
        let finalized_number = finalized.get("number").and_then(Value::as_str)
            .and_then(parse_hex_u128).ok_or(RpcError::Unavailable)?;
        if finalized_number < tx_block {
            return Ok(TxVerification::Pending);
        }
        let block = self.rpc("eth_getBlockByNumber", json!([format!("0x{tx_block:x}"), false])).await?;
        if !block.get("hash").and_then(Value::as_str).is_some_and(|value| value.eq_ignore_ascii_case(tx_block_hash)) {
            return Ok(TxVerification::Pending);
        }
        Ok(TxVerification::Verified {
            tx_hash: tx_hash.to_owned(),
            payer: tx_from,
            amount_wei: tx_value,
            block_number: tx_block,
            block_hash: tx_block_hash.to_ascii_lowercase(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RpcError {
    Unavailable,
    WrongNetwork,
    NameUnresolved,
    InvalidTransaction,
    FailedTransaction,
}

enum TxVerification {
    Pending,
    Verified { tx_hash: String, payer: String, amount_wei: u128, block_number: u128, block_hash: String },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PaymentConfig {
    chain_id: u64,
    treasury_address: Option<String>,
    donation_enabled: bool,
    subscriptions_enabled: bool,
    monthly_price_wei: Option<String>,
    annual_price_wei: Option<String>,
    currency: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EnsNameRequest {
    ens_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SiweFinishRequest {
    challenge_id: Uuid,
    signature: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PaymentClaimRequest {
    tx_hash: String,
    plan: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DonationLinkRequest {
    amount_wei: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SiweChallenge {
    challenge_id: Uuid,
    address: String,
    message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccessStatus {
    tier: &'static str,
    fossil_nodes: i64,
    fossil_node_limit: Option<i64>,
    paid_through: Option<String>,
    linked_ens_name: Option<String>,
}

pub(super) fn router(state: CoordinatorCryptoState) -> Router {
    Router::new()
        .route("/api/crypto/config", get(payment_config))
        .route("/api/crypto/donation/link", post(donation_link))
        .route("/api/crypto/subscription/claim", post(claim_subscription))
        .route("/api/crypto/access", get(access_status))
        .route("/api/auth/eth/login/start", post(start_wallet_login))
        .route("/api/auth/eth/login/finish", post(finish_wallet_login))
        .route("/api/auth/eth/link/start", post(start_wallet_link))
        .route("/api/auth/eth/link/finish", post(finish_wallet_link))
        .route("/api/auth/eth/link", get(linked_wallet))
        .layer(middleware::from_fn_with_state(state.auth.clone(), auth_context))
        .with_state(state)
}

async fn auth_context(State(auth): State<crate::auth::AuthState>, mut request: Request, next: Next) -> Response {
    request.extensions_mut().insert(auth);
    next.run(request).await
}

async fn payment_config(State(state): State<CoordinatorCryptoState>) -> Json<PaymentConfig> {
    Json(PaymentConfig {
        chain_id: 1,
        treasury_address: state.treasury.clone(),
        donation_enabled: state.treasury.is_some(),
        subscriptions_enabled: state.subscriptions_enabled(),
        monthly_price_wei: state.monthly_price_wei.map(|price| price.to_string()),
        annual_price_wei: state.annual_price_wei.map(|price| price.to_string()),
        currency: "ETH",
    })
}

async fn donation_link(
    State(state): State<CoordinatorCryptoState>,
    Json(request): Json<DonationLinkRequest>,
) -> Response {
    if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
        return StatusCode::NOT_FOUND.into_response();
    }
    if state.treasury.is_none() || parse_positive_u128(&request.amount_wei).is_err() {
        return client_error("Enter a positive ETH amount while donations are configured");
    }
    if let Err(_) = state.ensure_mainnet().await { return unavailable(); }
    Json(json!({"recipient":state.treasury,"amountWei":request.amount_wei})).into_response()
}

async fn start_wallet_login(
    State(state): State<CoordinatorCryptoState>,
    Json(request): Json<EnsNameRequest>,
) -> Response {
    if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let ens_name = match normalize_ens_name(&request.ens_name) {
        Ok(name) => name,
        Err(()) => return client_error("Enter a valid ASCII .eth name, such as willjasen.eth"),
    };
    let address = match state.resolve_ens_address(&ens_name).await {
        Ok(address) => address,
        Err(RpcError::WrongNetwork | RpcError::Unavailable) => return unavailable(),
        Err(_) => return client_error("That ENS name does not currently resolve to an Ethereum address"),
    };
    let user_id = sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM rawrarr_eth_wallets WHERE address=$1")
        .bind(&address).fetch_optional(&state.pool).await;
    let user_id = match user_id {
        Ok(Some(id)) => id,
        Ok(None) => return client_error("This ENS wallet is not linked to a Rawrarr account. Sign in with your passkey to link it first."),
        Err(_) => return unavailable(),
    };
    create_challenge(&state, &address, &ens_name, "wallet_login", Some(user_id)).await
}

async fn finish_wallet_login(
    State(state): State<CoordinatorCryptoState>,
    Extension(auth): Extension<crate::auth::AuthState>,
    Json(request): Json<SiweFinishRequest>,
) -> Response {
    let challenge = match load_challenge(&state, request.challenge_id, "wallet_login").await {
        Ok(Some(row)) => row,
        Ok(None) => return client_error("This sign-in request expired or was already used. Start again."),
        Err(_) => return unavailable(),
    };
    let address = challenge.try_get::<String, _>("address").unwrap_or_default();
    let ens_name = challenge.try_get::<String, _>("ens_name").unwrap_or_default();
    let message = challenge.try_get::<String, _>("message").unwrap_or_default();
    if !verify_siwe_signature(&message, &request.signature, &address) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match state.resolve_ens_address(&ens_name).await {
        Ok(current) if current == address => {}
        Ok(_) | Err(RpcError::NameUnresolved | RpcError::InvalidTransaction | RpcError::FailedTransaction) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return unavailable(),
    }
    let mut tx = match state.pool.begin().await { Ok(tx) => tx, Err(_) => return unavailable() };
    let locked = sqlx::query("SELECT user_id,address,ens_name,message FROM rawrarr_eth_auth_challenges WHERE id=$1 AND flow='wallet_login' AND expires_at>now() AND consumed_at IS NULL FOR UPDATE")
        .bind(request.challenge_id).fetch_optional(&mut *tx).await;
    let Ok(Some(locked)) = locked else { return client_error("This sign-in request expired or was already used. Start again."); };
    let Some(user_id) = locked.try_get::<Option<Uuid>, _>("user_id").ok().flatten() else { return unavailable(); };
    if locked.try_get::<String,_>("address").ok().as_deref() != Some(address.as_str())
        || locked.try_get::<String,_>("ens_name").ok().as_deref() != Some(ens_name.as_str())
        || locked.try_get::<String,_>("message").ok().as_deref() != Some(message.as_str())
    { return StatusCode::UNAUTHORIZED.into_response(); }
    let mapped_user = sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM rawrarr_eth_wallets WHERE address=$1")
        .bind(&address).fetch_optional(&mut *tx).await;
    if !matches!(mapped_user, Ok(Some(mapped)) if mapped == user_id) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if sqlx::query("UPDATE rawrarr_eth_auth_challenges SET consumed_at=now() WHERE id=$1")
        .bind(request.challenge_id).execute(&mut *tx).await.is_err()
        || sqlx::query("UPDATE rawrarr_eth_wallets SET last_auth_at=now(), ens_name=$2 WHERE address=$1")
            .bind(&address).bind(&ens_name).execute(&mut *tx).await.is_err()
    {
        return unavailable();
    }
    let token = match crate::auth::issue_session(&mut tx, user_id, &auth.node_id).await {
        Ok(token) => token,
        Err(_) => return unavailable(),
    };
    if tx.commit().await.is_err() { return unavailable(); }
    crate::auth::session_response(token, auth.secure_cookie, auth.next_path)
}

async fn start_wallet_link(
    State(state): State<CoordinatorCryptoState>,
    Extension(auth): Extension<crate::auth::AuthState>,
    headers: HeaderMap,
    Json(request): Json<EnsNameRequest>,
) -> Response {
    if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let ens_name = match normalize_ens_name(&request.ens_name) {
        Ok(name) => name,
        Err(()) => return client_error("Enter a valid ASCII .eth name, such as willjasen.eth"),
    };
    let address = match state.resolve_ens_address(&ens_name).await {
        Ok(address) => address,
        Err(RpcError::WrongNetwork | RpcError::Unavailable) => return unavailable(),
        Err(_) => return client_error("That ENS name does not currently resolve to an Ethereum address"),
    };
    match sqlx::query_scalar::<_, Uuid>("SELECT user_id FROM rawrarr_eth_wallets WHERE address=$1")
        .bind(&address).fetch_optional(&state.pool).await
    {
        Ok(Some(existing)) if existing != user_id => return client_error("That wallet is already linked to another Rawrarr account"),
        Err(_) => return unavailable(),
        _ => {}
    }
    create_challenge(&state, &address, &ens_name, "wallet_link", Some(user_id)).await
}

async fn finish_wallet_link(
    State(state): State<CoordinatorCryptoState>,
    Extension(auth): Extension<crate::auth::AuthState>,
    headers: HeaderMap,
    Json(request): Json<SiweFinishRequest>,
) -> Response {
    if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(session_user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let challenge = match load_challenge(&state, request.challenge_id, "wallet_link").await {
        Ok(Some(row)) => row,
        Ok(None) => return client_error("This wallet-link request expired or was already used. Start again."),
        Err(_) => return unavailable(),
    };
    let address = challenge.try_get::<String, _>("address").unwrap_or_default();
    let ens_name = challenge.try_get::<String, _>("ens_name").unwrap_or_default();
    let message = challenge.try_get::<String, _>("message").unwrap_or_default();
    let challenge_user_id = challenge.try_get::<Option<Uuid>, _>("user_id").ok().flatten();
    if challenge_user_id != Some(session_user_id) || !verify_siwe_signature(&message, &request.signature, &address) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match state.resolve_ens_address(&ens_name).await {
        Ok(current) if current == address => {}
        Ok(_) | Err(RpcError::NameUnresolved | RpcError::InvalidTransaction | RpcError::FailedTransaction) => return StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => return unavailable(),
    }
    let mut tx = match state.pool.begin().await { Ok(tx) => tx, Err(_) => return unavailable() };
    let locked = sqlx::query("SELECT user_id,address,ens_name,message FROM rawrarr_eth_auth_challenges WHERE id=$1 AND flow='wallet_link' AND expires_at>now() AND consumed_at IS NULL FOR UPDATE")
        .bind(request.challenge_id).fetch_optional(&mut *tx).await;
    let Ok(Some(locked)) = locked else { return client_error("This wallet-link request expired or was already used. Start again."); };
    if locked.try_get::<Option<Uuid>, _>("user_id").ok().flatten() != Some(session_user_id) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if locked.try_get::<String,_>("address").ok().as_deref() != Some(address.as_str())
        || locked.try_get::<String,_>("ens_name").ok().as_deref() != Some(ens_name.as_str())
        || locked.try_get::<String,_>("message").ok().as_deref() != Some(message.as_str())
    { return StatusCode::UNAUTHORIZED.into_response(); }
    let inserted = sqlx::query("INSERT INTO rawrarr_eth_wallets(address,user_id,ens_name) VALUES ($1,$2,$3) ON CONFLICT(address) DO UPDATE SET ens_name=EXCLUDED.ens_name WHERE rawrarr_eth_wallets.user_id=EXCLUDED.user_id")
        .bind(&address).bind(session_user_id).bind(&ens_name).execute(&mut *tx).await;
    if !matches!(inserted, Ok(result) if result.rows_affected() == 1) {
        return client_error("That wallet is already linked to another Rawrarr account");
    }
    if sqlx::query("UPDATE rawrarr_eth_auth_challenges SET consumed_at=now() WHERE id=$1")
        .bind(request.challenge_id).execute(&mut *tx).await.is_err()
        || tx.commit().await.is_err()
    {
        return unavailable();
    }
    Json(json!({"ok":true,"ensName":ens_name,"address":address})).into_response()
}

async fn linked_wallet(
    State(state): State<CoordinatorCryptoState>,
    Extension(auth): Extension<crate::auth::AuthState>,
    headers: HeaderMap,
) -> Response {
    if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let rows = match sqlx::query("SELECT address,ens_name FROM rawrarr_eth_wallets WHERE user_id=$1 ORDER BY linked_at")
        .bind(user_id).fetch_all(&state.pool).await
    {
        Ok(rows) => rows,
        Err(_) => return unavailable(),
    };
    let wallets = rows.into_iter().filter_map(|row| Some(json!({
        "address": row.try_get::<String,_>("address").ok()?,
        "ensName": row.try_get::<String,_>("ens_name").ok()?
    }))).collect::<Vec<_>>();
    Json(json!({"wallets":wallets})).into_response()
}

async fn create_challenge(
    state: &CoordinatorCryptoState,
    address: &str,
    ens_name: &str,
    flow: &str,
    user_id: Option<Uuid>,
) -> Response {
    let challenge_id = Uuid::new_v4();
    let nonce = Uuid::new_v4().simple().to_string();
    let message = match siwe_message(&state.origin, address, ens_name, &nonce) {
        Some(message) => message,
        None => return unavailable(),
    };
    if sqlx::query("DELETE FROM rawrarr_eth_auth_challenges WHERE expires_at<=now() OR consumed_at IS NOT NULL")
        .execute(&state.pool).await.is_err()
        || sqlx::query("INSERT INTO rawrarr_eth_auth_challenges(id,flow,address,ens_name,message,nonce,user_id,expires_at) VALUES ($1,$2,$3,$4,$5,$6,$7,now()+($8*interval '1 second'))")
            .bind(challenge_id).bind(flow).bind(address).bind(ens_name).bind(&message).bind(nonce)
            .bind(user_id).bind(SIWE_TTL_SECONDS).execute(&state.pool).await.is_err()
    {
        return unavailable();
    }
    let mut response = Json(SiweChallenge { challenge_id, address: address.to_owned(), message }).into_response();
    response.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn load_challenge(
    state: &CoordinatorCryptoState,
    challenge_id: Uuid,
    flow: &str,
) -> Result<Option<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query("SELECT address,ens_name,message,user_id FROM rawrarr_eth_auth_challenges WHERE id=$1 AND flow=$2 AND expires_at>now() AND consumed_at IS NULL")
        .bind(challenge_id).bind(flow).fetch_optional(&state.pool).await
}

async fn claim_subscription(
    State(state): State<CoordinatorCryptoState>,
    Extension(auth): Extension<crate::auth::AuthState>,
    headers: HeaderMap,
    Json(request): Json<PaymentClaimRequest>,
) -> Response {
    if !state.subscriptions_enabled() {
        return client_error("ETH subscriptions are not configured yet");
    }
    if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Some(user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let Some((amount, period_seconds)) = state.plan_price(&request.plan) else {
        return client_error("Select a configured monthly or annual plan");
    };
    let plan = request.plan.clone();
    let tx_hash = match normalize_tx_hash(&request.tx_hash) {
        Ok(value) => value,
        Err(()) => return client_error("Enter a valid Ethereum transaction hash"),
    };
    let payer = match sqlx::query_scalar::<_, String>("SELECT address FROM rawrarr_eth_wallets WHERE user_id=$1 ORDER BY linked_at LIMIT 1")
        .bind(user_id).fetch_optional(&state.pool).await
    {
        Ok(Some(address)) => address,
        Ok(None) => return client_error("Link an ENS wallet before subscribing"),
        Err(_) => return unavailable(),
    };
    match sqlx::query("SELECT account_id FROM rawrarr_eth_payments WHERE tx_hash=$1")
        .bind(&tx_hash).fetch_optional(&state.pool).await
    {
        Ok(Some(row)) if row.try_get::<Uuid,_>("account_id").ok() == Some(user_id) => {
            return subscription_response(&state, user_id).await;
        }
        Ok(Some(_)) => return client_error("That transaction has already been used"),
        Err(_) => return unavailable(),
        _ => {}
    }
    let verified = match state.verify_subscription_tx(&tx_hash, &payer, amount).await {
        Ok(TxVerification::Pending) => return (StatusCode::ACCEPTED, Json(json!({"status":"pending","message":"Waiting for a finalized Ethereum mainnet transaction."}))).into_response(),
        Ok(TxVerification::Verified { tx_hash, payer, amount_wei, block_number, block_hash }) => (tx_hash, payer, amount_wei, block_number, block_hash),
        Err(RpcError::WrongNetwork | RpcError::Unavailable) => return unavailable(),
        Err(RpcError::FailedTransaction) => return client_error("The Ethereum transaction failed"),
        Err(_) => return client_error("The transaction does not match the selected ETH subscription plan"),
    };
    let (tx_hash, payer, amount_wei, block_number, block_hash) = verified;
    let mut tx = match state.pool.begin().await { Ok(tx) => tx, Err(_) => return unavailable() };
    let prior = sqlx::query("SELECT account_id FROM rawrarr_eth_payments WHERE tx_hash=$1 FOR UPDATE")
        .bind(&tx_hash).fetch_optional(&mut *tx).await;
    match prior {
        Ok(Some(row)) if row.try_get::<Uuid,_>("account_id").ok() == Some(user_id) => {
            return match tx.commit().await { Ok(()) => subscription_response(&state,user_id).await, Err(_) => unavailable() };
        }
        Ok(Some(_)) => return client_error("That transaction has already been used"),
        Err(_) => return unavailable(),
        _ => {}
    }
    let inserted = sqlx::query("INSERT INTO rawrarr_eth_payments(tx_hash,account_id,payer_address,plan,amount_wei,recipient_address,block_number,block_hash) VALUES ($1,$2,$3,$4,$5::numeric,$6,$7::numeric,$8) ON CONFLICT(tx_hash) DO NOTHING")
        .bind(&tx_hash).bind(user_id).bind(&payer).bind(&plan).bind(amount_wei.to_string())
        .bind(state.treasury.as_deref().unwrap_or_default()).bind(block_number.to_string()).bind(&block_hash)
        .execute(&mut *tx).await;
    match inserted {
        Ok(result) if result.rows_affected() == 1 => {}
        Ok(_) => return client_error("That transaction has already been used"),
        Err(_) => return unavailable(),
    }
    let updated = sqlx::query("UPDATE rawrarr_eth_subscriptions SET paid_through=GREATEST(paid_through,now())+($2*interval '1 second'),updated_at=now() WHERE account_id=$1")
        .bind(user_id).bind(period_seconds).execute(&mut *tx).await;
    match updated {
        Ok(result) if result.rows_affected() == 1 => {}
        Ok(_) => if sqlx::query("INSERT INTO rawrarr_eth_subscriptions(account_id,paid_through) VALUES ($1,now()+($2*interval '1 second'))")
            .bind(user_id).bind(period_seconds).execute(&mut *tx).await.is_err() { return unavailable(); },
        Err(_) => return unavailable(),
    }
    if tx.commit().await.is_err() { return unavailable(); }
    subscription_response(&state, user_id).await
}

async fn subscription_response(state: &CoordinatorCryptoState, user_id: Uuid) -> Response {
    match sqlx::query("SELECT to_char(paid_through AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS paid_through FROM rawrarr_eth_subscriptions WHERE account_id=$1")
        .bind(user_id).fetch_optional(&state.pool).await
    {
        Ok(Some(row)) => Json(json!({"status":"paid","paidThrough":row.try_get::<String,_>("paid_through").ok()})).into_response(),
        Ok(None) => unavailable(),
        Err(_) => unavailable(),
    }
}

async fn access_status(
    State(state): State<CoordinatorCryptoState>,
    Extension(auth): Extension<crate::auth::AuthState>,
    headers: HeaderMap,
) -> Response {
    let Some(user_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let row = sqlx::query("SELECT (u.trial_started_at IS NOT NULL AND u.trial_started_at+interval '3 days'>now()) AS trial_active, CASE WHEN s.paid_through>now() THEN to_char(s.paid_through AT TIME ZONE 'UTC','YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') ELSE NULL END AS paid_through, (SELECT count(*) FROM rawrarr_nodes n WHERE n.user_id=u.id AND n.role='metadata') AS fossil_nodes, (SELECT ens_name FROM rawrarr_eth_wallets w WHERE w.user_id=u.id ORDER BY linked_at LIMIT 1) AS ens_name FROM rawrarr_users u LEFT JOIN rawrarr_eth_subscriptions s ON s.account_id=u.id WHERE u.id=$1")
        .bind(user_id).fetch_optional(&state.pool).await;
    let Ok(Some(row)) = row else { return unavailable(); };
    let trial_active_from_db = row.try_get::<bool,_>("trial_active").unwrap_or(false);
    let paid_through = row.try_get::<Option<String>,_>("paid_through").ok().flatten();
    let fossil_nodes = row.try_get::<i64,_>("fossil_nodes").unwrap_or_default();
    let is_public = env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public");
    let trial_active = is_public && trial_active_from_db;
    let paid_active = paid_through.is_some();
    let (tier, fossil_node_limit) = if !is_public || trial_active || paid_active {
        (if !is_public { "internal" } else if paid_active { "paid" } else { "trial" }, None)
    } else {
        ("free", Some(FREE_FOSSIL_NODE_LIMIT))
    };
    let linked_ens_name = row.try_get::<Option<String>,_>("ens_name").ok().flatten();
    Json(AccessStatus { tier, fossil_nodes, fossil_node_limit, paid_through, linked_ens_name }).into_response()
}

pub(super) async fn free_tier_denies_fossil_node(pool: &PgPool, user_id: Uuid, current_node_id: Option<&str>) -> Result<bool, sqlx::Error> {
    if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
        return Ok(false);
    }
    let count = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM (SELECT id AS node_id FROM rawrarr_nodes WHERE user_id=$1 AND role='metadata' UNION SELECT node_id FROM rawrarr_node_enrollments WHERE user_id=$1 AND role='metadata' AND approved_at IS NULL AND expires_at>now() AND node_id<>COALESCE($2,'')) fossil_nodes")
        .bind(user_id).bind(current_node_id).fetch_one(pool).await?;
    if count < FREE_FOSSIL_NODE_LIMIT {
        return Ok(false);
    }
    let entitled = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_users u WHERE u.id=$1 AND u.trial_started_at IS NOT NULL AND u.trial_started_at+interval '3 days'>now()) OR EXISTS(SELECT 1 FROM rawrarr_eth_subscriptions WHERE account_id=$1 AND paid_through>now())")
        .bind(user_id).fetch_one(pool).await?;
    Ok(!entitled)
}

pub(super) async fn free_tier_allows_existing_fossil(pool: &PgPool, user_id: Uuid, node_id: &str) -> Result<bool, sqlx::Error> {
    if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
        return Ok(true);
    }
    let allowed = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM rawrarr_users u WHERE u.id=$1 AND u.trial_started_at IS NOT NULL AND u.trial_started_at+interval '3 days'>now()) OR EXISTS(SELECT 1 FROM rawrarr_eth_subscriptions WHERE account_id=$1 AND paid_through>now()) OR $2=(SELECT min(n.id) FROM rawrarr_nodes n WHERE n.user_id=$1 AND n.role='metadata')")
        .bind(user_id).bind(node_id).fetch_one(pool).await?;
    Ok(allowed)
}

fn normalize_ens_name(input: &str) -> Result<String, ()> {
    let trimmed = input.trim();
    let without_one_dot = trimmed.strip_suffix('.').unwrap_or(trimmed);
    let name = without_one_dot;
    if name.ends_with('.') { return Err(()); }
    if !name.is_ascii() || name.len() > 255 || !name.to_ascii_lowercase().ends_with(".eth") {
        return Err(());
    }
    let name = name.to_ascii_lowercase();
    let labels = name.split('.').collect::<Vec<_>>();
    if labels.len() < 2 || labels.last() != Some(&"eth")
        || labels.iter().any(|label| label.is_empty() || label.len() > 63
            || label.starts_with('-') || label.ends_with('-')
            || !label.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-'))
    {
        return Err(());
    }
    Ok(name)
}

fn siwe_message(origin: &str, address: &str, ens_name: &str, nonce: &str) -> Option<String> {
    let origin = Url::parse(origin).ok()?;
    let domain = origin.host_str()?;
    let port = origin.port().map(|port| format!(":{port}")).unwrap_or_default();
    let issued_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let expiration = (chrono::Utc::now() + chrono::Duration::seconds(SIWE_TTL_SECONDS))
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let uri = format!("{}/auth", origin.as_str().trim_end_matches('/'));
    Some(format!(
        "{domain}{port} wants you to sign in with your Ethereum account:\n{address}\n\nSign in to rawrarr with ENS name {ens_name}.\n\nURI: {uri}\nVersion: 1\nChain ID: 1\nNonce: {nonce}\nIssued At: {issued_at}\nExpiration Time: {expiration}"
    ))
}

fn verify_siwe_signature(message: &str, signature: &str, expected_address: &str) -> bool {
    let Some(bytes) = decode_hex(signature) else { return false; };
    if bytes.len() != 65 { return false; }
    let Ok(signature) = Signature::from_slice(&bytes[..64]) else { return false; };
    let recovery = match bytes[64] {
        0 | 1 => bytes[64],
        27 | 28 => bytes[64] - 27,
        _ => return false,
    };
    let Ok(recovery_id) = RecoveryId::try_from(recovery) else { return false; };
    let digest = eip191_hash(message.as_bytes());
    let Ok(key) = VerifyingKey::recover_from_prehash(&digest, &signature, recovery_id) else { return false; };
    verifying_key_address(&key).is_some_and(|address| address == expected_address)
}

fn eip191_hash(message: &[u8]) -> [u8; 32] {
    let prefix = format!("\x19Ethereum Signed Message:\n{}", message.len());
    let mut hasher = Keccak256::new();
    hasher.update(prefix.as_bytes());
    hasher.update(message);
    hasher.finalize().into()
}

fn verifying_key_address(key: &VerifyingKey) -> Option<String> {
    let encoded = key.to_encoded_point(false);
    let bytes = encoded.as_bytes();
    if bytes.len() != 65 { return None; }
    let hash = Keccak256::digest(&bytes[1..]);
    Some(format!("0x{}", encode_hex(&hash[12..])))
}

fn namehash(name: &str) -> [u8; 32] {
    let mut node = [0u8; 32];
    for label in name.split('.').rev() {
        let label_hash = Keccak256::digest(label.as_bytes());
        let mut hasher = Keccak256::new();
        hasher.update(node);
        hasher.update(label_hash);
        node = hasher.finalize().into();
    }
    node
}

fn encode_universal_resolve(name: &str, resolver_data: &[u8]) -> Vec<u8> {
    let mut dns_name = Vec::new();
    for label in name.split('.') {
        dns_name.push(label.len() as u8);
        dns_name.extend_from_slice(label.as_bytes());
    }
    dns_name.push(0);
    let padded_name_len = (dns_name.len() + 31) / 32 * 32;
    let padded_data_len = (resolver_data.len() + 31) / 32 * 32;
    let mut arguments = Vec::with_capacity(64 + 64 + padded_name_len + padded_data_len);
    arguments.extend_from_slice(&word(64));
    arguments.extend_from_slice(&word((64 + 32 + padded_name_len) as u128));
    arguments.extend_from_slice(&word(dns_name.len() as u128));
    arguments.extend_from_slice(&dns_name);
    arguments.resize(64 + 32 + padded_name_len, 0);
    arguments.extend_from_slice(&word(resolver_data.len() as u128));
    arguments.extend_from_slice(resolver_data);
    arguments.resize(64 + 32 + padded_name_len + 32 + padded_data_len, 0);
    let mut calldata = vec![0x90, 0x61, 0xb9, 0x23]; // resolve(bytes,bytes)
    calldata.extend(arguments);
    calldata
}

fn decode_address_return(data: &[u8]) -> Option<String> {
    if data.len() < 64 { return None; }
    let offset = usize::try_from(u128::from_be_bytes(data[16..32].try_into().ok()?)).ok()?;
    if offset.checked_add(64)? > data.len() { return None; }
    let length = usize::try_from(u128::from_be_bytes(data[offset..offset + 32][16..32].try_into().ok()?)).ok()?;
    if length < 32 || offset.checked_add(32 + length)? > data.len() { return None; }
    let address_word = &data[offset + 32..offset + 64];
    if address_word[..12].iter().any(|byte| *byte != 0) { return None; }
    Some(format!("0x{}", encode_hex(&address_word[12..])))
}

fn word(value: u128) -> [u8; 32] {
    let mut output = [0u8; 32];
    output[16..].copy_from_slice(&value.to_be_bytes());
    output
}

fn normalize_address(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let raw = input.strip_prefix("0x").or_else(|| input.strip_prefix("0X")).ok_or("address must start with 0x")?;
    if raw.len() != 40 || !raw.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("address must contain exactly 40 hexadecimal characters".into());
    }
    Ok(format!("0x{}", raw.to_ascii_lowercase()))
}

fn normalize_tx_hash(input: &str) -> Result<String, ()> {
    let raw = input.strip_prefix("0x").ok_or(())?;
    if raw.len() != 64 || !raw.bytes().all(|byte| byte.is_ascii_hexdigit()) { return Err(()); }
    Ok(format!("0x{}", raw.to_ascii_lowercase()))
}

fn parse_positive_u128(input: &str) -> Result<u128, Box<dyn std::error::Error>> {
    let value = input.parse::<u128>()?;
    if value == 0 { return Err("ETH price must be greater than zero".into()); }
    Ok(value)
}

fn parse_hex_u128(input: &str) -> Option<u128> {
    u128::from_str_radix(input.strip_prefix("0x")?, 16).ok()
}

fn decode_hex(input: &str) -> Option<Vec<u8>> {
    let raw = input.strip_prefix("0x").unwrap_or(input);
    if raw.len() % 2 != 0 { return None; }
    raw.as_bytes().chunks_exact(2).map(|pair| Some((hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?)).collect()
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes { result.push(HEX[(byte >> 4) as usize] as char); result.push(HEX[(byte & 15) as usize] as char); }
    result
}

fn client_error(message: &'static str) -> Response {
    (StatusCode::BAD_REQUEST, Json(json!({"error":message}))).into_response()
}

fn unavailable() -> Response {
    (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error":"Ethereum service is temporarily unavailable"}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use k256::ecdsa::SigningKey;

    fn sign(message: &str, key: &SigningKey) -> String {
        let digest = eip191_hash(message.as_bytes());
        let (signature, recovery_id) = key.sign_prehash_recoverable(&digest).unwrap();
        let mut bytes = signature.to_bytes().to_vec();
        bytes.push(recovery_id.to_byte());
        format!("0x{}", encode_hex(&bytes))
    }

    fn address_for(key: &SigningKey) -> String {
        verifying_key_address(key.verifying_key()).unwrap()
    }

    async fn verifier_with_rpc_responses(results: Vec<Value>) -> (CoordinatorCryptoState, tokio::task::JoinHandle<()>) {
        use std::sync::{Arc, Mutex};
        let responses = Arc::new(Mutex::new(results));
        let app = Router::new().route("/", post(mock_rpc)).with_state(responses);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://rawrarr:unused@127.0.0.1/unused")
            .unwrap();
        let web_origin = Url::parse("https://rawrarr.example").unwrap();
        let webauthn = webauthn_rs::prelude::WebauthnBuilder::new("rawrarr.example", &web_origin)
            .unwrap().rp_name("rawrarr").build().unwrap();
        let auth = crate::auth::AuthState {
            pool: pool.clone(), webauthn, secure_cookie: true,
            node_id: "coordinator".to_owned(), next_path: "/",
        };
        let state = CoordinatorCryptoState {
            pool, auth, origin: "https://rawrarr.example".to_owned(),
            rpc_url: Url::parse(&format!("http://{address}/")).unwrap(),
            treasury: Some("0x2222222222222222222222222222222222222222".to_owned()),
            monthly_price_wei: Some(25_000_000_000_000_000), annual_price_wei: None,
            client: reqwest::Client::new(),
        };
        (state, server)
    }

    async fn mock_rpc(
        State(responses): State<std::sync::Arc<std::sync::Mutex<Vec<Value>>>>,
        Json(_request): Json<Value>,
    ) -> Json<Value> {
        let result = responses.lock().unwrap().remove(0);
        Json(json!({"jsonrpc":"2.0","id":1,"result":result}))
    }

    fn finalized_rpc_responses() -> Vec<Value> {
        vec![
            json!("0x1"),
            json!({"from":"0x1111111111111111111111111111111111111111","to":"0x2222222222222222222222222222222222222222","value":"0x58d15e17628000","input":"0x","chainId":"0x1","blockNumber":"0x64","blockHash":format!("0x{}","a".repeat(64))}),
            json!({"status":"0x1","blockNumber":"0x64","blockHash":format!("0x{}","a".repeat(64))}),
            json!({"number":"0x65"}),
            json!({"hash":format!("0x{}","a".repeat(64))}),
        ]
    }

    #[tokio::test]
    async fn rpc_subscription_claim_requires_finalized_canonical_exact_mainnet_transfer() {
        let (state, server) = verifier_with_rpc_responses(finalized_rpc_responses()).await;
        let result = state.verify_subscription_tx(
            &format!("0x{}", "b".repeat(64)),
            "0x1111111111111111111111111111111111111111",
            25_000_000_000_000_000,
        ).await.unwrap();
        assert!(matches!(result, TxVerification::Verified { block_number: 100, .. }));
        server.abort();
    }

    #[tokio::test]
    async fn rpc_subscription_claim_stays_pending_until_finality_and_matching_canonical_block() {
        let mut unfinalized = finalized_rpc_responses();
        unfinalized[3] = json!({"number":"0x63"});
        let (state, server) = verifier_with_rpc_responses(unfinalized).await;
        let result = state.verify_subscription_tx(&format!("0x{}", "b".repeat(64)), "0x1111111111111111111111111111111111111111", 25_000_000_000_000_000).await.unwrap();
        assert!(matches!(result, TxVerification::Pending));
        server.abort();

        let mut reorged = finalized_rpc_responses();
        reorged[4] = json!({"hash":format!("0x{}","c".repeat(64))});
        let (state, server) = verifier_with_rpc_responses(reorged).await;
        let result = state.verify_subscription_tx(&format!("0x{}", "b".repeat(64)), "0x1111111111111111111111111111111111111111", 25_000_000_000_000_000).await.unwrap();
        assert!(matches!(result, TxVerification::Pending));
        server.abort();
    }

    #[tokio::test]
    async fn rpc_subscription_claim_rejects_wrong_amount_and_failed_receipt() {
        let mut wrong_amount = finalized_rpc_responses();
        wrong_amount[1]["value"] = json!("0x1");
        let (state, server) = verifier_with_rpc_responses(wrong_amount).await;
        let result = state.verify_subscription_tx(&format!("0x{}", "b".repeat(64)), "0x1111111111111111111111111111111111111111", 25_000_000_000_000_000).await;
        assert!(matches!(result, Err(RpcError::InvalidTransaction)));
        server.abort();

        let mut failed_receipt = finalized_rpc_responses();
        failed_receipt[2]["status"] = json!("0x0");
        let (state, server) = verifier_with_rpc_responses(failed_receipt).await;
        let result = state.verify_subscription_tx(&format!("0x{}", "b".repeat(64)), "0x1111111111111111111111111111111111111111", 25_000_000_000_000_000).await;
        assert!(matches!(result, Err(RpcError::FailedTransaction)));
        server.abort();
    }

    #[test]
    fn siwe_message_is_bound_to_origin_chain_nonce_and_expiration() {
        let message = siwe_message("https://rawrarr.example", "0x1111111111111111111111111111111111111111", "willjasen.eth", "0123456789abcdef").unwrap();
        assert!(message.starts_with("rawrarr.example wants you to sign in with your Ethereum account:"));
        assert!(message.contains("URI: https://rawrarr.example/auth"));
        assert!(message.contains("Chain ID: 1\nNonce: 0123456789abcdef"));
        assert!(message.contains("Expiration Time:"));
        assert!(!message.contains("seed phrase"));
    }

    #[test]
    fn verifies_personal_sign_and_rejects_tampering_bad_recovery_and_another_address() {
        let key = SigningKey::from_bytes((&[7u8; 32]).into()).unwrap();
        let address = address_for(&key);
        let message = "example.eth wants you to sign in";
        let signature = sign(message, &key);
        assert!(verify_siwe_signature(message, &signature, &address));
        assert!(!verify_siwe_signature("changed", &signature, &address));
        assert!(!verify_siwe_signature(message, &signature, "0x1111111111111111111111111111111111111111"));
        assert!(!verify_siwe_signature(message, "0x00", &address));
        let mut bad_v = decode_hex(&signature).unwrap();
        bad_v[64] = 35;
        assert!(!verify_siwe_signature(message, &format!("0x{}", encode_hex(&bad_v)), &address));
    }

    #[test]
    fn ens_names_are_ascii_normalized_and_invalid_names_fail_closed() {
        assert_eq!(normalize_ens_name("  WillJasen.ETH. ").unwrap(), "willjasen.eth");
        assert!(normalize_ens_name("willjasen.com").is_err());
        assert!(normalize_ens_name("willjasen..eth").is_err());
        assert!(normalize_ens_name("willjasen.eth.elsewhere").is_err());
        assert!(normalize_ens_name("wіlljasen.eth").is_err());
        assert!(normalize_ens_name("-willjasen.eth").is_err());
    }

    #[test]
    fn ens_namehash_and_universal_resolver_abi_are_stable() {
        assert_eq!(encode_hex(&namehash("eth")), "93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae");
        let call = encode_universal_resolve("willjasen.eth", &[0x3b,0x3b,0x57,0xde]);
        assert_eq!(&call[..4], &[0x90,0x61,0xb9,0x23]);
        assert_eq!(&call[4..36], &word(64));
        let dns_name = &call[4 + 64 + 32..];
        assert!(dns_name.starts_with(&[9, b'w',b'i',b'l',b'l',b'j',b'a',b's',b'e',b'n', 3,b'e',b't',b'h',0]));
    }

    #[test]
    fn universal_resolver_output_rejects_truncated_or_non_address_data() {
        let address = decode_hex("0000000000000000000000001234567890123456789012345678901234567890").unwrap();
        let mut result = Vec::new();
        result.extend_from_slice(&word(64));
        result.extend_from_slice(&[0u8;32]);
        result.extend_from_slice(&word(32));
        result.extend_from_slice(&address);
        assert_eq!(decode_address_return(&result).as_deref(), Some("0x1234567890123456789012345678901234567890"));
        assert!(decode_address_return(&result[..63]).is_none());
        assert!(decode_address_return(&[]).is_none());
    }

    #[test]
    fn crypto_values_are_strictly_normalized() {
        assert_eq!(normalize_tx_hash(&format!("0x{}", "AB".repeat(32))).unwrap(), format!("0x{}", "ab".repeat(32)));
        assert!(normalize_tx_hash("0x1234").is_err());
        assert_eq!(parse_hex_u128("0xde0b6b3a7640000"), Some(1_000_000_000_000_000_000));
        assert_eq!(parse_hex_u128("0x"), None);
        assert_eq!(parse_positive_u128("0").is_err(), true);
        assert!(normalize_address("0x0000000000000000000000000000000000000000").is_ok());
    }
}
