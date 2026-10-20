# rawrarr roadmap

This document is the single index of what the repository has completed and what remains. It reflects the local prototype and product decisions recorded in the project docs; it does not imply a public release or a production multi-node deployment. No delivery dates are committed.

## Recommended next work

1. **Verify an unattended storage scan.** Source and the standalone storage Compose file schedule a bounded scan after the first 24-hour interval, or at an owner-configured interval. The [manual Mini scan](runbooks/2026-10-18-clarify-storage-scan-action.md) completed and synced three encrypted sidecars, and the [later container update](runbooks/2026-10-18-update-mac-mini-containers.md) rebuilt storage. A scheduled run on that image has not yet been observed.
2. **Build a durable cross-node catalog.** Owner-approved node enrollment, heartbeats, selected-node pages, direct routes, and scan commands exist. Add persistent library/device records, scan generations, stale/offline inventory, and reconciliation without moving media bytes through roarlay.
3. **Finish export beyond movies.** Settings can export all indexed movie text and matching available artwork in a browser-built ZIP. Add other metadata classes, cancellation, bounded streaming, and explicit partial-failure reporting before calling it a complete archive.
4. **Verify the intended public reverse-proxy route.** The local Tailscale HTTPS route is distinct from the planned Cloudflare/public route. Test `/api/events` streaming, reconnects, buffering, and timeouts through the exact public proxy configuration.

This sequence verifies deployed storage behavior, then addresses the missing catalog and export durability. Revisit it if product priorities change.

5. **Evolve profile dinosaur generations through visual comparison.** Keep the public-age-recipient digest as the stable identity seed. Build named renderer generations and a fixed-seed preview that compares controlled mutations in dinosaur anatomy, pose, palette, lighting, texture, framing, and prehistoric backgrounds. Use human-selected improvements as the parent for later generations; promote a generation only after visual review at sidebar and profile sizes. The current style direction is premium 3D with cinematic violet-blue lighting and varied prehistoric trees. See [avatar generation comparison](runbooks/2026-10-21-version-avatar-generations.md).

## Completed in the repository

- **Standalone Rust application:** Axum web/API service, local PostgreSQL migrations and state, `/health`, Docker Compose, and container health checks.
- **Role-separated local deployment:** roarlay keeps passkeys and PostgreSQL; fossil (metadata) and boneyard (storage) are subscriber nodes with separate local state. Owner-approved enrollment, node heartbeats, a roarlay workspace, scoped browser direct routes with relay fallback, and owner-requested scans are implemented. A Mac mini bounded manual scan and encrypted sidecar synchronization were verified; the scheduled scan still needs observation.
- Use **dino** for a user-deployed Rawrarr node. A dino may run fossil and either boneyard or tarpit, but boneyard and tarpit must be mutually exclusive. Tarpit is planned, not implemented.
- **Local passkey onboarding:** first-account registration and sign-in use WebAuthn with passkeys and sessions stored in that node's PostgreSQL database.
- **Age-encrypted shared settings:** create or paste a post-quantum age identity; encrypt configuration as `/shared/config.aged`. The private identity is sent to the subscriber web process for unlock and retained there while unlocked. An explicit **Keep this ID on this device** option saves it with owner-only permissions in node-local `/data/age-identity`; it never belongs in `/shared`, PostgreSQL, or the operator service. Generated IDs use a copy pop-up and copy acknowledgement, followed by an explicit **Continue** click before vault unlock and dashboard navigation. Pasted IDs unlock directly with **Unlock vault**, without copy acknowledgement. The dialog has no Close action. Character verification is temporarily disabled.
- **Radarr and local library prototype:** configure Radarr, inspect movie inventory and connection status, scan a configured local media path, and create age-encrypted metadata and artwork. Media stays on its storage host.
- **Radarr recovery and migration:** keep the latest native Radarr backup age-encrypted in the shared Syncthing folder on a six-hour cycle, and provide an owner-run helper to restore it to a Radarr container deployed on a chosen node. Media is not copied; matching root-folder mounts or manual repair are required.
- **Sonarr integration:** configure and validate an HTTPS Sonarr connection, store its settings in the subscriber's age-encrypted configuration, and read series inventory through authenticated APIs. The browser sends credentials directly to metadata; roarlay never receives the API key. Radarr and Sonarr share library presentation. A [dated live verification](runbooks/2026-10-18-automatic-sonarr-metadata-backfill.md) observed 228 TV series and 226 encrypted text and artwork records; that snapshot does not establish current connectivity or complete coverage.
- **Syncthing and status foundation:** configure supported Syncthing peer/folder settings, expose per-node device identity and scan status, write compressed allowlisted shared logs, and stream dashboard updates using server-sent events.
- **Browser movie export:** decrypt all indexed movie text metadata and matching available artwork in the browser and download a ZIP. The current ZIP is assembled in memory and is limited to movie metadata.
- **Local Docker data and test workflow:** persist shared data outside the source tree, retain PostgreSQL and node-local state in Docker-managed storage, and check web/PostgreSQL health. The `rawrarr-testing-base:local` image caches external tools and Rust dependencies; each test run creates a unique container from that base, copies in current source, and retains its log. The base is rebuilt only when missing or explicitly requested.
- **Project foundations:** standalone Git repository, product and data-boundary documentation, design guide, user guide, and test suites for current Rust and browser behavior.

