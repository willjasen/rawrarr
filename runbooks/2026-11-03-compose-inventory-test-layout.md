# Compose inventory and isolated test layout

Issue: [#154](https://github.com/willjasen/rawrarr/issues/154).

## Scope and baseline

First increment only: inventory the 22 root Compose files and their repository
callers, then relocate the isolated test environment. Started in the clean
task-owned `willjasen-docker-compose-cleanup` worktree, fast-forwarded to freshly
fetched GitHub main `7be69d7`. No shared-checkout changes or remote deployment.

The [inventory](../docs/compose-layout.md) separates base stacks, overlays,
companions, historical Mac coordinator definitions, generated subscriber output,
and do3 deployment definitions. It records exact-path callers, the regression
fixture glob, and historical evidence. No root definitions are declared unused
solely because a script reference is missing. Remaining consolidation is planned,
not implemented.

## Changes and invariants

- Moved the test definition to `deploy/testing/compose.yml`, its Caddy config to
  `deploy/testing/arr.Caddyfile`, and Headscale config to
  `deploy/testing/headscale/config.yaml`. Reduced root Compose files from 22 to 21.
- Preserved all services, runtime settings, named volumes, networks, images,
  secrets, and port bindings. Added the explicit default project
  `rawrarr-test-environment`; the integration runner still passes it with `-p`.
  Rebased its build context to `../../rust-arr`.
- Updated executable callers and the historical runbook's links. The integration
  runner now resolves the Compose path relative to its script rather than the
  invoking directory. Existing regression snapshots already copy `deploy/`;
  added a guard for that fixture-copy contract.
- Excluded the deleted old test filename from the internal target classifier
  and coordinator updater. A deletion reported by Git must not trigger live
  deployment selection just because it had a root Compose filename. Added guards
  for both old and new paths.
- Left the other 21 root definitions and all do3 Compose paths unchanged. No
  subscriber generator or role/data boundary changes.

## Adoption, failure, and rollback

No persistent data, database schema, encrypted config, or application protocol
changes; no data migration is required. Only the isolated test command changes.
Use `-p rawrarr-test-environment -f deploy/testing/compose.yml`, preserving any
additional overrides and protected settings. Static configuration binds point to
the moved checked-in files; secret and runtime state paths remain unchanged.

External commands using the deleted filename must be updated before their next
run; there is deliberately no root compatibility wrapper. Existing containers
are not reconfigured by moving files. Docker labels may still record the old
path until a later deliberate test-stack recreation. Do not copy secret files,
delete volumes, reuse a historical project name, or run `down` for this change.
Actual starts/recreations retain normal lock and published-main requirements.

If a later startup fails, inspect the resolved config and paths before changing
state. Roll back to the prior published revision and file argument with the same
project/volumes. No data rewrite limits rollback.

## Verification

- Captured the original and relocated `docker compose config --format json`
  models using project `rawrarr-test-environment` and `--env-file /dev/null`.
  Compared the complete models after allowing exactly the two static bind-source
  relocations: identical, including the resolved Rust build context.
- The new Compose definition resolves successfully in OrbStack.
- Resolved the definition without `-p` and confirmed its default project is
  `rawrarr-test-environment` with the same 13 services.
- Compared both static fixture files to the prior Git revision: byte-for-byte
  unchanged.
- Focused Python regressions passed: layout (3), internal target classifier (8),
  coordinator auto-updater (10), and regression fixture copy (2).
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` passed all Node.js,
  Playwright, and Rust suites in the reusable OrbStack clone
  `rawrarr-test-regression-tests-mw0tyr9r`; Rust reported 123 passing tests.
  The base image was reused, not rebuilt. Test suites ran offline after clone
  preparation. The stopped clone retains its logs.
- `git diff --check` passed.
- No `rawrarr-test-environment` containers existed at inspection. Did not start
  the full environment or exercise peer enrollment/Arr APIs, inspect remote
  deployment labels, or deploy internal/public services.

## Remaining work

Issue #154 remains open. Next increments must resolve canonical coordinator and
subscriber bases, optional companions, and historical-stack retirement using the
inventory and actual deployment labels before moving/removing active definitions.
The current project instructions prohibit a runbook index update; none is made.
