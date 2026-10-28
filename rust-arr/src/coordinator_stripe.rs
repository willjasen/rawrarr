use super::CoordinatorState;
use crate::auth::AuthState;
use axum::{
    Extension, Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    http::HeaderValue,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;
use std::{env, fs, sync::Arc, time::Duration};
use url::Url;
use uuid::Uuid;
use zeroize::Zeroizing;

const STRIPE_API: &str = "https://api.stripe.com/v1";

fn no_store(response: impl IntoResponse) -> Response {
    let mut response = response.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

#[derive(Clone)]
pub(super) struct BillingState {
    config: Option<Arc<TestBillingConfig>>,
    client: reqwest::Client,
}

struct TestBillingConfig {
    secret_key: Zeroizing<String>,
    price_id: String,
}

impl BillingState {
    pub(super) fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let secret_path = env::var("STRIPE_TEST_SECRET_KEY_FILE").ok();
        let price_path = env::var("STRIPE_TEST_PRICE_ID_FILE").ok();
        let config = match (secret_path, price_path) {
            (None, None) => None,
            (Some(secret_path), Some(price_path)) => {
                let secret_key = Zeroizing::new(fs::read_to_string(secret_path)?);
                let price_id = fs::read_to_string(price_path)?.trim().to_owned();
                let config = test_config(secret_key, price_id)?;
                if !env::var("RAWARR_BILLING_AUDIENCE").is_ok_and(|value| value == "public") {
                    return Err("Stripe Test-mode checkout is only enabled for the public billing audience".into());
                }
                Some(Arc::new(config))
            }
            _ => return Err("Stripe Test-mode key and price must be configured together".into()),
        };
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .build()?;
        Ok(Self { config, client })
    }

    fn config(&self) -> Option<&TestBillingConfig> {
        self.config.as_deref()
    }
}

fn test_config(secret_key: Zeroizing<String>, price_id: String) -> Result<TestBillingConfig, &'static str> {
    let secret_key = Zeroizing::new(secret_key.trim().to_owned());
    let price_id = price_id.trim();
    if !secret_key.starts_with("sk_test_") {
        return Err("Stripe account checkout requires a Test-mode secret key");
    }
    if !price_id.starts_with("price_")
        || price_id.len() < 12
        || !price_id.chars().all(|character| character.is_ascii_alphanumeric() || character == '_')
    {
        return Err("Stripe test price file must contain a Stripe Price ID");
    }
    Ok(TestBillingConfig {
        secret_key,
        price_id: price_id.to_owned(),
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BillingStatus {
    checkout_enabled: bool,
    customer_linked: bool,
    subscription_status: Option<String>,
    current_period_end: Option<String>,
    cancel_at_period_end: bool,
}

struct AccountBillingRecord {
    customer_id: Option<String>,
    subscription_status: Option<String>,
    current_period_end: Option<String>,
    cancel_at_period_end: bool,
}

async fn account_billing_record(
    state: &CoordinatorState,
    account_id: Uuid,
) -> Result<Option<AccountBillingRecord>, sqlx::Error> {
    let row = sqlx::query(
        "SELECT c.stripe_customer_id, s.status, to_char(s.current_period_end AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') AS current_period_end, COALESCE(s.cancel_at_period_end, FALSE) AS cancel_at_period_end FROM rawrarr_users u LEFT JOIN rawrarr_stripe_customers c ON c.account_id=u.id LEFT JOIN LATERAL (SELECT status,current_period_end,cancel_at_period_end FROM rawrarr_stripe_subscriptions WHERE account_id=u.id ORDER BY stripe_event_created_at DESC LIMIT 1) s ON TRUE WHERE u.id=$1",
    )
    .bind(account_id)
    .fetch_optional(&state.pool)
    .await?;
    row.map(|row| {
        Ok(AccountBillingRecord {
            customer_id: row.try_get("stripe_customer_id")?,
            subscription_status: row.try_get("status")?,
            current_period_end: row.try_get("current_period_end")?,
            cancel_at_period_end: row.try_get("cancel_at_period_end")?,
        })
    })
    .transpose()
}

pub(super) async fn status(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
) -> Response {
    let Some(account_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return no_store(StatusCode::UNAUTHORIZED);
    };
    let Some(_) = state.stripe_billing.config() else {
        return no_store(Json(BillingStatus {
            checkout_enabled: false,
            customer_linked: false,
            subscription_status: None,
            current_period_end: None,
            cancel_at_period_end: false,
        }));
    };
    match account_billing_record(&state, account_id).await {
        Ok(Some(record)) => no_store(Json(BillingStatus {
            checkout_enabled: true,
            customer_linked: record.customer_id.is_some(),
            subscription_status: record.subscription_status,
            current_period_end: record.current_period_end,
            cancel_at_period_end: record.cancel_at_period_end,
        })),
        Ok(None) => no_store(StatusCode::NOT_FOUND),
        Err(_) => no_store(StatusCode::SERVICE_UNAVAILABLE),
    }
}

#[derive(Serialize)]
struct HostedSession {
    url: String,
}

fn same_origin(state: &CoordinatorState, headers: &HeaderMap) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        == Some(state.web_origin.as_str())
}

fn hosted_stripe_url(value: &str, allowed_host: &str) -> Option<String> {
    let url = Url::parse(value).ok()?;
    (url.scheme() == "https" && url.host_str() == Some(allowed_host)).then(|| url.to_string())
}

fn checkout_params(
    origin: &str,
    price_id: &str,
    account_id: Uuid,
    customer_id: Option<&str>,
) -> Vec<(String, String)> {
    let account_id = account_id.to_string();
    let mut params = vec![
        ("mode".into(), "subscription".into()),
        ("line_items[0][price]".into(), price_id.to_owned()),
        ("line_items[0][quantity]".into(), "1".into()),
        ("client_reference_id".into(), account_id.clone()),
        ("metadata[rawrarr_account_id]".into(), account_id.clone()),
        (
            "subscription_data[metadata][rawrarr_account_id]".into(),
            account_id,
        ),
        (
            "success_url".into(),
            format!("{origin}/billing?billing=success"),
        ),
        (
            "cancel_url".into(),
            format!("{origin}/billing?billing=cancelled"),
        ),
    ];
    if let Some(customer_id) = customer_id {
        params.push(("customer".into(), customer_id.to_owned()));
    }
    params
}

async fn create_session(
    state: &CoordinatorState,
    path: &str,
    params: &[(String, String)],
    allowed_host: &str,
) -> Result<String, StatusCode> {
    let config = state.stripe_billing.config().ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let response = state
        .stripe_billing
        .client
        .post(format!("{STRIPE_API}/{path}"))
        .bearer_auth(config.secret_key.as_str())
        .form(params)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let value: Value = response.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let url = value
        .get("url")
        .and_then(Value::as_str)
        .and_then(|url| hosted_stripe_url(url, allowed_host))
        .ok_or(StatusCode::BAD_GATEWAY)?;
    Ok(url)
}

async fn test_price_is_one_usd_monthly(
    state: &CoordinatorState,
) -> Result<bool, StatusCode> {
    let config = state
        .stripe_billing
        .config()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    let response = state
        .stripe_billing
        .client
        .get(format!("{STRIPE_API}/prices/{}", config.price_id))
        .bearer_auth(config.secret_key.as_str())
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    if !response.status().is_success() {
        return Err(StatusCode::BAD_GATEWAY);
    }
    let price: Value = response.json().await.map_err(|_| StatusCode::BAD_GATEWAY)?;
    let recurring = price.get("recurring");
    Ok(price.get("active").and_then(Value::as_bool) == Some(true)
        && price.get("livemode").and_then(Value::as_bool) == Some(false)
        && price.get("unit_amount").and_then(Value::as_i64) == Some(100)
        && price.get("currency").and_then(Value::as_str) == Some("usd")
        && recurring.and_then(|value| value.get("interval")).and_then(Value::as_str)
            == Some("month")
        && recurring.and_then(|value| value.get("interval_count")).and_then(Value::as_i64)
            == Some(1))
}

pub(super) async fn checkout(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
) -> Response {
    if !same_origin(&state, &headers) {
        return no_store(StatusCode::FORBIDDEN);
    }
    let Some(account_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return no_store(StatusCode::UNAUTHORIZED);
    };
    let Some(config) = state.stripe_billing.config() else {
        return no_store(StatusCode::SERVICE_UNAVAILABLE);
    };
    match test_price_is_one_usd_monthly(&state).await {
        Ok(true) => {}
        Ok(false) => return no_store(StatusCode::SERVICE_UNAVAILABLE),
        Err(status) => return no_store(status),
    }
    let record = match account_billing_record(&state, account_id).await {
        Ok(Some(record)) => record,
        Ok(None) => return no_store(StatusCode::NOT_FOUND),
        Err(_) => return no_store(StatusCode::SERVICE_UNAVAILABLE),
    };
    if matches!(
        record.subscription_status.as_deref(),
        Some("active" | "trialing" | "past_due" | "unpaid" | "incomplete" | "paused")
    ) {
        return no_store(StatusCode::CONFLICT);
    }
    let params = checkout_params(
        &state.web_origin,
        &config.price_id,
        account_id,
        record.customer_id.as_deref(),
    );
    match create_session(&state, "checkout/sessions", &params, "checkout.stripe.com").await {
        Ok(url) => no_store(Json(HostedSession { url })),
        Err(status) => no_store(status),
    }
}

pub(super) async fn portal(
    State(state): State<CoordinatorState>,
    Extension(auth): Extension<AuthState>,
    headers: HeaderMap,
) -> Response {
    if !same_origin(&state, &headers) {
        return no_store(StatusCode::FORBIDDEN);
    }
    let Some(account_id) = crate::auth::authenticated_user_id(&auth, &headers).await else {
        return no_store(StatusCode::UNAUTHORIZED);
    };
    let record = match account_billing_record(&state, account_id).await {
        Ok(Some(record)) => record,
        Ok(None) => return no_store(StatusCode::NOT_FOUND),
        Err(_) => return no_store(StatusCode::SERVICE_UNAVAILABLE),
    };
    let Some(customer_id) = record.customer_id else {
        return no_store(StatusCode::NOT_FOUND);
    };
    let params = vec![
        ("customer".into(), customer_id),
        (
            "return_url".into(),
            format!("{}/billing", state.web_origin),
        ),
    ];
    match create_session(&state, "billing_portal/sessions", &params, "billing.stripe.com").await {
        Ok(url) => no_store(Json(HostedSession { url })),
        Err(status) => no_store(status),
    }
}

#[cfg(test)]
mod tests {
    use super::{checkout_params, hosted_stripe_url, test_config};
    use uuid::Uuid;
    use zeroize::Zeroizing;

    #[test]
    fn only_trusted_https_stripe_checkout_and_portal_hosts_are_accepted() {
        assert!(hosted_stripe_url("https://checkout.stripe.com/c/pay/cs_test", "checkout.stripe.com").is_some());
        assert!(hosted_stripe_url("http://checkout.stripe.com/c/pay/cs_test", "checkout.stripe.com").is_none());
        assert!(hosted_stripe_url("https://attacker.example/", "checkout.stripe.com").is_none());
        assert!(hosted_stripe_url("https://checkout.stripe.com.attacker.example/", "checkout.stripe.com").is_none());
    }

    #[test]
    fn checkout_binds_the_configured_price_and_authenticated_account_metadata() {
        let account_id = Uuid::parse_str("9e1a96b4-0a43-4f40-9eb4-1e5e9ae68f17").unwrap();
        let params = checkout_params(
            "https://rawrarr.willjasen.net",
            "price_test_1234567890",
            account_id,
            Some("cus_test_123"),
        );
        let params: std::collections::HashMap<_, _> = params.into_iter().collect();
        assert_eq!(params.get("mode").map(String::as_str), Some("subscription"));
        assert_eq!(params.get("line_items[0][price]").map(String::as_str), Some("price_test_1234567890"));
        assert_eq!(params.get("client_reference_id").map(String::as_str), Some("9e1a96b4-0a43-4f40-9eb4-1e5e9ae68f17"));
        assert_eq!(params.get("metadata[rawrarr_account_id]"), params.get("subscription_data[metadata][rawrarr_account_id]"));
        assert_eq!(params.get("customer").map(String::as_str), Some("cus_test_123"));
        assert_eq!(params.get("success_url").map(String::as_str), Some("https://rawrarr.willjasen.net/billing?billing=success"));
        assert_eq!(params.get("cancel_url").map(String::as_str), Some("https://rawrarr.willjasen.net/billing?billing=cancelled"));
    }

    #[test]
    fn test_checkout_rejects_live_keys_and_malformed_prices() {
        assert!(test_config(Zeroizing::new("sk_live_not-for-test".into()), "price_test_123456".into()).is_err());
        assert!(test_config(Zeroizing::new("sk_test_only".into()), "prod_not-a-price".into()).is_err());
        assert!(test_config(Zeroizing::new("sk_test_example".into()), "price_test_123456".into()).is_ok());
    }
}
