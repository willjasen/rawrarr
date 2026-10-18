# Optional Tailscale integration plan

Status: current deployment notes; role, device-code, and two-host container direction revised 2026-10-18. Syncthing relay fallback was verified in the two-Mac test, and the previous Tailscale TCP forward was removed.

## Current MacBook routes (2026-10-18)

The local MacBook deployment uses two distinct paths. The coordinator shares the Tailscale container's network namespace, and Tailscale Serve on 443 proxies directly to `http://127.0.0.1:8080`. Tailscale terminates HTTPS; no Caddy process or coordinator-proxy container is needed. The Tailscale container publishes the coordinator's diagnostic port 8080 only as host loopback port 7298. Metadata and storage nodes use their configured `RAWARR_COORDINATOR_URL` over HTTPS and do not depend on a coordinator Compose service or local network alias.

Syncthing peer traffic uses global discovery and the public relay pool when direct connectivity is unavailable. The old Tailscale Serve TCP forward on port 22000 has been removed, and no inbound Syncthing port is published. Each Syncthing GUI remains bound to its Mac's loopback at port 8385. Production onboarding must wait for explicit owner approval before configuring a new node's Syncthing peers or folders. Confirm the coordinator HTTPS handler with `tailscale serve status --json`; it is the only MacBook Serve route.

This describes the current local deployment. Any alternate design must preserve the HTTPS origin and passkey RP/origin, keep the coordinator separate from metadata, and provide the loopback backend expected by Serve. See [container deployment](container-deployment.md) and [the route runbook](../runbooks/2026-10-18-coordinator-https-routing.md).

## Two-host container deployment (2026-10-18)

Use the dedicated `rawrarr-macbook-tailscale` container for the MacBook coordinator's private Serve endpoint. A Tailscale container on the Mini is optional and is not required for storage or Syncthing: storage uses the coordinator's HTTPS URL, while Syncthing uses global discovery and public relays when direct connectivity is unavailable. If a Mini Tailscale container is used for another private route, give it separate persistent node-local state; never share state between the Macs or put auth material in synchronized `/shared` data.

On the MacBook, the Tailscale container must reach the coordinator's internal Docker listener (`coordinator:8080`) and run Tailscale Serve to publish only that coordinator over tailnet HTTPS. A dedicated Compose network shared by coordinator and Tailscale container is preferred; if Serve must proxy to `127.0.0.1:8080`, first verify a supported shared network namespace design. Do not route through the MacBook's native Tailscale service or publish the coordinator/API or PostgreSQL ports publicly. The local metadata UI remains on `127.0.0.1:7297`.

On the Mini, the storage node connects directly to the coordinator's HTTPS endpoint over ordinary outbound HTTPS. Tailscale can be used as an alternate route, but is not a prerequisite. Media remains local and read-only, and `/shared`, `/data`, PostgreSQL, passkeys, and node credentials retain their existing boundaries. PostgreSQL remains private to the coordinator's Docker network and must never be reachable through Tailscale.

On the MacBook, inspect OrbStack's support for the Tailscale container networking mode. The image defaults to userspace networking, while kernel mode requires TUN and added capabilities; do not add those privileges without a concrete need. Prototype Serve ingress from a second tailnet device and confirm that Serve remains tailnet-private without Funnel. Verify coordinator health over HTTPS from the Mini storage container's normal network. Use interactive enrollment or a protected secret-injection path; never bake auth keys into Compose, logs, or Git.

Deployment sequence: (1) confirm the Mini's OrbStack Docker engine and safe persistent paths; (2) configure MacBook Serve to the coordinator only; (3) test the HTTPS endpoint from a second tailnet device; (4) set the Mini storage node's `RAWARR_COORDINATOR_URL` to the public HTTPS endpoint, provision/transfer only its own node token securely, then link it; (5) enable Syncthing global discovery and public relays on both peers and verify a relay connection when direct connectivity is unavailable; (6) verify coordinator access and Syncthing operation with the Mini Tailscale sidecar stopped. PostgreSQL remains private to the coordinator Docker network.

## Product decision

Tailscale is an optional connectivity layer. Rawrarr remains usable through another trusted private transport. It carries coordinator web/API, authenticated device control, and application data only; never media files/streams or PostgreSQL access. Tailscale membership does not authorize Rawrarr data access or decrypt ciphertext.

The MacBook's current local deployment is the subscriber **coordinator**. The Mac mini is the planned **storage** worker without PostgreSQL or user passkeys. **Shard** means encrypted app-data replica, with further serving duties undecided. Coordinator and storage may be combined elsewhere. This supersedes the earlier full-web-app-on-every-node direction. See [node types](node-types.md).

## Private coordinator endpoint

Prefer a stable named Tailscale Service such as `https://rawrarr.<tailnet>.ts.net` with the coordinator as its sole initial backend. The name is illustrative, not a verified or published endpoint. Per-device Tailscale Serve can first verify access under the coordinator host's MagicDNS name. Serve exposes a local endpoint; a named Service supplies a stable service identity and routing. Neither enrolls Rawrarr devices by itself.

Only an actual coordinator can serve this endpoint. Do not advertise storage/shard replicas as interchangeable backends or claim failover from shared ciphertext. Multiple coordinators would require a separate database, authentication, and consistency design.

Keep the existing local app at `127.0.0.1:7297`. A host-level Serve proxy can target that listener if supported by the installed client. Before publication, inspect existing Serve routes and avoid overwriting them. Named Services require compatible clients, tagged service hosts, service definition/host approval, and access policy; verify current requirements and the installed macOS client support before choosing host-level versus a dedicated sidecar identity. Do not retag the user's whole MacBook merely to fit the design without assessing its existing identity/access. A dedicated sidecar avoids conflating host and app identity, but needs separate enrollment and verified container routing.

For the MacBook sidecar, use independent persistent Tailscale state and the app's explicit Docker network address; sidecar localhost is not the MacBook host. Compare userspace and kernel/TUN requirements before adding privileges. Verify the Mini can reach the coordinator over ordinary HTTPS from inside its OrbStack worker container; the Mini host or container does not need Tailscale. Do not place auth keys in Compose, logs, `/shared`, or encrypted app configuration. Enrollment uses interactive login or a scoped secret mechanism, and any private Tailscale state remains node-local.

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
