# Add decrypted configuration and age verification page

## Objective and initial state

Show the currently readable configuration intended for Rawrarr peers and report whether its stored form is age-encrypted. Initially, Radarr settings were only available through a settings form and age status showed only whether the identity was unlocked and a file existed. Related references: [encrypted config](2026-10-17-rawrarr-age-encrypted-config.md), [age identity onboarding](2026-10-17-rawrarr-age-identity-onboarding.md), and [product direction](../docs/product-direction.md).

## Actions

- Added a read-only `/api/configuration` view and Configuration page with a readable JSON preview. API-key material is redacted in the browser.
- Reports ciphertext existence, recognized age format, identity-unlocked state, and successful decryption/config parsing. It distinguishes locked, verified, unencrypted runtime settings, unrecognized ciphertext, and verification failure.
- Labels peer synchronization as not implemented; the current configuration file is local to this Rawrarr instance.

## Verification

- Rust 1.85 formatting, locked `cargo check`, locked release build, and `git diff --check` passed.
- An isolated local flow generated a PQ age identity, saved a test Radarr config, and verified the API reported recognized ciphertext and successful decryption while redacting the test API key.
- Restarting the isolated instance without its identity correctly reported a locked config, recognized the age format, and withheld decrypted configuration.
- The current `127.0.0.1:8082` page was opened and the Configuration view displays the JSON preview area and encryption verification fields. Its status is `not_configured` because this development instance has no saved config.
- `127.0.0.1:8080`, which holds the existing live Radarr connection, was not changed.

## Final state and remaining work

The Configuration page and read-only `/api/configuration` endpoint are available in the updated local build. The current payload masks API-key contents and identifies whether configuration is age encrypted and successfully decryptable. Peer-to-peer configuration synchronization is not implemented yet.
