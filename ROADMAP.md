# rawrarr roadmap

This document is the single index of what the repository has completed and what remains. It reflects the local prototype and product decisions recorded in the project docs; it does not imply a public release or a production multi-node deployment. No delivery dates are committed.

## Recommended next work

1. **Verify an unattended storage scan.** Source and the standalone storage Compose file schedule a bounded scan after the first 24-hour interval, or at an owner-configured interval. The [manual Mini scan](runbooks/2026-10-18-clarify-storage-scan-action.md) completed and synced three encrypted sidecars, and the [later container update](runbooks/2026-10-18-update-mac-mini-containers.md) rebuilt storage. A scheduled run on that image has not yet been observed.
2. **Build a durable cross-node catalog.** Owner-approved node enrollment, heartbeats, selected-node pages, direct routes, and scan commands exist. Add persistent library/device records, scan generations, stale/offline inventory, and reconciliation without moving media bytes through roarlay.
3. **Finish export beyond movies.** Settings can export all indexed movie text and matching available artwork in a browser-built ZIP. Add other metadata classes, cancellation, bounded streaming, and explicit partial-failure reporting before calling it a complete archive.
4. **Verify the intended public reverse-proxy route.** The local Tailscale HTTPS route is distinct from the planned Cloudflare/public route. Test `/api/events` streaming, reconnects, buffering, and timeouts through the exact public proxy configuration.

This sequence verifies deployed storage behavior, then addresses the missing catalog and export durability. Revisit it if product priorities change.

## Completed in the repository

- **Standalone Rust application:** Axum web/API service, local PostgreSQL migrations and state, `/health`, Docker Compose, and container health checks.
- **Role-separated local deployment:** roarlay keeps passkeys and PostgreSQL; metadata and apex are subscriber nodes with separate local state. Owner-approved enrollment, node heartbeats, a roarlay workspace, scoped browser direct routes with relay fallback, and owner-requested scans are implemented. A Mac mini bounded manual scan and encrypted sidecar synchronization were verified; the scheduled scan still needs observation.
- **Local passkey onboarding:** first-account registration and sign-in use WebAuthn with passkeys and sessions stored in that node's PostgreSQL database.
- **Age-encrypted shared settings:** create or paste a post-quantum age identity; encrypt configuration as `/shared/config.aged`. The private identity is sent to the subscriber web process for unlock and retained there while unlocked. An explicit **Keep this ID on this device** option saves it with owner-only permissions in node-local `/data/age-identity`; it never belongs in `/shared`, PostgreSQL, or the operator service. Generated IDs use a copy pop-up and copy acknowledgement, followed by an explicit **Continue** click before vault unlock and dashboard navigation. Pasted IDs unlock directly with **Unlock vault**, without copy acknowledgement. The dialog has no Close action. Character verification is temporarily disabled.
- **Radarr and local library prototype:** configure Radarr, inspect movie inventory and connection status, scan a configured local media path, and create age-encrypted metadata and artwork. Media stays on its storage host.
- **Sonarr integration:** configure and validate an HTTPS Sonarr connection, store its settings in the subscriber's age-encrypted configuration, and read series inventory through authenticated APIs. The browser sends credentials directly to metadata; roarlay never receives the API key. Radarr and Sonarr share library presentation. A [dated live verification](runbooks/2026-10-18-automatic-sonarr-metadata-backfill.md) observed 228 TV series and 226 encrypted text and artwork records; that snapshot does not establish current connectivity or complete coverage.
- **Syncthing and status foundation:** configure supported Syncthing peer/folder settings, expose per-node device identity and scan status, write compressed allowlisted shared logs, and stream dashboard updates using server-sent events.
- **Browser movie export:** decrypt all indexed movie text metadata and matching available artwork in the browser and download a ZIP. The current ZIP is assembled in memory and is limited to movie metadata.
- **Local Docker data and test workflow:** persist shared data outside the source tree, retain PostgreSQL and node-local state in Docker-managed storage, and check web/PostgreSQL health. The `rawrarr-testing-base:local` image caches external tools and Rust dependencies; each test run creates a unique container from that base, copies in current source, and retains its log. The base is rebuilt only when missing or explicitly requested.
- **Project foundations:** standalone Git repository, product and data-boundary documentation, design guide, user guide, and test suites for current Rust and browser behavior.

## Outstanding work

### Multi-node subscriber application

- Durable device and library catalog with stale/offline state, scan generations, and reconciliation.
- Define library configuration and scanning for mixed-media roots and category-specific roots distributed across multiple storage nodes; combine them in the instance catalog without moving media bytes.
- Complete device credential rotation and revocation behavior, identity continuity, and a durable, narrowly scoped command/catalog channel. The current approval, heartbeats, and scan request are the foundation.
- Keep user sign-in and PostgreSQL on roarlay. Multi-account invitations and multiple-coordinator auth/failover remain future work.
- Publish role-specific packaging for roarlay, apex, metadata, and an isolated operator service; the existing Compose files are local source-build examples.
- Implement explicit per-user sharing grants. Syncthing peer/folder configuration does not currently implement invitations, trust decisions, or selective sharing.

