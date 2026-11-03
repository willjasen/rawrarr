# Deploy the rawrs voice Talk fix

## Objective and tracking

Deploy the fix for the coordinator voice tab Talk button, tracked by [issue #195](https://github.com/willjasen/rawrarr/issues/195) and merged in [PR #196](https://github.com/willjasen/rawrarr/pull/196). Published `main` is `fc229749cecd47a2282a75719a0cdb9e9e801fce`.

## Source and local builds

- Local `main` was synchronized to the merged published commit before deployment. The target classifier selected coordinator and nodes for `rust-arr/src/coordinator-ai.mjs` and the related test path.
- Downloaded the authenticated GitHub tarball for the full published commit and verified SHA-256 `00dedd37e3992679aff7af8973fbb71c3c72cadd157e5207e488f2ac0fccabce` on the workstation and both internal Macs.
- Built the `linux/amd64` image locally on the Mac mini and locally on the MacBook Pro from that archive. Both images carry OCI revision label `fc229749cecd47a2282a75719a0cdb9e9e801fce`. The mini image ID is `sha256:9d477b2d41ee462cecab9fe6e6b72d85f8cacc67ac43dc582466bf3a434ee7d5`; the MacBook image ID is `sha256:9f8f3024ce91b95ab670e884970cafb82a32a6f19ad1a6031b5625c72ae1f5cd`.
- Host Docker context was `orbstack` on both Macs. The Mac mini ran the amd64 image with its existing emulation support. No GitHub-built container image was deployed.

## Deployment and verification

- Atomically claimed separate `.container-rebuild-roarlay.lock` and `.container-rebuild-fossil.lock` markers. Recreated only `roarlay` in `rawrarr-internal-mini`, then only the MacBook `fossil` in `rawrarr-node-internal`; released each lock after verification.
- Roarlay is healthy with zero restarts on the locally built `fc22974` amd64 image. `rawrarr.risk-mermaid.ts.net:443` Serve status remains `/` -> `http://127.0.0.1:8080`; `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200 and the Roarlay health page.
- PostgreSQL remained healthy and the coordinator Tailscale sidecar remained running; neither was recreated.
- MacBook fossil is healthy with zero restarts on the locally built `fc22974` amd64 image. Its HTTPS `/health` and `/direct-api/health` endpoints returned successfully. Its Tailscale sidecar remained running.
- The authenticated assistant page was not verified in the browser because the available Chrome session was signed out. The merged revision's GitHub Basic script tests and private UI bundle publication passed; local `node --test scripts/tests/test_rawrarr_ai_chat.mjs` passed all 8 tests before merge.

## Recovery

The previous coordinator image is retained as `rawrarr:rollback-roarlay-fc22974`; the previous MacBook image is retained as `rawrarr:rollback-fossil-fc22974`. If a service needs rollback, recreate only that service under its deployment lock and verify its health and route before releasing the lock.
