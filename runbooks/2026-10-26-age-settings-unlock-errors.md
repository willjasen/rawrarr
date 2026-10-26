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
- The live internal host could not be reached from this session: its MagicDNS hostname did not resolve. No service was changed or deployed.

## Verification

- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed: Node 112/112, Playwright 14/14, Rust 96/96. The Python integration phase is intentionally skipped by this runner; its disposable PostgreSQL and approved-node environment was not started.
- `python3` AST parse of `scripts/tests/test_rawrarr_configuration.py` — passed.
- `git diff --check` — passed.
- Host `cargo` and `rustfmt` are unavailable, so formatting and the direct host Rust command were not run. The isolated Rust build and test suite compiled and passed.
- Follow-up focused Rust regression coverage was added for both the successful legacy-config migration and rejection without key-file creation. It has not yet been run.
