# Persistent Rayfish coordinator access path

Issue: [#446](https://github.com/willjasen/rawrarr/issues/446).

## Objective and initial state

Make the internal do3 Roarlay namespace able to reach the private Rayfish
discovery and relay HTTPS endpoints through a durable, unpublished TLS proxy.
The path must persist through coordinator namespace-holder recreation without
changing the default route, Tailscale Serve, public services, or live Fossil
routes. No live configuration is changed by this task.

At task start, local `main` was clean and synchronized to `origin/main` at
`b9e1b976472b532c4e3936e5ee3934d8ec5c88f3`; this task uses branch
`codex/rayfish-persistent-coordinator-path` in its own worktree. The existing
internal coordinator uses `rawrarr-internal-do3-control` and
`rawrarr-internal-do3-egress`; Roarlay shares the Tailscale container's network
namespace. The deployed relay and discovery services use separate internal
backend networks. The earlier temporary probe used a private bridge and an
unpublished TLS proxy, and confirmed unchanged default routing and Tailscale
Serve before and after detachment.

## Implementation record

Read-only do3 inspection confirmed the temporary probe topology and existing
backend networks: the relay and discovery each have a private internal bridge;
the internal coordinator currently uses only its control and egress bridges.
The probe's temporary `172.31.99.0/29` range is unused in the current Docker
network list and host IPv4 routes. The namespace-holder default route and
Tailscale Serve were read before and after that probe and were unchanged.

Added an opt-in, internal-only Compose network with fixed coordinator/proxy
addresses, an unpublished Caddy proxy with source filtering and the existing
canonical TLS names, a coordinator-only Compose overlay, and a sidecar-only
`extra_hosts` overlay. The proxy reaches only the existing Rayfish backend
networks. The coordinator auto-updater excludes the topology overlay from app
image rebuild classification and retains it when recovering the active Compose
file list for later Roarlay recreations. The guide documents published-source
build, activation ordering, checks and topology rollback. No application,
database, subscriber or synchronized-data migration is needed.

## Verification

- `python3 scripts/tests/test_rayfish_coordinator_path.py` — passed (4 tests).
  This rendered the coordinator, proxy and generated sidecar Compose models
  with Docker Compose; it did not contact the Docker daemon.
- `python3 -m unittest discover -s scripts/tests -p test_do3_coordinator_auto_updater.py`
  — passed (19 tests), including preservation of the optional overlay.
- `docker build -f deploy/rayfish/Dockerfile.https -t rawrarr-internal-rayfish-https:test .`
  — passed on OrbStack.
- The resulting Caddy image validated the coordinator Caddyfile using an
  ephemeral synthetic certificate and `--network none` — passed.
- `git diff --check` — passed.

The proxy was not deployed or started on do3. Persistent behavior across an
actual coordinator namespace-holder recreation and authorized Roarlay-to-node
traffic remain unverified and are required before selecting any live Rayfish
route.

## Final state and remaining gates

No internal/public service, Tailscale route, or Fossil was changed. The task
changes were committed locally in `70b5192` on
`codex/rayfish-persistent-coordinator-path`; they have not been pushed. Before
future adoption, publish the work to `main`, apply the Compose overlays under
the established locks, verify coordinator route/health and authenticated
Roarlay-to-node traffic across namespace-holder recreation, and rehearse
rollback. The MacBook canary remains a separate gated task.
