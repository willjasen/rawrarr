mod stripe_webhook;

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{env, net::SocketAddr, sync::Arc};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    webhook_secret: Arc<String>,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("BILLING_DATABASE_URL")?;
    let webhook_secret = env::var("STRIPE_WEBHOOK_SECRET")?;
    if webhook_secret.trim().is_empty() {
        return Err("STRIPE_WEBHOOK_SECRET must not be empty".into());
    }
    let address: SocketAddr = env::var("BILLING_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8090".to_owned())
        .parse()?;
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/webhooks/stripe", post(stripe_webhook))
        .layer(DefaultBodyLimit::max(1_048_576))
        .with_state(AppState {
            pool,
            webhook_secret: Arc::new(webhook_secret),
        });
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(Health { status: "ok" })),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(Health { status: "unavailable" }),
        ),
    }
}

async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let Some(signature) = headers
        .get("stripe-signature")
        .and_then(|value| value.to_str().ok())
    else {
        return StatusCode::BAD_REQUEST;
    };
    let now = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };
    if stripe_webhook::verify_signature(&body, signature, &state.webhook_secret, now).is_err() {
        return StatusCode::BAD_REQUEST;
    }
    let event: stripe_webhook::StripeEvent = match serde_json::from_slice(&body) {
        Ok(event) => event,
        Err(_) => return StatusCode::BAD_REQUEST,
    };
    if event.id.is_empty() || event.data.object.id.is_empty() {
        return StatusCode::BAD_REQUEST;
    }
    if !stripe_webhook::is_billing_event(&event.event_type) {
        return StatusCode::NO_CONTENT;
    }
    let mut transaction = match state.pool.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE,
    };
    let explicit_account_id = match event.data.object.rawrarr_account_id() {
        Ok(account_id) => account_id,
        Err(()) => return StatusCode::BAD_REQUEST,
    };
    let customer_id = event.data.object.stripe_customer_id();
    // When Stripe provides account metadata, reject an existing customer mapping
    // that points elsewhere. A valid webhook must never reassign a customer.
    if let (Some(account_id), Some(customer_id)) = (explicit_account_id, customer_id) {
        match sqlx::query_scalar::<_, Uuid>("SELECT account_id FROM rawrarr_stripe_customers WHERE stripe_customer_id=$1")
            .bind(customer_id)
            .fetch_optional(&mut *transaction)
            .await
        {
            Ok(Some(existing)) if existing != account_id => return StatusCode::CONFLICT,
            Ok(_) => {}
            Err(_) => return StatusCode::SERVICE_UNAVAILABLE,
        }
    }
    let account_id = if let Some(account_id) = explicit_account_id {
        Some(account_id)
    } else if let Some(customer_id) = customer_id {
        match sqlx::query_scalar::<_, Uuid>("SELECT account_id FROM rawrarr_stripe_customers WHERE stripe_customer_id=$1")
            .bind(customer_id)
            .fetch_optional(&mut *transaction)
            .await
        {
            Ok(account_id) => account_id,
            Err(_) => return StatusCode::SERVICE_UNAVAILABLE,
        }
    } else {
        None
    };
    // Billing events without an authenticated account/customer association must
    // not be acknowledged as processed or become candidates for entitlement.
    if account_id.is_none() {
        return StatusCode::BAD_REQUEST;
    }
    let inserted = sqlx::query(
        "INSERT INTO stripe_webhook_events(event_id, event_type, account_id, stripe_created_at) VALUES ($1, $2, $3, to_timestamp($4)) ON CONFLICT(event_id) DO NOTHING",
    )
    .bind(&event.id)
    .bind(&event.event_type)
    .bind(account_id)
    .bind(event.created as f64)
    .execute(&mut *transaction)
    .await;
    let inserted = match inserted {
        Ok(result) => result.rows_affected() == 1,
        Err(_) => return StatusCode::SERVICE_UNAVAILABLE,
    };
    if !inserted {
        return match transaction.commit().await {
            Ok(()) => StatusCode::NO_CONTENT,
            Err(_) => StatusCode::SERVICE_UNAVAILABLE,
        };
    }
    if let (Some(account_id), Some(customer_id)) = (account_id, customer_id) {
        let mapped = sqlx::query(
            "INSERT INTO rawrarr_stripe_customers(account_id, stripe_customer_id) VALUES ($1, $2) ON CONFLICT(account_id) DO UPDATE SET updated_at=now() WHERE rawrarr_stripe_customers.stripe_customer_id=EXCLUDED.stripe_customer_id",
        )
        .bind(account_id)
        .bind(customer_id)
        .execute(&mut *transaction)
        .await;
        match mapped {
            Ok(result) if result.rows_affected() == 1 => {}
            _ => return StatusCode::SERVICE_UNAVAILABLE,
        }
        if event.event_type.starts_with("customer.subscription.") {
            if let Some(status) = event.data.object.status.as_deref() {
                let period_end = event.data.object.current_period_end.map(|value| value as f64);
                let cancel_at_period_end = event.data.object.cancel_at_period_end.unwrap_or(false);
                let subscription = sqlx::query(
                    "INSERT INTO rawrarr_stripe_subscriptions(stripe_subscription_id, account_id, stripe_customer_id, status, current_period_end, cancel_at_period_end, stripe_event_created_at) VALUES ($1, $2, $3, $4, to_timestamp($5), $6, to_timestamp($7)) ON CONFLICT(stripe_subscription_id) DO UPDATE SET status=EXCLUDED.status, current_period_end=EXCLUDED.current_period_end, cancel_at_period_end=EXCLUDED.cancel_at_period_end, stripe_event_created_at=EXCLUDED.stripe_event_created_at, updated_at=now() WHERE rawrarr_stripe_subscriptions.account_id=EXCLUDED.account_id AND rawrarr_stripe_subscriptions.stripe_customer_id=EXCLUDED.stripe_customer_id AND rawrarr_stripe_subscriptions.stripe_event_created_at <= EXCLUDED.stripe_event_created_at",
                )
                .bind(&event.data.object.id)
                .bind(account_id)
                .bind(customer_id)
                .bind(status)
                .bind(period_end)
                .bind(cancel_at_period_end)
                .bind(event.created as f64)
                .execute(&mut *transaction)
                .await;
                if subscription.is_err() {
                    return StatusCode::SERVICE_UNAVAILABLE;
                }
            }
        }
    }
    match transaction.commit().await {
        Ok(()) => StatusCode::NO_CONTENT,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
