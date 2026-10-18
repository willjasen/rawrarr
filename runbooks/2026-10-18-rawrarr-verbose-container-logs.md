# Make Rawrarr container logs verbose

## Objective and initial state

Show routine HTTP activity in the Docker container's stdout logs by default and provide an environment option to suppress routine request lines later. The app previously printed startup messages and wrote a small set of structured compressed events under `/shared/logs`; it did not report individual HTTP requests to container logs.

## Changes

- Add request logging for method, path (without query string), response status, and elapsed time. Never log headers, query parameters, or request/response bodies, which may contain credentials or private identities.
- Keep server-error request summaries visible when quiet mode is enabled.
- Add `RAWARR_QUIET_LOGS=true` to suppress successful/client-error request lines; default to verbose (`false`).
- Print a numeric-only library scan summary to container logs.

## Verification

The focused Rust quiet-option unit test passed, `cargo fmt --check` passed, and the complete Rust test suite passed (9/9) in a temporary Rust 1.85 container with the age tools bundled in the built app image. The production Docker image built successfully. `/health` returned `ok`, and the container logged `request GET /health -> 200`; a query-string sentinel was absent from logs. Compose defaults to verbose request logging; `RAWARR_QUIET_LOGS=true` is covered by unit tests. No request bodies or headers are logged.
