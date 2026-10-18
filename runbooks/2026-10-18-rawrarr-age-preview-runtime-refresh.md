# Refresh the running age setup page

## Objective and initial state

The user reported that the age setup screen still showed the complete private identity despite the source change for a shortened preview. The worktree was clean at commit `53f3224`; `rust-arr/src/age_setup.html` already contained the shortened preview implementation and the Node DOM test asserted the full generated fixture was absent from page elements. The live OrbStack app at port 8084 served the older copy-only page, confirming the running container did not contain the current source.

## Actions

- Rebuilt and recreated only the `rawrarr` service in Compose project `rawrarr-local`, using OrbStack and `--no-deps` to leave PostgreSQL running.
- Kept the full private identity absent from visible page content; it is used only in page memory for explicit clipboard copy or the unlock request.

## Verification

- The first rebuild attempt lacked Compose overrides and would have used port 8080; no app data was changed. Recreated the app with the existing protected password-file path and `RAWARR_HOST_PORT=8084`, leaving PostgreSQL running.
- Reloaded the user's existing Chrome tab at `http://localhost:8084/setup/age`; the live page now says a shortened preview is shown and the complete private identity is never displayed. The old copy-only page text is gone.
- `node --test scripts/tests/test_rawrarr_age_setup.mjs` passed (1 test). `http://127.0.0.1:8084/health` returned `ok`; the app is running on port 8084 and PostgreSQL remains healthy.
- Did not generate or inspect a real private identity during verification.
