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
establish those results. Coordinator loopback relay selection also requires
absence of `RAWARR_NODE_RELAY_SOCKS5_URL` until explicit proxy bypass is tested.

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
Short soak smoke checks and 2 soak-report unit tests passed. A 72-hour soak can
run using `--integration --soak-hours 72 --report /protected/report.json`;
its passing report, if obtained, does not override the failed discovery gate.

Native ARM64 execution could not start: the saved OpenSSH alias failed
authentication, and Tailscale SSH stopped at strict host-key verification.
Neither host verification nor credentials were changed or bypassed.
