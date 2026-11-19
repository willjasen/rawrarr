# Optional Rayfish transport pilot

Issue: [#446](https://github.com/willjasen/rawrarr/issues/446).

Rayfish 0.5.16 is packaged as a separate, disabled-by-default Linux sidecar.
It is experimental, pre-1.0 software, not an independently audited replacement
for Tailscale. Its closed mesh avoids a continuously online membership server
for established peers, but membership changes still need a network-key holder.
Discovery and traffic relays remain infrastructure dependencies.

## Boundaries

Roarlay retains accounts, passkeys, node approvals and application coordination.
Fossil and boneyard remain subscriber roles. Existing Tailscale clients, HTTPS
Serve routes, browser origins and passkey RP IDs do not change.

The intended product boundary is a separate closed Rayfish network per user,
with the coordinator participating separately in each network. The current
pilot does **not** provision or bind networks and routes to Rawrarr accounts
automatically. Network names are local aliases, not proof of ownership or
isolation. Do not deploy a shared pilot mesh as per-user isolation. A synthetic
test exercises two separate networks, but account-bound provisioning, cross-user
authorization, revocation and recovery still require implementation and review.
The intended end state is to remove user-node dependence on SwarmSail/Headscale,
while keeping Tailscale available for coordinator access. That replacement
requires proving every affected route and service, including Syncthing and
configuration-peer traffic, plus a migration and rollback path. It is not yet
implemented and does not authorize removal of existing services.
The eventual replacement must pass the complete isolated environment with
subscriber Tailscale containers stopped. The current environment joins those
containers' network namespaces, so it cannot demonstrate that outcome merely
by stopping them. A separate namespace topology and qualified HTTPS, config
peer and Syncthing paths are required; coordinator-side Tailscale access may
remain.

The rootless pilot can expose a selected subscriber's loopback TCP 8080 and,
separately, opt-in Syncthing data traffic on TCP 22000.
Configured forwards bind namespace-local IPv4 loopback ports 17000–17999 and
target a recorded Rayfish mesh IPv6 address on TCP 8080 by default or on TCP
22000 with `"service": "syncthing"` on subscriber forwards. Firewall admission also
pins the full authenticated endpoint identity and local network alias. Use the
status `endpoint` field, not the optional paired-user `identity` field.
Addresses are recorded from `networks[].my_ipv6`, not guessed from hostnames.
Before opening any forward the runtime uses its native `b3sum` tool to verify
the address against Rayfish's pinned endpoint-to-IPv6 derivation. A mismatched
address/identity fails startup even when both endpoints are allowed callers.

No TUN, host routing, DNS changes, privileged mode, Docker socket, NET_ADMIN,
published ports or host network are needed. The sidecar receives only public
transport configuration and its own dedicated node-local state volume. Never
mount `/shared`, `/media`, application `/data`, PostgreSQL, private age identities,
passkeys or application credential files into it. Transport admission is not
application authorization.

The firewall defaults deny in both directions. New inbound TCP is allowed only
from explicitly pinned callers to the permitted port. `allow_in` grants app
access; `allow_syncthing_in` independently grants Syncthing data access.
Rayfish 0.5.16 tracks only locally
initiated flows: accepted server connections therefore also require outbound TCP
permission to that exact caller on arbitrary destination ports for ephemeral-port
replies. **This is not a source-port-restricted, stateful reply rule.** The
rootless exposure and forward configuration constrain normal application use.
The mandatory authenticated SOCKS listener is bound to loopback on a random
port; its credentials remain in private transport state. Do not distribute them
to applications or use it as a general proxy. A stricter stateful outbound policy
requires upstream changes and is a rollout review item.

The wrapper rejects imported state, node/role/namespace/network reassignment,
open networks, file auto-accept, remote-management grants and recorded device
pairings. It rewrites the managed firewall before each start and holds a
node-state lock across daemon execution. Do not enable extra Rayfish features
through its administrative CLI while the pilot runs; the wrapper validates saved
policy on restart, not continuously.

## Build and generate

Use a published GitHub `main` revision before any internal deployment. The
following examples describe setup, not authorization to deploy. Check available
space and the target's native Docker architecture first; never prune to make room.

```sh
docker info --format '{{.OSType}}/{{.Architecture}}'
# Use linux/arm64 on an ARM64 host, not emulation.
docker build --platform linux/amd64 -f deploy/rayfish/Dockerfile \
  -t rawrarr-rayfish:0.5.16 .
docker image inspect rawrarr-rayfish:0.5.16
```

The Dockerfile pins the release downloads by SHA-256 for AMD64 and ARM64.
The pinned upstream source is
[`c29c4a17`](https://github.com/rayfish/rayfish/tree/c29c4a17d9ba76ef30dd9e307f3ba0cb1e4efaba)
(MPL-2.0).
This is a separate transport image, not a change to the host-local Rawrarr app
base image. It must already exist locally; Compose never pulls it.

Place configuration and generated Compose outside synchronized data and Git.
Keep their containing host directory private (0700); the public configuration
file must be readable by container UID 10001 (for example 0644 inside that
private directory). Secret fields are not accepted in this configuration.
Initially use a bootstrap configuration with no forwards or application exposure:

```json
{
  "version": 1,
  "name": "rayfish-fossil",
  "namespace": "existing-fossil-tailscale-1",
  "role": "metadata",
  "network": "rawrarr-pilot",
  "lan_discovery": false
}
```

`namespace` must identify the already-running container whose network namespace
contains that specific application's loopback listener. Inspect the real
container names and listeners first. Do not select the Docker host or a database.
LAN discovery is off unless explicitly enabled; enabling it publishes discovery
information on the local network. The isolated tests explicitly enable it.

For a private deployment, set both infrastructure endpoints on every sidecar
(including the enrollment endpoint) before its first start:

```json
{
  "infrastructure": {
    "discovery": "https://discovery.example.net/pkarr",
    "relay": "https://relay.example.net:3340"
  }
}
```

These are fields to add to the bootstrap JSON above. The wrapper configures
Rayfish to replace its default servers with exactly these endpoints. Provision
the signed-packet discovery service and the iroh traffic relay separately;
this option does not deploy them. Build the private discovery server with
`docker build -f deploy/rayfish/Dockerfile.discovery -t rawrarr-rayfish-discovery:local .`.
Run it on a private backend network with a fresh persistent volume at `/state`;
do not publish its HTTP port directly. Put both services behind trusted TLS
and restrict ingress to approved pilot hosts.
HTTP is accepted only for IPv4 loopback or single-label container service
names; use those forms only on an isolated local Docker network. URLs cannot
carry credentials, query parameters, or fragments. Leaving `infrastructure`
out preserves legacy defaults on unconfigured state. An existing sidecar
must use a fresh transport volume and be re-enrolled to switch infrastructure;
the wrapper refuses to reuse state with mismatched saved settings. Retain the
original volume and route as rollback, and do not copy private endpoint keys.

The discovery server does not federate or send packets to a DHT. It verifies
the Ed25519 signature against the requested record key and atomically retains
the newest signed packet in SQLite. It has no list endpoint or eviction;
it rejects new keys after 512 records and continues accepting updates to
existing keys. Keep its database on service-local storage, back it up, and
monitor for capacity errors. Do not silently restore an older snapshot over
a newer database: doing so loses the newer signed records, including possible
revocations. Starting this server requires no application data migration.

```sh
python3 scripts/rayfish_transport.py compose --config /protected/pilot.json \
  > /protected/rayfish.compose.json
docker compose -p rawrarr-rayfish-fossil -f /protected/rayfish.compose.json \
  --profile rayfish config --quiet
docker compose -p rawrarr-rayfish-fossil -f /protected/rayfish.compose.json \
  --profile rayfish up -d --pull never
```

Keep a distinct Compose project and state volume for each transport sidecar.
The existing application and Tailscale containers are not dependencies managed
by this generated definition. If a namespace holder is recreated, deliberately
recreate the associated transport sidecar after verifying its new namespace.

## Admission and route selection

Create a dedicated closed network on the enrollment-only endpoint. Do not
grant this endpoint application ingress or configure any forwards:

```sh
docker exec RAYFISH_CONTAINER ray --rootless create --closed \
  --name rawrarr-pilot --hostname mesh-owner
docker exec RAYFISH_CONTAINER ray --rootless files auto-accept rawrarr-pilot off
docker exec RAYFISH_CONTAINER ray --rootless status --json
```

Generate an invitation using `ray --rootless invite rawrarr-pilot --json` and
transfer its `code` through a protected channel. Do not put invitation codes in
Compose, Git, ordinary logs or `/shared`. On each traffic endpoint, use a
separate invitation and a distinct hostname, for example
`ray --rootless join CODE --name rawrarr-pilot --hostname fossil-a
--no-auto-accept-files`. Do not accept firewall auto-management or device pairing.
The creator must turn file auto-accept off before its next restart too.

After recording the traffic endpoint IDs and mesh addresses, add explicit inbound peers
and/or forwards. This example uses **synthetic** values that must be replaced:

```json
{
  "version": 1,
  "name": "rayfish-fossil",
  "namespace": "existing-fossil-tailscale-1",
  "role": "metadata",
  "network": "rawrarr-pilot",
  "lan_discovery": false,
  "expose_app": true,
  "allow_in": ["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"],
  "forwards": [{
    "node_id": "already-approved-fossil",
    "peer_id": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "peer_ipv6": "200::1",
    "local_port": 17297
  }]
}
```

Update the public configuration, then restart only its Rayfish sidecar. Never
use membership, peer addresses or successful TCP as substitutes for Rawrarr
owner/role/session checks. A coordinator can have forwards but cannot expose
its own ingress in this pilot.

For a coordinator, `relay-targets --config /protected/coordinator-pilot.json
--existing /protected/original-relay-targets.json` emits a new route map.
Only named, already-configured node routes change to `http://127.0.0.1:PORT/`;
the supplied original map is untouched. Save both maps separately. Applying
the emitted map to `RAWARR_NODE_RELAY_TARGETS` remains an explicit operator
deployment step. There is no automatic fallback or automatic app reconfiguration.
If `RAWARR_NODE_RELAY_SOCKS5_URL` is set, explicitly set
`RAWARR_NODE_RELAY_BYPASS_LOOPBACK_PROXY=true` to make Rayfish loopback targets
independent of that SOCKS proxy. The tested Tailscale SOCKS server could also
reach a loopback forward when the bypass was off, but that route depends on the
proxy being available. The bypass applies to all configured `127.0.0.1` node targets;
other targets still use the configured proxy. The setting defaults to `false`
to preserve existing behavior. It does not bypass proxies for `localhost`, IPv6
loopback, or arbitrary local addresses. Existing Tailscale HTTPS routes remain
the fallback configuration, not a silent per-request retry path. Restoring the
original target map and removing the opt-in returns to the previous behavior.

### Persistent internal coordinator access on do3

The internal coordinator uses the same canonical HTTPS infrastructure URLs as
the Macs: `https://do3.risk-mermaid.ts.net:18444/pkarr` for discovery and
`https://do3.risk-mermaid.ts.net:18443` for the traffic relay. Its Tailscale
namespace holder cannot reach the tailnet-only listener through the normal host
egress path. Use the opt-in access proxy files below to give that namespace a
private route while preserving TLS hostname verification:

- `deploy/rayfish/docker-compose.coordinator-access.yml` runs an unpublished,
  non-root TLS proxy on an internal Docker bridge. It reaches only the existing
  relay and discovery backend networks and accepts requests only from the
  coordinator endpoint address.
- `deploy/do3/docker-compose.internal-rayfish-coordinator.yml` attaches the
  coordinator Tailscale namespace holder to that same internal bridge. The
  bridge is not a default gateway. The overlay is separate from the standard
  coordinator Compose file and must be included whenever the namespace holder
  is recreated.
- `deploy/rayfish/compose.roarlay-coordinator-hosts.yml` maps the canonical
  hostname to the proxy's fixed bridge address for the generated
  `rayfish-roarlay` sidecar only. Roarlay, the Macs, and Tailscale Serve keep
  their existing name resolution and routes.

The do3 bridge uses `172.31.99.0/29` (`.2` proxy, `.3` coordinator). This range
was checked against do3's current Docker networks and host IPv4 routes on
2026-11-18; recheck it before creating the network on any host. The private
network is internal-only and the proxy publishes no host ports. The HTTPS
certificate and key remain host-local at `/opt/rawrarr-internal-rayfish/tls.crt`
and `tls.key`; set `RAWARR_RAYFISH_TLS_CERT_PATH` and
`RAWARR_RAYFISH_TLS_KEY_PATH` only if the protected host-local paths differ.

Prepare from a published `main` archive on do3. First check the Docker network
range, source revision and image; build the proxy image from the checked-out
published source with `docker build -f deploy/rayfish/Dockerfile.https -t
rawrarr-internal-rayfish-https:local .`. Then validate and start the proxy
project, including its external backend networks:

```sh
docker compose -p rawrarr-internal-rayfish-access \
  -f deploy/rayfish/docker-compose.coordinator-access.yml config --quiet
docker compose -p rawrarr-internal-rayfish-access \
  -f deploy/rayfish/docker-compose.coordinator-access.yml up -d --pull never
```

When applying the internal coordinator Compose definition, add
`-f deploy/do3/docker-compose.internal-rayfish-coordinator.yml` to its existing
file list. This changes the Tailscale namespace holder's network attachment, so
coordinate the holder and Roarlay recreation with their existing deployment
locks. Confirm the default route and `tailscale serve status --json` before and
after. The proxy is in a separate Compose project; stopping that project and
removing the coordinator overlay restores the previous topology.

Generate the coordinator sidecar configuration with the two canonical URLs
above, set its name to `rayfish-roarlay` and namespace to
`rawrarr-int-tailscale`, then include
`-f deploy/rayfish/compose.roarlay-coordinator-hosts.yml` when validating and
starting its generated Compose. Verify TLS health for both services from the
sidecar namespace, then verify authenticated Roarlay-to-Fossil responses,
anonymous denial and the normal coordinator HTTPS health route before selecting
any Rayfish relay target. This access path alone does not change a node route or
authorize a Fossil cutover.

Signed configuration-peer RPC can opt into a pinned `config-peer` TCP 8443
forward between metadata nodes; see the [peer repair protocol](metadata-peer-repair-protocol.md)
for the loopback HTTP listener and peer-map requirements. Automatic peer
provisioning is not implemented. Syncthing's data path was tested
with manually paired devices and a loopback-only listener; its GUI and discovery,
media, database traffic and browser ingress are not forwarded. Existing configs
retain their behavior when the optional Syncthing settings are absent; no
persistent-state migration is required. Roll back by restoring the prior
Syncthing address and removing the optional grant and forward.

## Verification, adoption and rollback

The image health check verifies only local rootless daemon liveness; it can pass
before membership or peers are ready. Verify real peer/application responses,
application authentication failures, denied peers, Tailscale routes and service
health separately.

```sh
python3 scripts/tests/test_rayfish_transport.py
python3 scripts/tests/test_rayfish_transport.py --integration
python3 scripts/tests/test_orbstack_arr_connectivity.py --start --fresh-fixtures
python3 scripts/tests/test_rayfish_transport.py --environment
python3 scripts/tests/run_rawrarr_tests_in_docker.py
```

The real-Rayfish tiers use separate Docker namespaces, a private test-only signed
record store and direct LAN discovery. Public traffic relays are disabled in
these fixtures. The environment tier reaches both real fossils' authenticated,
node-specific recipient endpoints, verifies byte-identical responses and
anonymous HTTP 401s, and checks bidirectional Tailscale pings during a Rayfish
outage without restarting either existing app/Tailscale container. Synthetic
transfer tests additionally compare a 950,272-byte payload hash.

For existing deployments this is additive: no database, `/data`, encrypted
configuration or `/shared` migration is required. Transport state is new,
private and schema-versioned; incompatible/imported state is rejected rather
than rewritten. An internal trial remains gated on a passing regression baseline
and explicit deployment of published `main`. Public adoption remains planned.

Rollback restores the saved original application route map and disables/stops
only the pilot sidecars. Keep their volumes for diagnosis and identity-preserving
recovery; do not delete state as a rollback step. A stopped sidecar must produce
no successful response through an unintended route. A request may instead remain
pending; a bounded coordinator response timeout has not been qualified.

The browser-direct route requested for this pilot is not implemented. Nodes should
continue to contact the coordinator over HTTPS. A browser can probe a node's
`host.network.ray` address only when the browser's own device has suitable
Rayfish connectivity and DNS; the node's rootless sidecar provides neither to
the browser. Direct HTTPS also requires a browser-trusted node certificate and
an owner-bound authorization handoff. WebRTC is a possible alternative that
does not require a browser-side Rayfish client, but needs authenticated signaling,
verified node identity, a node-side endpoint and possibly a TURN relay. It is not
a normal HTTPS URL and has not been implemented or qualified.

Outstanding gates include independent security review, NAT/traffic-relay
fallback, discovery outages, sleep/wake, cold restart and relay-metadata/privacy
assessment, a 72-hour soak, and native ARM64 execution. Direct-LAN success is
not evidence that these pass.

Expanded qualification on 2026-11-13 found that established connections survive
loss of the private discovery store, but simultaneous cold restart of both peers
with discovery still unavailable fails to reconnect within the test deadline.
Restoring the persistent store and restarting peers restores byte-exact traffic.
The expanded integration exits nonzero for that failed gate rather than treating
its other passing checks as overall success. Container process pause/resume is
tested separately; it is not a hardware sleep/wake test.

For the isolated soak, add `--soak-hours 72 --report /protected/report.json` to
`--integration`. Keep the host awake. The runner checks bytes, authentication and
endpoint stability every 30 seconds without retrying broken transfers. More than
90 seconds between sampling rounds invalidates continuity. The private atomic
JSON report distinguishes soak results from the failed cold/discovery gate;
successful soak measurements do not approve deployment.
Use `python3 scripts/tests/test_rayfish_transport.py --soak-status --report
/protected/report.json` to inspect freshness. A report that says `running`
but has not been updated in 90 seconds is displayed as `stale`, not as an
active soak; this does not prove why it stopped. The command returns nonzero
unless both the soak and the separate discovery qualification passed.

Private traffic-relay qualification now also passes on native AMD64. The
test-only `scripts/tests/Dockerfile.rayfish-relay` pins official iroh-relay 1.3.0
binaries; `python3 scripts/tests/rayfish_relay_docker.py` uses it with no public
ports or public relays. Separate synthetic peers disable LAN discovery.
Short-lived test helpers apply IPv4/IPv6 egress filters only within those
disposable namespaces, allowing loopback and private discovery/relay TCP while
blocking direct paths. Production sidecars still need no NET_ADMIN.
The harness rechecks isolation after each namespace recreation, verifies both
payload hashes and application 401s, accounts for transferred bytes using relay
counters, and proves relay outage stops transfers and restoration resumes them.
Payload, credential and HTTP-path markers are absent from the tested relay logs.
Relay operators can still observe endpoint/address, timing and traffic-volume
metadata; this is not an exhaustive confidentiality audit. NAT hole-punching
and native ARM64 execution remain unverified.

The three-endpoint owner-offline test also passes: after the closed mesh's
network-key holder is stopped, two already-admitted members exchange application
data, and one cold-restarts successfully while the owner stays offline.
Private discovery remains available in that test. This supports the narrower
claim that established membership does not require a continuously online
network owner, not that discovery infrastructure can be removed.

For the pilot, keep the network key in a dedicated enrollment-only endpoint
with no application listener or forwards. Enroll Roarlay's sidecar and both
subscriber sidecars as ordinary members. The standalone OrbStack fixture
restarts all three traffic sidecars with that key holder stopped and private
discovery paused, then checks authorized relay traffic, anonymous denial,
signed config repair, and bidirectional Syncthing. The private traffic relay
remains online on the same Docker bridge; this does not qualify recovery
across hosts or NATs.

Existing Tailscale deployments require no application or data migration for
this topology. For an existing Rayfish pilot that put the owner on a traffic
sidecar, enroll ordinary peers in a new closed mesh with separate state; do
not copy the owner state to another sidecar. Retain the old routes and
identities until the new pins and routes are verified. Roll back by restoring
the previous route and sidecar settings without deleting either mesh's state.

An additional direct-LAN test with a separate, stopped key holder found that
two ordinary members can both cold-restart and exchange authenticated
application responses while private discovery is paused. This is a narrower
result than the two-peer key-holder test: the key holder's own restart remains
blocked by its destruction-check timeout. Cached membership also cannot
establish that a newly revoked peer is excluded while signed discovery is
unavailable; treat that as a security and rollout gate, not a basis for
automatically failing open. Keep independent application authorization and the
existing Tailscale route. In a disposable test, both members could still
exchange application responses after the key holder kicked one while they were
offline and the corresponding discovery record changed, then they restarted
while discovery was paused. The key holder no longer listed the kicked member.
When discovery and the key holder returned, the remaining member removed the
kicked peer and its Rayfish connection to the app failed. This observed stale
revocation window is a rollout blocker; transport membership must not replace
Rawrarr's own authorization.

An operator who can still reach the receiving hosts can remove a peer's pinned
grants from their `allow_in`, `allow_config_peer_in`, `allow_syncthing_in`, and
any matching `forwards`, as applicable, and restart each affected sidecar.
This stops its existing connections and rewrites the deny-by-default ACL before
restart. The standalone fixture verifies the config-peer case while discovery
is offline. This is local containment, not automatic revocation on hosts the
operator cannot reach; a mesh `kick` alone is not equivalent to this procedure.

The private-relay discovery-outage qualification also fails: neither peer
serves forwarded application requests within 90 seconds after both cold-restart
with discovery stopped, despite the private relay remaining available.
Restoring the persistent discovery service recovers bidirectional byte-exact
traffic and application 401s without another peer restart. Direct paths remain
blocked, and relay byte counters account for the recovered payloads. A traffic
relay is therefore not a demonstrated substitute for membership/discovery
infrastructure. This is a rollout blocker, not a reason to bypass signed roster
restoration or weaken membership checks.
