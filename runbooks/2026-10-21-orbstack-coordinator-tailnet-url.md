# OrbStack coordinator tailnet URL

## Objective and boundaries

Give the isolated OrbStack coordinator its own Tailscale identity on the `risk-mermaid.ts.net` tailnet and expose its HTTPS endpoint with Tailscale Serve. The two subscriber nodes remain enrolled only in the test Headscale instance. Do not reuse the production `rawrarr-macbook-tailscale` identity or edit its Serve routes.

Compose project: `rawrarr-orbstack-test`. Intended URL: `https://rawrarr-orbstack-test-coordinator.risk-mermaid.ts.net/`. Test-only state is stored in named volume `rawrarr-orbstack-test_orbstack-test-ts-coordinator`. Coordinator remains published on host loopback at `http://127.0.0.1:17397`.

Related procedure: [isolated coordinator and two-node test](2026-10-21-orbstack-coordinator-two-node-test.md).

## Configuration and operation

- `tailscale-coordinator` is a separate `tailscale/tailscale:stable` service with a dedicated state volume and hostname. `roarlay` shares its network namespace, allowing Serve to proxy `127.0.0.1:8080`; the sidecar publishes port 17397 only on host loopback.
- `tailscale-a` and `tailscale-b` continue to use the private Headscale test service and their existing separate state volumes.
- The sidecar runs `tailscaled` directly so its daemon remains available while interactive login waits. Start login with `docker exec -it rawrarr-orbstack-test-tailscale-coordinator-1 tailscale --socket=/tmp/tailscaled.sock up --hostname=rawrarr-orbstack-test-coordinator --accept-dns=false`; use the `AuthURL` in `tailscale --socket=/tmp/tailscaled.sock status --json` for one-time owner authorization. Never put that link or generated auth material in Git or this runbook.
- After authorization, verify the node is online on the expected tailnet, then configure and verify Serve only on the test coordinator identity:

  ```sh
  docker exec rawrarr-orbstack-test-tailscale-coordinator-1 tailscale --socket=/tmp/tailscaled.sock serve --bg --yes --https=443 http://127.0.0.1:8080
  docker exec rawrarr-orbstack-test-tailscale-coordinator-1 tailscale --socket=/tmp/tailscaled.sock serve status --json
  curl -fsS https://rawrarr-orbstack-test-coordinator.risk-mermaid.ts.net/health
  ```

- WebAuthn origin and RP ID in the test coordinator use the HTTPS MagicDNS hostname. A new test database has no existing passkeys to migrate.
- Verify node peer connectivity independently through Headscale; joining the operator tailnet does not join test subscriber nodes to it.

## Initial implementation state

- The Tailscale status on the existing MacBook sidecar confirmed tailnet `risk-mermaid.ts.net`; its existing Serve routes were read and left unchanged.
- The test Compose configuration validates and the coordinator, PostgreSQL, Headscale, and both metadata services report healthy. The coordinator root and `/health` respond on loopback port 17397.
- The new coordinator Tailscale sidecar is running with a dedicated persistent state volume and a persistent `tailscaled` daemon. The `tailscale up` flow is waiting for interactive authorization. No operator-tailnet membership or Serve URL is claimed until that authorization is reflected as online and the route is verified.
- Both metadata nodes can reach `http://coordinator:8080/health`; two Tailscale ping requests in each direction through test Headscale succeed; loopback health on `127.0.0.1:17397/health` returns 200.

## Remaining work

Complete the one-time device authorization, configure HTTPS Serve on the test coordinator identity, verify Serve status and the HTTPS health route, and recheck Headscale peer pings in both directions. Do not change or reset any Serve handler on the existing MacBook Tailscale node.
