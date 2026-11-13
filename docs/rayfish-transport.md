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

The rootless pilot exposes only a selected subscriber's loopback TCP 8080.
Configured forwards bind namespace-local IPv4 loopback ports 17000–17999 and
target a recorded Rayfish mesh IPv6 address on TCP 8080. Firewall admission also
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
from explicitly pinned callers to 8080. Rayfish 0.5.16 tracks only locally
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

Create a dedicated closed network on one pilot endpoint:

```sh
docker exec RAYFISH_CONTAINER ray --rootless create --closed \
  --name rawrarr-pilot --hostname fossil-a
docker exec RAYFISH_CONTAINER ray --rootless files auto-accept rawrarr-pilot off
docker exec RAYFISH_CONTAINER ray --rootless status --json
```

Generate an invitation using `ray --rootless invite rawrarr-pilot --json` and
transfer its `code` through a protected channel. Do not put invitation codes in
Compose, Git, ordinary logs or `/shared`. On the other endpoint, use
`ray --rootless join CODE --name rawrarr-pilot --hostname fossil-b
--no-auto-accept-files`. Do not accept firewall auto-management or device pairing.
The creator must turn file auto-accept off before its next restart too.

After recording both endpoints and mesh addresses, add explicit inbound peers
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
If `RAWARR_NODE_RELAY_SOCKS5_URL` is set, do not select these loopback targets
until proxy bypass has been implemented and tested; the current explicit
coordinator proxy can intercept them. Existing Tailscale HTTPS routes remain
the fallback configuration, not a silent per-request retry path.

Existing signed configuration-peer RPC can also use an explicitly configured
loopback forward, but automatic peer-map generation/adoption is not implemented.
Syncthing, media, database traffic and browser ingress are outside this pilot.

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
an explicit selected-route failure, not silently route elsewhere.

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

The private-relay discovery-outage qualification also fails: neither peer
serves forwarded application requests within 90 seconds after both cold-restart
with discovery stopped, despite the private relay remaining available.
Restoring the persistent discovery service recovers bidirectional byte-exact
traffic and application 401s without another peer restart. Direct paths remain
blocked, and relay byte counters account for the recovered payloads. A traffic
relay is therefore not a demonstrated substitute for membership/discovery
infrastructure. This is a rollout blocker, not a reason to bypass signed roster
restoration or weaken membership checks.
