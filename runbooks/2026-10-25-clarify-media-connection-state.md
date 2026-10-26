# Clarify media connection state

## Objective and initial state

Clarify the coordinator Radarr and Sonarr settings when the selected metadata node has no service configured, while preserving the browser-to-node direct-only credential path. The initial UI displayed a route warning in this state, despite the route being healthy; current work starts from `origin/main` on `codex/clarify-media-connection-route`. GitHub issue [#87](https://github.com/willjasen/rawrarr/issues/87) tracks the change. Related implementation history: [Finish Sonarr connection settings](2026-10-18-sonarr-settings-flow.md).

## Changes

- Unconfigured settings now say the named service is not configured on the selected metadata node and explain that credentials go directly from the browser to the node.
- A true missing direct route now reports that this browser cannot reach the selected node, with guidance to use a browser that can.
- Added browser regression assertions for the unconfigured Radarr and Sonarr states. Existing direct-only credential assertions remain in the same test.

## Verification

- `git diff --check` and JavaScript syntax checks passed.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` passed in OrbStack: Node 128/128, Playwright browser 1/1, and Rust 82/82. Python integration checks were skipped by the standard runner because they need disposable service fixtures.
- An initial run exposed that the new unconfigured Radarr assertion ran before age-vault unlock; the assertion was moved after settings load and the full suite was rerun successfully.

## Final state and remaining work

Implementation and regression coverage are complete on `codex/clarify-media-connection-route`. No deployment was requested; UI copy has not been verified against the live internal page. Commit and PR status are recorded in the completion report.
