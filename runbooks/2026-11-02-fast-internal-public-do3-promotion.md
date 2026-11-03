# Fast internal-to-public do3 promotion

Issue: [#345](https://github.com/willjasen/rawrarr/issues/345)

## Objective

Reduce the time between a published Rawrarr coordinator change and having it available in both do3 environments. Keep Rust dependencies precompiled in do3's persistent BuildKit cache, deploy the internal coordinator first, then promote the same immutable image to public/live.

## Workflow

1. Before an iteration likely to need deployment, build a recent published `main` source archive on do3 for native `linux/amd64`. This warms the Dockerfile's Cargo registry, Git, and target BuildKit caches without changing a running service. Check storage first and retain all caches. If an exact revision image is already present and its OCI revision/platform labels match, reuse it instead of rebuilding.
2. After a change is merged, fetch its authenticated GitHub archive by full commit SHA, verify its digest, and build once from `rust-arr/` on do3. Record the full revision label, architecture, and image ID. Preserve the old reusable base and service image IDs under rollback tags before advancing `rawrarr:internal-base-amd64`.
3. Recreate only internal `roarlay` under `/opt/rawrarr-internal-do3/.container-rebuild-roarlay.lock`, using a local image override with `pull_policy: never` and Compose `--no-deps --pull never`. Verify health, revision, architecture, local health, the Tailscale Serve mapping, and `https://rawrarr.risk-mermaid.ts.net/health` before releasing the lock.
4. After internal verification, recreate only public `roarlay` under `/opt/rawrarr-roarlay-prod/.container-rebuild-roarlay.lock`, using the same immutable SHA image and the existing public Compose project. Verify health, revision, architecture, and `https://roarlay.rawrarr.net/health` before releasing the lock.
5. Compare the published revision with the previously deployed SHA using `scripts/classify_internal_deploy_targets.py`. If subscriber code is affected, deploy native amd64/arm64 images to the appropriate Macs through their separate workflow.

Keep coordinator databases, Tailscale state, UI volumes, public ingress services, subscriber state, and secrets unchanged. Never pull or rebuild between the internal and public stages: both should run the same do3 image ID. If either health check fails, stop at that stage and restore only that coordinator's saved image under its lock. Schema changes need their own backup, compatibility, and rollback plan.

## Compatibility and adoption

This is an operational workflow only. It adds no application state, schema, or protocol change, so no migration is required. Existing do3 deployments adopt it through their current published-archive and local-image path; the separate Compose projects and persistent volumes remain as-is.

## Verification

- Reviewed the current Dockerfile's BuildKit cache mounts and do3 coordinator Compose definitions.
- Cross-checked the procedure against the recent internal coordinator, public coordinator, image-cache, and subscriber deployment runbooks.
- The initial published `main` revision was `7af3d2780271b878a8751fb7f5522bb7731c3704`; the internal coordinator already ran it as native `linux/amd64`, image ID `sha256:b5c578a84beef40ab43ab702fb163bf13dcb12f8ce310ae7655c430415a3eba0`. Internal local `/health` and HTTPS `/health` returned HTTP 200, and Tailscale Serve JSON mapped `/` to `http://127.0.0.1:8080`. Public/live was `e2bd850a8b6464702c26b58515bccc5da1f7efce`; it was promoted to `7af3d27` using the already-built image with no compilation.
- GitHub `main` then advanced to `b29e11b076c7102e2865706a8f1a8bf778e6282b` during this task. A concurrent internal rollout had already built and deployed that exact revision as `linux/amd64`, image ID `sha256:6abd868bb1a70b405d1462bd9e7684cb10ae9572d0c70d46ee4a69977b15b287`. Verified internal local port 7299 and HTTPS `/health` returned HTTP 200, the Serve route, and the full OCI revision label. Preserved public's `7af3d27` image as `rawrarr:rollback-public-before-b29e11b-amd64`, then promoted the same `b29e11b` image to public Roarlay with `--no-deps --pull never`. Public reports healthy with zero restarts and the expected revision/platform; `https://roarlay.rawrarr.net/health` returned HTTP 200. Released the public lock after verification. No additional image build or dependency compilation was needed.
- The change classifier selected coordinator only; no subscriber nodes needed an update. The published changes contain no database migration.
- The linked OrbStack regression container `rawrarr-test-regression-tests-mw06cfid` used the existing source-independent `rawrarr-testing-base:local` image (`sha256:6eb2b6d92c319a05bb9b986c7c527f33cc26db7e26ace20fe5851a24c5661ff5`, linux/arm64). It copied the current shared-checkout snapshot, prepared dependencies, disconnected from the bridge, ran the suites offline, and was removed afterward. Node, Playwright, and Rust suites failed; Python integration was skipped by the runner. Observed failures included stale age-public-ID assertions, topology-visibility browser timeouts, and Rust compile error `WorkspaceNode` missing `signing_root_fingerprint` at `rust-arr/src/coordinator.rs:1611`. The snapshot included existing uncommitted changes in the shared checkout; those files were not modified by this task. The reusable base image was retained. The separate multi-node `rawrarr-test-environment` stack was stopped and was not started.
