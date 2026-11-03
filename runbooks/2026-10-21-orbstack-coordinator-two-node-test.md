# OrbStack coordinator and two-node Headscale test

> Historical record. Since 2026-11-03 the definition and static fixtures live
> under `deploy/testing/`, and the current project is `rawrarr-test-environment`.
> Use [the current testing guide](../TESTING.md#isolated-coordinator-and-two-fossil-environment)
> and [the cleanup/adoption record](2026-11-03-compose-inventory-test-layout.md)
> instead of replaying the old commands below.

## Objective and boundaries

Run an isolated Rawrarr coordinator, PostgreSQL, Headscale, and two metadata subscribers in OrbStack. Verify coordinator/node health, node heartbeats, Headscale enrollment, and bidirectional Tailscale peer pings. This test does not use production `do2`, Swarmsail's active projects, or the existing Rawrarr Tailscale client identities.

Compose project: `rawrarr-orbstack-test`. Coordinator: `http://127.0.0.1:17397`. Headscale is private to the project network. Subscriber Tailscale addresses are `100.108.0.1` and `100.108.0.2`. Headscale serves HTTPS using a test-only local certificate; the clients trust its test CA. Application and database secrets, Headscale data, and all test Tailscale state live outside the repository under `~/Library/Application Support/rawrarr/orbstack-test/` and named test volumes.

Related procedures: [Headscale container client staging](2026-10-18-headscale-container-client-staging.md) and [subscriber node enrollment](2026-10-18-subscriber-node-enrollment.md).

## Initial state and cleanup

OrbStack was already active. The old `rawrarr-local` coordinator, metadata, Redis artwork-cache, and PostgreSQL containers were stopped and removed after inspecting their mounts. Their named volumes—including PostgreSQL and metadata state—and the `/shared` synchronized-data bind were preserved. The existing Rawrarr Syncthing service, the established Rawrarr Tailscale clients and their state directories, and Swarmsail/Headscale services were left running.

The main checkout contained user changes at task start. They were not reverted or staged. The test uses separate Compose project and volume names and loopback port 17397, leaving the established 7297/7298 listeners alone.

## Configuration and operation

- [`deploy/testing/compose.yml`](../deploy/testing/compose.yml) defines PostgreSQL, coordinator, Headscale, and two metadata subscribers. Only the coordinator binds a host port, on `127.0.0.1:17397`.
- [`deploy/testing/headscale/config.yaml`](../deploy/testing/headscale/config.yaml) gives the test its own `100.108.0.0/24` allocation range, SQLite database, and MagicDNS suffix.
- Host-only files contain random PostgreSQL and Rawrarr node credentials, a synthetic approved-node registry, one-use Headscale preauth keys, and the self-signed TLS certificate/key. Never copy them into Git or another environment.
- Bring the existing stack back up with:

  ```sh
  docker compose -p rawrarr-orbstack-test -f docker-compose.orbstack-test.yml up -d
  ```

- Coordinator health is at `http://127.0.0.1:17397/health`. Container health checks cover PostgreSQL, Headscale, coordinator, and each Rawrarr node.
- Inspect Headscale peers with `docker exec rawrarr-orbstack-test-headscale-1 headscale nodes list`.
- Check connectivity from either node with `docker exec rawrarr-orbstack-test-tailscale-a-1 tailscale ping 100.108.0.2` and the reverse from `tailscale-b` to `100.108.0.1`.

Do not run `docker compose down -v` unless intentionally deleting every test database, node state, Headscale state, and Tailscale identity in this isolated project.

## Verification results

- `docker compose ... config --quiet` passed; Headscale `configtest` passed.
- Coordinator, PostgreSQL, Headscale, and both metadata nodes report healthy. Coordinator HTTP `/` returned 200 and `/health` returned the Roarlay health page.
- Both metadata node `/health` routes returned HTTP 200 from their shared Tailscale network namespaces.
- Coordinator PostgreSQL contains current heartbeats for `test-metadata-a` and `test-metadata-b`.
- The isolated Headscale instance lists exactly the two test peers as online under the synthetic `rawrarr-orbstack-test` user: `100.108.0.1` and `100.108.0.2`.
- Three `tailscale ping` requests succeeded from A to B and three from B to A. Each returned a peer pong via the peer's OrbStack network endpoint in about 1 ms, confirming peer-to-peer connectivity rather than only a coordinator health signal.
- Existing Swarmsail/Headscale projects and established Rawrarr Tailscale clients remain running and unchanged.

## Scope and remaining work

This verifies the local coordinator/node control and Headscale peer-connectivity path. No live Radarr or Sonarr service was configured in this test, so Arr API connection and library behavior remain unverified here. The test uses preapproved synthetic Rawrarr node tokens rather than interactive account signup and per-account subscriber provisioning. Keep this stack available for follow-up review; remove only project `rawrarr-orbstack-test` when it is no longer needed, and preserve other Docker projects and volumes.
