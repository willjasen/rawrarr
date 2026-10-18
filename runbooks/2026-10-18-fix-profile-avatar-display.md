# Fix profile avatar display

## Objective and initial state

Fix the missing profile images shown in the coordinator sidebar and Settings. The age-ID-derived avatar depended on a successful recipient fetch from an available metadata node; when that fetch failed, no avatar URL was assigned and the Settings image exposed its alt text.

## Actions

- Kept the age-ID-derived avatar as the preferred image and added a deterministic name-derived seed fallback when recipient retrieval fails.
- Clipped the Settings avatar image inside its circular frame.
- Added focused regression assertions for the fallback seed and avatar frame.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (13/13); `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Rebuilt only roarlay in `rawrarr-local` with the coordinator and MacBook Tailscale Compose overlays. Roarlay is healthy; local and tailnet `/health` returned HTTP 200; Serve still routes `/` to `http://127.0.0.1:8080` and preserves `/direct-api` at `http://127.0.0.1:8082`.
- Released the roarlay deployment lock after verification.

## Final state and remaining work

- Both profile locations now receive a generated avatar even when no metadata node can supply the public age ID. A live authenticated browser screenshot was not captured to verify visual appearance after deployment.
- No commit was created because the relevant files already contain unrelated user changes; staging whole files would include work outside this task.
