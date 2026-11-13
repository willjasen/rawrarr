# Pull published images for do3 coordinator deployments

Issue: [#349](https://github.com/willjasen/rawrarr/issues/349)

## Change

The do3 coordinator updater continues to use its host-local GitHub token for
reading `main` and comparing revisions, but now discovers the latest published
app image by anonymously pulling `ghcr.io/willjasen/rawrarr:main`. It explicitly
requests `linux/amd64`, verifies the image's revision label and architecture,
and compares that source revision with GitHub `main`. If coordinator/runtime
changes remain unpublished, it waits for the image workflow rather than
deploying stale code. Otherwise it pins the local image under its immutable
`sha-<image-revision>` tag. The verified image ID is promoted from internal
roarlay to public/live; the updater no longer downloads source archives or
compiles Rawrarr on do3.

The publish workflow creates the SHA manifest and advances the `main` tag after
the AMD64 and ARM64 images finish. If the updater sees source changes on `main`
before the image is published, it detects the mismatch and retries on a later
timer poll. Compose still uses `pull_policy: never` and `--pull never` after the
updater has explicitly pulled and verified the image.

## Compatibility and adoption

This changes image delivery only. It does not change application data,
databases, encrypted configuration, or node/coordinator protocols, so no
migration is needed. The public GHCR package must remain public. Existing do3
deployments adopt the updater by installing it from a verified archive of the
published `main` revision with `deploy/do3/install-coordinator-auto-updater.sh`;
the installer enables the periodic timer.

## Deployment status

The previously installed source-building updater and its timer were stopped
while this change is prepared. Its in-progress build for revision
`40f38ec3dce806c65d400efe25ede735b213be10` was cancelled before either
coordinator was recreated. At that time, internal and public/live were both
healthy on `cff88a1384dba852a575ddfd0cd6595b12c00956`, with no deployment locks
held. The updater timer must be reinstalled from published `main` before
automatic updates resume. The old timer was subsequently disabled to prevent a
reboot from restarting the source-building updater. Automatic polling remains
paused until the replacement is installed from a verified published `main`
archive.

A manual internal-first attempt then pulled
`ghcr.io/willjasen/rawrarr:sha-3c250566dcd6bdf1737db81fdb05e53d97ecd8b8` on
do3 and verified its `linux/amd64` platform and source revision label. Internal
verification failed, and the deployment helper restored the prior image. The
retained output does not identify which verification check failed, so the
deployment must not be retried until that failure is diagnosed. Public/live was
not changed. A read-only follow-up confirmed both coordinators are healthy on
`cff88a1384dba852a575ddfd0cd6595b12c00956`; the internal Serve route still
proxies `/` to `http://127.0.0.1:8080`, and the internal HTTPS health route
responds. The public edge health request returned HTTP 403 from do3, which the
updater treats as a warning rather than a local-service failure.

After PR #461 merged as `e892b0829e4550c539faabdf6ab8b91b8ae50eac`, its updater
commit did not get a SHA image because image publishing is triggered only by
`rust-arr/**` or publish-workflow changes. The published GHCR `main` tag still
points to the valid `3c250566dcd6bdf1737db81fdb05e53d97ecd8b8` image. The
follow-up updater therefore discovers the published image revision from that
tag, compares its source paths to current GitHub `main`, and waits if any
coordinator/runtime changes remain unpublished. It then pins the validated
image locally by its immutable SHA tag. This follow-up has not yet been
installed or deployed.

## Revision verification fix

After PR #462 merged as `6ca8f98e5abd031f7f3a52f9b09c9a1a31b69656`, its updater
was installed on do3 with the timer still disabled. A controlled internal-first
run pulled GHCR `main` (OCI revision
`3c250566dcd6bdf1737db81fdb05e53d97ecd8b8`), recreated the internal
coordinator, and rolled back because `/api/version` reported `unknown` instead
of that revision. Public/live was not changed; internal is healthy again on
`cff88a1384dba852a575ddfd0cd6595b12c00956`.

Root cause: the publish workflow set only the OCI revision label and did not
pass the Docker build argument `RAWARR_BUILD_REVISION`, so the binary's revision
file contained `unknown`. The workflow now passes `RAWARR_BUILD_REVISION=<sha>`
for both architecture builds. Separately, the updater's top-level error handler
referenced `tarfile.TarError` after the `tarfile` import was removed, turning a
handled failure into `NameError`; that stale exception type was removed.

Adoption: merging this change triggers a fresh GHCR publish for the merge SHA.
Deployment resumes only after that image's OCI label and `/api/version` both
report the merge SHA; internal deploys first, then public/live on the same
image. The timer stays disabled until both deployments verify. No persistent
data or protocol migration is involved.

## Verification

- `python3 scripts/tests/test_do3_coordinator_auto_updater.py` — 18 tests passed.
- `python3 -m py_compile deploy/do3/auto_update_coordinators.py` — passed.
- `git diff --check` — passed.
- Revision fix: `python3 scripts/tests/test_do3_coordinator_auto_updater.py` — 19 tests passed; `python3 scripts/tests/test_publish_container_workflow.py` — 4 tests passed; `py_compile` and `git diff --check` passed. `test_rawrarr_coordinator_build_info.mjs` was not run locally (Node.js unavailable on this host).
