# Fix pending node rejection

Issue: [#264](https://github.com/willjasen/rawrarr/issues/264)

The coordinator rejection route deserialized its request as the full signed approval payload. The workspace sends only `requestId`, so Axum returned HTTP 422 before the handler ran. Rejection now has its own request type with only the request ID; the existing owner-scoped deletion still removes only unapproved enrollments.

This change affects no persistent data or protocol compatibility. Existing deployments adopt it by updating the coordinator application; no migration is required.

Verification: `node --test scripts/tests/test_rawrarr_node_approval_ui.mjs` passed (8 tests); `git diff --check` passed. Rust compilation was not verified because `cargo` is unavailable in this shell.
