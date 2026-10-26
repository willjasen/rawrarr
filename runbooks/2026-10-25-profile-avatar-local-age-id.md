# Generate the profile avatar from the saved age ID

## Objective and initial state

Track [issue #78](https://github.com/willjasen/rawrarr/issues/78). Settings showed a loading message while waiting for a metadata node, despite the coordinator workspace restoring the owner's private age ID in browser-local storage. A fallback existed only after a node fetch failed; when no eligible node was available, the function returned without drawing an avatar.

## Actions

- Derive the public age recipient locally from the saved browser age ID using the existing pinned age library, then hash it for the deterministic profile avatar.
- If no age ID is available or derivation fails, render the existing deterministic name-based fallback immediately. Do not request a node recipient for this profile flow.
- Keep the private age ID in browser memory and never include it in coordinator or node requests.
- Add regression coverage and rebuild the embedded browser bundle as needed.

## Verification

- `npm ci --ignore-scripts` and `npm run build` in `rust-arr/web-export` completed; the pinned `esbuild` bundle was regenerated. npm reported one moderate advisory in the dependency tree; no dependency versions were changed.
- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (24/24).
- `node --check rust-arr/src/coordinator-workspace.mjs` and `git diff --check` passed.

## Final state and remaining work

The profile avatar derives its stable seed from the saved private age ID entirely in the browser. When no ID is saved or derivation fails, a stable name-derived avatar renders without a node request. The private ID is not included in network requests. No deployment or authenticated browser visual check was requested or performed.
