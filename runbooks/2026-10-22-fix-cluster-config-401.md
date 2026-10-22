# Fix cluster configuration fetch returning 401

## Objective and starting state

Restore the Configuration view when the browser reads age-encrypted config from an approved metadata node over its direct HTTPS route. The authenticated internal coordinator page showed `Could not fetch encrypted metadata (401)`; `/health` returned HTTP 200. The shared `main` checkout had unrelated edits, so this fix was isolated in a dedicated worktree.

## Change

`fetchBytes` now uses `credentials: 'include'`. The direct-route setup endpoint places a short-lived browser session cookie on the metadata-node origin; browser fetch defaults omit cookies on cross-origin requests, which caused the observed 401. The request still sends only the encrypted config and the browser decrypts it locally. The generated metadata-export bundle was rebuilt from `rust-arr/web-export/metadata-export.js`.

## Verification

- `node --test --test-name-pattern='encrypted metadata fetches include' scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed.
- A full run of that test file has unrelated failures against this worktree's older base snapshot; it is not a clean baseline for this focused fix.
- Deployed only the two metadata export source/bundle files to the Mac mini coordinator source, after backing them up under `~/Library/Application Support/rawrarr/internal-coordinator/backups/pre-cluster-config-401/`. Rebuilt only `roarlay` in `rawrarr-internal-mini`; PostgreSQL and Tailscale remained running.
- Roarlay reports healthy; HTTPS `/health` returns HTTP 200. Read-back Tailscale Serve status confirms `/` still proxies to `http://127.0.0.1:8080`.
- Opened a temporary authenticated Chrome tab to the Configuration page. It loaded the shared config and showed `<REDACTED>` for secret fields. Closed the temporary tab; the user's original tab was left unchanged.
- The deployment marker was removed after verification.

## Recovery and remaining work

Revert the task commit to restore the old fetch behavior. Deploy only the coordinator `roarlay` service using the project deployment instructions, then reload the authenticated Configuration page and confirm the encrypted config displays with secrets redacted. Preserve its database and Tailscale state.
