# Remove the roarlay label from coordinator page headers

## Objective

Remove the “roarlay” text displayed in the top-right header across coordinator workspace pages. Issue: [#89](https://github.com/willjasen/rawrarr/issues/89).

## Initial state

- Started from fetched `origin/main` at `0888445` in a dedicated worktree and branch `codex/remove-roarlay-header-label`.
- The coordinator workspace template had a small “roarlay” brand element beside Sign out; the existing workspace regression test asserted its presence.

## Actions

- Removed the visible wordmark while retaining the header element and Sign out control.
- Updated coordinator workspace regression coverage to require no visible header label and retain Sign out.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed all 26 tests.
- `node --check scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed.
- `git diff --check` passed.
- Live deployment was not requested or performed.

## Final state

- The coordinator header no longer displays the roarlay label; Sign out remains present.
- Changes are committed locally on `codex/remove-roarlay-header-label`; no push or deployment was requested.
