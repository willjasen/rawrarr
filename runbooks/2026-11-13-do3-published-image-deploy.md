# Pull published images for do3 coordinator deployments

Issue: [#349](https://github.com/willjasen/rawrarr/issues/349)

## Change

The do3 coordinator updater continues to use its host-local GitHub token for
reading `main` and comparing revisions, but now pulls the matching
`ghcr.io/willjasen/rawrarr:sha-<full-sha>` multi-architecture image anonymously.
It explicitly requests `linux/amd64` and verifies the image's revision label and
architecture before deployment. The verified image ID is promoted from internal
roarlay to public/live; the updater no longer downloads source archives or
compiles Rawrarr on do3.

The publish workflow creates the SHA manifest after the AMD64 and ARM64 images
finish. If the updater sees a new `main` revision before its image is published,
the failed pull is reported and the next timer poll retries it. Compose still
uses `pull_policy: never` and `--pull never` after the updater has explicitly
pulled and verified the image.

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
automatic updates resume.

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

## Verification

- `python3 scripts/tests/test_do3_coordinator_auto_updater.py` — 15 tests passed.
- `python3 -m py_compile deploy/do3/auto_update_coordinators.py` — passed.
- `git diff --check` — passed.
