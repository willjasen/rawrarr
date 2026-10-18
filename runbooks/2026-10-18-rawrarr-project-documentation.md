# Consolidate Rawrarr technical and user documentation

## Objective and initial state

Create a clear project entry point that tells readers what Rawrarr can do now, how to run the Docker prototype, and which product ideas remain plans. The repository already had separate architecture docs and dated runbooks but no concise user guide or technical status matrix. Reviewed the current Compose files, Rust routes/UI, test index, and relevant passkey, age-config, scanning, metadata, Syncthing, backup, subscription, and node-type runbooks.

## Actions

- Added `docs/README.md`, `docs/technical-status.md`, and `docs/user-guide.md`, then linked the docs index from the repository README.
- Added `docs/device-signing-design.md` based on Tephramesh's local device-signing design notes. Recorded the transferable trust model and the need to choose a post-quantum or hybrid signature algorithm for Rawrarr.
- Updated the SwarmSail/Tailscale plan: one Headscale tailnet per subscriber account, a subscriber Tailscale sidecar, SwarmSail-managed communication over the overlay, with Rawrarr permissions and age recipients remaining the data-authorization layer.
- Recorded the chosen backup MVP: manual-only, age-encrypted archives to public recipient(s), destination node/path in shared encrypted config, and no automatic retention deletion until policy is chosen.
- Kept the new backup and device-signing functions explicitly marked as planned rather than available UI features.

## Verification and remaining work

`git diff --check` passed, relative Markdown link targets were checked in 117 files, and `node --test scripts/tests/test_rawrarr_age_setup.mjs` passed (1 test). The age setup UI preview has its own task runbook. The app health endpoint had previously returned `ok` after its OrbStack rebuild; no authenticated browser visual check was performed during this documentation task. No SwarmSail tailnet, backup archive, device-signing key, or new subscription service is created by these docs. Product implementation and production runtime claims remain separate from this documentation task.