### Shared clusters and owner-controlled membership

- Keep new clusters isolated by default; support an explicit one-way transition to shared mode. The owner alone may invite or remove members; invitees cannot extend membership.
- Establish a genesis-anchored, append-only trust history for every cluster, including isolated-mode device enrollment/removal, role changes, shared-mode transition, member changes, root rotation, and recovery. Replicate and back up history, pin accepted heads locally, and detect gaps, forks, and rollback.
- Define owner-managed loss handling: if the age identity is lost, original media remains and encrypted metadata/config must be rebuilt where possible; if the signing root is lost, the owner may explicitly re-enroll devices under a new genesis/trust chain. Rawrarr provides no secret recovery bypass, and a new signing root must never be trusted silently.
- Decide whether MLS is warranted for group epochs over the eventually consistent Syncthing folder or whether an owner-signed append-only membership log is sufficient. Prototype single-writer commits, offline catch-up, conflicts, and stale-node quarantine before selecting a protocol.
- Keep age as file encryption. Its per-file random key and recipient stanzas are part of each age file; every authorized recipient gets a stanza that can recover the same key, adding only modest header size while encrypting the payload once. Stanzas need not contain public recipient IDs verbatim; keep those IDs in authenticated membership records. Do not create a persistent cluster-wide pre-shared key. Validate multi-recipient age for new files and test a maintained implementation's ability to add recipients to existing ciphertext without re-encrypting payloads; otherwise choose a history migration policy.
- Implement revocation so removed members lose future access, while explaining that they may retain previously received data. Request cooperative cleanup from their Rawrarr nodes and report pending, node-reported complete, or unconfirmed status without claiming secure erasure.
- Define invitation history scope, per-person versus per-device membership, root-key recovery/succession, rollback resistance, and reconnect behavior before production implementation.

### Private access, operator service, and relay

- Cloudflare/reverse-proxy support for the intended public hostname, including verified SSE streaming behavior.
- Productionize the existing local Tailscale sidecars and tailnet-scoped routes while keeping transport separate from Rawrarr authorization and age-key access.
- Operator service node running SwarmSail/Headscale account coordination, with a one-time, revocable node claim flow.
- Blind relay and portal for reaching subscriber nodes through outbound tunnels. The subscriber node must retain application authentication and TLS termination; the relay must not decrypt or store subscriber app data or proxy media.

### Packaging and subscriptions

- Publish versioned, multi-architecture application images with release notes, provenance, and a documented upgrade path.
- Generate role-specific Compose files for private/subscribed coordinator, storage, shard, and combined coordinator + storage deployments. The current Compose files are local source-build examples.
- Hosted passkey account, checkout, trial, subscription/license issuance and enforcement. The selected direction is a shared-instance entitlement with signed licenses; provider, pricing, grace period, and final lock behavior remain undecided.

### Product capabilities and durability

- Restore private-key character verification after fixing the reported rejection of correct answers and validating the full browser flow.

- Add the optional roarlay AI chat, talking voice mode, and PostgreSQL request inbox described in [the design guide](docs/DESIGN.md#ai-chat-and-request-review-planned). First answer read-only library questions from verified records, then let an owner-run agent review gaps and propose scoped changes. Enforce five AI questions per account per UTC day during the 24-hour trial across text and voice. Consider a Twilio phone entry point after signed-in browser voice. Define cloud-data consent, transcript retention, caller authentication, approval, and the blind-content query path before implementation.

- General-purpose file/folder sharing beyond media.
- Sonarr posters/artwork, snapshots, metadata lookup, and episode mutations; Lidarr, Readarr, and Bazarr integrations; Prowlarr-style indexer management and search; concrete download-client adapters and fulfillment. The current download-client abstraction has no concrete adapter.
- Manual age-encrypted shared-folder backup to one node-local destination, including recovery verification and a retention policy.
- Device signing, approval, signed shared configuration, and revocation/revision checks.
- Static logo animation for selected moments, with reduced-motion behavior.

## Related documentation

- [Device and service architecture](docs/device-architecture.md) — current local prototype boundaries and target node model.
- [Product direction](docs/product-direction.md) — long-term product goals and trust boundaries.
- [Product and interface design](docs/DESIGN.md) — user-facing principles and flows.
- [Shared cluster plan](docs/shared-cluster-sharing-plan.md) — planned isolated-to-shared transition, owner-controlled membership, MLS evaluation, age file handling, revocation cleanup, and phased work.
- [Technical boundaries](docs/technical-status.md) — data placement and security boundaries.
- [User guide](docs/user-guide.md) — current local prototype usage.
- [Testing](TESTING.md) — reusable dependency image and isolated runner workflow.
- [Runbooks](runbooks/README.md) — dated task evidence.
