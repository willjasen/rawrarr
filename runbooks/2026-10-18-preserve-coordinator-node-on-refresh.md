# Preserve the selected node page on refresh

## Objective and initial state

Refreshing the coordinator while a metadata node was open in its iframe returned to the coordinator workspace. The coordinator kept the selected node only in JavaScript memory, and the embedded app's fragment was not reflected in the top-level URL. The current checkout has extensive unrelated changes; preserve them. The coordinator and metadata containers are in project `rawrarr-local`; the metadata node owns loopback port 7297, and the coordinator diagnostics use 7298.

Related: [Preserve the current page on refresh](2026-10-18-preserve-page-on-refresh.md), [Coordinator HTTPS routing](2026-10-18-coordinator-https-routing.md).

## Actions

- Save the selected node in the coordinator URL query and restore it from the approved, relay-available node list.
- Mirror the embedded Rawrarr view fragment into the top-level URL and restore that fragment in the iframe.
- Rebuild the shared app image and update the coordinator and metadata containers in the existing `rawrarr-local` project. Preserve PostgreSQL, volumes, and Tailscale Serve handlers.

## Verification and final state

- Focused coordinator workspace regression checks passed (3/3); `git diff --check` passed.
- Updated the coordinator and metadata containers in the existing `rawrarr-local` project using the protected password file path without reading its contents. An initial metadata-only `--no-deps` update left the shared-image coordinator in `Created`; started the coordinator explicitly with its original Compose file set. Both containers serve successfully afterward.
- `http://127.0.0.1:7297/health` returned HTTP 200. Public coordinator health returned HTTP 200 and identified the coordinator; the root returned the coordinator sign-in page. Tailscale Serve read-back still shows `rawrarr.risk-mermaid.ts.net:443` `/` proxying to `http://127.0.0.1:8080`.
- An authenticated browser refresh of the user-specific movie page was not exercised; verification covered the source regression assertions and the live coordinator/metadata health and routing. Existing unrelated source changes overlap the affected files, so the task changes remain uncommitted to avoid including them.

## Recovery

Revert only this task's URL persistence changes and rebuild the coordinator with the same Compose project and configuration. Keep the existing database, metadata node, and volumes.
