# Keep profile avatar traits stable across refreshes

## Objective and initial state

Track [issue #83](https://github.com/willjasen/rawrarr/issues/83). The profile avatar normally derives from the saved age ID, but its fallback hashed the generated display name. That fallback could change when profile naming changed, producing different colors and wing traits after a refresh.

## Actions

- Return the authenticated account UUID alongside the generated display name from the workspace profile endpoint.
- Use the account UUID as the fallback avatar seed input, preserving the age-ID-derived seed when local derivation succeeds.
- Add regression assertions for the stable fallback seed source.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs scripts/tests/test_rawrarr_profile_color.mjs` passed (28/28).
- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.

## Final state and remaining work

The signed-in profile fallback hashes the authenticated account ID, so the generated fallback name and avatar traits remain stable across refreshes. The local age-ID-derived seed remains the preferred path. No deployment was requested or performed.
