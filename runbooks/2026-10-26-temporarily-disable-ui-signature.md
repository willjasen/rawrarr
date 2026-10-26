# Temporarily disable UI signature enforcement

## Objective and tracking

Temporarily allow the Rawrarr coordinator to install checksum-verified UI releases without requiring publisher signatures, and remove the “UI signature unverified” badge that is confusing access troubleshooting. Tracked by issue [#107](https://github.com/willjasen/rawrarr/issues/107). Device signing and metadata signing are outside this change.

## Initial state

The Mac mini coordinator was healthy and its Tailscale Serve route mapped HTTPS 443 to `http://127.0.0.1:8080`. Its current UI bundle (`f7925d26652d683dc784c90ded2cadb3f6885a17`) had `bundle.signature-status=verified` and `/ui/update-status.json` reported `signed`. Therefore the warning seen by the owner did not match the active coordinator's recorded bundle state. The signature status was a workspace badge and release-update integrity feature, not an authentication gate.

The signed updater required `ui-bundle.zip.sig` and an image-pinned public key before extraction. SHA-256 checks, archive path/size limits, staged extraction, and atomic current/previous swaps already existed. GitHub Actions also required the signing secret and published detached signatures.

## Changes

- Remove publisher-signature verification from the updater and remove the signing-secret step and signature asset from GitHub Actions releases. Keep the pinned public key and signing secret available for a future restoration.
- Continue enforcing the GitHub release checksum, archive path/count/size validation, staged extraction, and atomic installation/rollback.
- Remove the workspace signature badge and its now-unused authenticated status endpoint.
- Keep node enrollment/device signatures and signed metadata records unchanged.

## Verification results

- `python3 scripts/tests/test_ui_bundle.py -v`: passed 5/5, including unsigned bundle install, signature-asset ignore, checksum/path rejection, and preserving current UI on invalid input.
- `node --test scripts/tests/test_rawrarr_ui_signature_disabled.mjs`: passed 5/5; the checks cover the obsolete badge/endpoint, unsigned release path, absence of release signing requirements, and unchanged device-signing.
- `python3 scripts/tests/test_internal_deploy_targets.py -v`: passed 6/6; the UI updater path is correctly classified as coordinator-only.
- `node --check` passed for the edited coordinator workspace module and topology browser test source.
- The topology browser test could not execute because this worktree's host Node environment lacks `playwright-core`.
- `cargo check --locked --offline` could not start: host `rustc` aborts because the configured LLVM dylib requires missing `libz3.5.1.dylib`.
- Before deployment, read-only checks returned HTTP 200 for `/`, `/health`, and `/api/auth/status`; `/api/auth/session` returned 401 without a browser session. The previous bundle marker reported version `f7925d26652d683dc784c90ded2cadb3f6885a17`, `verified`, and `signed`. This confirmed service reachability but did not reproduce the owner's authenticated browser symptom.

## Internal deployment verification

- PR #108 was merged at `f21e1b9`. Published `main` revision `5fedbadcb829869d37db128680f0b229c65137c1` passed GitHub Actions run [40268258968](https://github.com/willjasen/rawrarr/actions/runs/40268258968), which published the unsigned bundle and its SHA-256 checksum.
- Rebuilt and recreated only `roarlay` in Compose project `rawrarr-internal-mini` from that exact published revision. The active UI volume advanced to `5fedbadcb829869d37db128680f0b229c65137c1`; the installed checksum matched the release asset, and the signature marker and workspace badge are absent.
- Roarlay is healthy with zero restarts. PostgreSQL and Tailscale container IDs were unchanged. Tailscale Serve still maps HTTPS 443 `/` to `http://127.0.0.1:8080`; `/`, `/health`, and `/api/auth/status` return HTTP 200, and health identifies Roarlay.
- The per-container deployment lock was released and confirmed absent. The MacBook fossil was not changed because this is coordinator-only runtime behavior.
- `/api/auth/session` returned 401 without a browser session. The deployment therefore verifies the signature warning removal and coordinator availability, but does not verify that the owner's passkey login/access issue is resolved.

The internal rollout is complete. For future changes, continue to deploy only from an exact published `main` revision and preserve the existing coordinator Tailscale Serve route.

## Security tradeoff and restoration

While disabled, SHA-256 detects transfer corruption but does not authenticate the UI publisher. The GitHub access token remains required and private. Restore signature enforcement later by reinstating the pinned-key verification and signed status endpoint/badge, with tests proving missing, invalid, and altered signatures are rejected before atomic activation. This change must not be represented as restoring access until the actual login/access symptom is separately verified.

## Status

Implementation is merged and deployed to internal roarlay. The reported authenticated browser symptom still needs an owner-side login check.
