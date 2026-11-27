# Fix path traversal in config repair and metadata sizing

Issue: [#493](https://github.com/willjasen/rawrarr/issues/493)

The security scan identified config-repair writes in `config_repair.rs` and recursive metadata sizing in `main.rs`. Config repair and Valkey restoration now require the exact `config.aged` path under the existing canonical shared-data directory. They reject a non-directory or symlink at that directory and never create missing parents during repair. The storage-size walk does not follow symlinks and rejects nesting deeper than 64 directories to bound recursion. Existing `/shared/config.aged` and metadata subtree layouts remain unchanged; no data migration is needed.

## Verification

- `cargo test --locked --manifest-path rust-arr/Cargo.toml config_repair` — 12 passed.
- `cargo test --locked --manifest-path rust-arr/Cargo.toml metadata_storage_tests` — 7 passed.
- `git diff --check` — passed.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` — Node.js passed (156 tests); Playwright reported two existing cross-tab timeouts (`test_rawrarr_age_id_cross_tab_browser.mjs` desktop/mobile, 2.5 second wait); initial Rust compile identified environment mutation in newly added tests, corrected in this task. Focused Rust filters passed afterward. Python integration tier is skipped by the documented runner.

No deployment or live environment change was performed. Existing deployments adopt the fix with the next Rawrarr application image update; config formats and persisted data are unchanged.