## Outstanding work

### Multi-node subscriber application

- Provide direct browser access to a Radarr UI on its selected subscriber node without proxying the UI session through roarlay.
- Durable device and library catalog with stale/offline state, scan generations, and reconciliation.
- Define library configuration and scanning for mixed-media roots and category-specific roots distributed across multiple boneyard nodes; combine them in the instance catalog without moving media bytes.
- Complete device credential rotation and revocation behavior, identity continuity, and a durable, narrowly scoped command/catalog channel. The current approval, heartbeats, and scan request are the foundation.
- Keep user sign-in and PostgreSQL on roarlay. Multi-account invitations and multiple-coordinator auth/failover remain future work.
- Publish role-specific packaging for roarlay, fossil, boneyard, and an isolated operator service; the existing Compose files are local source-build examples.
- Implement explicit per-user sharing grants. Syncthing peer/folder configuration does not currently implement invitations, trust decisions, or selective sharing.
- Plan a future **tarpit** node type: an untrusted Syncthing destination for selected media replicated from trusted boneyard nodes, with destination-at-rest encryption so the destination cannot view the media. Not implemented; implementation has not started. Syncthing transport encryption alone is insufficient.

### Shared clusters and owner-controlled membership

- Keep new clusters isolated by default; support an explicit one-way transition to shared mode. The owner alone may invite or remove members; invitees cannot extend membership.
- Establish a genesis-anchored, append-only trust history for every cluster, including isolated-mode device enrollment/removal, role changes, shared-mode transition, member changes, root rotation, and recovery. Replicate and back up history, pin accepted heads locally, and detect gaps, forks, and rollback.
- Define owner-managed loss handling: if the age identity is lost, original media remains and encrypted metadata/config must be rebuilt where possible; if the signing root is lost, the owner may explicitly re-enroll devices under a new genesis/trust chain. Rawrarr provides no secret recovery bypass, and a new signing root must never be trusted silently.
- Decide whether MLS is warranted for group epochs over the eventually consistent Syncthing folder or whether an owner-signed append-only membership log is sufficient. Prototype single-writer commits, offline catch-up, conflicts, and stale-node quarantine before selecting a protocol.
- Keep age as file encryption. Its per-file random key and recipient stanzas are part of each age file; every authorized recipient gets a stanza that can recover the same key, adding only modest header size while encrypting the payload once. Stanzas need not contain public recipient IDs verbatim; keep those IDs in authenticated membership records. Do not create a persistent cluster-wide pre-shared key. Validate multi-recipient age for new files and test a maintained implementation's ability to add recipients to existing ciphertext without re-encrypting payloads; otherwise choose a history migration policy.
- Implement revocation so removed members lose future access, while explaining that they may retain previously received data. Request cooperative cleanup from their Rawrarr nodes and report pending, node-reported complete, or unconfirmed status without claiming secure erasure.
- Define invitation history scope, per-person versus per-device membership, root-key recovery/succession, rollback resistance, and reconnect behavior before production implementation.

### Private access, operator service, and relay

- Continue the private operator admin console from the sample-data preview: add server-enforced admin login/authorization before connecting production accounts, keep it in a separate container reachable only through localhost or Tailscale (never a public route), then add least-privilege live account reporting. Trial and subscription metrics must wait for implemented, authoritative billing state; do not infer them from account age.
- Cloudflare/reverse-proxy support for the intended public hostname, including verified SSE streaming behavior.
- Productionize the existing local Tailscale sidecars and tailnet-scoped routes while keeping transport separate from Rawrarr authorization and age-key access.
- Operator service node running SwarmSail/Headscale account coordination, with a one-time, revocable node claim flow.
- Provision a restrictive ACL with every Swarmsail-managed Headscale tailnet: allow nodes on that tailnet to communicate with one another only on TCP 443 and TCP 22000 (Syncthing), deny other traffic by default, and do not report provisioning complete until the ACL is applied and verified.
- Integrate one shared CrowdSec engine with SwarmSail-provisioned Headscale instances: register each instance's security logs when provisioned and remove the source when retired, then apply remediation at the serving ingress with decisions scoped to avoid unintended cross-user blocking.
- Blind relay and portal for reaching subscriber nodes through outbound tunnels. The subscriber node must retain application authentication and TLS termination; the relay must not decrypt or store subscriber app data or proxy media.

