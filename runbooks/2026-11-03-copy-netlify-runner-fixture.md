# Copy Netlify build fixture into isolated tests

Issue: [#360](https://github.com/willjasen/rawrarr/issues/360)

## Objective and change

Source-structure test `test_netlify_ignore_build.mjs` reads the root-level `netlify-ignore-build.sh` file. The isolated OrbStack runner copied other root fixtures but omitted this script, causing that test to fail with `ENOENT`. The runner now includes the script in its root fixture manifest and has focused Python regression coverage for copying it into `/workspace/`.

This only changes test setup. No production runtime, persistent data, or migration path is affected.

## Verification

- `python3 scripts/tests/test_rawrarr_test_runner_fixtures.py` — passed.
- `python3 -m py_compile scripts/tests/run_rawrarr_tests_in_docker.py scripts/tests/test_rawrarr_test_runner_fixtures.py` — passed.
- `node --experimental-global-webcrypto --test scripts/tests/test_netlify_ignore_build.mjs` — passed.
- Disposable OrbStack copy smoke check confirmed `/workspace/netlify-ignore-build.sh` and `/workspace/netlify.toml` are present.
- Full isolated OrbStack suite — not completed; a previous run stalled during Playwright and was stopped.
