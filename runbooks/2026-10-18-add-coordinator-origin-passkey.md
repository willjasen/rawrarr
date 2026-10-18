# Add a coordinator-origin passkey to the existing account

## Objective and initial state

Allow an owner signed in on a metadata node to register a passkey for the coordinator's HTTPS origin and attach it to the same PostgreSQL account. WebAuthn credentials are scoped to their relying-party origin; the existing `localhost` credential cannot authenticate directly at `rawrarr.risk-mermaid.ts.net`. Existing local services were healthy: metadata on `127.0.0.1:7297`, coordinator on `127.0.0.1:7298`, and PostgreSQL in Compose project `rawrarr-local`.

## Actions

- Added a short-lived, single-use coordinator link bound to the authenticated metadata session's user. The token is carried in the URL fragment, stored only as a SHA-256 digest, expires after five minutes, and is removed from browser history before the ceremony.
- Added the WebAuthn registration page and endpoints on the coordinator HTTPS origin. Completion inserts a credential for the linked `user_id`; it does not create or replace a session and does not remove the original credential.
- Exposed **Add passkey for coordinator** on the metadata dashboard and age setup page. The latter is the initial destination after sign-in when the local age vault has not yet been unlocked.
- Added migration `0005_coordinator_origin_passkey.sql` and coordinator relying-party configuration for `rawrarr.risk-mermaid.ts.net`.

## Verification

- Rust Docker image built successfully.
- Focused Node tests passed (9 tests), Python syntax compilation passed, and `git diff --check` passed.
- Disposable role integration passed. A headless Chromium virtual authenticator registered a metadata credential, signed in, followed the coordinator-origin handoff, and registered the second credential. PostgreSQL confirmed two credentials with one distinct `user_id`.
- Before rollout, created `/tmp/rawrarr-before-coordinator-passkey-20261018.dump` as a mode-0600 custom-format PostgreSQL backup.
- Built the `rawrarr:local` image, recreated the coordinator first (applying migration 0005), then recreated metadata in Compose project `rawrarr-local`.
- Migration verification showed versions 5 and 4 succeeded. Metadata, coordinator, and PostgreSQL containers are healthy. Local metadata and coordinator health pages and the Tailscale-served coordinator health page respond; `https://rawrarr.risk-mermaid.ts.net/passkeys/add` serves the passkey registration page.
- Live database remains at one user, one existing passkey, one credential owner. The owner still needs to select **Add passkey for coordinator** from the signed-in metadata page and complete the browser prompt; successful end-to-end credential creation was verified in the disposable integration environment.

## Recovery and remaining work

Migration 0005 is additive. If rollout fails, restore the previous coordinator and metadata image; the new nullable column/table can remain unused. The user must complete the browser passkey prompt within five minutes. This adds a coordinator-origin credential but does not itself implement the coordinator's future workspace UI or change the metadata node's local origin.

## Final state

Deployed to `rawrarr-local`; runtime health and registration page checks passed. Existing passkey remains in place. No user-issued coordinator-origin passkey has been registered yet.
