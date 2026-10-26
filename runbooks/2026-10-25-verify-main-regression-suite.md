# Verify local main regression suite

## Objective and initial state

Prepare an isolated branch from the clean local `main` revision so its regression suite passes, without changing the main checkout or the two pre-existing dirty task worktrees. Local `main` and the candidate branch base were both `9cf3d8822ee48369aea3210ea8f4ee86317d997b`. The candidate is `codex/local-regression-review-current-main` in `/Users/willjasen/.codex/worktrees/local-regression-fixes/rawrarr`. No changes have been committed or pushed.

The suite exposed stale test fixtures and expectations, a test runner that omitted required repository fixtures, and a real browser export bug. GitHub issue [#63](https://github.com/willjasen/rawrarr/issues/63) tracks the export fix.

## Changes made

- Updated the OrbStack runner's source copy to include the root Compose files, `admin/`, `docs/`, and storage-proxy Caddy config so all test modules can run against the intended workspace fixtures.
- Updated stale browser/static assertions to current `main` behavior, including dynamic age assets, artwork cache locks, direct routes, profile copy, title/year labels, device pictures, enrollment signing fields, and Sonarr progress wording. Retained checks for node signing, signing approval, avatar routes, signed UI updates, and updater polling.
- Restored coordinator browser-test fixtures for the imported node-avatar, profile-color, and AI modules.
- Fixed a name collision in `rust-arr/web-export/metadata-export.js`: the blob download URL no longer shadows the endpoint URL helper. Rebuilt the embedded `rust-arr/src/assets/metadata-export.js` bundle.
- Updated the mini-export, landing-page, age-ID, and movie snapshot assertions to match current UI output while preserving their behavior checks.

## Verification

Ran focused tests for the corrected browser modules in the OrbStack test image: 10 passed, 0 failed. Ran `git diff --check`: passed.

Final full regression command (using the required `rawrarr-test-regression` project name):

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack
```

Result: Node 122/122 passed; Playwright browser 1/1 passed; Rust 82/82 passed. Python integration checks were skipped by the runner because they require disposable PostgreSQL, age, or Syncthing fixtures. The separate OrbStack Syncthing peer test was not part of this request and was not run.

## Final state

The regression changes were committed as `a599749` and merged to GitHub `main` through [PR #64](https://github.com/willjasen/rawrarr/pull/64), producing squash commit `6ffed1623ad46fad739bf87944032293118b5b8e`. The required GitHub check passed; the private UI bundle publishing job was skipped. Issue [#63](https://github.com/willjasen/rawrarr/issues/63) is closed. The passing local suite ran against the exact source changes in that merge. No deployment was performed.

The shared main checkout and the two pre-existing dirty worktrees were left untouched during regression work. The task branch was pushed and merged; there are no remaining uncommitted regression changes in its original worktree.
