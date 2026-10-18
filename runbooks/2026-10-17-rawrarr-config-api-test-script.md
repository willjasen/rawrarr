# Add a Rawrarr configuration integration test

## Objective and initial state

Add a repeatable test script for the configuration and age-encryption API. Existing script tests are indexed in [the test README](../scripts/tests/README.md); the Configuration page behavior is documented in [its runbook](2026-10-17-rawrarr-configuration-page.md).

## Actions

- Added `scripts/tests/test_rawrarr_configuration.py`. It launches the selected Rawrarr binary on a free loopback port with a disposable `RAWARR_DATA_DIR`, generates a temporary post-quantum age identity, stores a dummy Radarr URL and API key, then checks ciphertext recognition, successful decryption, and API-key redaction.
- The script restarts the same temporary instance without an identity and confirms the encrypted configuration is reported as locked and withheld. The temporary identity, ciphertext, and data directory are removed when the run ends. Server output is discarded and failure messages do not include request contents.
- Indexed the script in [scripts README](../scripts/README.md) and [test README](../scripts/tests/README.md).

## Verification

- `python3 scripts/tests/test_rawrarr_configuration.py` passed against the existing release binary. It verified recognized age ciphertext, successful decrypt, API-key redaction, restart-to-locked behavior, and temporary-data cleanup.
- `sh scripts/tests/test_run_media_sidecar_throttled.sh` passed; `python3 -m py_compile scripts/tests/test_rawrarr_configuration.py` and `git diff --check` passed.

## Final state and remaining work

- The test script is available at `scripts/tests/test_rawrarr_configuration.py`. It requires Python 3, the `age` and `age-keygen` commands, and a built release binary. Docker behavior remains outside this test's scope.
