# Optional Tailscale integration plan

Status: planned; role and device-code direction revised 2026-10-18. No Tailscale configuration was changed for this plan.

## Product decision

Tailscale is an optional connectivity layer. Rawrarr remains usable through another trusted private transport. It carries coordinator web/API, authenticated device control, and application data only; never media files/streams or PostgreSQL access. Tailscale membership does not authorize Rawrarr data access or decrypt ciphertext.

The MacBook's current local deployment is the subscriber **coordinator**. The Mac mini is the planned **storage** worker without PostgreSQL or user passkeys. **Shard** means encrypted app-data replica, with further serving duties undecided. Coordinator and storage may be combined elsewhere. This supersedes the earlier full-web-app-on-every-node direction. See [node types](node-types.md).

## Private coordinator endpoint

Prefer a stable named Tailscale Service such as `https://rawrarr.<tailnet>.ts.net` with the coordinator as its sole initial backend. The name is illustrative, not a verified or published endpoint. Per-device Tailscale Serve can first verify access under the coordinator host's MagicDNS name. Serve exposes a local endpoint; a named Service supplies a stable service identity and routing. Neither enrolls Rawrarr devices by itself.

Only an actual coordinator can serve this endpoint. Do not advertise storage/shard replicas as interchangeable backends or claim failover from shared ciphertext. Multiple coordinators would require a separate database, authentication, and consistency design.

Keep the existing local app at `127.0.0.1:7297`. A host-level Serve proxy can target that listener if supported by the installed client. Before publication, inspect existing Serve routes and avoid overwriting them. Named Services require compatible clients, tagged service hosts, service definition/host approval, and access policy; verify current requirements and the installed macOS client support before choosing host-level versus a dedicated sidecar identity. Do not retag the user's whole MacBook merely to fit the design without assessing its existing identity/access. A dedicated sidecar avoids conflating host and app identity, but needs separate enrollment and verified container routing.

For a sidecar, use independent persistent Tailscale state and the app's explicit Docker network address; sidecar localhost is not the MacBook host. Compare userspace and kernel/TUN requirements before adding privileges. On the Mini, prove DNS and TLS reachability from inside the actual OrbStack worker container even if the host already uses Tailscale. Do not place auth keys in Compose, logs, `/shared`, or encrypted app configuration. Enrollment uses interactive login or a scoped secret mechanism, and all resulting private state remains node-local.

## Link storage to the coordinator

Configure the worker with the coordinator's trusted HTTPS base URL. The worker registers a key-bound pending device, then prints the `/devices/link` URL and a short-lived human code in local Docker logs. The owner signs in on the coordinator, enters the code, reviews the device, and explicitly approves. The worker polls with key proof and retains its own approved identity. See the [linking plan](coordinator-linking-plan.md) for expiry, rate limits, revocation, and log boundaries.

Tailscale reachability must exist before Rawrarr linking. The logged URL does not itself enroll Tailscale or open the tailnet to the browser. Use an authenticated browser/device that can reach the private coordinator.

## WebAuthn and outage behavior

Existing localhost RP credentials are not automatically usable at a new Tailscale hostname. Implement a deliberate HTTPS-origin/RP enrollment or migration strategy, test login and approval there, and retain existing local access. Do not merely change Compose origin variables and assume existing credentials survive. Device API authentication remains independent from browser passkeys.

When the MacBook sleeps or disconnects, linking and new commands wait; worker retries use bounded backoff and queued state. Local media stays available to its other consumers. Define whether approved scans can continue offline and how their results are queued before implementation. A shard does not automatically become the coordinator during an outage.

## Build order and acceptance

1. Separate worker startup from web/auth/PostgreSQL, building on `storage-scan-once`.
2. Implement coordinator device-code enrollment, owner approval, worker authentication, durable identity, and revocation.
3. Inspect host/tailnet state; choose the appropriate Serve/Service identity without disturbing existing routes. Verify named-Service platform support or first validate a per-device Serve endpoint.
4. Configure a controlled HTTPS/WebAuthn path, scoped network access, and a single coordinator backend. Keep transport optional and local listener usable; do not enable Funnel.
5. Prove DNS, certificate verification, enrollment, poll/command authorization, and encrypted inventory delivery from the Mini's actual container. Test denial, expired codes, revoked keys, service restart, coordinator outage, and worker restart.
6. Verify catalog consumption separately from `/health` and file synchronization. Test local access with Tailscale unavailable. Document exactly what remains unimplemented.

## Future operator infrastructure

SwarmSail/Headscale remains the future operator service direction for isolated subscriber networks and hosted account claims. It is separate from the private coordinator device-code flow. Do not assume Tailscale SaaS Serve/Services or certificate behavior is supported identically by Headscale; verify compatibility independently. The operator must never receive subscriber private identities, decrypted application data, or media.

## Official references

- [Tailscale Services](https://tailscale.com/docs/features/tailscale-services) — named services, host eligibility, approval, and routing.
- [Tailscale Serve CLI](https://tailscale.com/docs/reference/tailscale-cli/serve) — private endpoint proxying.
- [MagicDNS](https://tailscale.com/docs/features/magicdns) — device DNS names.
- [Docker integration](https://tailscale.com/docs/features/containers/docker/how-to/connect-docker-container) — container identity and networking.
