# Generate a dinosaur-themed account name

## Objective and initial state

Remove user-entered naming from first passkey setup and generate a dinosaur-themed account label. The existing sign-in page already omitted username entry, but first setup still requested a display name and the coordinator derived its internal username from that value. The workspace contained extensive unrelated edits before this task; those changes remain untouched.

Related context: [identity creation wizard](2026-10-18-rawrarr-identity-creation-wizard.md).

## Changes

- First setup now posts an empty registration-start payload and has no name field.
- The coordinator generates a distinct lowercase account name from dinosaur-era words, a dinosaur or prehistoric creature, and four hex digits derived from the new account ID. It uses the generated name for both username and display name.
- Updated the browser and passkey integration expectations and added Rust coverage for the generated format.
- Rebuilt roarlay in `rawrarr-local`. The default Compose file attempted to bind port 7298 directly and collided with the running Tailscale sidecar. Re-running with the three Compose files recorded in the existing deployment labels (`docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml`) recreated the intended service without changing the sidecar or metadata node.

## Verification

- `node --test scripts/tests/test_rawrarr_passkey_ui.mjs` passed all three checks.
- The isolated Rust suite compiled the updated code and passed the new account-name test; 46 of 48 Rust tests passed. Two unrelated avatar tests failed because the test container lacks the SVG rasterizer and JPEG XL tools.
- The isolated Node suite passed the updated passkey UI check. The broader run had 13 pre-existing failures among 45 tests, including missing browser globals, mismatched workspace fixtures, and a missing Compose file; unrelated failures are retained in the test log container `rawrarr-testing-tests-mve1865a`.
- Host `cargo fmt --check` and `git diff --check` passed. Host Rust compilation could not start because its LLVM installation requires a missing Z3 library.
- Rebuilt roarlay with the existing project configuration; its health status is `healthy`, and local plus HTTPS health routes return the Roarlay status page. `tailscale serve status --json` confirms HTTPS 443 `/` proxies to `http://127.0.0.1:8080`. The per-container deployment marker was removed after verification.

## Final state

First-passkey enrollment generates a unique dinosaur-themed account name without asking the user for a username or display name. Sign-in behavior is unchanged. The updated service is deployed and verified on roarlay.
