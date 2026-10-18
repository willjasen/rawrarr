# Show connected node pages from the coordinator

## Objective and initial state

After coordinator-origin passkey sign-in, open the registered metadata or storage node's Rawrarr page at `https://rawrarr.risk-mermaid.ts.net/`. Initially the coordinator root served only health. The local database had one owner-linked MacBook metadata node; its heartbeat was stale before deploying the new node heartbeat behavior. The mini storage node has no coordinator-reachable relay route yet.

## Actions

- Made coordinator `/` show the passkey page while signed out and the owner workspace when authenticated. Kept `/health` as its independent public liveness page.
- Added an owner-scoped connected-node list and pending-approval panel. The workspace selects the online metadata node by default and displays its Rawrarr UI in the coordinator page.
- Added `/nodes/{id}/...` relay routes that verify the coordinator session and node ownership, resolve only a trusted `RAWARR_NODE_RELAY_TARGETS` target, create a separate session scoped to that node, limit methods and request bodies, and rewrite app asset/API paths. The coordinator session and node bearer credential are not forwarded upstream.
- Added node-scoped coordinator auth routes for subscriber passkey/session operations. This keeps node ceremonies bound to the node's registered WebAuthn origin while coordinator web sign-in uses its Tailscale HTTPS origin.
- Added a 30-second node registration heartbeat so the node list can distinguish online from stale nodes.
- Updated local Compose and docs for relay targets/workspace behavior. The default local route maps `macbook-metadata` to `http://metadata:8080`; external node routes require explicit coordinator configuration.
- Rebuilt and recreated only the existing `rawrarr-local` coordinator and metadata services using the protected PostgreSQL password file path. PostgreSQL volumes and application data were preserved.

## Verification

- Passed focused Node tests for the coordinator workspace, linked coordinator passkeys, and passkey UI; 9 tests passed.
- Passed Python compilation and Node syntax checks for the disposable role integration test and its Playwright helper.
- `docker build -t rawrarr:roles-test -f rust-arr/Dockerfile rust-arr` succeeded with one existing unused-function warning.
- `python3 scripts/tests/test_rawrarr_role_deployment.py --apply` passed. Its real Chromium virtual-passkey flow signed into the coordinator, selected the online metadata node, loaded the metadata age-setup page inside the coordinator workspace, and verified node-scoped auth/enrollment behavior.
- Host `cargo check` and `cargo fmt --check` could not start because the host LLVM loader references a missing `libz3.5.1.dylib`; the full Docker release build compiled the Rust source successfully.
- After deployment, local coordinator `/` and `https://rawrarr.risk-mermaid.ts.net/` returned the passkey sign-in page, both `/health` routes returned HTTP 200, unauthenticated `/nodes/macbook-metadata/` returned HTTP 401, coordinator-to-metadata health was reachable, and PostgreSQL reported the metadata node recently online.
- The real user passkey ceremony at the live Tailscale origin was not performed. The isolated browser test exercised an equivalent virtual-passkey flow at its test origin.
- The user later asked to pause coordinator rebuilds because another task is redeploying it. This task's build and recreate had already completed before that instruction; no further rebuild or deployment commands were run afterward.

## Recovery and remaining work

- MacBook metadata page relay is active through the shared Docker network. Keep local port `7297` available for direct diagnosis.
- Configure `RAWARR_NODE_RELAY_TARGETS` with a trusted coordinator-reachable URL for each storage node before its page can be opened. The Mac mini Tailscale sidecar currently provides an outbound SOCKS proxy only; it does not expose the storage HTTP service to the coordinator.
- If the new workspace must be rolled back, restore the previous coordinator/metadata image and Compose source while preserving PostgreSQL and metadata volumes. Do not use `docker compose down -v`.
- The relay is a trusted application intermediary and can see proxied page/API plaintext; it has no `/media`, `/shared`, private age identity, or node-local data mount. Persistent authenticated outbound relay channels and multi-account invitations remain future work.

## Final state

- The signed-in coordinator workspace is implemented. At the time of verification, the local deployment served the new sign-in page and the MacBook metadata route was online; the other task may subsequently change the running container. Storage target reachability remains unconfigured.
