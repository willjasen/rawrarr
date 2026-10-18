# Remove the profile avatar status sentence

## Objective and initial state

Remove the success sentence under the coordinator profile avatar while preserving loading and error feedback during public age ID retrieval.

## Actions

- Clear the avatar status region after successful generation and added a regression assertion that the sentence stays absent.

## Verification

- The profile avatar and dinosaur spike tests passed with `node --test --test-name-pattern='signed-in user profile generates a dinosaur|dinosaur back spikes' scripts/tests/test_rawrarr_coordinator_workspace.mjs`; JavaScript syntax and `git diff --check` passed.
- The complete workspace suite ran 12 tests: 11 passed and the unrelated Sonarr settings assertion failed because it expects the old `fields.url` request-body expression; current code uses `copy.url`. No Sonarr code or assertion was changed.
- Confirmed port 7297 was unoccupied and port 7298 remained owned by OrbStack's Tailscale namespace. Rebuilt only `roarlay` in `rawrarr-local` with the coordinator and MacBook Tailscale overlays.
- The coordinator reports healthy; local and tailnet `/health` returned HTTP 200. The deployed workspace module clears the profile status text after avatar generation. Tailscale Serve still routes `/` to `http://127.0.0.1:8080` and preserves `/direct-api` at `http://127.0.0.1:8082`.

## Final state and remaining work

- The profile success sentence is gone. Loading and error feedback remain available while retrieving the public age ID.
