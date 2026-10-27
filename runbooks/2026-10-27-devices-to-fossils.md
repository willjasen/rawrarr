# Rename Devices to Fossils

## Scope

Rename the coordinator's user-facing Devices page to Fossils, update its browser URL, and generate new fossil node IDs with the `fossil-` prefix. Existing node IDs and the legacy `#devices` bookmark remain valid. Tracked by [issue #215](https://github.com/willjasen/rawrarr/issues/215).

Issue #215 acceptance criteria:

> **Title:** Rename the Devices workspace page to Fossils
>
> **Body:** Replace the coordinator UI's Devices page label with Fossils, move the page route to `/#fossils` while keeping existing bookmarks functional, and issue new subscriber node IDs with a `fossil-` prefix. Keep existing enrolled IDs and their route mappings valid.
>
> **Acceptance criteria:** The navigation, page heading, onboarding continuation, and in-context help say Fossils; `/#fossils` opens the page and `/#devices` redirects; newly generated IDs use `fossil-` and are accepted by Compose generation; existing IDs continue to work.

## Changes

- Renamed the coordinator page, navigation item, onboarding return link, help text, and guide references.
- Updated the page hash to `#fossils` and preserve `#devices` through a redirect.
- Updated onboarding ID generation and Compose validation to use `fossil-`; existing stored IDs are not migrated.
- The generated onboarding browser bundle was updated to match source. `npm run build:subscriber-onboarding` could not run because `esbuild` is not installed in this worktree.

## Verification

- `node --test scripts/tests/test_rawrarr_devices_add_button.mjs scripts/tests/test_rawrarr_device_names.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs scripts/tests/test_rawrarr_subscriber_compose.mjs` — passed (39 tests).
- `git diff --check` — passed.
- `npm run build:subscriber-onboarding` — not run successfully because the worktree has no installed `esbuild` executable; the checked-in bundle was updated and source/bundle IDs are covered by the focused test.
