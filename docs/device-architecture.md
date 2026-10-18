# Device and scanner architecture

The accepted subscriber roles are coordinator, storage, and shard. The coordinator runs the web/API, user authentication, PostgreSQL, catalog, and device coordination; storage is a read-only media worker without PostgreSQL or user passkeys; shard retains shared ciphertext with further serving duties undecided. Coordinator and storage capabilities can coexist. This MacBook fills the coordinator role; the Mac mini is planned as storage. The operator service is separate. See [node types](node-types.md) and [coordinator linking](coordinator-linking-plan.md).

## Current prototype boundary

The current deployment is one local Rawrarr web/API process with its own PostgreSQL database. PostgreSQL holds that node's passkey credentials, WebAuthn ceremony state, and sessions. A successful passkey registration/sign-in sends the browser to age setup when the encrypted config is still locked. Creating an ID happens in the browser-facing setup flow: the server returns the new identity for client-side copying, but the app does not save it in PostgreSQL, `/shared`, logs, or a service-node component. A generated identity opens the copy pop-up; the user copies it, confirms the copy, and enters three random characters after the fixed key prefix. A correct response unlocks the vault and navigates to the dashboard. Pasted identities use the same copy and verification steps, then require the user to select **Unlock vault**.

The unlocked configuration is saved as age ciphertext in `/shared/config.aged`. Dashboard snapshots are sent over the web process's `/api/events` server-sent events route. The `/health` route reports service health for the container health check. Current local Compose is a development/self-hosting shape; it does not provision subscriber instances or run the future operator service, claim protocol, Headscale automation, or public relay. Public-domain/proxy behavior for long-lived dashboard event streams still requires end-to-end deployment verification.

This local prototype boundary is distinct from the intended deployment roles below. The target storage and shard nodes, service node, and enrollment model are product architecture, not a claim that those systems are currently deployed.

## Ownership boundaries

- **Coordinator node:** subscriber web/API, passkeys, PostgreSQL, catalog, enrollment, and task coordination.
- **Shard node:** encrypted app-data replica; no implicit coordinator/database or media access. Additional serving behavior is undecided.
- **Storage node:** reads local media beneath `/media`, publishes encrypted inventory/metadata, and authenticates outbound to the coordinator with its own device identity. No local PostgreSQL, user passkeys, or private age identity is required for encryption-only scanning. Mixed and category-specific libraries may contribute to the coordinator catalog; multi-library aggregation remains planned.
- **Service node:** operator-owned SwarmSail/Headscale, account/subscription, and future blind-relay role. It coordinates tailnets but does not join or decrypt a subscriber's `/shared` data and does not proxy media.
- **Optional Tailscale sidecar:** joins the user's tailnet for private Rawrarr web/API and control/application-data traffic, plus optional Service advertisement. It has a node-local identity and state; Rawrarr can still start and be reached locally without it. Rawrarr never uses Tailscale to transfer media files or streams; `/media` remains on its storage node.
- **Download agent (future):** runs where its configured client or destination is available and receives narrowly scoped work from the coordinator.

## Data domains

Rawrarr's canonical path and synchronization rules are listed in [data domains](data-domains.md). `/media` is a storage-node-local read-only mount; `/metadata` is the metadata domain available on every node and aliases `/shared/metadata`; `/shared` is the Syncthing root, including compressed operational logs under `/shared/logs`; `/data` and PostgreSQL storage remain node-local. A web node may omit `/media` entirely.

## Local and shared app data

Node-local `/data` holds device identity and operational state. PostgreSQL has its own coordinator-local volume for user authentication and future catalog/control records. Storage progress or queues can use local files or a lightweight store if needed. Never sync a live PostgreSQL datadir.

The separate Syncthing folder holds portable shared user and application records, including age-encrypted configuration (`config.aged`) and encrypted metadata/artwork (`.aged`). The metadata path `/metadata` points into that folder at `/shared/metadata`; it does not introduce a second sync root. It also contains the public age recipient so a scanner can encrypt new records without receiving a private identity; the recipient is public key material, not a secret. Rawrarr may keep a local PostgreSQL projection/cache of those records, but it is not the shared source of truth. Ciphertext and its directories must be readable/traversable by Syncthing; private age identities and plaintext config never belong in the share. See the [roadmap](../ROADMAP.md) for device enrollment and catalog synchronization work.

Owners sign in on the coordinator before approving device links. Storage-only and shard nodes do not need copies of user passkeys or browser sessions. Future multiple-coordinator authentication/failover remains a separate design question, not a prerequisite for storage enrollment.


Rawrarr's wider direction is a unified media automation system with a Prowlarr-class discovery role plus library management and fulfillment. See [product direction](product-direction.md) for the intended design and the [roadmap](../ROADMAP.md) for implementation status.

