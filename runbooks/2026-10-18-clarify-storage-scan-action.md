# Clarify and deploy the storage media scan action

## Objective

Make the manual storage-node action clearly describe scanning media, calculating hashes, and running ffprobe; deploy the coordinator UI and current storage service; run the requested bounded sample.

## Initial state

- The user reported that no scan option was visible and clarified that the operation is a storage-node media scan/hash/ffprobe pass.
- Source had two actions, but the button said “Scan missing metadata”; the UI changes had not yet been deployed to roarlay.
- The roarlay service was healthy, port 7297 belonged to the existing OrbStack service, the configured password file existed, and the prior roarlay deployment lock was held by another task. Waited and atomically claimed it after it cleared.

## Actions

- Changed the actions to “Scan, hash & ffprobe media” and “Scan, hash & ffprobe 3 movies (1–2 GB)”; clarified that the worker reads media and writes encrypted sidecars to shared storage.
- Used the existing `rawrarr-local` Compose project and its coordinator/Tailscale overlay files to rebuild only `roarlay`.
- SSH access to `brandons-mac-mini.risk-mermaid.ts.net` was available for this one-time operation. The Mini’s OrbStack CLI is `/usr/local/bin/docker`; adding `/usr/local/bin` to the SSH command PATH made its configured macOS credential helper available to BuildKit.
- Confirmed the Mini storage deployment marker was clear, atomically claimed `~/Library/Application Support/rawrarr/.container-rebuild-storage.lock`, and created a source rollback archive at `deploy/backups/rust-arr-before-storage-scan-20261018.tar.gz`.
- Copied only `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `migrations/`, and `src/` into the existing Mini deployment source, preserving its `target/` and other deployment files. Rebuilt and force-recreated only the `storage` service.
- The first BuildKit attempt stopped before container changes because the SSH PATH omitted `/usr/local/bin/docker-credential-osxkeychain`; retrying with the scoped PATH adjustment succeeded.
- Delayed the automatic scan until one full 86400-second interval after service startup so deployment would not start an unbounded scan before the requested bounded sample.
- Clicked “Scan, hash & ffprobe 3 movies (1–2 GB)” on the authenticated Devices page. The Mini accepted the request with HTTP 202 and status polling returns HTTP 200.

## Verification

- `node --test scripts/tests/test_rawrarr_storage_scan_ui.mjs`, `node --check rust-arr/src/coordinator-workspace.mjs`, and `git diff --check` passed.
- Rebuilt and force-recreated only `roarlay` in the existing `rawrarr-local` Compose project. The container became healthy. The served coordinator module contains both updated button labels and the empty-response error text.
- Refreshed the signed-in Chrome tab and confirmed the Mac mini card visibly shows “Scan, hash & ffprobe media” and “Scan, hash & ffprobe 3 movies (1–2 GB)”.
- Local `/health` on port 7297 and the public `https://rawrarr.risk-mermaid.ts.net/health` respond with the Roarlay health page. Tailscale Serve readback still maps `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.
- The Mini storage container is healthy on the rebuilt image. Its direct scan-status endpoint returns HTTP 401 without an authenticated session (the route exists and is protected), direct health returns HTTP 204, and public Tailscale Serve readback preserves `/` to `127.0.0.1:8082` and `/direct-api` to `127.0.0.1:8083`.
- The initial observation caught the browser job during media-tree enumeration. On a later SSH check, Docker stats showed active CPU and increasing I/O, confirming work was progressing. A second button click received HTTP 409 because the original scan was still active; it did not start a duplicate.
- The original bounded scan then completed: 10,000 media files enumerated, 3 matching files checked, 3 encrypted sidecars written, 0 already complete, 0 sidecar errors, 0 scan issues, and `truncated=true` at the library enumeration cap. The UI reported “Scan complete · 3 files checked · 3 written · 0 already complete · 0 errors · scan limit reached”. The request options were movie kind, 1,000,000,000 to 2,000,000,000 bytes inclusive, and maxFiles=3.
- Verified three opaque `.json.aged` files in Mini `/shared/metadata/library/text` and the MacBook synchronized shared-folder copy. Their sizes were 4,790, 6,253, and 23,940 bytes, and each begins with the age encryption header. The node-local `/data/storage-sidecar-cache.json` was created with mode 0600. No plaintext sidecar or media-adjacent JSON was produced.
- The storage deployment marker was removed after health and route verification; the active scan itself is not a deployment operation and was left running. The source rollback archive remains on the Mini.

## Final state

The coordinator UI and updated Mini storage image are deployed. The bounded scan completed and its three encrypted sidecars are present in the synchronized shared folder. The scan reached the 10,000-entry library enumeration cap, which is surfaced in the UI; a future optimization could apply filters during traversal to avoid walking the full bounded inventory. The Roarlay and Mini storage deployment locks were removed after verification. SSH access was temporary for this operation and was not persisted.
