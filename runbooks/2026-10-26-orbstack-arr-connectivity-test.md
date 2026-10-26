# OrbStack Radarr and Sonarr connectivity test

## Objective and scope

Add disposable Radarr and Sonarr services to the isolated coordinator/node test environment and verify that fossil can reach both authenticated APIs. GitHub issue [#99](https://github.com/willjasen/rawrarr/issues/99) tracks this work. The existing [coordinator and two-node test](2026-10-21-orbstack-coordinator-two-node-test.md) remains the procedure for Headscale peer checks.

This test verifies fossil's container-network and Arr API connectivity. It does not configure Rawrarr's encrypted Arr settings or verify library loading: Rawrarr requires HTTPS Arr URLs, while the isolated Arr services expose HTTP and the Rust client trusts public roots. Do not weaken the HTTPS policy to make this local fixture pass.

## Environment and changes

- Compose project: `rawrarr-orbstack-test`; coordinator loopback URL: `http://127.0.0.1:17397`.
- Added LinuxServer Radarr and Sonarr containers with separate named `/config` volumes. They have no host-published ports or media mounts.
- The first synthetic metadata node is displayed as fossil. Its existing node ID and Tailscale identity remain `test-metadata-a` to preserve the pre-existing synthetic registry and persisted state.
- The test starts the coordinator first, reads its public instance-root fingerprint from `/api/nodes/signing-root`, and passes that fingerprint only to the local fossil Compose process. No root fingerprint is hard-coded.
- Tailscale test clients explicitly retain their non-default Headscale and DNS settings when starting with current Tailscale images.

## Run and verify

Run from the repository root on a Mac with OrbStack and the existing private test-state files under `~/Library/Application Support/rawrarr/orbstack-test/`:

```sh
python3 scripts/tests/test_orbstack_arr_connectivity.py --start
```

The script checks Docker context `orbstack`, validates Compose, starts Radarr, Sonarr, PostgreSQL, Headscale, the coordinator and fossil, waits for service health, reads each generated Arr API key inside its own container, and sends the authenticated status request from fossil. API keys are passed to the one-shot Python process over stdin and are not printed. Expected output:

```text
PASS fossil reached Radarr API
PASS fossil reached Sonarr API
```

Running without `--start` checks services already running in the test project:

```sh
python3 scripts/tests/test_orbstack_arr_connectivity.py
```

## Verification and result

- `docker compose -p rawrarr-orbstack-test -f docker-compose.orbstack-test.yml config --quiet` passed.
- `python3 scripts/tests/test_orbstack_arr_connectivity.py --start` passed for Radarr and Sonarr. Both returned the expected authenticated `/api/v3/system/status` identity from fossil's network namespace.
- Test API keys were not included in command output.
- Rawrarr settings-save, encrypted persistence, and library behavior against these plain-HTTP services remain unverified because the application requires HTTPS with a trusted certificate.

## Cleanup and remaining work

Keep the isolated project available for follow-up. If it needs to be stopped, stop only `rawrarr-orbstack-test`; preserve its named volumes unless intentionally removing all disposable test state. Do not use `down -v` as routine cleanup.

Issue #99 remains open for a future local strategy that can exercise the HTTPS-only settings save and library path with a trusted test certificate or a purpose-built TLS test fixture.
