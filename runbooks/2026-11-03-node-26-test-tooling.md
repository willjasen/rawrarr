# Move Rawrarr tooling to Node.js 26

Issue: [#273](https://github.com/willjasen/rawrarr/issues/273)

## Objective and starting state

Use Node.js 26 for Rawrarr host-side tooling and the reusable OrbStack regression image on both native architectures. The test image previously installed Debian Bookworm's Node.js 18; Mac hosts already have Node 26, and the Mac mini's test image had previously been moved to Node 24. The MacBook host run passed 150 nonbrowser tests on Node 26, but Playwright and Rust were not completed in that earlier run.

The do3 fast deploy builds the Rust app image from published source and deploys it to isolated internal and public coordinator services. Mac fossil deployments also run the Rust app image. These runtime images do not install Node.js; host-side `web-export` dependency setup is the Node-dependent build path and must use locked npm dependencies.

## Change

- Added `.nvmrc` with Node major 26 without changing a machine-wide default.
- Added `.nvmrc` to the isolated runner's root fixture copy so test coverage sees the same major selection as the repository.
- Changed the reusable test image to install Node.js 26 from the signed NodeSource apt repository.
- Changed linked-clone preparation to upgrade stale Node versions, verify major 26 and npm availability, then fetch the locked Rust dependencies.
- Added explicit Node 26 gating to the isolated suite entrypoint and regression coverage for its base and clone setup.
- Documented base refresh requirements and the fast-deploy/runtime Node boundary in `TESTING.md`.

This is tooling only. No app protocol, database, `/data`, `/shared`, identity, or credential migration is expected. Service deployment is not part of this implementation task. On rollout, inspect the exact internal and public coordinator and fossil images; redeploy only if an image unexpectedly includes a Node-dependent runtime component.

## Verification

- `node --test scripts/tests/test_rawrarr_docker_healthchecks.mjs` — passed (5/5); covers Node major selection, NodeSource setup, linked-clone upgrade, the `/usr/local` Playwright module path, and copied `.nvmrc`.
- `python3 scripts/tests/test_do3_coordinator_auto_updater.py` — passed (6/6); `python3 -m unittest scripts.tests.test_internal_deploy_targets -v` — passed (7/7).
- `sh -n scripts/tests/prepare-test-clone.sh scripts/tests/run-tests-in-container.sh` and `git diff --check` — passed.
- Built the native amd64 reusable base and verified `rawrarr-testing-base:local` reports `linux/amd64`, Node.js 26.13.0, npm 11.20.0, and the expected Playwright module path. Preserved the previous Node 18 image as `rawrarr-testing-base:node18-rollback-20261103-amd64`; also kept an intermediate Node 26 image tag while correcting the Playwright module prefix.
- Ran linked-clone preparation from the preserved Node 18.20.4 base. It upgraded to Node.js 26.13.0, provided npm 11.20.0 and retained the Playwright module, fetched the locked Cargo dependencies, and reported `TEST CLONE READY`. The disposable migration container was removed.
- On host Node.js 26.12.0/npm 11.20.0, `npm ci --prefix rust-arr/web-export` and all three locked `web-export` build scripts passed. Generated output was checked and unrelated generated asset churn was restored.
- Full current-source OrbStack runner — Node passed **150/150** and Rust passed **116/116**. The Playwright suite exercised Node 26 and the coordinator build-info fast-deploy check passed; one existing test, `test_rawrarr_coordinator_sonarr_settings.mjs`, timed out waiting for its Radarr unconfigured note. Python integration and OrbStack peer tests were skipped because their disposable fixtures are outside this runner.
- No internal/public coordinator or fossil application service was rebuilt or restarted. Their Rust image Dockerfile contains no Node installation; exact live image inspection is deferred until the published rollout stage, when the internal/public coordinator and fossil deployments can be classified from the exact published revision.

## Rollback

Retain each host's previous reusable test image under a clearly named rollback tag before rebuilding `rawrarr-testing-base:local`. Restore the previous tag if Node 26 breaks the test tooling. The `.nvmrc` can be reverted independently; host-wide Node installations and running application images remain untouched.
