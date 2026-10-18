# Show shared-folder health in the coordinator Devices view

## Objective
Display Syncthing shared-folder status on each enrolled device card in the live coordinator workspace.

## Initial state
The prior local-app implementation was not used by roarlay's coordinator Devices page. That page gets enrolled nodes from `/api/workspace/nodes`; node health must travel through the existing account-scoped `/nodes/{node_id}` relay. The MacBook Syncthing instance has `rawrarr-shared` configured, is idle at 100%, and reports the Mac mini peer connected at 100%. Rawrarr's metadata container had no Syncthing API configured. Syncthing's API key is stored only in its protected local configuration and a mode-0600 host-side runtime file for mounting as a Docker secret.

## Changes
Added an authenticated subscriber endpoint for shared-folder status, allowed that exact GET through the owner-scoped coordinator node relay, and added cached per-node health rendering to coordinator device cards. Syncthing peer names are matched against node IDs and browser-decrypted device names; ambiguity is displayed via the node's own health or unavailable state. Configured the MacBook metadata service to query the existing Syncthing companion through its Docker container name and mount its API key as a Compose secret.

## Verification
UI JavaScript syntax, focused whitespace checks, and resolved Compose configuration pass. Docker builds compile the Rust application successfully. The authenticated live node endpoint reports `rawrarr-shared` idle at 100%; `rawrarr-mini-syncthing` is connected and at 100%. The live coordinator Devices view displays the matching status on both device cards. Roarlay and metadata containers are healthy; local and tailnet `/health` return HTTP 200 and identify Roarlay. Tailscale Serve still maps HTTPS `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.

## Recovery
The API credential is runtime-only at `~/Library/Application Support/rawrarr/syncthing-api-key` and can be removed after reverting the metadata Compose secret/environment wiring. The named application and Syncthing data volumes are preserved; do not use `down -v`.

The Syncthing `system/status` endpoint does not include a version field in the installed Syncthing release; the client reads it from `system/version`. Folder matching uses explicit JSON string extraction. These fixes were confirmed by the live endpoint response above.

## Final state
The coordinator Devices page renders per-device shared-folder health and peer completion. Deployment and final route checks passed; the roarlay and metadata deployment locks were released.
