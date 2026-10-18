# Fix Movies age-ID state

## Objective

Keep Movies consistent with Settings when an age private ID is saved in this browser, including after the active tab has cleared its in-memory copy.

## Initial state

The coordinator Settings status reads browser `localStorage`, while Movies decryption reads `ageIdentityInMemory`. The page clears that variable on `pagehide`, but Movies navigation did not rehydrate it. Also, decryption errors unconditionally revealed the entry form, which could imply that the ID was absent even when the actual issue was a request or ciphertext error.

## Actions

- Re-read the saved browser age ID when opening Movies if the in-memory value is empty.
- Refresh the age-ID controls before attempting the automatic unlock.
- On unlock failure, preserve controls based on actual browser storage state and show the request/decryption error.
- Add a regression assertion and update the test index.
- Rebuilt only roarlay in the existing `rawrarr-local` deployment using the Compose files recorded on its Tailscale container: `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml`.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` passed.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (13 tests).
- Scoped `git diff --check` passed.
- Roarlay reports healthy; `http://127.0.0.1:7297/health` and `https://rawrarr.risk-mermaid.ts.net/health` both returned HTTP 200.
- Read back Tailscale Serve JSON: `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`; existing `/direct-api` to `http://127.0.0.1:8082` remains configured.
- Confirmed the served coordinator script on both local and HTTPS routes contains the age-ID rehydration fix.
- The first base-file-only deployment attempt hit the existing Tailscale port 7298 bind. Identified the live deployment's three Compose files from container metadata and retried with those files, leaving the Tailscale container running.
- Released the task-owned roarlay deployment marker after verification.

## Recovery

Revert the Movies navigation rehydration and failure-handler control update in `rust-arr/src/coordinator-workspace.mjs`.
