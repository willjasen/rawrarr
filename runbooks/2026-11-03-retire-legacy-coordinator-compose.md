# Retire legacy coordinator Compose definitions

Issue: [#154](https://github.com/willjasen/rawrarr/issues/154).
Follows the [inventory/test-layout increment](2026-11-03-compose-inventory-test-layout.md)
in the same dedicated task worktree (rebased milestone `eb7a25e`).

## Decision and evidence

The operator chose retirement after read-only deployment checks rather than
parameterizing the two duplicated staged Headscale clients. After SSH limitations
were reported, the operator explicitly chose: "Treat the old Mac coordinator
stacks as retired based on my confirmation".

- Local OrbStack Docker labels, including stopped containers, show the internal
  fossil uses host-local `rawrarr-node-internal/compose.yml` and an image override.
  It does not use the former Mini coordinator definition.
- do3's labels show separate internal/public host-local coordinator deployments,
  with distinct Compose projects and overlay chains. The former Mini file and
  empty Serve file are not in those chains. Docker commands ran inside a
  dedicated window of its existing `ai` tmux session.
- Remote Mac audit remains incomplete: the fallback Mini SSH identity was missing,
  and Tailscale SSH on both Macs stopped at strict host-key verification.
  No key was accepted or security check bypassed. The retirement is operator-
  confirmed where independent verification could not complete.
- No `.github/` workflow directly names these files. Repository references were
  current tests, the target classifier, deployment docs, and historical runbooks.
- `docker-compose.tailscale-serve.yml` contains only comments and `services: {}`.
- Retain `docker-compose.tailscale-macbook.yml` because the documented admin
  preview still depends on it. Retain generic coordinator/subscriber templates,
  optional app companions, and the staged Headscale clients.

The clean task worktree was inspected and `origin/main` fetched. Newer upstream
documentation work was observed, not merged into this task's commits. This
increment does not amend the first milestone or touch the shared checkout.

## Changes

Removed `docker-compose.internal-mini.yml` and `docker-compose.tailscale-serve.yml`,
reducing root definitions from 21 to 19. Analytics, trial, and configured relay/
direct-route tests now inspect the do3 coordinator, retaining their assertions
against the supported deployment instead of retaining a duplicate old topology.
Deployment classifiers ignore retired-path deletion events. Regression guards
check retirement and that supported generic/local/do3 definitions remain.

Updated the inventory, deployment guide, design notes, test index, and the two
source historical runbooks with retirement notices. Other historical transcripts
remain records of what ran, not current procedures.

## Adoption, failure, and rollback

No persistent data/schema, encrypted config, node protocol, service, or live
Serve configuration changes. No migration or deployment is needed for current
do3 coordinators or Mac subscribers. No remote file, container, volume, secret,
database, or Tailscale identity was removed.

Remove only the empty Serve `-f` argument from any external Compose command;
preserve the remaining file list and explicit project. Do not replay old Mini
coordinator deployment instructions or substitute the do3 file into the Mini's
project. Discovering an unexpected dependency means pausing that host's cleanup,
inspecting state, and planning migration separately. Restore the former
definitions from published commit `94fda4c` if required for recovery, retaining existing resources.
Never use `down`, recreate a database, or rewrite identities as part of retirement.

## Verification

- Focused Python checks passed: layout (4), internal deploy targets (9), do3
  coordinator auto-updater (11).
- The retained do3 coordinator definition passed `docker compose config --quiet`
  with a validation-only image tag; the relocated test definition also passed.
  No real secret contents were read by these checks.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py` passed Node.js,
  Playwright, and Rust suites in `rawrarr-test-regression-tests-mw0ueeoo`.
  Rust reported 123 passed. The reusable base was not rebuilt; test execution
  ran offline after clone preparation. Stopped-clone logs are retained.
- Confirmed 19 root definitions remain, with no change to the retained
  combined/coordinator/metadata/storage/MacBook transport or do3 Compose content.
- `git diff --check` passed.

No full environment startup, remote Mac Docker audit, internal/public deployment,
or remote cleanup was performed.
