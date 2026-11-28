# Owner-issued app-key grants

Issue: [#497](https://github.com/willjasen/rawrarr/issues/497).

## Objective and initial state

Implement browser-side creation of owner-root-signed, fossil-bound app-key grants. The node verifier format is defined by [PR #494](https://github.com/willjasen/rawrarr/pull/494); delivery and settings activation are outside this issue. The prepared app identity registry is encrypted to the shared group recipient, so possession of its ciphertext alone does not authorize issuance.

## Design and adoption

The owner client must decrypt the owner-wrapped group identity and then the prepared app identity registry locally; it must encrypt the selected app identity directly to the approved fossil's age recipient and sign the canonical grant payload with the owner-held encrypted root backup. Only the resulting public metadata and device-encrypted envelope may leave the client. This adds no migration or deployment change because the issuer is not connected to grant delivery or settings activation.

## Progress

- Fetched `origin/main` at `06007b367a73ddf91a0b7f2c84ce0d2b1e15dc83` and created dedicated worktree `codex/issue-497-owner-app-key-grants`.
- Preserved the unrelated untracked runbook in the shared checkout and the separate issue #500 worktree.
- Confirmed #497 is open and PR #494 remains open; its bundle schema is the target compatibility contract.
- Added browser crypto helpers to unwrap the prepared app identity only after decrypting the owner-wrapped group identity, then encrypt the app identity directly to one fossil recipient and sign the v1 canonical payload with the owner-held root backup. The builder requires an explicitly refreshed minimum accepted key epoch and a strictly greater new epoch. It returns the verifier-compatible bundle and does not submit it to the coordinator.
- Added focused cases for target-field signature binding, digest changes, device-only decryption, owner authorization, malformed/oversized registry data, missing app identity, invalid root access, and stale epoch rejection; rebuilt the browser asset.

## Verification and remaining work

- In an isolated, uniquely named AMD64 `rawrarr-test-regression` OrbStack container, copied the current worktree source/tests, ran `prepare-test-clone.sh` with network access, disconnected the clone, and ran `node --test scripts/tests/test_rawrarr_device_signing_crypto.mjs` — 4 passed, 0 failed. The same focused test exercises both source and generated browser modules.
- `git diff --check` — passed.
- Direct host execution was unavailable because the checkout has no web-export `node_modules`; using the prepared isolated clone resolved the dependency gap. The full Rust, Node, Playwright, and environment suites were not run. No end-to-end grant installation or delivery was exercised; PR #494 remains open and defines the node verifier contract.
- No existing data or protocol migration is required because issuance is client-side and is not wired to app settings or coordinator delivery. The utility requires callers to source the app identity through the owner-authorized decrypt helper and to refresh the target's accepted epoch before issuing. Grant delivery remains tracked separately by [#500](https://github.com/willjasen/rawrarr/issues/500).

## Final state

Implementation commit `94fbba6` is on `codex/issue-497-owner-app-key-grants` and pushed. Pull request [#504](https://github.com/willjasen/rawrarr/pull/504) targets `main`; it was open and mergeable at the last check, with the `Basic script tests` GitHub Action still running. No deployment or app-settings activation was performed. The shared checkout's unrelated untracked runbook and other task worktrees remain untouched.
