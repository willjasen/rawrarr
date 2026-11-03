# Optional Tailscale integration plan

Status: internal coordinator placement revised 2026-11-03. The internal
coordinator stays on do3; neither Mac is a coordinator deployment target.
Device-code linking and named-service failover remain planned, not implemented
by Compose cleanup. See [current deployment](container-deployment.md).

## Internal coordinator ingress on do3

The internal coordinator shares its dedicated do3 Tailscale sidecar namespace.
Serve HTTPS 443 `/` proxies directly to `http://127.0.0.1:8080`, preserving
`https://rawrarr.risk-mermaid.ts.net/`. Diagnostics are host-loopback-only.
The public coordinator on the same host has separate services and state.

After a coordinator/Serve change, read back its sidecar's
`tailscale serve status --json` and verify the HTTPS `/health` route. Preserve
unrelated handlers, including any established Syncthing TCP forwards; do not
reset Serve. A healthy container alone does not verify the tailnet route.

The former Mac coordinator and staged Headscale templates are retired. Their
dated runbooks describe historical deployments, not current setup. Source
retirement does not stop sidecars, revoke enrollment, or delete identities.

## Mac subscriber transport

Both Macs host fossil subscribers, using their established host-local Compose
definitions. Nodes connect to their configured `RAWARR_COORDINATOR_URL` over
HTTPS; they do not depend on a Mac-local coordinator service or network alias.
Inspect actual namespace, listener, and Serve configuration before maintenance.

Use separate persistent node-local transport state for each subscriber. Never
share identities between hosts or put keys/state under synchronized `/shared`.
Keep subscriber transport separate from do3 coordinator ingress and native host
Tailscale. Do not add TUN capabilities or change namespace design without need.

For storage-role nodes, media remains node-local and read-only. `/shared`,
`/data`, credentials, and private identities retain their data-domain boundaries.
PostgreSQL remains private to the coordinator Docker network, never through
Tailscale. Transport membership is not owner approval or application permission.

Configure Syncthing peers/folders only after explicit owner approval. Its GUI
stays loopback-only unless deliberately secured. Preserve the actual project's
verified peer routing; historical discovery/relay/TCP-forward experiments are
not authority to overwrite current settings.

Verify subscriber health and the intended coordinator/direct routes separately.
Never recreate coordinator or database services on a Mac to repair subscriber
connectivity. Preserve identities, credentials, volumes, and the published
coordinator origin. See [Compose retirement/adoption](compose-layout.md).

## Product decision

Tailscale is an optional connectivity layer. Rawrarr remains usable through another trusted private transport. It carries coordinator web/API, authenticated device control, and application data only; never media files/streams or PostgreSQL access. Tailscale membership does not authorize Rawrarr data access or decrypt ciphertext.

The internal coordinator stays on do3. Fossil (`metadata`) and boneyard (`storage`)
are subscriber roles; shard is superseded by metadata. Generic self-hosting
templates do not change the internal host assignment. See [node types](node-types.md).

## Private coordinator endpoint

The current internal endpoint is `https://rawrarr.risk-mermaid.ts.net/` on do3.
A future named Tailscale Service requires its own compatible clients, access
policy, host approval, and verification. It is not implemented by this cleanup
and does not enroll Rawrarr devices by itself.

Only an actual coordinator can serve this endpoint. Do not advertise storage/shard replicas as interchangeable backends or claim failover from shared ciphertext. Multiple coordinators would require a separate database, authentication, and consistency design.

Keep the configured HTTPS/WebAuthn origin and RP scope during maintenance.
Before any endpoint change inspect existing Serve routes; never reassign
the native identity of a Mac or promote a subscriber to coordinator implicitly.

Sidecar localhost is its own namespace, not the host. Verify routes from the
actual subscriber container. Do not put keys in Compose, logs, `/shared`, or
encrypted app config. Use protected scoped enrollment and node-local state.

## Link storage to the coordinator

Configure the worker with the coordinator's trusted HTTPS base URL. Current
onboarding uses explicit owner approval in the coordinator workspace. A
human-readable `/devices/link` code flow remains planned; do not present it as
an available installation step. See the [linking plan](coordinator-linking-plan.md).

Tailscale reachability must exist before Rawrarr linking. The logged URL does not itself enroll Tailscale or open the tailnet to the browser. Use an authenticated browser/device that can reach the private coordinator.

## WebAuthn and outage behavior

Existing localhost RP credentials are not automatically usable at a new Tailscale hostname. Implement a deliberate HTTPS-origin/RP enrollment or migration strategy, test login and approval there, and retain existing local access. Do not merely change Compose origin variables and assume existing credentials survive. Device API authentication remains independent from browser passkeys.

When a Mac sleeps, that subscriber becomes unavailable; the do3 coordinator
does not move with it. Coordinator outages and node outages are distinct.
Define offline behavior explicitly; replicated ciphertext does not provide
coordinator failover.

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
