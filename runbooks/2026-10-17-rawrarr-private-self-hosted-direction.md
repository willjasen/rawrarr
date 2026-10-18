# Rawrarr private self-hosted product direction

## Objective and initial state

Record the user's direction for how Rawrarr should be distributed and how a private owner may share with another person. Existing product direction covers unified media functions but does not define private ownership, subscription intent, or invitee trust; see [unified product direction](2026-10-17-rawrarr-unified-product-direction.md) and [Syncthing peer sync](2026-10-17-rawrarr-syncthing-peer-sync.md).

## Direction recorded

- Rawrarr should be deployable by users from a packaged Docker image. The code may stay closed source initially, with open sourcing considered later.
- The initial workspace is private to one owner. Subscription is a business direction to explore; no pricing, tiers, or hosted control plane have been selected.
- The owner may invite another person by pairing a device through Syncthing. Invitees begin untrusted; the owner selects any folders or data sets to share. Pairing alone grants neither trust nor decryption access.
- Private identities, credentials, and unshared configuration remain outside share grants. Age encryption recipients determine who can decrypt ciphertext.
- This is future product behavior. Current code configures Syncthing peers/folders but does not implement person-level invitations, trust policy, or selective share grants.

## Verification and remaining work

Updated `docs/product-direction.md` to distinguish the user's intended direction from current capabilities. `git diff --check` passed, and relative links in the changed product and runbook documents resolve. Define subscription value, packaging/release controls, invite lifecycle, and selective encrypted-sharing rules before implementing those features.
