# Clean up obsolete Rawrarr containers on the MacBook

## Objective and initial state

Free host ports 7297 and 7298 and remove obsolete Rawrarr containers after a subscriber Compose launch failed on port 7297. `rawrarr-macbook-tailscale` in Compose project `rawrarr-local` owned both loopback mappings. Its `/health` host routes returned an empty reply because the matching app container was no longer running. The sidecar still had the old `rawrarr.risk-mermaid.ts.net` Serve configuration and Syncthing TCP 22000 forward.

The current `rawrarr-orbstack-test` coordinator and two-node Headscale environment is separate. The active `rawrarr-local-syncthing-1` service carries user shared data and must remain running.

## Actions

- Remove the stale `rawrarr-macbook-tailscale` container to release 7297 and 7298. Keep its host Tailscale state directory and Compose configuration; the old Rawrarr HTTPS URL and Syncthing TCP forward go offline while this container is absent.
- Remove the old `rawrarr-headscale-macbook-tailscale` test client and stale `rawrarr-test-fossil-1`, `rawrarr-test-sonarr`, `rawrarr-testing-role-split`, and `base-test-image-mvdgto87` containers.
- Keep `rawrarr-orbstack-test-*`, `rawrarr-local-syncthing-1`, Swarmsail containers, and all Docker volumes and bind-mounted state.
- Remove the unused `rawrarr-test_default` bridge network if it becomes empty. Preserve the newly created `rawrarr-test` volumes.

## Verification

- Removed the six listed obsolete containers and the empty `rawrarr-test_default` network. Host ports 7297 and 7298 now have no TCP listeners.
- The OrbStack coordinator, PostgreSQL, Headscale, and both metadata nodes report healthy; coordinator health on `127.0.0.1:17397` returns HTTP 200.
- `rawrarr-local-syncthing-1` remains healthy. The Swarmsail-owned `headscale-rawrarr-local` container remains running.
- The `rawrarr-test_rawrarr-data` and `rawrarr-test_rawrarr-shared` volumes remain present. Tailscale state directories were not deleted.

## Recovery

The deleted Tailscale and test containers can be recreated from their existing Compose files and images. Tailscale node state remains in the bind-mounted Application Support directories. Do not remove Rawrarr or Syncthing volumes during this cleanup.
