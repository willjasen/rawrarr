# Coordinator-owned health page

## Objective and initial state

Give the coordinator its own styled health page, matching the metadata status design without forwarding to subscriber nodes. HTTPS already routes to coordinator; `/health` returns plain text and `/` returns 401. Existing staged and unstaged work is preserved.

References: [metadata health page](2026-10-18-health-page.md), [coordinator HTTPS routing](2026-10-18-coordinator-https-routing.md).

## Implementation

Serve a self-contained coordinator HTML page at `/health` and `/`, outside node authentication. Report coordinator web-process liveness only; database, authentication, and subscriber readiness are explicitly not checked. Keep protected API authorization unchanged. No metadata requests, data, or assets are used.

## Verification and recovery

- Docker release build passed; recreated only the coordinator with `docker compose -p rawrarr-local up -d --no-deps coordinator` and the existing protected password-file path.
- `python3 scripts/tests/test_coordinator_health.py --base-url http://127.0.0.1:7298` passed; the same test against `https://rawrarr.risk-mermaid.ts.net` passed. Both routes return identical self-contained HTML with coordinator-specific status, no redirect, explicit liveness scope, and an authenticated API still returning 401 anonymously.
- Metadata health on port 7297 remains HTTP 200. Its container and data were preserved.
- In-app browser preview failed with `ERR_BLOCKED_BY_CLIENT`; visual rendering is not verified.
- Initial rollback image tagging failed because the running image ID was no longer available in Docker's image store. Preserved the previous executable at `/tmp/rawrarr-coordinator-health-rollback-20261018/rust-arr` before recreation; no secret mounts were copied. Recovery can rebuild an image using that executable over the current runtime image.
- Python compilation and task whitespace checks passed.

## Final state

Coordinator health is native to the coordinator with no subscriber dependencies. New page, test, and runbook are task-owned. The route wiring in pre-existing untracked `coordinator.rs` and index entries in mixed-ownership README files remain uncommitted to avoid including others' work. No Git remote is configured.
