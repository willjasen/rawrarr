# Rename the workspace menu label

## Objective

Change the left navigation section label from “Workspace” to “rawrchive”.

## Initial state

- The label was in `rust-arr/src/ui.html`.
- The persistent `rawrarr-local` deployment serves the metadata role on `127.0.0.1:7297`.
- Port 7297 was owned by the running `rawrarr-local-metadata-1` container; its health page returned HTTP 200.
- The repository had extensive pre-existing changes. This task edits only the sidebar label and adds this runbook and index entry.

## Actions

- Changed the sidebar section label to `rawrchive`.
- Rebuilt the existing `metadata` Compose service under project `rawrarr-local`. Compose also attempted to recreate the coordinator and failed because another process already owns port 7298.

## Verification

- Confirmed the running metadata container is healthy and `/health` returns HTTP 200.
- Confirmed the rebuilt service binary contains `rawrchive`.
- The unauthenticated root redirects to `/auth`, so the authenticated workspace HTML was not read back over HTTP. Refresh the signed-in page to see the update.
- The coordinator is healthy after the interrupted Compose attempt; the MacBook Tailscale container remains running on port 7298.

## Recovery

Restore the section label in `rust-arr/src/ui.html` to `Workspace` and rebuild the same `rawrarr-local` metadata service. Use `--no-deps` to avoid recreating the coordinator and conflicting with the existing Tailscale port binding.
