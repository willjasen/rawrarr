#![forbid(unsafe_code)]

use axum::{routing::get, Router};
use std::{env, net::SocketAddr};

async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    let address = env::var("RUST_ARR_LISTEN_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_owned());
    let socket_address: SocketAddr = address
        .parse()
        .expect("RUST_ARR_LISTEN_ADDR must be a valid socket address");

    let app = Router::new().route("/health", get(health));
    let listener = tokio::net::TcpListener::bind(socket_address)
        .await
        .expect("failed to bind HTTP listener");

    println!("rust-arr listening on {socket_address}");
    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}
