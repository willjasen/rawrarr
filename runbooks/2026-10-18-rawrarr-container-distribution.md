# Plan Rawrarr container distribution

## Objective and initial state

Capture an eventual path for distributing the Rawrarr Docker container to self-hosted subscribers. The repository currently has a development Compose file that builds `rawrarr:local`; it has no published image or release automation. Existing [node types](../docs/node-types.md) and [Compose claim-flow runbook](2026-10-18-rawrarr-compose-claim-flow.md) define separate operator service and subscriber storage/shard roles.

## Decision recorded

- Publish versioned OCI images to a public registry, with immutable release tags and a digest/checksum for each release.
- Provide subscriber-friendly, role-specific Compose YAML. Normal installation should pull a prebuilt image, not require cloning or compiling the repository.
- Keep persistent state in `/data` and `/shared`, and media in node-local `/media`; never package subscriber data or credentials into the image.
- Make upgrade steps explicit: select a supported release, update the Compose image tag, pull, and recreate the app container while preserving its mounts.
- Consider multi-architecture builds for common self-hosting devices and publish release notes. Keep registry credentials unnecessary for ordinary pulls.
- Preserve the service-node boundary: share the subscriber image only if roles remain isolated at entrypoint and mount levels; otherwise use a separately published service image.

## Initial state and useful commands

- Checked `git status --short --branch`; the branch is `main` and contains pre-existing unrelated edits. Those files were preserved.
- Reviewed `rust-arr/Dockerfile`, `docker-compose.yml`, `README.md`, `docs/node-types.md`, and the existing claim-flow plan.
- Current Compose uses `build: ./rust-arr` and `image: rawrarr:local`; it is explicitly a development/local example.

## Verification and remaining work

- Reviewed the documentation changes and runbook index entry; run `git diff --check` before completion.
- No image was built or published. Registry selection, image naming, release support policy, update notification behavior, and signed provenance remain to decide when preparing the first distributable release.
