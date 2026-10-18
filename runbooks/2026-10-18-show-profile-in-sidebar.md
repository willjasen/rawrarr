# Show the signed-in profile in the sidebar

## Objective and initial state

Replace the coordinator sidebar's early-access tagline with the signed-in user's profile picture and display name. The coordinator workspace already loaded the account name and rendered a deterministic dinosaur avatar from the public age ID. The checkout contained extensive unrelated changes, so work was limited to the coordinator workspace and its focused regression assertion.

## Actions

- Replaced the tagline with a compact avatar and account-name row in the fixed sidebar footer.
- Connected it to the existing profile and avatar loading flow; profile errors show a neutral unavailable label.
- Extended the coordinator workspace regression test.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (13/13), and `git diff --check` passed.
- The first rebuild used only the base Compose file and failed because its port mapping collided with the running Tailscale sidecar on 127.0.0.1:7298. Retried with `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml`; roarlay is healthy, local and tailnet `/health` both return HTTP 200, and Serve still proxies `/` to `http://127.0.0.1:8080` while preserving `/direct-api` at `http://127.0.0.1:8082`.
- The roarlay deployment lock was released after health and route verification.

## Final state and remaining work

- The roarlay sidebar now displays the account name and generated dinosaur avatar using the existing signed-in profile flow. The authenticated UI was not interactively opened in a browser during this task.
- No commit was created because the relevant workspace files and runbook index already contain unrelated user changes; staging those whole files would include work outside this task. All changes remain in the working tree.
