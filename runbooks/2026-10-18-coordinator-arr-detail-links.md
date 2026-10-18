# Coordinator links to reachable Arr entries

## Objective and initial state

Show Radarr and Sonarr detail links beside years in the live coordinator Movies and TV tables, only when a browser-side probe confirms the saved service URL is reachable. The earlier link implementation targeted a separate UI renderer and was not visible in these tables. The worktree contains extensive concurrent changes; preserve them.

## Actions

- Include Radarr title slugs in the encrypted browser library response. Fetch live Radarr slugs through the selected metadata node and match them to decrypted movie records by TMDb ID. Sonarr series already carry title slugs.
- Probe each saved HTTPS Arr URL from the browser with no credentials, no referrer, and a three-second timeout. Add a keyboard-accessible external-link icon beside the year only after success and when a safe slug exists.
- Deploy only roarlay in the existing `rawrarr-local` project using `docker-compose.yml` and `docker-compose.tailscale-macbook.yml`; preserve the existing Tailscale Serve handlers.

## Verification

- Record focused browser fixture results, build, roarlay health, local and HTTPS health, and Tailscale Serve status here before completion.

## Recovery

Revert only the coordinator detail-link helper, markup styles, Radarr encrypted response slug field, and its browser mapping. Preserve the separate Rawrarr library renderer and all unrelated worktree changes.

- Focused verification: the Radarr browser link suite passed 5/5, including unreachable, timeout, and missing-slug cases. The new coordinator route/UI assertions passed 2/2. `node --check rust-arr/src/coordinator-workspace.mjs` and scoped `git diff --check` passed. Host Rust compilation was unavailable because the installed LLVM depends on the missing `libz3.5.1.dylib`; the deployment image build completed.
- The first Compose build did not replace the running container, so explicitly recreated only `roarlay`. The new container image ID matches `rawrarr:local`; container health is healthy. Local and HTTPS `/health` respond, and Tailscale Serve retains `/` -> `http://127.0.0.1:8080` plus `/direct-api` -> `http://127.0.0.1:8082`.
- The open browser is signed in but currently reports no selected metadata node after reload, so item rows and visible icons could not be confirmed in that tab. The coordinator will probe each service on its media page and only display the icon for reachable services with valid slugs.
