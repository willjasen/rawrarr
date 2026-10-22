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
- Pushed task commits through `02e8187`. GitHub Actions run [39854511390](https://github.com/willjasen/rawrarr/actions/runs/39854511390) passed and published `ui-02e818731a491f6427518f2102355f728273ae97`. PR #14 later advanced `main` to `06762cb` with deployment-target classification; it added no runtime files. That merge was included in the local `main` synchronization.
- Built roarlay from a clean detached mini worktree at `02e8187` using the main Compose file and no branch overlay. Recreated only `roarlay` with `--no-deps`. The PostgreSQL and Tailscale container IDs remained unchanged with zero restarts. Afterward, moved the mini's standard `github-source` checkout to a clean `main` branch at `06762cb`.
- The deployment-target classifier selected both coordinator and nodes. Built `rawrarr:local` from a clean detached worktree of published `main` at `06762cb` and recreated only `fossil` in `rawrarr-node-internal` with `--no-deps`. The MacBook Tailscale container ID stayed unchanged with zero restarts. The existing dirty `github-source` checkout was not modified.
- Removed both deployment locks after service verification. Removed the task-created temporary build worktrees.

## Verification and recovery

- Focused checks: UI bundle tests 3/3; node-avatar and subscriber Compose tests 6/6. The broader coordinator workspace test file reported 19 passes and 7 failures, including mismatches between legacy assertions and current page markup; these remain unresolved. GitHub Actions basic tests and bundle publication passed.
- Verified the GitHub bundle SHA-256, installed `/ui/current/bundle.version` and `.sha256`, and matched the served coordinator workspace module SHA-256 to the bundle asset.
- Roarlay was healthy with zero restarts; HTTPS `/health` returned 200; Tailscale Serve still mapped coordinator HTTPS 443 `/` to `http://127.0.0.1:8080`. Fossil was healthy with zero restarts; its Tailscale Serve configuration was unchanged and its HTTPS `/health` returned 200.
- Bundle updates verify SHA-256 and retain `/ui/previous`; publisher-authenticating signatures and browser-side signature verification remain future work. See [UI distribution design and tests](2026-10-22-github-ui-distribution.md).
