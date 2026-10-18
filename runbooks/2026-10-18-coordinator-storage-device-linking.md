# Coordinator roles and storage device-code linking

## Objective and initial state

Record the accepted coordinator/storage/shard/service roles and revise the Mac mini plan. The user identifies the current MacBook deployment as coordinator, wants Tailscale HTTPS connectivity from the Mini, and requests a URL plus human device code in Docker logs for linking, analogous to familiar device enrollment flows.

Reviewed the prior [Mac mini planning record](2026-10-18-mac-mini-storage-node-plan.md), node/device/data-domain docs, device-signing note, Tailscale plan, Compose shape, and startup source. `storage-scan-once` exits before PostgreSQL/web authentication startup; the full app still connects to PostgreSQL. This corrects the earlier overbroad claim that all startup paths require the database.

Branch is `main` with extensive unrelated edits. No remote is configured. Preserve unrelated changes and use an isolated staging index for task-only documentation.

## Changes and decisions

- Coordinator owns user authentication, PostgreSQL, catalog, and coordination; storage needs local device/scan state without user passkeys or PostgreSQL. Coordinator and storage may combine; shard serving responsibilities remain open.
- Revised Mac mini deployment and Tailscale plans. The worker reads silo media read-only and connects outbound to the MacBook coordinator; only the coordinator backs the planned named Service.
- Added the device-code linking plan: `/devices/link`, short-lived code in local Docker logs, owner sign-in/code entry/review/approval, device-key proof, expiry/rate limits/revocation. Keep human codes out of shared logs and long-lived credentials out of all logs.
- The user also requires local age private-ID entry on the Mini, separate from web/device linking. Planned hidden terminal input, protected local file/read-only secret mount, recipient validation, and no transfer through the coordinator.
- Recorded container-level reachability, WebAuthn origin migration, existing Serve-route preservation, worker lifecycle, and catalog ingestion as implementation work. Checked current official Tailscale documentation; did not publish a service or change networking.

## Verification, final state, and remaining work

Reviewed task-only diff, checked local Markdown links, and ran whitespace checks on the isolated staged changes. No runtime behavior changed; application tests, rebuilds, and live deployment checks do not apply. No SSH, media scan, Tailscale mutation, or code-flow implementation was performed. Existing source/config changes remain untouched.

Next: implement the persistent storage worker and coordinator enrollment, then verify Tailscale/HTTPS/WebAuthn from the actual browser and Mini container before deploying. Documented plan URLs and codes are placeholders, not working endpoints. No runtime rollback is needed; documentation can be reverted by its task commit if required.
