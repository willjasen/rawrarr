# Add the coordinator workspace sidebar

## Objective

Keep the working coordinator metadata-node workspace while adding navigation that matches the Rawrarr dashboard style.

## Changes

- Added a responsive left sidebar with Overview, Movies, Series, Configuration, and Settings navigation.
- Kept the coordinator movie workspace and existing visual palette; the current view follows the URL hash, so `#movies` opens Movies.
- Series and configuration/settings entries identify the existing workspace limitations without implying that these node-specific controls are implemented here.

## Verification

- `node --check rust-arr/src/coordinator-workspace.mjs` passed. `git diff --check` passed for the task files.
- The coordinator HTTPS health route returns HTTP 200, and Serve status confirms `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`.
- The UI rebuild was not run: `.container-rebuild-coordinator.lock` already exists in Application Support and belongs to another deployment task. The marker was left untouched.

## Remaining work

- Verify the layout in the user's authenticated browser session.
