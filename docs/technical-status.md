# Rawrarr technical status

Last reviewed: 2026-10-18.

This document separates implemented behavior from architecture that is still a plan. See the [user guide](user-guide.md) for hands-on instructions and the [runbook index](../runbooks/README.md) for dated implementation evidence.

## Implemented

- **Rust web application:** Axum-based web/API application, PostgreSQL-backed local passkey authentication, and Docker Compose packaging. The Rust crate forbids unsafe code. PostgreSQL data and runtime state remain node-local.
- **Passkey onboarding:** first account registration and sign-in use WebAuthn passkeys stored in the local PostgreSQL database. A new local account is not yet a cloud relay account.
- **Age vault:** create or import a post-quantum age identity; persist Rawrarr configuration as `config.aged` in `/shared`; unlock it in the app. The private identity is not persisted by the app. Setup shows only an abbreviated preview and public-recipient fingerprint, copies only the private key line in a pop-up, then asks for copy acknowledgement and three random characters after the fixed key prefix. Correct entry auto-unlocks a generated identity and opens the dashboard; pasted identities use the same checks before the user selects **Unlock vault**.
- **Radarr:** configure URL/API key in the UI, save settings age-encrypted, apply changes without restarting, show connection state and movies.
- **Local scanning and metadata:** scan an explicitly mounted media root; search Radarr metadata and cache artwork as age-encrypted `.aged` files. Scans can create age-encrypted JSON sidecars from a public age recipient without needing the private identity in the scanner process.
- **Syncthing:** configure supported peer/folder settings through the Syncthing REST API. Syncthing implements BEP for peer transfer; Rawrarr does not implement BitTorrent or BEP itself.
- **Shared logs:** write compressed allowlisted JSONL logs beneath `/shared/logs/<device-id>/`.
- **Docker:** the base Compose project runs Rawrarr and PostgreSQL; media and age-identity Compose overlays are available. Both containers have readiness health checks. `/shared` is a host bind mount, while `/data` and PostgreSQL use local Docker volumes.
- **Brand assets:** the setup and workspace pages prefer a lossless JPEG XL logo and retain PNG fallback for browsers without JPEG XL decoding.

## Planned, not implemented

- **More Arr integrations and native functions:** Sonarr, Lidarr, Readarr, Bazarr, broader Prowlarr-style discovery, concrete download clients, and fulfillment remain future work. Radarr is the current integration focus.
- **Multiple Rawrarr nodes:** the target roles are the operator `service` node, subscriber `storage` nodes, and media-less subscriber `shard` nodes. Each subscriber node should run the Rawrarr web/API service. Remote control/pairing and cross-node consistency are unfinished.
- **SwarmSail and Tailscale:** one Headscale tailnet per subscriber account; each subscriber Docker deployment runs a Tailscale sidecar. SwarmSail on the operator's service node will communicate with those containers over the overlay. The overlay is transport only; Rawrarr sharing and age recipients control data access. Do not send media over it.
- **Relay, claim flow, and subscriptions:** the production service node is intended to host passkey accounts, subscriptions/licensing, SwarmSail orchestration, and a blind relay. Claiming a new container through a temporary URL, hosted checkout/Stripe, signed licenses, and enforcement have not been implemented.
- **Shared identity records:** passkeys and user data are still stored in each node's local PostgreSQL. Portable encrypted passkey sharing and a passkey-first bootstrap for a new node remain unsolved.
- **Device signing:** per-device signing keys, enrollment approval, signatures over shared configuration, revocation, and revision checks are a future security feature. See [device signing design](device-signing-design.md).
- **Local shared-folder backups:** the agreed design is a manual backup action on one designated node, stored outside `/shared`, with the archive age-encrypted to public recipient(s). Destination device/path lives in `config.aged`. Retention is undecided; automatic deletion is out of scope. No backup UI or archive implementation exists.
- **Animated logo:** the JPEG XL format can carry animation, but the logo remains static. Occasional animation, timing, and reduced-motion behavior have not been implemented.

## Data and trust boundaries

- `/media` is a node-local, read-only media mount. Rawrarr, Syncthing, Tailscale, and the relay must not copy or proxy those media bytes as part of app-data operations.
- `/shared` is the Syncthing app-data root. `/metadata` aliases `/shared/metadata`. `config.aged`, artwork, and library sidecars are encrypted before synchronization; logs are compressed and allowlisted, but are not age-encrypted individually.
- `/data` and the live PostgreSQL database are local to each container/node. A Syncthing copy of files is not database replication or quorum.
- The operator's relay/service node may store account credentials, public age recipients, billing/license status, and opaque instance/device IDs. It must never receive private age identities or subscriber app-data plaintext.

## Verification inventory

Automated checks are indexed in [scripts/tests/README.md](../scripts/tests/README.md). Recent documented checks include Rust unit tests, a Node DOM test for age setup behavior, disposable PostgreSQL API tests, and isolated two-node Syncthing tests with OrbStack or local Syncthing processes. Not every test runs on every change. Check each test's prerequisites and its runbook before treating historical results as proof of current runtime health.
