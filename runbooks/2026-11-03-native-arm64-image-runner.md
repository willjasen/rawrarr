# Build the ARM64 image on a native GitHub runner

Issue: [#431](https://github.com/willjasen/rawrarr/issues/431)

## Objective and initial state

Replace QEMU emulation for the ARM64 image build with GitHub Actions' `ubuntu-24.04-arm` runner. The current multi-architecture workflow's ARM64 job ran for more than two hours in a previous run, while AMD64 completed in about seven minutes. Its logs show Rust dependencies compiling under QEMU. The task began on clean `origin/main` commit `7b2ccae` in a dedicated worktree.

## Actions

- Assign each architecture a matrix runner: AMD64 stays on `ubuntu-latest`; ARM64 uses `ubuntu-24.04-arm`.
- Remove QEMU setup and verify `uname -m` matches the expected native architecture before building.
- Add workflow regression coverage and update the script test index.
- The change only affects CI runner selection. It does not change image contents, runtime state, data, schemas, or subscriber/coordinator protocols; no deployment migration is needed.

## Verification

- Pending: `python3 scripts/tests/test_publish_container_workflow.py`.
- Pending: GitHub Actions workflow for both architecture builds and manifest publication after merge.
- The workflow's use of the GitHub-hosted ARM64 runner is supported by current GitHub-hosted runner documentation. Private-repository compute usage/rates depend on the account's plan.

## Current image run

The earlier publish run `41723284489` was canceled at Will's request while ARM64 was still compiling. Its AMD64 build had succeeded. No multi-architecture manifest was published by that canceled run.

## Final state

Implementation is in progress on `codex/native-arm64-image-runner`.
