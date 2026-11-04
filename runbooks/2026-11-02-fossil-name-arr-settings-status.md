# Show Fossil names in ARR settings status

Issue: https://github.com/willjasen/rawrarr/issues/323

## Change
Radarr and Sonarr status copy in the coordinator workspace now uses the selected node's user-managed name from the age-encrypted device-name map. If unavailable, it uses the enrollment display name, then a role label. This changes presentation only; no stored data, protocol, or migration is involved.

## Verification
- `git diff --check` passed.
- Focused Playwright test was attempted with `node --test scripts/tests/test_rawrarr_media_management_playwright.mjs` but could not start because `/usr/local/lib/node_modules/playwright-core/index.mjs` is missing in this environment. The configured and unconfigured cases are covered in the test source but remain unexecuted.
