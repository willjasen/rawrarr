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

- `python3 scripts/tests/test_publish_container_workflow.py` — 3 tests passed.
- PR #432's Basic script tests passed; its private UI bundle job was skipped.
- GitHub Actions publish run [41729388514](https://github.com/willjasen/rawrarr/actions/runs/41729388514) succeeded: AMD64 and ARM64 images built and published, then the multi-architecture `main` and commit tags were assembled.
- The ARM64 runner architecture check passed; the image build step took 5m49s and the full ARM64 job took 6m12s. The prior QEMU ARM64 image build step took 2h28m58s. That's an observed roughly 25x reduction, though not a controlled single-variable benchmark because the Rust dependency-layer change also landed between the two runs.
- GitHub's hosted-runner reference lists `ubuntu-24.04-arm` for private repositories. Private-repository compute usage/rates depend on the account's plan.

## Current image run

The earlier publish run `41723284489` was canceled at Will's request while ARM64 was still compiling. Its AMD64 build had succeeded. No multi-architecture manifest was published by that canceled run; the successful replacement run is `41729388514`.

## Final state

Merged in PR [#432](https://github.com/willjasen/rawrarr/pull/432) as commit `381b775`. The successful image workflow completed from that commit.
