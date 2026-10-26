# Complete OrbStack Arr connection testing

## Objective and context

Finish GitHub issue [#99](https://github.com/willjasen/rawrarr/issues/99) by testing Rawrarr's actual encrypted Radarr and Sonarr connection paths from fossil. The earlier [connectivity runbook](2026-10-26-orbstack-arr-connectivity-test.md) verified only authenticated upstream API access; this task extends it through settings persistence, status, and library reads.

## Changes

- Added optional `RAWARR_ARR_CA_CERT_FILE` support to the Arr HTTP client. It adds the specified PEM certificate as an extra trust root while preserving the built-in roots and HTTPS-only URL validation. When unset, existing trust behavior is unchanged.
- Added a Caddy TLS proxy for the isolated Arr containers. It serves `radarr.test` and `sonarr.test` with a task-generated certificate signed by a private CA stored under `~/Library/Application Support/rawrarr/orbstack-test/arr-tls/`. The CA private key, leaf private key, and certificate files are outside Git; the CA public certificate is passed to fossil as a read-only Compose secret.
- Extended the OrbStack integration script to seed a short-lived, node-scoped test session in the isolated PostgreSQL database, generate/persist a test-only age identity when needed, save both Arr settings through Rawrarr's API, verify redacted settings readback, and read each service status and library. It removes the synthetic user/session rows after each run. API keys and the age identity stay on stdin/in-process and are never printed.
- The test node keeps its own age identity in its isolated `/data` volume. Its encrypted settings remain in the isolated test `/shared` volume so repeat runs validate persistence.

Related procedure: [OrbStack coordinator and two-node test](2026-10-21-orbstack-coordinator-two-node-test.md).

## Run

On the OrbStack host, from the repository root:

```sh
python3 scripts/tests/test_orbstack_arr_connectivity.py --start
```

The script creates or validates the private test CA, validates Compose, builds the local Rawrarr image, starts or updates services only in Compose project `rawrarr-orbstack-test`, and provides the current coordinator signing-root fingerprint to the test fossil container. It then verifies the HTTPS proxies from fossil before saving settings through Rawrarr.

Expected connection output:

```text
PASS fossil saved encrypted Radarr settings; status and library APIs succeeded
PASS fossil saved encrypted Sonarr settings; status and library APIs succeeded
```

Run without `--start` to repeat the settings/readback/status/library checks against the already-running stack:

```sh
python3 scripts/tests/test_orbstack_arr_connectivity.py
```

## Verification

- `python3 scripts/tests/test_orbstack_arr_connectivity.py` passed against the running stack, and a subsequent `--start` pass also succeeded. The latter unlocked the persisted test configuration and confirmed repeatable encrypted readback, Radarr `/api/v3/system/status` and `/api/v3/movie`, and Sonarr `/api/v3/system/status` and `/api/v3/series` through fossil.
- `python3 -m py_compile scripts/tests/test_orbstack_arr_connectivity.py` passed.
- `docker compose -p rawrarr-orbstack-test -f docker-compose.orbstack-test.yml config --quiet` passed.
- `cargo fmt --check` could not run: the host formatter could not load its missing Homebrew `libz3.5.1.dylib`, and the Rust build image does not include the `rustfmt` component. The Docker image build compiled the modified Rust source successfully.
- `git diff --check` passed.

## Final state and cleanup

The isolated coordinator, fossil, Headscale, Radarr, Sonarr, and TLS proxy remain running. The test CA/keypair is under the owner-only OrbStack test-state directory; the age identity and encrypted Arr settings remain only in the isolated test node/shared volumes. No production service, identity, media, or database was changed.

Issue #99 acceptance criteria are met. Close the issue after this change is merged.
