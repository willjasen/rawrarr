# Test and deploy GitHub main in OrbStack

## Objective and issue

Run local regression and Playwright tests against clean GitHub `main`, then deploy its coordinator and metadata-node changes to the internal environment. Tracked by [issue #39](https://github.com/willjasen/rawrarr/issues/39).

## Source and initial state

- GitHub `main` was fetched at `f490f5806f6a39c83131bc53af8e211a650cb94b`; tests and deployment builds used a clean detached worktree at that commit. The shared `main` checkout had extensive unrelated modifications and deletions and was left untouched.
- The internal Mac mini source checkout was at `742f75d`. The target diff contains coordinator and metadata-node runtime changes, so both `roarlay` and `fossil` were selected.
- OrbStack was available on both Macs. Existing roarlay and fossil containers were healthy. Their deployment locks were absent before this task. The mini's protected GitHub UI token file existed with mode 600.

## Tests

- The first OrbStack test attempt stopped before running tests because the cached test image lacked web-export dependencies. Rebuilt the image with `python3 scripts/tests/run_rawrarr_tests_in_docker.py --rebuild-base` and reran the suite.
- Node.js regression suite: 69 passed, 23 failed. Failures included missing `/workspace/admin/index.html` fixtures and stale UI/source expectations in age setup, coordinator workspace, direct-route, and other tests. The full log remains in `rawrarr-testing-tests-mvk4wgxl` on the local OrbStack context.
- Playwright browser test: 1 passed.
- Rust unit suite: 69 passed, with two dead-code warnings.
- Focused ECDH/HKDF browser-node assertion: 1 passed using `node --test --test-name-pattern='browser and metadata node derive the encrypted library key' scripts/tests/test_rawrarr_coordinator_workspace.mjs`.

## Deployment and verification

- Built both images from the clean `f490f58` worktree: fossil `rawrarr:local` (`sha256:0322a4d6742dbbec8e04c101328cb9acdbb6b21c9ec13f7af26db1dbcb6d4f71`, amd64) and roarlay `rawrarr:internal-mini` (`sha256:66700e4a92f9e95186cdb856d3900e37edbc2b75ff2d72a15d182d6b224cb634`, arm64).
- Recreated only fossil in `rawrarr-node-internal` and only roarlay in `rawrarr-internal-mini`. PostgreSQL and Tailscale services were not recreated. Both node deployment locks were held through verification and then removed.
- Fossil is healthy with zero restarts; its HTTPS tailnet `/health` returned HTTP 200. Its Tailscale container ID was unchanged.
- Roarlay is healthy with zero restarts; coordinator HTTPS `/health` returned HTTP 200. PostgreSQL and Tailscale container IDs were unchanged, with zero restarts. Tailscale Serve read-back confirms `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`.
- Both services report installed bundle version `f490f5806f6a39c83131bc53af8e211a650cb94b` and SHA-256 `8a5f5c7dd5bf2725bef83a79ddba49110d4c3bf02cb33b1beecf882773dc4eef`, matching the published release asset.

## Recovery and follow-up

Both services use their existing Compose projects and persistent volumes. If a rollback is needed, rebuild the prior deployed source revision for the host architecture and recreate only the affected app service with `--no-deps`; retain PostgreSQL, Tailscale state, `/ui`, and protected secrets. The 23 failing Node.js assertions remain to be triaged; passing Playwright, Rust, and focused ECDH checks do not resolve those suite failures.
