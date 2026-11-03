# Fix failing Playwright browser tests

## Goal

Restore the Playwright suite after the 2026-10-26 full local run reported failures in movie library rendering, links, navigation, and Sonarr setup.

## Tracker

- GitHub issue: [#194](https://github.com/willjasen/rawrarr/issues/194)
- Task branch: `codex/fix-playwright-failures`

## Findings and changes

- Several browser fixtures omitted the current `/api/settings/{radarr,sonarr}` `configured` flag or returned an error for settings. The UI correctly treated those services as unconfigured and hid their mocked library rows. Fixtures now explicitly configure the service when testing its library behavior.
- The Sonarr settings test waited for a connection prompt inside the library container, which is hidden while Sonarr is unconfigured. It now checks the visible settings panel and confirms the library remains hidden.
- The configured Sonarr form is hidden after saving and can be reopened from the Connection settings action on the library panel; the browser test covers the save/clear flow through that control.

## Verification

- Passed: `python3 scripts/tests/run_rawrarr_tests_in_docker.py` — Node 112/112, all Playwright browser tests passed, Rust 97/97.
- Python integration tests were skipped by the runner because they require separate disposable PostgreSQL, age, or Syncthing fixtures.