### Packaging and subscriptions

- Publish versioned, multi-architecture application images with release notes, provenance, and a documented upgrade path.
- Generate role-specific Compose files for private/subscribed coordinator, storage, shard, and combined coordinator + storage deployments. The current Compose files are local source-build examples.
- **One-line subscriber node deployment (planned):** build on the existing fossil/boneyard Compose preview with a copied curl command that downloads a script containing the generated Compose definition, checks or helps install Docker and Compose, and starts the node. Preserve explicit owner approval, local-only secrets, safe reruns, and the manual Compose option. Public versioned images are a prerequisite; see the [deployment plan](docs/one-line-node-deployment-plan.md).
- Hosted passkey account, checkout, trial, subscription/license issuance and enforcement. The selected direction is a shared-instance entitlement with signed licenses; provider, pricing, grace period, and final lock behavior remain undecided.

### Product capabilities and durability

- Restore private-key character verification after fixing the reported rejection of correct answers and validating the full browser flow.

- Flesh out the public landing page into an account-free interactive demo: add a clearly labeled entry point, a guided tour of representative screens, fictional fixture data, and only read-only or explicitly simulated interactions. Keep it isolated from subscriber data and protected APIs, preserve existing sign-in/workspace behavior, and label implemented, planned, and simulated capabilities accurately. See the [public demo plan](docs/public-demo-plan.md).
- Extend the initial roarlay OpenAI chat, press-to-talk voice, and PostgreSQL request inbox described in [the design guide](docs/DESIGN.md#ai-chat-and-request-review). Replace the narrow intent classifier and canned unsupported-question response with schema-bounded Responses function calling over scoped Rawrarr operations. Keep decrypted catalog work local to the browser by default; consider an MCP adapter over the same authorized operations for a separate owner-run review agent. Add owner-run review and scoped configuration proposals, searchable and deletable history with retention controls, and richer library questions. Tie the existing five-per-day limit to trial entitlement when licensing exists; determine paid-plan limits. Consider a Twilio phone entry point after signed-in browser voice, with caller authentication and explicit cloud-data consent.
- General-purpose file/folder sharing beyond media.
- Sonarr posters/artwork, snapshots, metadata lookup, and episode mutations; Lidarr, Readarr, and Bazarr integrations; Prowlarr-style indexer management and search; concrete download-client adapters and fulfillment. The current download-client abstraction has no concrete adapter.
- Manual age-encrypted shared-folder backup to one node-local destination, including recovery verification and a retention policy.
- Device signing, approval, signed shared configuration, and revocation/revision checks.
- Replace the Mac mini's generated device-ID display alias with a randomly selected, scientifically recognized non-avian dinosaur species name from a vetted, substantial taxonomy list. Exclude candidates whose genus or species epithet matches any word in the owner's current profile name, ignoring case and punctuation; if no candidate remains, ask rather than weaken the exclusion. Change only the alias in the existing age-encrypted device-name config, preserving the stable device ID and verifying the UI still identifies the same node.
- Static logo animation for selected moments, with reduced-motion behavior.

## Related documentation

- [Device and service architecture](docs/device-architecture.md) — current local prototype boundaries and target node model.
- [Product direction](docs/product-direction.md) — long-term product goals and trust boundaries.
- [Product and interface design](docs/DESIGN.md) — user-facing principles and flows.
- [One-line node deployment plan](docs/one-line-node-deployment-plan.md) — planned embedded-Compose installer, Docker bootstrap, platform support, approval boundaries, and acceptance criteria.
- [Public demo plan](docs/public-demo-plan.md) — planned account-free public tour, synthetic data, security boundaries, and acceptance criteria.
- [Shared cluster plan](docs/shared-cluster-sharing-plan.md) — planned isolated-to-shared transition, owner-controlled membership, MLS evaluation, age file handling, revocation cleanup, and phased work.
- [Technical boundaries](docs/technical-status.md) — data placement and security boundaries.
- [User guide](docs/user-guide.md) — current local prototype usage.
- [Testing](TESTING.md) — reusable dependency image and isolated runner workflow.
- [Runbooks](runbooks/README.md) — dated task evidence.
