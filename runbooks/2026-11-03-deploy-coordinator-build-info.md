# Deploy coordinator build information to do3

Issue: [#349](https://github.com/willjasen/rawrarr/issues/349)

Pull request: [#350](https://github.com/willjasen/rawrarr/pull/350)

## Deployment

- Deployed published GitHub revision `e10250a8ef8ff694b41c44609c2a4962d7d3c030` to the internal and public/live `roarlay` services on do3.
- Downloaded the authenticated GitHub archive on do3 and verified SHA-256 `4a86b7bff9cb917806fec8cc7e37a8a9631a15e6c492b400ee72d8dc70f56892` before extraction.
- Built once on native `linux/amd64` with the host's BuildKit cache. Image ID: `sha256:c70678674ff0117b06914210fa47be5ec96a3ee23f421d943e422204d84ea952`; OCI revision label and embedded build revision both match the published SHA.
- Preserved the prior base and both service image IDs under `rawrarr:rollback-base-before-e10250a-amd64`, `rawrarr:rollback-internal-roarlay-before-e10250a-amd64`, and `rawrarr:rollback-public-roarlay-before-e10250a-amd64`.
- Recreated only internal `roarlay`, verified its healthy state, exact image ID/platform, local health endpoint, `/api/version`, internal HTTPS `/health`, and the existing Tailscale Serve mapping. The deployment timestamp is `2026-11-03T05:21:10Z`.
- Promoted the same image ID to public/live `roarlay`. Its container is healthy; `/health` and `/api/version` pass from inside the coordinator's network namespace. The public HTTPS check returned Cloudflare HTTP 403 from do3 for both `/health` and `/api/version`, so the public edge route could not be verified. The Cloudflare tunnel container remained running; no Cloudflare or ingress settings were changed. The deployment timestamp is `2026-11-03T05:21:10Z`.
- Both deployment locks were released after verification. Coordinator databases, Tailscale state, UI volumes, secrets, and unrelated services were left in place.
- After deployment, `origin/main` advanced to `ad5a631`. Comparing `e10250a` to the current published `main` with `scripts/classify_internal_deploy_targets.py` selected no runtime targets; the later changes are documentation and the separate landing page.

## Adoption and limits

This change adds no persistent data, database schema, or protocol format, so no migration is required. Existing deployments report the embedded build revision and use `RAWARR_DEPLOYED_AT` for the footer timestamp; the do3 override files are stored under each revision's host-local build directory.

PR #350 adds build metadata and the coordinator footer only. It does not install a GitHub Actions workflow or a do3 polling service to deploy future `main` commits automatically. Automatic deployment remains outstanding under issue #349. The public HTTPS route also needs a successful external health check; do not alter Cloudflare security settings based only on the 403 observed from do3.

## Verification

- Focused build-info Node tests: passed (2 tests).
- Linked OrbStack regression runner: Rust passed (116 tests); Node failed because `netlify-ignore-build.sh` was missing from the test clone; one existing Playwright Radarr/Sonarr settings test timed out. Other Playwright tests in the run passed.
- GitHub PR check `Basic script tests`: passed.
- Internal coordinator health, version metadata, and Tailscale Serve route: passed.
- Public coordinator container health and in-container health/version endpoints: passed. Public HTTPS through Cloudflare returned HTTP 403 and remains unverified.
