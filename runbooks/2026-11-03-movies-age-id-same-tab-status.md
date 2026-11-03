# Keep Movies age ID state clear after Settings save

Issue: [#358](https://github.com/willjasen/rawrarr/issues/358)

## Findings and change

The live internal coordinator serves current GitHub `main`, including the earlier cross-tab age-ID synchronization. Settings and Movies share the same in-memory age identity in a tab, but returning to Movies did not refresh the initial “Enter your age private ID” status. This made a saved ID look absent even though the unlock controls already knew about it.

Movies now refreshes the age-ID controls and status on navigation, and resets the initial prompt only when no identity is available. Saving while Movies is already open also starts its normal unlock flow. This affects only browser UI state; no stored data, schema, node protocol, or migration changes are needed.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- `test_rawrarr_age_id_cross_tab_browser.mjs` passed in the isolated OrbStack container (2 tests), including cross-tab import and same-tab Settings-to-Movies navigation/unlock.
- Full `python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack` did not complete: Node had one unrelated missing `/workspace/netlify-ignore-build.sh` fixture (149/150 passed), and the existing signing-topology Playwright test remained active for over four minutes. The isolated test container was stopped; its exit was 137. The focused age-ID browser test passed afterward in that same prepared container.
- No deployment was performed. The fix requires the next internal coordinator deployment from published GitHub `main`.
