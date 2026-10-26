# Deploy tested GitHub main to the internal Macs

## Objective and tracking

Deploy the tested, published `main` revision to the internal Mac mini coordinator (`roarlay`) and MacBook subscriber (`fossil`). GitHub issue [#68](https://github.com/willjasen/rawrarr/issues/68) tracks this deployment. Do not deploy the separate local bytecode-only commit.

## Source and initial state

- GitHub `main` is `02c3461c4535555d249533c94ea88130bd04954a`. GitHub Actions run [40252774607](https://github.com/willjasen/rawrarr/actions/runs/40252774607) passed the required Python script tests and published the signed UI release.
- Local main has that same source commit plus local bytecode-only commit `dcb8045`; the local OrbStack Node, Playwright, Rust, Python, Syncthing peer, and role integration tests passed against the same application source.
- `scripts/classify_internal_deploy_targets.py --base 204e31a29e9ece95b1ad3c45d2155f6c484b4706 --head origin/main` selects both coordinator and nodes.
- No GitHub Actions service-image deployment workflow exists. GitHub Actions publishes the UI bundle; service images must be built from the published GitHub revision on the target Macs.
- Before deployment, fossil and roarlay are healthy, their Tailscale sidecars are running, the coordinator PostgreSQL is healthy, and both per-container deployment locks are absent. The coordinator Serve route reads back `/` -> `http://127.0.0.1:8080`.
- Existing apps use Compose projects `rawrarr-node-internal` and `rawrarr-internal-mini`. Preserve their volumes, PostgreSQL, Tailscale containers, routes, secrets, and browser windows.

## Deployment actions

- On the Mac mini, fetched GitHub `main` into the existing clean GitHub source checkout and created a separate detached worktree at `02c3461c`. The existing `github-source` branch remained unchanged. Built `rawrarr:internal-mini` from that exact source; image ID `sha256:d911d1e755c5c254ff4e8ca83fb658da311f3f9917c659d0cfa86df59e974b01` (`arm64`). Tagged the previous roarlay image as `rawrarr:rollback-roarlay-02c3461` (`sha256:66700e4a92f9e95186cdb856d3900e37edbc2b75ff2d72a15d182d6b224cb634`).
- Noninteractive SSH on the Mac mini omits `/usr/local/bin` from `PATH`; remote Docker and GitHub CLI commands worked after exporting `PATH=/usr/local/bin:/opt/homebrew/bin:$PATH`. The Docker context reported `orbstack`.
- Before updating roarlay, wrote a mode-600 custom-format PostgreSQL dump under `~/Library/Application Support/Rawrarr/backups/pre-main-02c3461/` and confirmed it with `pg_restore --list`. The fossil `/data` volume was copied to the same protected backup folder on the MacBook; its directory mode is 700 and group/other access was removed.
- Built the MacBook image from the clean GitHub-based Codex worktree; image ID `sha256:d46f3f9d41b00d1e8a0b0645f1eb3e246faab7348d648310d5c74ee28e3f0cc0` (`amd64`). The old running image blob was missing from Docker's image store. `docker commit` could not snapshot it, so its container root filesystem was exported into rollback image `rawrarr:rollback-fossil-02c3461` (`sha256:d7631e6b8d107f9b341f0d144fe403082cb12521428483e8e89143f42a720617`); mounted `/data`, `/shared`, `/ui`, and token paths were excluded from that image.
- The first fossil recreation with the new binary failed and entered a restart loop because the existing host Compose file does not pass `RAWARR_INSTANCE_ROOT_FINGERPRINT`. The new binary requires this independently checked pin. No pin was guessed. The owner chose a clean account/trust bootstrap rather than spending time recovering the prior fingerprint. Fossil was stopped; its application-side state and volumes were retained.
- Followed the signing rollout sequence and deployed roarlay from the exact GitHub `02c3461c` Compose and source checkout using `--no-deps`. For the clean bootstrap, created a separate mode-600 empty approved-node registry file (`approved-nodes-reset.json`) because OrbStack could not re-bind the atomically replaced original file. The reset registry mounted successfully. Recreated only the coordinator database; PostgreSQL, Tailscale, and their volumes remained running. The roarlay deployment lock was released after verification. The fossil deployment lock remains held while that stopped service awaits onboarding.

## Verification and current state

- On the tested application source commit: OrbStack Node 122/122, Playwright 1/1, Rust 82/82, Python 34/34, Syncthing peer integration passed, and role integration passed. GitHub Actions run [40252774607](https://github.com/willjasen/rawrarr/actions/runs/40252774607) passed `Basic script tests` and signed UI bundle publication for the same commit.
- Roarlay is running the new arm64 image, healthy with zero restarts. SQLx migrations 1–8 succeeded. The account, passkey, node, enrollment, and instance signing-root tables are empty. The protected PostgreSQL dump remains available and valid. PostgreSQL and Tailscale retained their original uptime and zero restarts. The active approved-node registry parses as an empty list and has mode 600.
- `docker exec rawrarr-internal-mini-tailscale-1 tailscale serve status --json` confirms `rawrarr.risk-mermaid.ts.net:443` `/` still proxies to `http://127.0.0.1:8080`. Coordinator HTTPS `/health` returned HTTP 200 and the route serves the Roarlay health page. Fossil is stopped pending the new owner root and generated Compose; its Tailscale sidecar and its `/data`, `/shared`, and `/ui` volumes were not reset. A current MacBook SSH verification could not be completed: the tailnet address resolves, but SSH host-key verification failed, and the `fossil` short hostname does not resolve from this workstation.
- The coordinator's persistent UI volume contains bundle versions `6ffed1623ad46fad739bf87944032293118b5b8e` and `02c3461c4535555d249533c94ea88130bd04954a`; the latest version is the tested GitHub `main` commit.
- Fossil previously ran the exported pre-update application snapshot with zero restarts and HTTPS `/health` HTTP 200, then it was stopped for the authorized clean bootstrap. It has not been updated to `02c3461c`; its existing Compose lacks the required instance-root pin.

## Owner-authorized clean bootstrap

The owner prefers to avoid recovering the old root backup and authorized clearing internal account/trust state if that is the faster route. The published onboarding flow can create a new encrypted instance-root backup and matching fingerprint, then generate a fossil Compose file containing the pin. The coordinator database and approved-node registry were reset. The Mac mini Tailscale identity, MacBook Tailscale sidecar, `/data` (including node age identity, signing key, and token), `/shared` metadata, `/ui` bundles, and all Docker volumes remain intact. Backups of PostgreSQL, the approved-node registry, and fossil `/data` are stored with owner-only permissions.

## Remaining work

The coordinator is ready for the owner to open `https://rawrarr.risk-mermaid.ts.net/`, create the first passkey, complete new root setup, save the encrypted root backup and fingerprint, and download/provide the generated fossil Compose configuration. The user said they do not know where to obtain the old fingerprint, so do not delay onboarding for backup recovery. Keep fossil stopped until the new root is created and its matching pin is available; then deploy the matching configuration and verify owner approval, node health, and coordinator-to-fossil routes. Before resuming fossil deployment, resolve the MacBook SSH host-key failure using its already trusted host identity/configuration; do not bypass host-key checking.

## Recovery

If an app fails verification, use its previously recorded image ID with the same Compose project and `--no-deps`, while holding only that app's deployment lock. Keep PostgreSQL and Tailscale state running. Verify health and routes before releasing the lock.
