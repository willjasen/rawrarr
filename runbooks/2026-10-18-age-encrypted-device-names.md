# Edit device names in the coordinator

## Objective and initial state

Allow an owner to rename approved nodes from the coordinator workspace while keeping each name private from the coordinator service. Device names were not stored; the coordinator showed stable node IDs sourced from PostgreSQL. The shared `config.aged` is the durable subscriber-owned settings file.

Related context: [blind coordinator boundary](2026-10-18-blind-coordinator-content.md), [shared age config](2026-10-17-rawrarr-shared-age-config.md), and [coordinator workspace](2026-10-18-coordinator-workspace.md).

## Implementation

- Added a backward-compatible `device_names` map to the age config, keyed by stable node ID.
- Added a browser flow that decrypts the shared config with the user-entered age identity, edits one name, and re-encrypts it using the published age recipient. The private identity and plaintext names stay in the browser.
- Added metadata-node endpoints to read the public recipient and atomically replace bounded age ciphertext. The coordinator relay allows only the corresponding ciphertext read/write paths and validates the encrypted file header on writes.
- Added a Devices tab containing the existing node cards and pending approvals; Overview is intentionally minimal. Device names use inline text fields that save on Enter or blur, without a persistent Rename button. Names are rendered from browser-decrypted config; PostgreSQL node records are unchanged.
- Refined device cards after screenshot review: the saved device name is now the main heading and editable in place; the generic “Open node” label is removed. Clicking elsewhere on the card opens the node, with role, route status, and compact instructions beneath the heading.

## Follow-up: metadata fetch returned 502

The user reported that renaming a device failed with “Could not fetch encrypted metadata (502).” Container inspection showed `roarlay` sharing the current `rawrarr-macbook-tailscale` namespace while `metadata` had no Docker network attachment and retained a stale `container:<id>` namespace. The `/direct-api` and coordinator Serve handlers remained correctly configured. Recreate both app services from the current Compose overlays so metadata attaches to the active Tailscale namespace; do not recreate PostgreSQL or Tailscale. Also correct the age ciphertext prefix check: it must match an actual newline after `age-encryption.org/v1`, not literal backslash-n bytes.

Rebuilt `roarlay` and `metadata` together with the existing `rawrarr-local` Compose project, without recreating PostgreSQL or Tailscale. Both app containers are healthy and now share the active `rawrarr-macbook-tailscale` namespace. From the coordinator container, metadata `/health` returned HTTP 200. The config and public-recipient routes returned HTTP 401 when probed without a delegated node session, confirming that requests reached the metadata service and its owner/session gate remains active. Tailscale Serve still maps HTTPS `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`; the public coordinator HTTPS `/health` returned HTTP 200. The focused device-name tests (3/3) and `git diff --check` passed. An authenticated browser rename/save could not be exercised in this turn, so that final interaction remains unverified.

## Verification and remaining work

`npm run build` regenerated the browser age bundle. The focused device-name assertions passed after the Devices-tab change, as did JavaScript syntax and `git diff --check`. The project-wide Docker suite reported unrelated existing working-tree Node failures, and its Rust tests could not resolve uncached crates because crates.io DNS was unavailable. Host `cargo check --locked` could not start because the local LLVM install requires a missing `libz3.5.1.dylib`; however, the coordinator Docker release build completed successfully.

Deployed the `roarlay` service in the existing `rawrarr-local` Compose project after the node-type naming task completed. After screenshot review, removed the generic “Open node” heading and made each saved device name the primary editable card heading. The card itself opens its node when clicked outside the name field; role and route state are secondary details. `rawrarr-local-roarlay-1` is healthy. The live HTTPS health page identifies Roarlay; Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and preserves `/direct-api` at `http://127.0.0.1:8082`. The live workspace asset contains the updated device card and inline editor. The metadata and PostgreSQL containers were not recreated. Device-name save was not exercised with a signed-in browser session.
