# Integrate Prowlarr for indexer discovery and search

Issue: [#289](https://github.com/willjasen/rawrarr/issues/289)

## Scope

Implement Prowlarr as an owner-configured, replaceable indexer-discovery adapter. Service configuration belongs to the selected metadata node and is encrypted using the existing age-encrypted config. The coordinator browser talks to that node directly; roarlay receives neither the Prowlarr API key nor search results. Searches are explicit, bounded, and read-only; Rawrarr does not download or fulfill results.

The managed-node deployment/adoption phase must preserve the external Prowlarr instance and its data, use Prowlarr-supported export/import mechanisms, and provide review, verification, and rollback before cutover.

## Compatibility and adoption

Prowlarr credentials fit the existing version-1 encrypted `services` array, whose deserializer already preserves unknown service entries. Keep old encrypted configurations readable and do not rewrite them merely to add Prowlarr support. No PostgreSQL schema, `/shared` file format, node protocol, or existing service configuration changes are required.

Managed Prowlarr runs as an owner-started Compose overlay alongside the existing fossil Compose project. The overlay shares the fossil's Tailscale network namespace, stores `/config` in a new node-local `prowlarr-config` volume, and adds a `127.0.0.1:9696:9696` host binding to the Tailscale sidecar so setup is available only from the node host. It does not mount Docker sockets into Rawrarr or send service credentials/backups through roarlay. Applying the overlay can recreate the Tailscale sidecar and interrupt the node; run it in a maintenance window, preserve existing volumes, and never use `docker compose down -v`.

Adoption is owner reviewed and uses Prowlarr's native System → Backup/Restore UI. Keep the external instance running and unchanged, save its URL/API key privately before cutover, export its native backup, and restore only into a fresh managed instance with a compatible Prowlarr version. Compare indexers, enabled states, and connected applications in the source and target UIs. Test the managed node-local URL (`http://127.0.0.1:9696`) and target API key through Settings before saving; then verify Discover status and a manual search. Rollback by restoring the source URL/API key in Rawrarr. Keep the managed volume until rollback is verified. Backup archives may contain credentials and remain local to the owner.

## Verification

Phase 1 is implemented: selected-node encrypted settings, connection validation, normalized indexer status, explicit bounded search, and a read-only discovery view. Phase 2 offers a reviewable local Compose overlay and documents native backup/restore adoption, cutover checks, and rollback. Radarr/Sonarr's shared Arr transport and coordinator settings flow are reused. The browser sends the Prowlarr API key and search requests directly to the selected node; roarlay does not receive credentials or search results.

`python3 scripts/tests/run_rawrarr_tests_in_docker.py` passed after both phases: Node.js 157/157, Playwright including the Prowlarr overlay-download/adoption guidance and desktop/mobile flow, and Rust 126/126. Python integration tests were skipped by the runner because they require disposable PostgreSQL, age, or Syncthing fixtures. `cargo fmt --check` could not run: the isolated toolchain lacks `rustfmt`, while the host toolchain's LLVM cannot load its missing Homebrew `libz3.5.1.dylib`. Prowlarr and the generated Compose overlay have not yet been exercised against a live service/host.

No automatic Docker deployment or backup transfer is implemented. The owner reviews/runs Compose and uses Prowlarr's supported UI backup/restore process; external-instance preservation and cutover/rollback verification remain owner responsibilities.

## Internal backend rollout

After PR #426 merged, both internal fossils returned HTTP 200 for
`/direct-api/health` and HTTP 401 for `/direct-api/api/settings/radarr`, but HTTP
404 for `/direct-api/api/settings/prowlarr`. Both running images identified
revision `c001b779a31da9ecbee0f3cdae1935a2bb94773b`, predating Prowlarr support.
The updated browser UI therefore reached nodes without the new backend routes.
Will authorized deploying the merged backend to both internal fossils.

The published target inspected was
`f67b072a45df6771f9430d8fa523aaecf419e7e9`. Deployment classification selected
coordinator and nodes relative to the old image, but this operation is explicitly
limited to the two fossils; do3 and public services remain untouched.

The MacBook Pro is the local host, uses Docker context `orbstack`, and reports
`linux/x86_64`. Its active service is `rawrarr-node-internal-fossil-1`, with
Compose files in `~/rawrarr-node-internal/`. The Mac mini reports
`linux/aarch64`; its active internal service is `rawrarr-internal-fossil-1`,
with Compose files in `~/rawrarr-internal/`. Preserve their existing `/data`,
`/shared`, and any `/ui` volumes. An unrelated restarting subscriber on the mini
is outside this rollout.

The configured SSH identity file was absent. Mini access succeeded using the
existing default identity explicitly with the saved `brandons-mac-mini` host
key and strict host-key checking enabled. No SSH keys or configuration changed.
No fossil deployment markers were present at inspection.

Pre-build storage checks found about 19 GiB free on the MacBook Pro and only
5.6 GiB free on the mini. Deployment paused before any image build, retagging,
or service recreation because the mini lacked comfortable build headroom.
Nothing had been pruned or deleted at that point. Resumption required resolving
storage and rechecking the published target and per-service deployment markers.

### Completed rollout

Will approved removing exactly two unused mini test images:
`rawrarr-testing-base:node24-rollback-20261028`
(`1dce271ce34e528e0976b95faa9bd1b9bd028ee1e7f43a324c34d01688d6969c`)
and dangling test image
`f29b882b2018cfa4e1c6a26214b04ef40b9ca670e704051db42fc5fb807958c5`.
BuildKit retained their layers after image removal. Separate approvals covered
seven obsolete unshared test-cache records and their 15 small descendant
records; these were removed by exact record ID, leaf-first. No broad cache
prune was used. The current reusable test base, application compiler/dependency
caches, application rollback images, containers, and volumes were preserved.
Available mini space rose to about 11 GiB before the build and remained about
10 GiB afterward.

Downloaded the exact published target above using authenticated `gh api`.
The archive root included the full target SHA, and its SHA-256 was
`c50caaf7231004ad14bbfe88a79d7f999af17fc1a0c5504c79b2d498a90c30ce`,
verified again after transfer to the mini. Each host built from a new
`~/builds/prowlarr-f67b072-rollout-20261103/rust-arr/` directory, using its native
platform and existing BuildKit cache. No working-tree source or registry app
image was deployed.

Each fossil's `.container-rebuild-fossil.lock` was acquired atomically before
building and held through recreation and verification. Previous base and service
images were retained under
`rawrarr:rollback-prowlarr-f67b072-before-base-<arch>` and
`rawrarr:rollback-prowlarr-f67b072-before-fossil-<arch>`.
Only `fossil` was recreated in each existing Compose project, using the existing
files and local-image override with `--no-deps --no-build --pull never`.

The resulting immutable images are:

- MacBook Pro: `rawrarr:f67b072a45df6771f9430d8fa523aaecf419e7e9-amd64`,
  image ID
  `sha256:c80e7b9f42b2c0e7f426df3ceb56733c41e6b93d30eca98abfb8ee4e04005431`,
  verified `linux/amd64`.
- Mac mini: `rawrarr:f67b072a45df6771f9430d8fa523aaecf419e7e9-arm64`,
  image ID
  `sha256:19d534cec012de86f7ae766bee25a94ec0eedcd24ccb44274335336fa231718f`,
  verified `linux/arm64`.

The reusable `internal-base-<arch>` tags now point to those images. Both running
containers' revision labels and embedded build-revision files match the target.
Both are healthy, with successful container-local `/health` and tailnet
`/direct-api/health` requests. On both nodes, unauthenticated
`/direct-api/api/settings/prowlarr` and `/direct-api/api/prowlarr/indexers` now
return HTTP 401 instead of 404, matching the existing Radarr endpoint.
This verifies backend route availability, not an authenticated browser flow or
an actual Prowlarr search. No Prowlarr service was installed or reconfigured.

Before/after mount snapshots matched, and all other containers in the targeted
Compose projects retained their IDs. Both task-owned deployment markers were
removed after verification. The unrelated restarting subscriber on the mini was
left untouched; no coordinator or public service was changed.
