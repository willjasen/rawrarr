# rawrarr roadmap

This document is the single index of what the repository has completed and what remains. It reflects the local prototype and product decisions recorded in the project docs; it does not imply a public release or a production multi-node deployment. No delivery dates are committed.

## Recommended next work

1. **Verify SSE through the intended Cloudflare/reverse-proxy route.** The app currently streams dashboard updates from `/api/events`, but the deployed path has not been tested. Confirm streaming, reconnect behavior, buffering, and connection timeouts using the exact planned proxy configuration.
2. **Make a local two-node subscriber setup coherent.** Persist device and library records, define pairing and revocation, add heartbeat/remote command behavior, and resolve local passkey/session and catalog consistency across nodes. Keep each node's PostgreSQL local; do not synchronize database files.
3. **Add private node access and enrollment.** Once the node/auth contract works, add the optional subscriber Tailscale sidecar and tailnet-scoped service, followed by one-time claim through the operator service node. Transport membership must not grant data access.

This sequence closes a known deployment question first, then builds the node coordination needed by the larger hosted design. Revisit it if product priorities change.

## Completed in the repository

- **Standalone Rust application:** Axum web/API service, local PostgreSQL migrations and state, `/health`, Docker Compose, and container health checks.
- **Local passkey onboarding:** first-account registration and sign-in use WebAuthn with passkeys and sessions stored in that node's PostgreSQL database.
- **Age-encrypted shared settings:** create or paste a post-quantum age identity; encrypt configuration as `/shared/config.aged`. The private identity stays in browser memory and is never saved by the app. Generated IDs use a copy pop-up, copy acknowledgement, random three-character challenge, then automatic vault unlock and dashboard navigation. Pasted IDs use the same copy check, followed by **Unlock vault**.
- **Radarr and local library prototype:** configure Radarr, inspect movie inventory and connection status, scan a configured local media path, and create age-encrypted metadata and artwork. Media stays on its storage host.
- **Syncthing and status foundation:** configure supported Syncthing peer/folder settings, expose per-node device identity and scan status, write compressed allowlisted shared logs, and stream dashboard updates using server-sent events.
- **Local Docker data and test workflow:** persist shared data outside the source tree, retain PostgreSQL and node-local state in Docker-managed storage, and check web/PostgreSQL health. The `rawrarr-testing-base:local` image caches external tools and Rust dependencies; each test run creates a unique container from that base, copies in current source, and retains its log. The base is rebuilt only when missing or explicitly requested.
- **Project foundations:** standalone Git repository, product and data-boundary documentation, design guide, user guide, and test suites for current Rust and browser behavior.

## Outstanding work

### Multi-node subscriber application

- Durable device and library catalog with stale/offline state, scan generations, and reconciliation.
- Pairing, approval, revocation, device identity continuity, heartbeats, and a narrowly scoped remote command channel.
- Consistent login/session and user identity behavior across subscriber web nodes. Passkeys and live PostgreSQL data are currently node-local; portable user/passkey records and secure bootstrap remain unresolved.
- Prove storage and shard nodes each run the intended web/API service while media remains local to storage nodes.
- Implement explicit per-user sharing grants. Syncthing peer/folder configuration does not currently implement invitations, trust decisions, or selective sharing.

### Private access, operator service, and relay

- Cloudflare/reverse-proxy support for the intended public hostname, including verified SSE streaming behavior.
- Optional Tailscale sidecars and a tailnet-scoped Rawrarr service, with transport kept separate from Rawrarr authorization and age-key access.
- Operator service node running SwarmSail/Headscale account coordination, with a one-time, revocable node claim flow.
- Blind relay and portal for reaching subscriber nodes through outbound tunnels. The subscriber node must retain application authentication and TLS termination; the relay must not decrypt or store subscriber app data or proxy media.

### Packaging and subscriptions

- Publish versioned, multi-architecture application images with release notes, provenance, and a documented upgrade path.
- Generate role-specific Compose files for private/subscribed storage and shard deployments. The current Compose files are local source-build examples.
- Hosted passkey account, checkout, trial, subscription/license issuance and enforcement. The selected direction is a shared-instance entitlement with signed licenses; provider, pricing, grace period, and final lock behavior remain undecided.

### Product capabilities and durability

- General-purpose file/folder sharing beyond media.
- Sonarr, Lidarr, Readarr, and Bazarr integrations; Prowlarr-style indexer management and search; concrete download-client adapters and fulfillment. The current download-client abstraction has no concrete adapter.
- Manual age-encrypted shared-folder backup to one node-local destination, including recovery verification and a retention policy.
- Device signing, approval, signed shared configuration, and revocation/revision checks.
- Static logo animation for selected moments, with reduced-motion behavior.

## Related documentation

- [Device and service architecture](docs/device-architecture.md) — current local prototype boundaries and target node model.
- [Product direction](docs/product-direction.md) — long-term product goals and trust boundaries.
- [Product and interface design](docs/DESIGN.md) — user-facing principles and flows.
- [Technical boundaries](docs/technical-status.md) — data placement and security boundaries.
- [User guide](docs/user-guide.md) — current local prototype usage.
- [Testing](TESTING.md) — reusable dependency image and isolated runner workflow.
- [Runbooks](runbooks/README.md) — dated task evidence.
