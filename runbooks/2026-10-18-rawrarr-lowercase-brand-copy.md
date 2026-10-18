# Display rawrarr in lowercase

## Objective

Ensure user-facing Rawrarr interface text, runtime errors, and container logs use the lowercase product name `rawrarr`.

## Initial state

The setup page, account placeholder, dashboard, device ID errors, and shared-log/server messages contained `Rawrarr`. Technical identifiers such as `RAWARR_*`, Rust module documentation, and project documentation titles are not displayed by the app and remain unchanged.

## Changes

- Lowercased the product name in HTML page copy, the default Syncthing folder label, device ID errors, and runtime log messages.
- Recorded the display rule in `docs/DESIGN.md`.
- Added a Node regression test for HTML pages and quoted runtime messages, and updated the age setup test's expected copy.

## Verification

- `node --test scripts/tests/test_rawrarr_age_setup.mjs scripts/tests/test_rawrarr_brand_case.mjs` passed (2 tests).
- `git diff --check` passed.
- Docker Compose config validation and `docker compose ... up -d --build rawrarr` succeeded; the service container is running.
- `http://localhost:8084/health` returned `ok`.
- The live `/auth` page displayed the lowercase placeholder `How rawrarr should address you`. `/setup/age` redirected to an authenticated route and returned 401 without a browser session, so the live age setup page was not inspected.
- The container's `/shared` mount remains `/Users/willjasen/Library/Application Support/Rawrarr/shared-data`.

## Final state

User-facing app copy and runtime messages use lowercase `rawrarr`. Technical identifiers and non-user-facing project prose retain their existing casing. The focused automated checks and local service health check passed.
