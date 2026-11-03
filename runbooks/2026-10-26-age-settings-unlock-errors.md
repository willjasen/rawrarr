# Report accurate age settings unlock errors

Issue [#164](https://github.com/willjasen/rawrarr/issues/164).

## Objective

When the browser unlocks a node's age vault before saving Radarr or Sonarr settings, report the actual failure category instead of labeling every vault initialization error as an invalid user identity.

## Initial state

The Movies and Settings flows correctly install the group identity encrypted to the node, then wrap the user's age identity to the node's device identity. In `unlock_age_settings`, every error from `AgeVault::from_identity` was converted to HTTP 400 with `Import a valid post-quantum age identity`. That also hid node age-tool failures, local/shared storage failures, and encrypted-config failures.

GitHub issue search found no existing issue for this behavior. Issue #110 describes the future per-app identity model and is not the current unlock failure. Created issue #164 before implementation.

## Changes

- The node previously mapped every `AgeVault::from_identity` error to HTTP 400 with “Import a valid post-quantum age identity.” This concealed age CLI, storage, group-key mismatch, and encrypted-config failures.
- Added stage-specific safe error messages and HTTP statuses for invalid/mismatched IDs, unavailable age tools, inaccessible key storage, encryption/decryption failures, and malformed config. No identity, ciphertext, or credential data is included in errors.
- Added Rust mapping tests and an integration assertion for a genuinely wrong group identity. Updated the test index.
- Follow-up: when `group-age-recipient` is missing, a mismatching legacy `age-recipient` is no longer treated as conclusive proof of a wrong user ID. The submitted identity must decrypt a valid existing `config.aged` before Rawrarr creates any new group-key files. A nonmatching identity or missing/unreadable config leaves the shared and node-local key state unchanged.
- The original error-reporting fix merged as PR [#165](https://github.com/willjasen/rawrarr/pull/165). The guarded legacy-recipient migration fix merged as PR [#172](https://github.com/willjasen/rawrarr/pull/172), merge commit `ecd5ec725d9dc6879797e933d9d5e6f577bb3b15`.

## Verification

- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed: Node 112/112, Playwright 14/14, Rust 96/96. The Python integration phase is intentionally skipped by this runner; its disposable PostgreSQL and approved-node environment was not started.
- `python3` AST parse of `scripts/tests/test_rawrarr_configuration.py` — passed.
- `git diff --check` — passed.
- Host `cargo` and `rustfmt` are unavailable, so formatting and the direct host Rust command were not run. The isolated Rust build and test suite compiled and passed.
- Follow-up `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` passed: Node.js 112/112, Playwright browser suite passed, and Rust 97/97 including the legacy-recipient migration regression. Python integration tests were skipped by the isolated runner.

## Internal deployment

- Published GitHub `main` advanced to `ef2fa2752496ff09ce5f50ed0444d88e47b3d535` before deployment. The deployment classifier selected both coordinator and node targets from previously deployed `d00e3b3`; builds used only the `rust-arr` source snapshot archived from that exact published commit.
- Rebuilt the Mac mini arm64 image (`sha256:887b85385ca6df5b5ad5dba7f7380355349e6959ce59b9bf27deda0d222775e0`) and MacBook amd64 image (`sha256:719d78078814ee8bf2665afb71d585d10aadb4c1c28d91a1ca92de0dc13d3070`).
- Recreated only `fossil` in mini project `rawrarr-internal`, `roarlay` in `rawrarr-internal-mini`, and `fossil` in MacBook project `rawrarr-node-internal`, each with `--no-deps`. All three report healthy with zero restarts. PostgreSQL, Tailscale, Valkey, and node data volumes remained running/preserved with zero restarts where health is reported.
- Coordinator `https://rawrarr.risk-mermaid.ts.net/health` and both fossil health routes returned HTTP 200. The mini Tailscale Serve read-back still maps coordinator HTTPS `/` to `http://127.0.0.1:8080`; both fossil `/direct-api/health` endpoints returned `status: ok`.
- All three per-container deployment locks were released and confirmed absent. The mini's previous image was tagged for rollback. The MacBook's prior live image digest was not available as an image tag after the build, so no new exact-image rollback tag was created; earlier rollback tags remain available.
- No live unlock with a personal age identity was attempted. After the deployed update, retry importing the same user age ID and saving the Radarr URL/key; Radarr and Sonarr do not need separate age IDs for this fix.
