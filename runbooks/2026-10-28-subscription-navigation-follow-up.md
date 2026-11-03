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
- On 2026-10-28, PR #296 added the token mount to the internal coordinator Compose file and added the public coordinator updater overlay. The same GitHub `main` source archive for `62256ac78eda05332966d55444942364854108ea` was checksum-verified on do3 before installing those files.
- After Will updated `/root/gh-token` with a repository-authorized token, both coordinator containers were recreated with `--no-deps --pull never` using their existing `c001b779a31da9ecbee0f3cdae1935a2bb94773b` image. Both `/ui/current/bundle.version` files report `62256ac78eda05332966d55444942364854108ea`, and both `coordinator_workspace.html` files contain the Subscription navigation link.
- Verification passed: internal roarlay healthy; internal Tailscale Serve still proxies `rawrarr.risk-mermaid.ts.net:443` to `http://127.0.0.1:8080`; internal HTTPS `/health` succeeded; public roarlay healthy; public HTTPS `/health` returned HTTP 200; public memory use was 15.6 MiB of its 96 MiB limit. PostgreSQL, Tailscale, Caddy, Stripe and subscriber services were not recreated.
- Deployment status: complete on 2026-10-28. No migration was required; existing data, protocols, and the compiled application image were unchanged.

## Rollback

- Revert the navigation link and its regression test. The existing `/billing` route and Settings link remain functional.
