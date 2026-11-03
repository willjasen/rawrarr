# Complete age settings unlock error classification

Issue [#164](https://github.com/willjasen/rawrarr/issues/164). Follow-up to [the original implementation](2026-10-26-age-settings-unlock-errors.md).

## Objective

Distinguish malformed user age IDs from valid IDs that cannot unlock the shared group identity, and provide regression coverage for failures at the settings unlock API.

## Changes

- Added a dedicated `GroupIdentityUnavailable` vault error for missing, mismatched, or unreadable group identity envelopes. Malformed age IDs remain `InvalidIdentity`; inability to decrypt encrypted settings remains `DecryptionFailed`; malformed decrypted settings remain `InvalidConfig`.
- Kept low-level device-envelope failures separate from validation of the decrypted user identity, so a malformed identity reaches the vault validation path and receives the correct user-facing error.
- Added API integration cases for malformed identities, wrong identities, corrupt group envelopes, and corrupt encrypted config. The test can run those cases alone with `python3 scripts/tests/test_rawrarr_role_deployment.py --apply --age-unlock-only --image rawrarr:issue-164-test`.
- Improved disposable enrollment-test diagnostics and aligned its browser helper with the current Fossils navigation.

## Verification

- Current-source image build: `docker build -t rawrarr:issue-164-test -f rust-arr/Dockerfile rust-arr` — passed.
- Focused disposable API integration: `python3 scripts/tests/test_rawrarr_role_deployment.py --apply --age-unlock-only --image rawrarr:issue-164-test` — passed owner setup, signed metadata enrollment, node-scoped session checks, encrypted Radarr config, and all age-unlock error cases.
- Full role integration remains blocked by a pre-existing, unrelated storage-sidecar fixture assertion (`shared storage sidecars are missing or unencrypted`) after the focused age-unlock cases pass.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed: Node.js 153/153, Playwright browser tests passed, Rust 122/122. The runner skipped its broader Python integration phase; the focused API integration above exercised this issue's unlock behavior.
- `ast.parse` of the modified Python tests and `git diff --check` — passed.
- Rust formatting was not independently verified: `rustfmt --check` reports extensive existing formatting differences in the touched Rust modules, so no whole-file formatting was applied.

## Compatibility and deployment

No persisted format, database schema, node protocol, or encrypted configuration format changes. Existing deployments adopt the improved error reporting with the normal application image update; no data migration is required.
