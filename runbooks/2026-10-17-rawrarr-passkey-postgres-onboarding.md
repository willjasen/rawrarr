# Rawrarr passkey and PostgreSQL onboarding

## Objective and initial state

Add a minimal passkey-first sign-in flow, followed by age identity creation/import, then the existing dashboard. Persist passkey credentials and authentication state in PostgreSQL. The app previously had no user authentication and embedded age identity setup in the dashboard. See [age identity onboarding](2026-10-17-rawrarr-age-identity-onboarding.md) and [encrypted configuration](2026-10-17-rawrarr-age-encrypted-config.md).

## Implementation

- Use `webauthn-rs` for browser WebAuthn ceremony validation; store credential records, pending ceremony state, hashed sessions, and users in PostgreSQL.
- Keep the registration/authentication ceremony state server-side in Postgres; set only an opaque session token in an HttpOnly cookie.
- First account registration creates the local owner. Subsequent account invitations are not implemented yet.
- Route unauthenticated users to a minimal passkey screen. After successful authentication, route users without an unlocked age identity to a dedicated minimal age setup screen; then continue to the existing dashboard.
- Add PostgreSQL to the base Docker Compose deployment with an internal-only network endpoint and persistent database volume.

## Verification and remaining work

- `cargo fmt`, `cargo test --manifest-path rust-arr/Cargo.toml`, and `cargo build --release --locked --manifest-path rust-arr/Cargo.toml` passed using the installed Rust 1.85 toolchain; three unit tests passed, including username validation.
- Python test harness compiles and its `--help` path runs. Full API integration test requires an empty disposable PostgreSQL database named with `_test`; no PostgreSQL server/client is installed here.
- `docker-compose config` rendered the PostgreSQL-backed deployment successfully. Docker daemon is unavailable, so container startup and health checks were not exercised.
- A release build is in progress. Browser ceremony and the complete age onboarding flow remain unverified end-to-end without a live PostgreSQL service and a browser authenticator.

Browser passkey operation requires an HTTPS secure context or localhost and a configured relying-party ID/origin. Do not store private age identities in PostgreSQL.
