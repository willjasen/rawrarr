# Rayfish parallel transport pilot

Issue: [#446](https://github.com/willjasen/rawrarr/issues/446).
Task base: published `d54bccc`; branch `willjasen-rayfish-parallel-transport`.

## Scope and decisions

Implement an opt-in rootless Rayfish sidecar beside existing Tailscale, then
verify separate peers and the complete isolated coordinator/two-fossil
environment before proposing internal adoption. No internal/public deployment,
application origin change, automatic failover, public rollout or media/Syncthing
migration is part of this implementation.

See [the operator guide](../docs/rayfish-transport.md) for build, bootstrap,
protected admission, peer-pinned route selection, verification and rollback.
The transport has its own state volume and never mounts subscriber application
data, identities or credentials. Application permissions remain unchanged.

Version 0.5.16 is pinned by binary checksums, with native AMD64/ARM64 selection.
The tested OrbStack host and image are Linux AMD64. ARM64 packaging is defined
but native ARM64 execution remains unverified.

## Findings incorporated

- Rootless forwarding targets mesh names/addresses, not raw endpoint IDs.
  Addresses come from network status; runtime `b3sum` verifies Rayfish's exact
  BLAKE3-of-lowercase-endpoint-hex derivation before exposing any forward.
- Default outbound deny blocked server replies to callers' ephemeral ports.
  Upstream conntrack only opens locally initiated flows. The fix permits
  outbound TCP to exactly pinned inbound callers, not arbitrary peers. This
  rule is broader than a stateful return-flow rule and is documented as such.
- Closed membership alone does not authorize the app. A third admitted peer
  lacking inbound approval cannot access it; anonymous application requests
  still receive HTTP 401.
- Creator network defaults enable file auto-accept. Bootstrap turns that off;
  restart rejects open networks, file/firewall auto-accept and unsafe grants.
- Local daemon health can pass without usable membership. Readiness checks
  must exercise actual authenticated application responses.

## Verification performed

| Command/check | Result |
| --- | --- |
| Native `deploy/rayfish/Dockerfile` build, binary version and image architecture | Passed: Rayfish 0.5.16, Linux AMD64 |
| `python3 scripts/tests/test_rayfish_transport.py` | 12 passed |
| `python3 -m unittest discover -s scripts/tests -p test_orbstack_signed_fixture.py` | 2 passed |
| `python3 scripts/tests/test_rayfish_transport.py --integration` | Passed: distinct peer namespaces, bidirectional 950,272-byte payload hashes, HTTP 401, admitted-peer denial, TCP/UDP deny verdicts, outage, restart identity and revocation |
| `python3 scripts/tests/test_orbstack_arr_connectivity.py --start --fresh-fixtures` | Passed: complete coordinator/two-fossil/Headscale environment; encrypted Radarr and Sonarr settings, status and library APIs |
| `python3 scripts/tests/test_rayfish_transport.py --environment` | Passed: both real fossils' authenticated node-recipient responses are byte-identical over Rayfish; anonymous HTTP 401; Tailscale pings both ways survive Rayfish outage; existing app/Tailscale IDs/start times unchanged |
| `python3 scripts/tests/run_rawrarr_tests_in_docker.py` | Final rerun passed: Node 153, Rust 141 plus 1 ignored, all Playwright tests; 292 seconds |
| Final generated-Compose/package smoke test in a network-disconnected namespace | Passed: nonroot packaged wrapper, local health, concurrent state-writer rejection; owned smoke resources removed |

Initial full regression failed only the existing topology browser case
“Fossils renders before stalled diagnostics and recovers on a later poll.”
The complete rerun passed without changing that test or application UI code.
Do not treat this as an identified/fixed browser root cause; retain the initial
failure as a possible timing/load flake.

Initial full-environment startup failed because the older preapproved fixtures
lacked signed node certificates. With explicit user approval, added
`--fresh-fixtures`: new versioned, uniquely named volumes, native cached ML-DSA
fixture tooling, valid root-signed node certificates, and subscriber-local seed
installation. Existing volumes were preserved. Private root material exists
only transiently in the test signer, never in coordinator state; private node
seeds enter only each new subscriber's `/data`.

The successful fixture override is stored outside Git at:
`~/Library/Application Support/rawrarr/orbstack-test/signed-fixture-v1-c2e69b5398fe40fdb32da1eae8e50ea8/compose.json`.
Keep it when managing these test services. Recreating from only the original
base Compose file would select the older volumes. Rerunning the connectivity
check without `--start` exercises the current services without that change.
Generated Rayfish peers/volumes/networks and temporary test sessions were removed;
the complete isolated Rawrarr environment remains available.

## Existing deployments and rollout gates

This is additive and disabled by default: existing deployments need no database,
node-data or synchronized-data migration. New transport state is marked schema
v1 and incompatible/imported state is rejected. Route maps are emitted separately
without overwriting the original Tailscale map. There is no automatic adoption.

Before an internal trial, publish the task through a focused PR and GitHub
`main`, follow the existing serialized deployment procedure, verify native
architecture and preserve original routes/state. This task did not push, open
a PR, merge or deploy.

Independent security review, stricter return-flow policy assessment, NAT and
traffic-relay fallback, public-discovery outage/privacy tests, sleep/wake and
cold-start behavior, native ARM64 execution and the 72-hour soak remain gates.
The current direct-LAN fixtures disable public traffic relays and cannot
establish those results. When the explicit SOCKS proxy is configured, selecting
coordinator loopback relay targets requires the opt-in described below.

Rollback restores the saved original app route map and stops only pilot
sidecars; retain their identity volumes. Never change Tailscale Serve, passkey
origins, PostgreSQL, `/shared`, `/data` or unrelated services for this rollback.

## Continued qualification

User requested continued isolated testing without merging or live deployment.
An agent security review reported no confirmed exploitable findings. This is not
an independent professional audit and does not close that gate.

Expanded direct-LAN qualification found a reproducible failure: established
connections work without the discovery store, but simultaneous peer cold restart
while it remains offline cannot resume within 30 seconds (the first experiment
also exceeded 90 seconds). Peers recover after the persistent store returns and
both restart. The fixture now persists signed packets across stop/start, so this
is not merely lost test-store contents. New-peer admission works after restoration.
Process suspension yields explicit request failure and byte-exact recovery on
resume. This is not a Mac hardware sleep/wake test.

The expanded synthetic integration intentionally exits nonzero for the cold-start
qualification failure after performing remaining controls. The real-fossil
environment check still passes and leaves its services unchanged.
Short soak smoke checks and 3 soak-report unit tests passed. A 72-hour soak can
run using `--integration --soak-hours 72 --report /protected/report.json`;
its passing report, if obtained, does not override the failed discovery gate.

Native ARM64 execution could not start: the saved OpenSSH alias failed
authentication, and Tailscale SSH stopped at strict host-key verification.
Neither host verification nor credentials were changed or bypassed.

### Private relay qualification

Built the checksum-pinned native AMD64 official iroh-relay 1.3.0 test image.
The real Rayfish fork depends on iroh-relay 1.3.0; the fixture exercises observed
wire compatibility, not every fork-specific behavior.

The first relay-isolation attempt correctly refused to claim success:
OrbStack routed between the two Docker bridges. Added strict IPv4/IPv6 egress
allowlists using temporary NET_ADMIN helpers in **only** the disposable peer
namespaces. Another attempt detected lost filters after Docker stop/start
recreated namespaces; the harness now reapplies and asserts isolation after
restart. No host firewall or live namespace was altered.

Final `python3 scripts/tests/rayfish_relay_docker.py` passed: mDNS disabled,
bidirectional byte-exact 950,272-byte payloads, app HTTP 401s, forwarded-byte
counter evidence, failure during relay outage and recovery after restoration.
Tested plaintext payload, application-token and HTTP-path markers are absent
from relay logs. All owned relay/peer containers, volumes and networks were
removed. Metadata privacy beyond these markers and real NAT hole-punching are
still pending.

The first 72-hour direct-LAN soak was started in a detached session process from
commit `0f40e54`. It failed the maximum-90-second sampling-gap guard after 21
successful bidirectional samples and about 650 seconds of monotonic runtime
(39,911,424 payload bytes). No byte/auth mismatch was reported. This invalidates
continuity; it is not a successful soak. Its private atomic report
and log are under this conversation's `files/` directory as
`rayfish-soak-72h.json` and `rayfish-soak-72h.log`. Sampling every 30 seconds,
failed transfers are not retried and gaps over 90 seconds invalidate continuity.
Stopping it with SIGTERM records interruption and cleans its owned fixtures.
Even a completed soak cannot override the failed cold/discovery qualification.

Added largest wall-clock and monotonic sampling gaps to reports so a subsequent
failure retains better diagnostics. An awake retry uses temporary `caffeinate -i`
around the test process, not a persistent OS sleep-setting change; forced sleep
or clock discontinuities can still invalidate it. Its report/log names are
`rayfish-soak-72h-awake.json` and `rayfish-soak-72h-awake.log`.

`python3 scripts/tests/rayfish_owner_offline_docker.py` passed with three distinct
endpoints: after stopping the network key holder entirely, the other two
admitted members exchanged byte-exact application responses. One member then
cold-restarted and recovered without the owner. Discovery was still available;
this does not resolve the simultaneous-cold-start/discovery-outage failure.
Application 401s and endpoint identities remained enforced. Owned fixtures were
removed, and no live service was used.

### Private-relay discovery-outage continuation

`python3 scripts/tests/rayfish_relay_docker.py --discovery-outage` repeated the
passing private-relay payload/auth/counter/outage checks, then cold-restarted both
peers while the private discovery container was stopped. The relay stayed
available, but forwarded requests timed out throughout the 90-second
qualification window; local daemon liveness remained intact. The overall command
correctly exited nonzero for this failed gate.

Restoring discovery recovered bidirectional 950,272-byte responses and 401s
without another peer restart. Relay counters accounted for both payloads, and
endpoint identities remained unchanged. This narrows the earlier direct-LAN
finding: an available traffic relay alone does not solve this cold-start failure.
Upstream's pinned `src/daemon/network_registry.rs` distinguishes local
key-holder restoration from member restoration needing the signed pkarr roster;
cached transport paths do not establish that roster's availability.

The experiment also exposed a fixture detail: Docker removes a stopped
container's DNS alias and can reassign addresses. Filters now reuse previously
resolved discovery/relay addresses while discovery is offline, and refresh them
inside only the owned disposable peer namespaces when discovery returns.
Direct-path checks resolve the peer's current address and verify its probe
listener is live after each restart; stale addresses cannot stand in for
isolation evidence. Five focused fixture tests pass. Final experiment evidence is
in the private session file `rayfish-relay-discovery-outage-final.log`; owned
containers, volumes and networks were cleaned up. The separate awake soak is
unaffected and still has not completed its requested 72 hours.

### 2026-11-18: interrupted soak and separate key-holder experiment

The awake soak's last report was written on 2026-11-13 at 05:09:35 -0500,
after 87 samples and approximately 2,705 seconds. Its process was absent when
checked on 2026-11-18; the three test containers had exited with status 255.
The host/runtime interruption's cause is not established. The report still
said `running` because abrupt termination prevented its cleanup handler from
writing a final state. The soak is incomplete, not passed.

Added a read-only `--soak-status --report PATH` check which treats a `running`
report with no update for over 90 seconds as `stale` and exits nonzero. It
preserves the original report, distinguishes stale evidence from a confirmed
process exit, and cannot override the separate discovery qualification.

Discovery diagnostics with a paused server confirmed the original outage is
not merely Docker removing the server's DNS alias. The network-key holder
reported `timed out checking network destruction` and did not restore its
network; an ordinary member warm-resumed from its cached verified roster.

`python3 scripts/tests/rayfish_owner_offline_docker.py --discovery-outage`
passed with a separate stopped key holder and a paused discovery server whose
DNS name remained resolvable: both ordinary members cold-restarted, exchanged
byte-exact 950,272-byte responses, enforced HTTP 401s, retained endpoint
identities and remained reachable when discovery returned. This does not fix the
key holder's cold-start failure or establish fresh revocation enforcement during
discovery loss. Cached roster freshness is an explicit security gate. In a
separate diagnostic run, two cached members remained able to communicate after
the owner requested a kick while they were offline. A repeated run verified
that the exact network's discovery packet changed after the kick before
pausing discovery; both members still exchanged authenticated fixture
responses after restarting with the key holder and discovery unavailable.
The new packet was not independently decoded to verify its roster, so the
precise published revocation remains unverified. Keep this as a security gate;
do not treat cached membership as proof of current authorization.

### Offline revocation and reconvergence

The repeatable command
`python3 scripts/tests/rayfish_owner_offline_docker.py --discovery-outage --revocation-outage`
checks that the key holder lists both members, then kicks one while they are
stopped. The key holder no longer lists it, and the exact network's discovery
packet changes. With that holder stopped and discovery paused, both members
restart from cached rosters and the kicked member still receives authenticated
fixture responses. After discovery and the key holder return, the remaining
member removes that peer and its Rayfish connection to the healthy local app
fails. The final observed reconvergence took 5.7 seconds after discovery and
the key holder returned. The command intentionally exits nonzero for the stale-revocation
qualification; if transfer is unavailable but removal is not observable, it
also fails as inconclusive instead of claiming a safe denial.

This is an isolated synthetic test, not evidence that a real Rawrarr session
or node authorization was revoked. It proves transport revocation cannot be
assumed fresh across a discovery partition. Do not remove independent Rawrarr
authorization or relax the network destruction check to make recovery pass.
Use original Tailscale routes for rollback; automatic fallback remains absent.

### Opt-in coordinator loopback proxy bypass

The coordinator's explicit Tailscale SOCKS proxy previously intercepted the
Rayfish forward's `127.0.0.1` target. Added the default-off
`RAWARR_NODE_RELAY_BYPASS_LOOPBACK_PROXY=true` setting to bypass that proxy for
configured IPv4 loopback relay targets only. Remote targets and the pre-existing
`fossil` exception keep their previous behavior, redirects remain disabled, and
invalid values fail startup. This affects no existing deployment unless
explicitly enabled; it requires no state, schema, or protocol migration.
Rollback removes the setting and restores the original route map. It is not
automatic failover and does not address the discovery or revocation blockers.
The generic coordinator Compose templates expose the flag with default
`false`; environments using different Compose definitions must add an explicit
override before enabling it. An isolated native regression clone passed the two
real-socket Rust proxy tests and 36 coordinator workspace tests. Both generic
Compose definitions resolve the default `false` and explicit `true` correctly.

### Per-user networks and real coordinator integration

Will requested one closed Rayfish network per user, with pinned ACLs and a
separate coordinator participant for each user. He also requested that nodes
continue using HTTPS to reach the coordinator and that the browser prefer an
authorized node's `.ray` URL when reachable. WebRTC was raised as a possible
browser-direct alternative. These are new requirements, not completed features.
The intended end state also removes SwarmSail/Headscale from user-node
networking while preserving optional Tailscale access to the coordinator. Will
requires a complete environment test with subscriber Tailscale containers
stopped before that replacement can be considered working. Current fossil and
Rayfish containers share those namespace holders, so the existing environment
cannot test this by simply stopping them. The synthetic user-network test uses
no Tailscale containers, but it does not exercise the complete environment.
The new topology must independently qualify coordinator HTTPS, application and
config-peer flows, Syncthing, authentication, failure/recovery and rollback.
Rayfish's pinned upstream documents names of the form `node.network.ray`; its
rootless mode does not install routes or DNS on the browser device, and a
browser-trusted HTTPS identity is still needed. Existing direct route validation
does not accept arbitrary `.ray` origins. No origin checks or TLS verification
were relaxed.

`python3 scripts/tests/rayfish_user_isolation_docker.py` passed with two closed
synthetic user networks and separate Rayfish coordinator peers sharing a network
namespace. Each reached its own authenticated fixture and a deliberate
cross-user route failed. Authenticated rootless SOCKS requests resolved each
user's own `.ray` name and rejected the other user's name. This is not a browser
DNS, browser HTTPS or certificate test. It does not establish account-bound automatic
provisioning or prove which of the network and ACL denied the cross-user route.
The network name is a local alias, not an ownership proof. Account binding,
secure invitations, port allocation, revocation and key recovery remain gates.
Loopback forwards are also reachable by other processes in the shared
coordinator network namespace; they are not per-process isolation.

Restored the existing stopped signed-fixture environment without replacing its
volumes. Its Arr connectivity check and real two-fossil Rayfish environment
test passed again. Built the current coordinator binary from the worktree in a
prepared native isolated clone and exercised it on a separate loopback listener
against the real fossil, using
`python3 scripts/tests/rayfish_coordinator_docker.py --binary /protected/binary`.
The original coordinator and Tailscale services were not restarted by this
test. Authenticated bytes matched the real fossil, a delegated session was
created for the correct owner/node, anonymous and wrong-user requests were
denied, and an admitted but unapproved Rayfish peer could not reach the fossil.
Firewall verdicts allowed the pinned app path and denied selected unrelated
TCP/UDP ports, coordinator ingress and the unapproved peer. Verdicts are
policy evaluations, not packet tests for every protocol/port.

With an actual Tailscale userspace SOCKS server running, the disabled bypass
also returned HTTP 200: that proxy could reach the loopback forward in this
fixture. Stopping SOCKS made disabled-bypass requests return 502, while the
enabled bypass worked with SOCKS either stopped or running. This corrects the
earlier assumption that SOCKS interception necessarily prevents all such
requests; bypass removes the dependency, rather than proving every proxy fails.
Stopping the Rayfish fossil did not produce a successful fallback. One run
remained pending beyond the 15-second client deadline rather than immediately
returning 502; bounded upstream response failure remains unverified.

Per-user provisioning, browser `.ray` HTTPS identity/trust and secure direct
handoff, browser-side network reachability, WebRTC, actual internal HTTPS
ingress, hardware/network tests, stale revocation and the uninterrupted 72-hour
soak remain unimplemented or unqualified. Existing deployed state has not
changed; no migration is needed for these additional test fixtures and docs.
