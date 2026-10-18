# Implement the Tailscale direct node route

## Objective

Move eligible browser requests for the MacBook metadata node directly through Tailscale Serve to a narrow node API listener, with visible automatic selection and coordinator relay fallback.

## Initial state

- The worktree has extensive pre-existing user changes, including the coordinator workspace, role-specific Compose files, and architecture documentation. Preserve those changes and stage only task-owned paths/hunks.
- The active `rawrarr-local` project has healthy coordinator and metadata containers. The MacBook Tailscale container shares their namespace; Serve currently routes `/` to coordinator port 8080. Existing metadata listens on loopback port 8081.
- No coordinator or metadata rebuild lock markers were present at inspection time. Host port 7297 had no listener; no replacement service has been started.
- The existing coordinator proxy is allowlisted and the browser decrypts the metadata ciphertext. This task adds a separate route only for encrypted metadata export and encrypted library responses.

## Planned actions

- Add a loopback-only metadata direct API listener with only the existing encrypted library/ciphertext endpoints and a public health probe.
- Add a coordinator-authenticated, short-lived session-cookie issuer scoped to the direct API path and selected owner node.
- Advertise direct availability to the workspace, probe automatically, show the selected path, and fall back to the existing `/nodes/{id}` relay on probe or request failure.
- Add Tailscale Serve's `/direct-api/` path handler while preserving `/` and Syncthing TCP forwarding; rebuild only locked coordinator/metadata containers.
- Add focused tests, update architecture/runbook indexes, and verify health and route read-back.

## Verification

- `node --test scripts/tests/test_rawrarr_direct_route.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 9 passed.
- Rust test build compiled successfully; 36 passed, including direct route validation and direct-session cookie tests. Four unrelated existing tests failed because their fixture files and `age-keygen` are absent in the generic Rust image.
- Full project test runner: Playwright passed; existing Node suite had 10 unrelated failures (including several pre-existing stale/fixture-dependent checks), and Rust offline dependency resolution could not find `p256`. Direct-route Node tests passed separately after the runner's earlier stale failure report.
- Compose merged configuration validated using the protected PostgreSQL password-file path and `RAWARR_NODE_ID=macbook-metadata`.
- Rebuilt only the `coordinator` and `metadata` services in Compose project `rawrarr-local`; both are healthy and PostgreSQL remains healthy.
- Metadata listener and Tailscale `/direct-api/api/direct/health` both return HTTP 204. `https://rawrarr.risk-mermaid.ts.net/health` returns the coordinator health page.
- Tailscale Serve read-back confirms `/` still proxies to `http://127.0.0.1:8080` and `/direct-api` proxies to `http://127.0.0.1:8082`. The pre-change snapshot had no TCP 22000 handler; none was changed or added.
- Port 7297 remains without a listener. The prior standalone `rawrarr` container is in `Created` state under the separate `rawrarr` Compose project; the active `rawrarr-local` project now contains role-specific coordinator and metadata services, so this task did not start or alter that stale container.
- Host `rustfmt` remains unavailable due to a broken Homebrew LLVM/Z3 linkage; the Docker production build and Rust test compilation succeeded.

## Final state

The MacBook metadata node advertises an owner-scoped direct URL. The coordinator issues a five-minute path-scoped session cookie, and the browser automatically chooses the direct route when reachable, reports the active route, and falls back to the coordinator relay on failure. Same-LAN routing and hosted-portal support remain planned. Deployment markers were removed after health and Serve verification.

## Recovery

Remove only the `/direct-api/` Serve handler if its route fails; preserve `/`, TCP forwarding, and Tailscale identity. Roll back the app image/Compose changes without deleting volumes or shared data.
