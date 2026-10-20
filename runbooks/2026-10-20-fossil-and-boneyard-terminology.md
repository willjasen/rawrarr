# Name fossil and boneyard nodes

## Decision and implementation

The user-facing names are **fossil** for metadata nodes and **boneyard** for storage nodes. The user requested this mapping on 2026-10-20, superseding the earlier terminology decision recorded in commit `c67ab0f`.

The names are reflected in the onboarding choices and generated Compose services, coordinator node labels and prompts, standalone Compose service names, and current architecture guides. Runtime role values remain `metadata` and `storage`; stable node IDs, database role values, API route paths, and data directories remain unchanged.

## Future encrypted Syncthing destination

The user will choose a separate name later for an untrusted Syncthing destination that stores replicated media encrypted at rest. That destination is not implemented and is not called boneyard. Syncthing transport encryption alone does not provide destination-at-rest encryption.

## Files updated

- Updated [node types](../docs/node-types.md), [product and interface design](../docs/DESIGN.md), [deployment](../docs/container-deployment.md), [user guide](../docs/user-guide.md), [technical status](../docs/technical-status.md), [device architecture](../docs/device-architecture.md), and [coordinator linking](../docs/coordinator-linking-plan.md).
- Updated onboarding and coordinator UI labels, generated Compose output, and Compose service names while retaining internal role identifiers.
- Updated focused UI/Compose tests and the testing index.

## Verification

Both browser bundles were rebuilt with `npm run build` and `npm run build:subscriber-onboarding` from `rust-arr/web-export`. The subscriber Compose test and fossil/boneyard selection test pass. The MacBook Tailscale namespace test passes; the Mini storage test fails on its existing expectation for a `RAWARR_COORDINATOR_SOCKS5_URL` setting that is absent from `docker-compose.storage.yml`. The combined Node run also encounters unrelated existing coordinator workspace assertions and cannot load the onboarding browser test because `playwright-core` is unavailable outside the isolated test image. Rust tests could not start because the local `rustc` binary aborts while loading a missing Homebrew `libz3.5.1.dylib`. `git diff --check` passes. No deployment or runtime data was changed.
