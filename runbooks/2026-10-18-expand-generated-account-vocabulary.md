# Expand generated account vocabulary

## Objective and initial state

Expand the generated account-name pools with the eras, materials, habitats, and prehistoric creatures supplied by Brandon. The generator uses two lowercase terms plus a four-character account-ID suffix; names must remain within the 32-character username limit.

Related context: [generate a dinosaur-themed account name](2026-10-18-generate-dinosaur-account-name.md) and [display the generated account name](2026-10-18-display-generated-account-name.md).

## Changes

- Added the supplied Type 1 terms to the first-word pool and Type 2 creatures to the second-word pool.
- Added coverage that each possible word pair stays valid and within the username limit.
- Generated labels still use a four-character account-ID suffix; the longest possible word pair is 29 characters including separators and suffix.

## Verification

- `node --test scripts/tests/test_rawrarr_passkey_ui.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 16 checks.
- The isolated Rust account-name test passed and validated every prefix/creature combination against the username rules.
- The roarlay release image built; its health status is `healthy`. HTTPS health identifies Roarlay, and Tailscale Serve JSON points HTTPS 443 `/` to `http://127.0.0.1:8080`.
- `git diff --check` passed. The local browser connection closed during a post-deploy refresh, so the newly generated name was not re-read in the browser.

## Final state

The supplied terms are active in the generator. Additional candidates to consider, not yet added: Type 1 — `mesozoic`, `paleolithic`, `icebound`, `shale`, `obsidian`, `craggy`, `marshy`, `boggy`, `tundra`, `badlands`; Type 2 — `mosasaur`, `ichthyosaur`, `pterodactyl`, `oviraptor`, `hadrosaur`, `therizinosaur`, `parasaurolophus`, `ceratopsian`, `glyptodon`, `gorgonops`. Each suggestion fits the existing 32-character username limit with the current longest prefix and suffix.
