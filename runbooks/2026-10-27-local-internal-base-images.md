# Local architecture-specific internal base images

## Objective and tracking

Create host-local full Rawrarr images that can be reused by coordinator and subscriber containers on each internal Mac, using the Mac's native processor architecture. Tracked by [issue #198](https://github.com/willjasen/rawrarr/issues/198).

## Work performed

- Added the linked-clone image workflow to `AGENTS.md`: exact published GitHub source, host-local native builds, architecture-specific stable and immutable tags, cache reuse, `pull_policy: never`, per-container deployment locks, service-only recreation, and a no-prune rule.
- Published source revision was `fc229749cecd47a2282a75719a0cdb9e9e801fce`. Its verified GitHub archive SHA-256 was `00dedd37e3992679aff7af8973fbb71c3c72cadd157e5207e488f2ac0fccabce`.
- MacBook Pro Docker reports `linux/x86_64`. Tagged the locally built full app image as `rawrarr:internal-base-amd64`; image ID `sha256:9f8f3024ce91b95ab670e884970cafb82a32a6f19ad1a6031b5625c72ae1f5cd`, OCI revision `fc229749cecd47a2282a75719a0cdb9e9e801fce`.
- Recreated only the MacBook `fossil` service from this local base using a Compose override with `pull_policy: never`. It restarted while the coordinator was unreachable, but it has since recovered: the service is healthy and `/health` responds. Removed this task's MacBook fossil deployment lock after verification. Its Tailscale sidecar was left running. `/direct-api/health` is not a valid route and returns 404; service verification uses `/health`.
- The Mac mini reports `linux/aarch64`. The first ARM build command failed because `--cache-from rawrarr:valkey-metadata-main-arm64` was treated as a registry cache reference and denied. A retry without that option began the native build, but OrbStack then stopped with a storage-full error. The mini has recovered and reports Docker context `orbstack`, `linux/aarch64`, and a healthy roarlay and mini fossil. Tailscale Serve reports `/` -> `http://127.0.0.1:8080`, and the tailnet `/health` route responds.
- The current exact-source archive was recreated from the published commit with `git archive`, transferred to `~/builds/internal-base-fc22974/source.tar.gz` on the mini, and checksum-verified on both Macs (`b063996ecae2e1851a37a16cd732e76cf14bf4ed37d82d2b314dc0bb513d8024`). The build directory is `~/builds/internal-base-fc22974/rust-arr`.
- The mini's free space fell from 4.2 GB to 2.4 GB during inspection and archive transfer. Docker has 1.348 GB of reclaimable BuildKit cache and 2.073 GB of reclaimable image layers, but the cache supports faster native builds and the images are prior rollback artifacts. A concurrent `rawrarr-test-regression` container finished with exit 1 and still references a 4.38 GB `rawrarr-testing-base:local` test image; its failure logs may be needed by that task, so both artifacts were preserved. No image, volume, cache, or user file was removed. More storage headroom is needed before another build attempt.

## Remaining work

After the Mac mini has several GB of free storage and OrbStack/Docker is healthy again:

1. MacBook fossil recovery and lock release are complete. `/health` is the valid application health route.
2. Rebuild published revision `fc229749cecd47a2282a75719a0cdb9e9e801fce` on the Mac mini with `--platform linux/arm64`, verify `linux/arm64` and its revision label, and tag it `rawrarr:<sha>-arm64` plus `rawrarr:internal-base-arm64`.
3. Back up the current mini service image IDs as rollback tags. Under separate locks, update only `roarlay` in `rawrarr-internal-mini` and `fossil` in `rawrarr-internal`; keep PostgreSQL and all Tailscale sidecars untouched. Verify image architecture/revision, service health, and the coordinator Serve route before releasing each lock.

Do not free disk space by pruning Docker images, volumes, BuildKit caches, or user data without Will's explicit approval.
