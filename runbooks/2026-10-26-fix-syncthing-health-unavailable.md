# Fix unavailable Syncthing health on online device cards

## Objective
Restore shared-folder Syncthing health in the coordinator Devices cards for an online configured subscriber. Issue [#135](https://github.com/willjasen/rawrarr/issues/135).

## Initial state
The task branch is based on `origin/main` commit `271ed38`. The screenshot shows the `fossil` MacBook Pro device online with a direct latency reading but its card says “Syncthing health unavailable.” Existing code fetches `/nodes/{node_id}/api/syncthing/shared-health` through the coordinator node relay; node API failures are collapsed into `configured: true, folders: null`.

## Investigation and changes
The shared-health endpoint already treated local folder completion as optional, but the same per-peer Syncthing completion lookup was easy to turn into a report-wide failure as the response path evolved. That failure causes the endpoint to return `folders: null`, which the coordinator renders as the screenshot's generic unavailable label. Updated peer completion to stay best-effort and added a mocked Syncthing API regression asserting folder state, local completion, and peer connectivity remain present when a peer completion query returns HTTP 404.

## Verification
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack --remove-after` passed against this worktree: Node 110/110, all serial Playwright browser tests, and Rust 85/85 (including the new Syncthing regression). Temporary test container was removed.
- `python3 scripts/tests/test_rawrarr_syncthing_orbstack.py` passed with Syncthing 2.1.5: isolated web/storage peers connected, shared data and encrypted config synced, and all temporary containers, network, and fixture data were removed.
- Host Rust compiler/formatter could not run because the installed LLVM library requires a missing `libz3` dylib; container compilation and tests passed.
- Internal owner-authenticated browser relay was not exercised, and no production service was deployed.

## Final state
The code and regression coverage are complete on the task branch. Internal deployment was not performed; deployment remains separate from this merge request.
