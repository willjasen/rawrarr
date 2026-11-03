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
