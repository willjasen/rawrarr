# Device and scanner architecture

Every subscriber Rawrarr node is intended to run the full web/API service. Storage is an optional node capability: a **shard** omits `/media`, while a **storage node** runs the same service with its local media root. The operator-owned **service node** is a separate deployment role that will eventually run SwarmSail and coordinate subscriber tailnets; it is not part of a subscriber instance. See [node types](node-types.md) and the [Tailscale integration plan](tailscale-integration-plan.md).

## Ownership boundaries

- **Shard node:** runs the dashboard/API, local database, device coordination, metadata providers, and future download/import orchestration. It keeps a replica of the shared folder and has no `/media` mount.
- **Storage node:** runs the same dashboard/API and local database, syncs the same shared folder, and mounts its local media root read-only at `/media` for scanning. Media bytes stay on their storage node; only encrypted metadata and app data are shared unless a user explicitly configures otherwise.
- **Service node:** operator-owned SwarmSail/Headscale, account/subscription, and future blind-relay role. It coordinates tailnets but does not join or decrypt a subscriber's `/shared` data and does not proxy media.
- **Optional Tailscale sidecar:** joins the user's tailnet for private Rawrarr web/API and control/application-data traffic, plus optional Service advertisement. It has a node-local identity and state; Rawrarr can still start and be reached locally without it. Rawrarr never uses Tailscale to transfer media files or streams; `/media` remains on its storage node.
- **Download agent (future):** runs where its configured client or destination is available and receives narrowly scoped work from the coordinator.

## Data domains

Rawrarr's canonical path and synchronization rules are listed in [data domains](data-domains.md). `/media` is a storage-node-local read-only mount; `/metadata` is the metadata domain available on every node and aliases `/shared/metadata`; `/shared` is the Syncthing root, including compressed operational logs under `/shared/logs`; `/data` and PostgreSQL storage remain node-local. A web node may omit `/media` entirely.

## Local and shared app data

The local app-data volume holds node-specific operational state, including PostgreSQL's live data directory, session tokens, and authentication challenges. Rawrarr must never sync a live PostgreSQL data directory through Syncthing.

The separate Syncthing folder holds portable shared user and application records, including age-encrypted configuration (`config.aged`) and encrypted metadata/artwork (`.aged`). The metadata path `/metadata` points into that folder at `/shared/metadata`; it does not introduce a second sync root. It also contains the public age recipient so a scanner can encrypt new records without receiving a private identity; the recipient is public key material, not a secret. Rawrarr may keep a local PostgreSQL projection/cache of those records, but it is not the shared source of truth. Ciphertext and its directories must be readable/traversable by Syncthing; private age identities and plaintext config never belong in the share. Portable user/passkey record replication and reconciliation remain future work beyond the currently persisted encrypted settings/artwork.

The owner prefers passkey sign-in before age-identity unlock on a newly joined web node. Because the shared passkey records are themselves age encrypted, the bootstrap path that lets a new node verify the passkey before it can decrypt those records remains to be designed. Do not resolve this by synchronizing a PostgreSQL datadir or storing the private age identity in the share.


Rawrarr's wider direction is a unified media automation system with a Prowlarr-class discovery role plus library management and fulfillment. See [product direction](product-direction.md). This is not implemented yet: indexer management and provider search remain future work.

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

The app exposes `/api/devices` with a persistent per-node ID and reports whether `RAWARR_LIBRARY_PATH` is configured. The web API can scan its local root when the age identity is unlocked. A separate `storage-scan-once` command currently scans a storage root without PostgreSQL or the web server and uses the shared public age recipient to write encrypted JSON sidecars. Keep it as a utility, but the target persistent storage-node deployment runs the full web/API service too. The remote pairing/command channel and cross-node web/auth consistency are future work; see the [Tailscale plan](tailscale-integration-plan.md). The operator service node and its future SwarmSail provisioning integration are documented in [node types](node-types.md).

Next implementation: persist the device/library catalog, add one-time device pairing and revocation, and define remote command/heartbeat handling. Upgrade persistent storage nodes to the full web/API deployment while keeping `/media` local to each storage-capable node. Resolve cross-node auth/session and database-projection consistency before advertising multiple web nodes behind one Tailscale Service. A web host must never gain direct access to another node's media filesystem. The future public relay is a separate component and must not become a trusted Rawrarr application server.
