# Prompt for age identity during Rawrarr setup

## Objective and initial state

Make first-run age identity setup visible immediately, including while the Movies view is selected. The identity form existed inside the Overview settings panel, which the Movies navigation hides. Related references: [encrypted config](2026-10-17-rawrarr-age-encrypted-config.md) and [Radarr movie library](2026-10-17-rawrarr-radarr-movies.md).

## Actions

- Moved the single age identity form into a prominent first-run panel above the workspace views.
- The panel remains visible until encrypted settings are unlocked, prompts creation or import, and changes its copy to request the existing identity when encrypted config is present.
- No identity is generated automatically; private key creation still requires the user to select the button.

## Verification

- Rust 1.85 formatting, locked `cargo check`, and locked release build passed; `git diff --check` passed.
- Restarted only the current task's unconfigured development server on `127.0.0.1:8082`; its health endpoint returns `ok`.
- Reloaded the open UI and verified the prominent first-run panel, create/import controls, and explanatory private-key backup text are visible on initial load. The page still displays the Movies navigation and Radarr setup controls.
- The existing configured server on `127.0.0.1:8080` was left running.

## Final state and remaining work

First-run users are now prompted to create or import a post-quantum age identity before using encrypted settings. Identity generation remains an explicit user action. The new app build is running at `http://127.0.0.1:8082/`.
