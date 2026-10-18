# Add identity-derived background scenes to dinosaur avatars

## Objective and initial state

The profile dinosaur SVGs used a solid circular color field with subtle circles. Add recognizable illustrated scenery while retaining the existing locally generated SVG and deterministic public-age-ID features.

## Actions

- Added volcanic hills with a lava flow, a forest scene with trees, and a hilly sunset scene behind each dinosaur.
- Chose the scene deterministically from an otherwise unused byte in the avatar digest so existing dinosaur traits remain unchanged.
- Added a focused test for repeatable selection across all three backgrounds and updated design and test documentation. The avatar remains an inline SVG data URL, not a JPEG XL or uploaded image.

## Verification

- `node --test --test-name-pattern='dinosaur' scripts/tests/test_rawrarr_coordinator_workspace.mjs`: 5 passed.
- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.
- Confirmed port 7297 belonged to Rawrarr and port 7298 to OrbStack. Waited for the roarlay deployment lock, then rebuilt only `roarlay` in `rawrarr-local`.
- `rawrarr-local-roarlay-1` reports healthy. Local and tailnet `/health` returned HTTP 200. The deployed workspace asset contains the scenery generator. Tailscale Serve routes `/` to `http://127.0.0.1:8080` and retains `/direct-api` at `http://127.0.0.1:8082`.

## Final state and remaining work

- Dinosaur profile images now render one of three public-ID-derived scene backgrounds locally in the browser. The deployment lock was released after health and route verification.