A device may publish multiple named libraries. Their host paths stay on that device. The web catalog identifies files by device, library, and relative path; it never needs the device's absolute mount path. If a device goes offline, its last inventory remains visible and is marked stale.

## Device record

A device record needs a stable ID, display name, role/capabilities, connection type, software version, last-seen time, and its library summaries. Libraries need stable IDs and display names. Scan snapshots carry relative file path, media kind, size, optional modification time, and optional content fingerprint. A future implementation should report removals as tombstones or full-snapshot generations so stale entries can be reconciled safely.

## Shared instance identity

`instance_id` identifies one logical Rawrarr shared folder/namespace. Every node that synchronizes that folder has the same `instance_id`; each node still has its own unique `device_id`. A device may participate in multiple instances, and an instance may have multiple devices. The ID represents the shared folder identity, not its host path, a person, a running process/container, or a physical device. Moving the folder or changing sync providers must not change it.

Keep the Rawrarr `instance_id` independent from the transport's folder identifier. For Syncthing, map the Rawrarr instance to a Syncthing folder ID; a future transport can map the same Rawrarr instance to its own identifier. The shared instance manifest records the stable `instance_id` and is replicated with the shared state. Creating an independent copy/fork should mint a new ID; restoring or adding a node to the same shared instance preserves the existing ID.

An `instance_id` is an identifier, not a secret or authorization token. Access still requires an authenticated device membership and a valid user/share grant. The public relay may use the `instance_id` to find participating device IDs and live routes, but knowing it must never permit a connection or reveal instance contents.

## Pairing and transport

The agent should initiate outbound HTTPS requests to the web service; the web service does not open inbound connections to media hosts. Pairing should create a per-device random token shown once, store only a token hash centrally, support revocation/rotation, and scope each token to that device. Tailscale HTTPS can carry Rawrarr web/API, control, and app-data traffic when the services are private to the tailnet. Tailscale is not a media transport: heartbeats and scan uploads must never include API keys, absolute media paths, or media bytes, and Rawrarr must not route media files or streams through it.

## Cloud portal and public relay boundary (future)

The cloud portal should be an access and rendezvous service, not the application controller. It may authenticate a portal account, list authorized opaque instance/device IDs, and find online nodes. The self-hosted node remains responsible for the Rawrarr application, passkey authentication, authorization, and configuration access. Treat the relay as a routing control plane and an opaque byte-forwarding data plane, not as a reverse proxy that terminates application TLS. The Rawrarr node initiates and maintains an outbound authenticated tunnel so it can work behind NAT; the browser's TLS session must terminate at that node.

The portal/relay may retain account data needed for portal sign-in, an opaque stable `instance_id`, member `device_id`s, public keys needed to authenticate outbound connections, and public age recipients associated with the passkey-authenticated portal account. Public age recipients are not credentials. It will also need transient routing state such as which devices are online and which tunnel to use. Network metadata (source IP, connection times, bytes transferred, and likely a routable hostname or SNI) is visible to the relay even when payloads are encrypted; minimize retention and do not log payloads. Instance and device IDs are identifiers, not access credentials. Use short-lived, audience-bound routing tickets and enforce access at the node. The portal must not have a server-side admin channel into subscriber nodes. It must never receive a private age identity or decrypt subscriber app data; pass subscriber traffic through as end-to-end encrypted bytes without retaining a payload copy.

Serve the Rawrarr UI through the node's end-to-end encrypted session so portal/relay-controlled page code cannot inspect application responses. The portal may provide discovery or bootstrap navigation, but it must not serve executable UI code for a session that claims relay-blind privacy. Prefer direct peer connections when available and use the relay as fallback. This is a design constraint, not an implemented feature; TLS passthrough, certificates for instance hostnames, routing, abuse controls, and failure behavior remain to be designed and tested. See the [cloud access architecture](cloud-access-architecture.md) for the proposed component model.

## Current implementation and next step

The app exposes `/api/devices` with a persistent per-node ID and reports whether `RAWARR_LIBRARY_PATH` is configured. The web API can scan its local root when the age identity is unlocked. A separate `storage-scan-once` command scans a storage root without PostgreSQL or the web server and uses the shared public age recipient to write encrypted JSON sidecars. This is the existing database-free foundation for a future persistent storage worker; device linking, authenticated command polling, durable retries, and coordinator catalog consumption are not implemented. See the [roadmap](../ROADMAP.md) for pairing, cross-node consistency, and the operator service milestone, and the [Tailscale plan](tailscale-integration-plan.md) for transport design.

See the [roadmap](../ROADMAP.md) for the next implementation milestone and the remaining multi-node work. The boundary in this guide remains: a web host must never gain direct access to another node's media filesystem, and the future public relay must not become a trusted Rawrarr application server.
