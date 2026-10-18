use super::{allowed_path, coordinator_path, validate_url};
#[test]
fn subscriber_payloads_cannot_be_proxied_to_coordinator() {
    for path in [
        "/api/settings/age",
        "/api/settings/radarr",
        "/api/radarr/movies",
        "/api/library/scan",
        "/api/auth/login/start/../settings/age",
    ] {
        assert!(!allowed_path(path));
    }
    assert!(allowed_path("/api/auth/login/start"));
    assert!(allowed_path("/api/auth/session"));
    assert!(allowed_path("/api/nodes/enroll"));
    assert!(allowed_path("/api/nodes/enroll/status"));
    assert!(allowed_path("/api/nodes/pending"));
    assert!(allowed_path("/api/nodes/approve"));
    assert!(allowed_path("/api/auth/passkeys/coordinator/link"));
    assert!(!allowed_path("/api/nodes/pending/other"));
    assert_eq!(coordinator_path("/api/auth/register/start"), "/api/nodes/auth/register/start");
    assert_eq!(coordinator_path("/api/auth/session"), "/api/nodes/auth/session");
    assert_eq!(coordinator_path("/api/auth/passkeys/coordinator/link"), "/api/auth/passkeys/coordinator/link");
}
#[test]
fn coordinator_transport_rejects_unsafe_origins() {
    for value in [
        "http://remote.example",
        "https://user:password@example.com",
        "https://example.com/path",
        "https://example.com/?token=x",
        "https://example.com/#fragment",
    ] {
        assert!(validate_url(&value.parse().unwrap(), true).is_err());
    }
    assert!(validate_url(&"https://coordinator.example".parse().unwrap(), false).is_ok());
    assert!(validate_url(&"http://coordinator:8080".parse().unwrap(), false).is_err());
    assert!(validate_url(&"http://coordinator:8080".parse().unwrap(), true).is_ok());
    assert!(validate_url(&"http://rawrarr-mini-tailscale:8081".parse().unwrap(), true).is_ok());
    assert!(validate_url(&"http://untrusted-sidecar:8081".parse().unwrap(), true).is_err());
}

#[test]
fn node_coordinator_socks5_proxy_url_is_supported() {
    assert!(reqwest::Proxy::all("socks5h://tailscale:1055").is_ok());
}
