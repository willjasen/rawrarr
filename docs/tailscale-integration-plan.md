# Optional Tailscale integration plan

## Product decision

Tailscale will be an optional connectivity layer for Rawrarr. A subscriber Docker deployment will run a separate Tailscale container/sidecar. SwarmSail on the operator's service node will eventually create one Headscale tailnet per subscriber account, enroll that account's Rawrarr containers, and communicate with them over the overlay. Rawrarr must remain usable on a local network or another private network when Tailscale is not installed, logged in, or reachable.

Tailscale is an overlay transport only: private access to Rawrarr web/API service and Rawrarr control/application-data exchanges between nodes. Membership does not authorize data access or decrypt shared content; Rawrarr permissions and age recipients govern that. Tailscale must not carry media files, media streams, or other `/media` storage payloads. Keep media on its storage node. The shared Rawrarr app-data folder is synchronized separately through Syncthing; Tailscale may provide private network reachability for Rawrarr endpoints but is not a media replication mechanism.

Every Rawrarr node will run the full Rawrarr web/API service. A node's role comes from its mounted data domains:

- **Web-only node:** runs the web/API service and its local database, mounts `/shared` and `/metadata`, and has no `/media` mount.
- **Web + storage node:** runs the same web/API service and local database, mounts `/shared` and `/metadata`, and also mounts its own `/media` read-only for scanning.

This replaces the planned long-term split where a storage/scanner node runs only a headless process. The existing `storage-scan-once` command remains a useful scanner/testing mode, but is not the target deployment for a persistent storage node.

## Name and routing model

Each node gets its own stable Tailscale machine identity and MagicDNS device name. In addition, a user's tailnet can define a Tailscale Service named `rawrarr`; its MagicDNS name is the stable entry point, for example `https://rawrarr.<tailnet-name>.ts.net`. Tailscale Services can advertise multiple service hosts, letting clients use one service name while Tailscale routes to available hosts. They are tailnet-scoped, not public DNS or a replacement for the future Rawrarr relay. Individual node MagicDNS names remain useful for diagnostics and explicitly targeting one node.

The service name is only a network entry point. It does not choose a particular user's library, make the node databases consistent, or tell Rawrarr which storage device owns a media path. Rawrarr's `instance_id`, device identity, shared-folder data, and app-level authorization remain authoritative. Direct node names are used when an operation specifically targets one device; the shared service name is used for general web access.

## Docker shape

Package the optional Tailscale client as a separate sidecar in both subscriber deployment shapes. Keep the core Rawrarr web/API container identical; the storage role mounts local `/media`, while the shard role does not. When enabled, the sidecar gets its own per-node persistent state and unique machine hostname, and exposes the Rawrarr endpoint to the account tailnet. SwarmSail coordinates enrollment and management from the operator service node. When disabled, local Compose port and private-network access continue to work. Do not treat Tailscale membership as a Rawrarr authorization grant.

The integration must not require a Tailscale auth key in the Compose file, shared folder, or `config.aged`. Support an interactive first login or a narrowly scoped secret-based enrollment flow; persist the resulting Tailscale state only in a node-local volume. Never share a Tailscale state directory between nodes. Start with the least privileged networking mode supported by the target Docker runtime; compare userspace networking with kernel/TUN mode for inbound Service/Serve behavior before fixing the Compose implementation.

Tailscale Serve is the per-node private HTTPS option. Tailscale Services is the multi-host stable-name option. A Service host requires a tag-based Tailscale identity, and advertising a Service may require tailnet administrator approval. Current Tailscale documentation lists version 1.86.0 or later for stable Services, TCP endpoints, and a host configured with Tailscale; recheck requirements and plan compatibility when implementation begins. Do not enable Funnel or public exposure as part of this feature.

## Build order

1. **Make each node a full Rawrarr node.** Replace the long-term scanner-only deployment assumption with web/API + local database on storage nodes; retain `/media` as an optional capability. Define per-node health, readiness, and device identity.
2. **Make arbitrary-node web access correct.** Identify which data currently exists only in local PostgreSQL, which auth/session state can be shared safely, and how shared age identity unlock works. Before putting multiple nodes behind one Service, prove that a request can land on any healthy backend without losing sign-in, settings, device visibility, or catalog consistency. Syncthing file replication is not database replication or quorum.
3. **Add the optional sidecar to both Compose shapes.** Implement independent persistent state per node, safe enrollment, stable hostname configuration, health/status reporting, and clean disable/removal behavior. Keep Rawrarr boot and local access independent from Tailscale availability.
4. **Expose one node privately.** Use Tailscale Serve to reach a node's web UI over its MagicDNS HTTPS name. Verify that passkey authentication remains enabled and that access is still bounded by the tailnet ACLs and Rawrarr's own authorization.
5. **Add the stable `rawrarr` Service.** Define a Service in the user's tailnet and advertise eligible Rawrarr nodes as hosts. Verify joining/leaving behavior, health-aware routing, TLS, service-host tag requirements, approval, and access policies. Only advertise a node that is ready to serve the complete shared instance.
6. **Test outage and no-Tailscale paths.** With two nodes online, use the stable Service name; take one node offline and confirm the remaining eligible node serves. Then disable/remove Tailscale and confirm the local/private-network Rawrarr URL still works.

## Acceptance checks

- Both web-only and web+storage Compose deployments run the full web/API app.
- A storage node can scan only its own `/media`; a web-only node has no media mount.
- Each node has independent Rawrarr and Tailscale identities/state while sharing the same Rawrarr `instance_id` and Syncthing folder.
- `https://rawrarr.<tailnet-name>.ts.net` works when Service routing is enabled and more than one eligible node exists; a per-node MagicDNS URL can still reach a selected node.
- Passkey login, encrypted configuration access, metadata, and device status work regardless of which eligible backend handles the request.
- Tailscale secrets and private state never enter Syncthing-shared files or logs. Tailscale disabled/unavailable does not prevent local app startup or local access.
- No public Funnel or public listener is created by default.
- Tailscale carries only Rawrarr web/API, control, and app-data traffic; media files and streams remain on storage nodes and are never routed as Tailscale payloads by Rawrarr.

## Official references

- [Tailscale Services](https://tailscale.com/docs/features/tailscale-services) — stable MagicDNS Service names, multiple hosts, TCP endpoints, tag-based hosts, and current client requirements.
- [Tailscale Serve](https://tailscale.com/docs/features/tailscale-serve) — private access to a per-node local service and HTTPS prerequisites.
- [Tailscale Docker Compose guide](https://tailscale.com/docs/features/containers/docker/how-to/connect-docker-container) — container enrollment and local persistent state.
- [Userspace networking for containers](https://tailscale.com/docs/concepts/userspace-networking) — userspace mode and Docker `TS_USERSPACE` behavior.
- [MagicDNS](https://tailscale.com/docs/features/magicdns) — device hostnames and tailnet-specific fully qualified names.
