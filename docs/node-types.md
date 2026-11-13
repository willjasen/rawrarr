# Rawrarr node types

## Current roles

The implemented roles are identified internally as `coordinator`, `storage`, and `metadata`. Their user-facing names are **roarlay** (coordinator), **boneyard** (storage), and **fossil** (metadata). The names do not change capabilities: roarlay handles central authentication, passkey persistence, and node coordination; fossil and boneyard nodes both run the subscriber UI and shared app-data functionality, while boneyard nodes also have read-only local media access and scanning. Metadata replaces shard, and subscriber describes ownership rather than a container type. The former service role is retired. Compose services and generated containers use `roarlay`, `fossil`, and `boneyard` for these roles. Rust `RAWARR_ROLE` values, database role values, API route paths, and stable node IDs remain `coordinator`, `storage`, and `metadata` for compatibility.

## Dino deployments

A **dino** is a Rawrarr node deployed by a user. It describes the user's deployed node, not an additional capability or internal role. A dino may run a **fossil** metadata node and may also run exactly one media role: either a **boneyard** storage node with access to its local media, or a **tarpit** encrypted-media node. A dino must not run both boneyard and tarpit roles. The tarpit combination is planned only; tarpit is not implemented.

Roarlay remains the coordinator role; these dino combinations describe the fossil and media roles and do not change coordinator behavior.

See [container deployment](container-deployment.md) for the authoritative capability matrix, startup behavior, trust limitations, local migration, and Mac mini setup. The coordinator is the established central control plane. The MacBook fossil node is its seeded first participant; later fossil and boneyard nodes join that coordinator. No subscriber role connects directly to PostgreSQL.

## Planned tarpit nodes

**Tarpit** is the planned user-facing name for an untrusted Syncthing destination that receives selected media from a trusted boneyard and stores it encrypted at rest. The name evokes a dark, difficult-to-escape place, matching the intended property that the destination cannot view the encrypted media. Tarpit is a future node type, not an implemented Rawrarr role; no implementation work has started. A boneyard remains the Rawrarr storage role. Ordinary Syncthing transport encryption alone does not provide destination-at-rest encryption.

Manual, account-scoped node approval and node-scoped central authentication are implemented for the coordinator's current single-owner setup. Browser device-code enrollment, multi-account invitations, and SwarmSail provisioning remain planned. The sections below retain future distribution/networking direction; references to the old service/shard packaging are superseded by the three-role deployment guide.

## Instance and enrollment model

- A subscriber subscription is attached to the shared Rawrarr `instance_id`, not to a container. It covers that instance's coordinator, storage, and shard nodes under the agreed plan/device limits.
- Public-service direction: SwarmSail creates one isolated Headscale tailnet per subscriber account. The account's authorized Rawrarr nodes use that overlay, even if the account later has more than one Rawrarr instance; the precise instance/account subscription mapping remains a separate product decision. Internal deployments may instead enroll each node in an owner-controlled Tailscale tailnet. The subscriber installer selects the control server; this does not change Rawrarr approval, coordinator access, or Syncthing sharing.
- Internal Mac mini topology: the intended coordinator URL is the mini's HTTPS Tailscale Serve name. Subscriber nodes join the same owner tailnet and use that URL for Rawrarr registration and API calls. The Compose generator accepts the URL; this documents the target and does not mean the current mini deployment has been migrated from boneyard to coordinator.
- Eligibility for the planned account tailnet begins with a verified active trial or subscription, not sign-in alone. The account starts isolated. Its owner may permanently make the tailnet shared; returning to a private security domain requires creating a new tailnet and reenrolling devices. SwarmSail's local mode transition exists, but automated provisioning, owner authentication, cross-tailnet routes, and live deployment are not implemented.
- Shared tailnet access is planned as an explicit device/service pair chosen by the owner, not a blanket link among all shared tailnets. Show a grant as active only after the cross-instance route and narrow policy are installed and verified. An address prefix such as `/30` can size a point-to-point link; it does not itself enforce membership or permission.
- Tailscale is optional on subscriber deployments. The MacBook coordinator may use a Tailscale sidecar for private Serve; fossil and boneyard nodes can connect outbound to its HTTPS endpoint without a Tailscale identity. SwarmSail on the operator's service node may use an overlay for authorized control and short-lived enrollment.
- A node keeps its own Headscale/Tailscale identity and state locally. Enrollment secrets are transient and must never enter Compose files, shared Syncthing data, age-encrypted app configuration, logs, or the service database after use.
- Tailscale is an overlay transport only. It does not determine Rawrarr sharing membership or data permissions; Rawrarr's own authorization and age recipients determine which data a person can access or decrypt. Do not couple tailnet membership to data access in the initial design.
- The Headscale network carries Rawrarr app access, control, and app-data traffic only. It must not carry media files or streams. Policies must explicitly allow only required Rawrarr endpoints; Headscale's empty/unconfigured policy can allow all node-to-node traffic.
- SwarmSail can coordinate tailnet membership and device visibility. It does not make the service node a trusted Rawrarr application backend, database replica, or media proxy.

