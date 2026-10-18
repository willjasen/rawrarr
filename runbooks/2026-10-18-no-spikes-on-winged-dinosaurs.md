# Remove back spikes from winged dinosaurs

## Objective and initial state

The public-age-ID-derived dinosaur generator independently selected optional wings and zero to three back spikes, so some winged variants had both features.

## Actions

- Made the generated spike count zero whenever the deterministic wing trait is present. Non-winged dinosaurs retain their existing zero-to-three spike variation and sizing.
- Extended the tail-and-wings regression to assert every generated winged dinosaur has no back spikes.

## Verification

- `node --test --test-name-pattern='dinosaur' scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 4 passed.
- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Confirmed port 7297 was unoccupied and port 7298 belonged to OrbStack. Rebuilt only `roarlay` in the `rawrarr-local` Compose project.
- `rawrarr-local-roarlay-1` reports healthy; local and tailnet `/health` returned HTTP 200. Tailscale Serve still maps `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.

## Final state and remaining work

- Winged dinosaurs render without back spikes; non-winged spike variants are unchanged. Deployment lock released after health and route verification.
