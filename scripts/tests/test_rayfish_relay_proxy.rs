use super::{build_relay_client, parse_relay_loopback_bypass};
use axum::{Router, routing::get};
use std::time::Duration;
use tokio::{net::TcpListener, time::timeout};

#[test]
fn loopback_bypass_is_explicit_and_strict() {
    assert_eq!(parse_relay_loopback_bypass(None), Ok(false));
    assert_eq!(parse_relay_loopback_bypass(Some("false")), Ok(false));
    assert_eq!(parse_relay_loopback_bypass(Some("true")), Ok(true));
    assert!(parse_relay_loopback_bypass(Some("yes")).is_err());
    assert!(parse_relay_loopback_bypass(Some("")).is_err());
}

#[tokio::test]
async fn configured_proxy_is_bypassed_only_for_opted_in_loopback_targets() {
    let target = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_url = format!("http://{}/", target.local_addr().unwrap());
    let target_task = tokio::spawn(async move {
        axum::serve(target, Router::new().route("/", get(|| async { "rayfish-forward" })))
            .await
            .unwrap();
    });

    let proxy = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_url = format!("socks5h://{}", proxy.local_addr().unwrap());
    let proxy_task = tokio::spawn(async move {
        let (stream, _) = timeout(Duration::from_secs(3), proxy.accept()).await.unwrap().unwrap();
        drop(stream);
    });
    let client = build_relay_client(Some(&proxy_url), false).unwrap();
    assert!(client.get(&target_url).timeout(Duration::from_secs(3)).send().await.is_err());
    proxy_task.await.unwrap();

    let proxy = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_url = format!("socks5h://{}", proxy.local_addr().unwrap());
    let client = build_relay_client(Some(&proxy_url), true).unwrap();
    let response = client.get(&target_url).timeout(Duration::from_secs(3)).send().await.unwrap();
    assert_eq!(response.text().await.unwrap(), "rayfish-forward");
    assert!(timeout(Duration::from_millis(100), proxy.accept()).await.is_err());

    let proxy_task = tokio::spawn(async move {
        for _ in 0..2 {
            let (stream, _) = timeout(Duration::from_secs(3), proxy.accept()).await.unwrap().unwrap();
            drop(stream);
        }
    });
    assert!(client.get("http://localhost/")
        .timeout(Duration::from_secs(3)).send().await.is_err());
    assert!(client.get("http://tailnet-node.invalid/")
        .timeout(Duration::from_secs(3)).send().await.is_err());
    proxy_task.await.unwrap();
    target_task.abort();
}