## Container configuration direction

### Image distribution

The implementation publishes public multi-architecture images to `ghcr.io/willjasen/rawrarr` on relevant pushes to `main`. The generated subscriber Compose uses the rolling `main` tag with `pull_policy: always`; the workflow also publishes a commit-SHA tag. Subscriber nodes pull anonymously and do not need a GitHub token. A signed image provenance policy remains future work.

Treat the application image as non-secret. Persist subscriber state only in the documented `/data` and `/shared` mounts. Keep original local media under `/media`; the target design allows only explicitly selected copies to be published under `/shared/out`. Upgrades replace the container while preserving its volumes. Do not bake private age identities, passkeys, credentials, media, or node-local databases into the image. The operator's `service` role may share the published application image only if its entrypoint and dependencies remain role-isolated; otherwise publish a separate service image. The service node must never receive subscriber-node mounts through a shared deployment template.

The development Compose files continue to build `rawrarr:local` from source. Automated public GHCR publishing and anonymous subscriber Compose pulls are implemented; release version policy, update notifications, and a signed provenance mechanism remain open.

The eventual Docker packaging should expose a validated role setting, with syntax still to be chosen for `coordinator`, `storage`, `shard`, and `service`, including coordinator + storage combinations, and reject unknown or incompatible settings at startup. Compose examples should make role-specific mounts and services explicit. In particular:

- `service` starts operator services only and does not mount subscriber `/shared` or `/media`.
- `coordinator` starts the subscriber web/API, passkey authentication, PostgreSQL, catalog, and device coordination.
- `storage` starts the media worker without PostgreSQL or user-login routes, mounts authorized shared data and its local read-only `/media`, and connects outbound to its coordinator.
- `shard` maintains an authorized encrypted app-data replica without media or an implicit web/auth/database stack; additional serving behavior remains open.

Whether these roles use one image with separate entrypoints or distinct images remains an implementation choice. Do not let a single broad Compose mode blur the service-node privacy boundary.

## Docker Compose deployment and claim flow (planned)

The private storage-to-coordinator link is a separate flow from operator account claims. The worker logs a coordinator HTTPS linking URL and short-lived device code, the owner signs in, enters the code and approves, and the worker proves its device key. See [coordinator linking](coordinator-linking-plan.md). The operator portal below is not required for this private setup.

The Rawrarr site should provide a Compose file tailored to the user's selected role (`coordinator`, `storage`, `shard`, or coordinator + storage) and deployment options. The user saves it locally and runs `docker compose up -d`; the node then initiates an outbound connection to the operator's service node so it can be discovered and claimed without requiring an inbound port on the user's network.

On first boot, the node should create its device identity and register as a pending device. The owner signs in to the Rawrarr portal with a passkey and explicitly claims that pending device into a private workspace or subscribed account. The node should prove continuity with its locally generated device key. A log message may include a non-secret URL or device identifier to help the owner find the pending claim, but must not expose a reusable bearer credential. The Compose YAML must not contain an enrollment secret, age private identity, or passkey material. Any short-lived bootstrap proof must be single-use, audience-bound, and delivered or confirmed without persisting it in Compose, logs, or shared app data.

The portal should distinguish private deployments from subscribed deployments, apply the relevant entitlement only after claim, and explain which account/service information is sent upstream. Claiming grants the service node routing and account metadata only; it does not make the service node a member of the subscriber's Syncthing folder or expose media, app data, age identities, or node-local databases. A private deployment should remain useful if optional subscription networking is unavailable, subject to its chosen private-mode access model.

Compose generation is planned product behavior; the current repository does not yet provide a claim portal or authenticated node enrollment protocol. See [cloud access architecture](cloud-access-architecture.md) for the hosted service boundary.

## Still to decide

- Whether each subscriber instance gets its own SwarmSail organization and dedicated Headscale instance, or a dedicated organization/tailnet on a shared Headscale instance. SwarmSail currently models isolated organizations against assigned Headscale instances; deployment capacity and automation need review before choosing.
- How a Rawrarr account's tailnet relates to multiple Rawrarr instances, if the product later allows them.
- Whether an invited collaborator joins the owner's account tailnet or connects from a separate account tailnet through an explicitly limited bridge. Defer this decision; it does not change Rawrarr's independent sharing and age-key authorization.
- Whether the service node and SwarmSail management API are one deployment or separate containers on the same operator host.
- Subscriber node/device limits, revocation on cancellation, trial expiry, or device removal; retain user data after subscription lapse.
- Least-privilege Headscale grants and exact app ports/protocols; deny media sharing and broad host/network access.
- How a subscriber node receives its one-time enrollment secret without persisting it in static Compose configuration or exposing it in process listings/logs.

## Related architecture

- [Device architecture](device-architecture.md) — per-instance device and storage behavior.
- [Cloud access architecture](cloud-access-architecture.md) — hosted portal, relay, and subscriber-data boundary.
- [Subscription model](subscription-model.md) — instance-based entitlement.
- [Tailscale integration plan](tailscale-integration-plan.md) — optional Rawrarr app-data connectivity.
