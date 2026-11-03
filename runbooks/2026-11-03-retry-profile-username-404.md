# Retry generated username lookup failures

Issue: [#362](https://github.com/willjasen/rawrarr/issues/362)

## Objective and initial state

The coordinator workspace loads the generated account name with `GET /api/workspace/profile`. A 404 on the initial request was rendered as the username and remained until refresh. Work is isolated in `codex/settings-generated-username-404-fix`, based on fetched `origin/main` at `e14b8ba`.

## Changes

- Retry profile HTTP 404 responses up to three total attempts with 250 ms and 500 ms delays.
- Preserve the loaded username if a later profile detail or avatar request fails.
- Add regression coverage for bounded retry handling and error preservation.
- No persistent data, database, or protocol changes; no migration is required.

## Verification

The focused bounded-retry regression passed in the isolated OrbStack test container (1 passed, 34 unrelated cases skipped by name filter). The full isolated runner completed: Node.js passed 151/151 and Rust passed 116/116. Playwright had three unrelated failures: Radarr settings timed out waiting for its node note, the Docker healthcheck test expected an older runner source pattern, and signing-key topology failed its desktop title visibility assertion. The new profile retry regression passed in the Node.js suite.

## Final state

The workspace retries the profile endpoint after transient 404 responses and retains a successfully loaded generated name across later detail errors. Changes were committed on the task branch and submitted in PR #364; no migration or deployment was performed.
