# Rawrarr node types

## Roles

Rawrarr has three deployment roles. They share the packaged product, but the operator-owned service node and subscriber-owned Rawrarr nodes are separate trust and data boundaries.

| Role | Owner and purpose | Runs | Data and capabilities |
| --- | --- | --- | --- |
| **Service node** (`service`) | Rawrarr operator's hosted main node | SwarmSail/Headscale coordination, hosted account and subscription services, and the future blind relay | Stores service accounts, passkey records, public age recipients, billing/license state, instance/device IDs, and Headscale provisioning state. It does not join a subscriber's Rawrarr data folder, receive private age identities, or store/decrypt subscriber app data or media. |
| **Storage node** (`storage`) | Subscriber's device that holds media | Full Rawrarr web/API plus local storage scanning | Has the subscriber's instance and device IDs, local app database/state, replica of `/shared`, and one or more local media roots mounted read-only under `/media`. Media is local to this node and is never copied through SwarmSail/Headscale. |
| **Shard node** (`shard`) | Subscriber's media-less device | Full Rawrarr web/API | Has the subscriber's instance and device IDs, local app database/state, and replica of `/shared`; it has no `/media` mount. It can serve the UI/API and participate in the subscriber's app-data mesh. |

`service`, `storage`, and `shard` are explicit deployment roles, not user privilege levels. A storage node and shard node are both subscriber Rawrarr nodes and both run the web/API. The distinction is whether a local media root is mounted. The service node is not a subscriber node and is not part of their `instance_id` or Syncthing folder.

## Instance and enrollment model

- A subscriber subscription is attached to the shared Rawrarr `instance_id`, not to a container. It covers that instance's storage and shard nodes under the agreed plan/device limits.
- SwarmSail creates one isolated Headscale tailnet per subscriber account. The account's authorized Rawrarr nodes use that overlay, even if the account later has more than one Rawrarr instance; the precise instance/account subscription mapping remains a separate product decision.
- Each subscriber Docker deployment runs a Tailscale container/sidecar. SwarmSail on the operator's service node communicates with authorized Rawrarr containers over that account's overlay and coordinates short-lived, single-use enrollment.
- A node keeps its own Headscale/Tailscale identity and state locally. Enrollment secrets are transient and must never enter Compose files, shared Syncthing data, age-encrypted app configuration, logs, or the service database after use.
- Tailscale is an overlay transport only. It does not determine Rawrarr sharing membership or data permissions; Rawrarr's own authorization and age recipients determine which data a person can access or decrypt. Do not couple tailnet membership to data access in the initial design.
- The Headscale network carries Rawrarr app access, control, and app-data traffic only. It must not carry media files or streams. Policies must explicitly allow only required Rawrarr endpoints; Headscale's empty/unconfigured policy can allow all node-to-node traffic.
- SwarmSail can coordinate tailnet membership and device visibility. It does not make the service node a trusted Rawrarr application backend, database replica, or media proxy.

## Container configuration direction

The eventual Docker packaging should expose a validated role setting, provisionally `RAWRARR_MODE=service|storage|shard`, and reject unknown or incompatible settings at startup. Compose examples should make role-specific mounts and services explicit. In particular:

- `service` starts operator services only and does not mount subscriber `/shared` or `/media`.
- `storage` starts Rawrarr web/API and node-local services, mounts that instance's shared app-data folder, and mounts its media root(s) read-only under `/media`.
- `shard` starts the same Rawrarr web/API and shared app-data participation as `storage`, but has no media mount or scanner access to host paths.

Whether these roles use one image with separate entrypoints or distinct images remains an implementation choice. Do not let a single broad Compose mode blur the service-node privacy boundary.

## Docker Compose deployment and claim flow (planned)

The Rawrarr site should provide a Compose file tailored to the user's selected role (`storage` or `shard`) and deployment options. The user saves it locally and runs `docker compose up -d`; the node then initiates an outbound connection to the operator's service node so it can be discovered and claimed without requiring an inbound port on the user's network.

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
