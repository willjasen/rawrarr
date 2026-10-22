# Deploy main to the internal Mac services

## Objective and starting state

Deploy the committed `main` revision `204e31a29e9ece95b1ad3c45d2155f6c484b4706` to the internal Mac mini coordinator (`rawrarr-internal-mini` / `roarlay`) and the MacBook Pro end-user metadata node (`rawrarr-node-internal` / `fossil`). The shared checkout contained unrelated staged, modified, and untracked work; build inputs came from a detached worktree at the exact commit. OrbStack was active on both Macs. Both services and the mini PostgreSQL were healthy before deployment.

## Deployment and verification

- Atomically claimed the per-service `roarlay` and `fossil` deployment markers. Used the MacBook OrbStack engine to build `rawrarr:local` for Linux/amd64 and the Mac mini OrbStack engine to build `rawrarr:internal-mini` for Linux/arm64 from the same `rust-arr` source.
- Recreated only `fossil` in project `rawrarr-node-internal`, then only `roarlay` in project `rawrarr-internal-mini`. Neither node data nor coordinator PostgreSQL/Tailscale state was changed.
- MacBook fossil image: `sha256:e2bbf68cb7f5ee6cc7f55ca6605fbcba01a8910df7a286d10ec398dbd5edb4d7`. Mini roarlay image: `sha256:cc29edef137872ed801b892e08871fd57b0a6c6d9738fd64c09682a3cafd1e91`.
- Verified both app containers healthy with zero restarts. Mini loopback health, mini coordinator HTTPS health, and MacBook node HTTPS health returned HTTP 200. PostgreSQL accepted connections. Tailscale Serve still maps coordinator HTTPS `/` to `http://127.0.0.1:8080`.
- Verified coordinator-to-fossil app health and `/direct-api/api/direct/health` both returned HTTP 200. A first direct-route probe omitted the API path and returned 404; the corrected route passed.
- Released and confirmed removal of both deployment markers. No automated test suite was run; validation was through native builds, container health, PostgreSQL readiness, Serve read-back, and live HTTP routes.

## Recovery and remaining work

If either deployment needs rollback, claim only that service's deployment marker, rebuild the prior source revision for that host's architecture, and recreate only its app service with its existing Compose project and secret paths. Preserve PostgreSQL, Tailscale state, and subscriber volumes. The original shared checkout changes remain untouched.
