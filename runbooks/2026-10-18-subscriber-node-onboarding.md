# Subscriber node onboarding on roarlay

## Objective and starting state

After passkey sign-in, present age and encrypted root signing-key setup before offering a Compose file for a subscriber-owned node. Only metadata and apex are offered. The signed-out landing page and existing local development Compose files are the starting points. Public OCI images and signed node approvals are not yet released; see [container distribution](2026-10-18-rawrarr-container-distribution.md) and [root design](../docs/device-signing-design.md).

## Work and verification

- Added authenticated `/setup/node`, reached after first coordinator passkey registration and from a workspace overview link. Returning sign-ins still enter the workspace.
- Added browser-only age generation/import and encrypted ML-DSA root generation/import validation. Generated backups require copying and acknowledgement before the next step. The page drops its private-material references on exit and does not submit them to roarlay.
- Added a subscriber-only Compose generator for metadata and apex. It uses named `/data` and `/shared` volumes, a loopback UI, optional read-only media bind, and local node enrollment. It rejects coordinator and never embeds private IDs or root backups.
- The UI and deployment guide explicitly identify the missing public image and inactive signing-root trust anchor. The download uses `rawrarr:local` and is a development preview.
- Focused Node.js crypto and Compose tests passed. Chromium exercised the generated age/root sequence, backup gates, and role list; it passed. Both generated role files passed `docker compose config --quiet`. `git diff --check` passed. A separate host Rust check could not run because host rustc cannot load its Homebrew Z3 library; the reusable container lacks the `redis` crate in its offline index. The actual Docker Compose deployment build compiled the release binary successfully.
- Checked the roarlay deployment marker, protected password-file path, port 7297 owner, current Compose labels, and Serve route. Claimed the roarlay marker atomically. Updated only roarlay in `rawrarr-local` using its recorded three-file Compose set, `--no-deps --build`, and the existing password-file path. Follow-up scoped rebuilds included local-passkey/Syncthing instructions and stricter Compose input validation. Compose warned about existing orphan containers; none were removed. The final roarlay image is `sha256:9e01f38c7c0b4517f58d1b9a12cb7f2875664eda16ca9961b847c3032fa78597` and the container is healthy.
- Local and tailnet `/health` returned HTTP 200 and identified roarlay. Tailscale Serve JSON still maps HTTPS `/` to `http://127.0.0.1:8080`. Signed-out `/setup/node` redirects to `/auth`; the new script is served over tailnet HTTPS, and the public landing page still responds. Authenticated live setup and a real subscriber deployment were not exercised.
- The served browser bundle hash matched the checked-out bundle. A temporary Docker named `/shared` volume was writable by the image's non-root user; that test volume was removed. Released the roarlay deployment marker after verification and confirmed its absence.

## Recovery and remaining work

Revert only this task's UI, Rust route, browser bundle/source, tests, documentation, and runbook commits and redeploy only roarlay to remove the preview; no runtime data or secret was changed by the implementation. Publish a versioned multi-architecture image before describing this as a general install path. Implement immutable/public root pinning and signed-approval validation before claiming the generated root authorizes nodes. Verify the full first-account flow against an isolated PostgreSQL instance and a real subscriber host.
