# Rawrarr technical status

Last reviewed: 2026-10-18.

The [roadmap](../ROADMAP.md) is the source of truth for completed and outstanding work. This page records the architectural constraints that apply across current and future features. See the [user guide](user-guide.md) for current usage and the [runbook index](../runbooks/README.md) for dated implementation evidence.

## Data and trust boundaries

- `/media` is a node-local, read-only media mount. Rawrarr, Syncthing, Tailscale, and the relay must not copy or proxy those media bytes as part of app-data operations.
- `/shared` is the Syncthing app-data root. `/metadata` aliases `/shared/metadata`. `config.aged`, artwork, and library sidecars are encrypted before synchronization; logs are compressed and allowlisted, but are not age-encrypted individually.
- `/data` and the live PostgreSQL database are local to each container/node. A Syncthing copy of files is not database replication or quorum.
- The operator's relay/service node may store account credentials, public age recipients, billing/license status, and opaque instance/device IDs. It must never receive private age identities or subscriber app-data plaintext.

## Current-source and runtime status

These descriptions reflect the current checkout, which may include uncommitted implementation changes. They do not establish which revision is running in `rawrarr-local`. Historical runbook tests and deployment results apply to their recorded task only. This documentation review did not rebuild the app or re-run integration tests.

`/health` serves an HTML status page and checks web-process responsiveness only. A successful response does not establish database, integration, or peer readiness. Remembered private age identities remain in node-local `/data/age-identity` with owner-only permissions; this opt-in storage is separate from shared ciphertext and from passkey authentication.

## Verification inventory

Automated checks are indexed in [scripts/tests/README.md](../scripts/tests/README.md). Recent documented checks include Rust unit tests, a Node DOM test for age setup behavior, disposable PostgreSQL API tests, and isolated two-node Syncthing tests with OrbStack or local Syncthing processes. Not every test runs on every change. Check each test's prerequisites and its runbook before treating historical results as proof of current runtime health.
