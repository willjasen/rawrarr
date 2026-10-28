# Make Subscription discoverable in workspace navigation

Issue: [#290](https://github.com/willjasen/rawrarr/issues/290)

## Request

After the ETH and Stripe controls moved to `/billing`, the Settings page linked to it as “Manage subscription”, but the coordinator sidebar had no Subscription entry. The page existed but was not discoverable from the main navigation shown in the report.

## Change

- Added a Subscription link to `/billing` in the main workspace sidebar, near Overview, with an explicit accessible name for compact mobile navigation.
- Kept Settings' “Manage subscription” link as a second entry point.
- No storage, database, encrypted configuration, protocol, or existing-state changes; no migration is needed.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (34 tests).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed: 144 Node.js tests, all Playwright tests including Subscription navigation and `/billing` interaction at 1280×800 and 375×812, and 113 Rust tests. Python integration fixtures were skipped by the runner as documented.
- `git diff --check` — passed.

## Deployment

- Published UI release: `ui-37aaf1124cc600fa8a6c337c2074e52702faf195`.
- Both do3 coordinator services run the UI bundle updater in their application image. Their Compose setup must mount a fine-grained GitHub token with **Contents: read** access at `/run/secrets/rawrarr_ui_github_token` and provide a persistent `/ui` volume. The internal coordinator already had the volume; the public coordinator receives one through `deploy/do3/docker-compose.public-ui-updater.yml`.
- On do3, the configured token source is `/root/gh-token`. It is a host-local secret and must not be copied into the repository or emitted in logs. The coordinator containers retrieve and checksum-verify the private UI bundle from GitHub; no app image rebuild is needed for this UI-only release.
- Deploy by updating only the internal and public `roarlay` coordinator services, preserving PostgreSQL, Tailscale, Caddy and all subscriber services. Verify `/ui/current/bundle.sha256` and `/ui/current/bundle.version` in both services and check `/billing` plus the Subscription sidebar link on both coordinator URLs.
- Deployment status: pending.

## Rollback

- Revert the navigation link and its regression test. The existing `/billing` route and Settings link remain functional.
