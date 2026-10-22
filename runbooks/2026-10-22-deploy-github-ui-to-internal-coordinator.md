# Deploy GitHub UI updates to internal nodes

## Objective and issue

Deploy private GitHub UI bundle updates and converge the internal coordinator and affected subscriber code on GitHub `main`. Issue [#13](https://github.com/willjasen/rawrarr/issues/13) tracks the work. Coordinator and subscriber services must be deployed only from a clean, published `main` commit. Local `main` and `origin/main` must point to the same final commit.

## Initial state

- GitHub `main` was `5ba6f80`, with UI bundle release `ui-5ba6f80daae78da1a18da79f1febcaec6047d3d8`.
- The Mac mini `github-source` checkout was on `codex/age-session-response-github` at `933d905`; its Compose build overlay pointed to a separate avatar branch. Active age-wrapped browser-session and node-avatar changes were missing from `main`.
- The mini's `roarlay`, PostgreSQL, and Tailscale sidecar were running; roarlay was healthy. Tailscale Serve mapped HTTPS 443 `/` to `http://127.0.0.1:8080`.
- The MacBook `fossil` project used `/Users/willjasen/rawrarr-node-internal/compose.yml`. Its `github-source` checkout had local modifications on the age-session branch; these were preserved.
- The mini's fine-grained GitHub read token was present with mode 600. The roarlay deployment lock was claimed before its rebuild. The fossil deployment lock was free before its rebuild.

## Changes and deployments

- Added age-wrapped browser session and node-avatar changes to `main`, retained the UI updater, and documented that deploys must use published `main` and that local `main` must match GitHub `main`.
- Pushed task commits through `02e8187`. GitHub Actions run [39854511390](https://github.com/willjasen/rawrarr/actions/runs/39854511390) passed and published `ui-02e818731a491f6427518f2102355f728273ae97`. PR #14 advanced `main` to `06762cb` with deployment-target classification and no runtime file changes. The task runbook commit advanced `main` to `d659b9d`; GitHub Actions run [39855598812](https://github.com/willjasen/rawrarr/actions/runs/39855598812) passed and published `ui-d659b9dd676b32ce66a7f1c410fe99a71a73b167`.
- Built roarlay from a clean detached mini worktree at `02e8187` using the main Compose file and no branch overlay. Recreated only `roarlay` with `--no-deps`. The PostgreSQL and Tailscale container IDs remained unchanged with zero restarts. Moved the mini's standard `github-source` checkout to clean `main` at `d659b9d`. The running updater advanced `/ui/current` from `06762cb` to `d659b9d` after that release was published.
- The deployment-target classifier selected both coordinator and nodes. Built `rawrarr:local` from a clean detached worktree of published `main` at `06762cb` and recreated only `fossil` in `rawrarr-node-internal` with `--no-deps`. The MacBook Tailscale container ID stayed unchanged with zero restarts. Added the protected token file at `~/Library/Application Support/Rawrarr/internal-node/secrets/rawrarr-ui-github-token` with mode 600, and updated the local Compose file with the GitHub release token secret and persistent `/ui` volume. Backed up the Compose file at `~/Library/Application Support/Rawrarr/backups/pre-node-ui-update-2026-10-22/compose.yml`. Fossil installed bundle `d659b9d` with SHA-256 `e2557be101e8609b3b76a42e90a35dfc52f00392fcee075ed27fa29119555bda`.
- Preserved the existing dirty MacBook `github-source` checkout and created a clean `github-source-main` worktree tracking `main` at `d659b9d`. Local `main` and `origin/main` both point to `d659b9dd676b32ce66a7f1c410fe99a71a73b167`; the shared checkout's unrelated staged and untracked files remain untouched. Removed both deployment locks after service verification and removed the task-created temporary build worktrees.

## Verification and recovery

- Focused checks: UI bundle tests 3/3; node-avatar and subscriber Compose tests 6/6. The broader coordinator workspace test file reported 19 passes and 7 failures, including mismatches between legacy assertions and current page markup; these remain unresolved. GitHub Actions basic tests and bundle publication passed.
- Focused bundle tests passed 3/3; node-avatar and subscriber Compose tests passed 6/6. `main` Actions tests and bundle publication passed. The broader coordinator workspace test file reported 19 passes and 7 failures, including mismatches between legacy assertions and current page markup; these remain unresolved.
- Verified the GitHub release bundle SHA-256, installed `/ui/current/bundle.version` and `.sha256` on both nodes, and matched the served coordinator workspace module SHA-256 to the bundle asset.
- Roarlay and fossil were healthy with zero restarts; both HTTPS `/health` checks returned 200. Tailscale Serve continued to map coordinator HTTPS 443 `/` to `http://127.0.0.1:8080`; the MacBook Tailscale Serve configuration and container were unchanged.
- Bundle updates verify SHA-256 and retain `/ui/previous`; publisher-authenticating signatures and browser-side signature verification remain future work. See [UI distribution design and tests](2026-10-22-github-ui-distribution.md).
