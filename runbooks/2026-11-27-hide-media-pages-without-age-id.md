# Hide media pages until age ID is available

## Objective

Keep Discover, TV, Movies, and Music free of confusing node/configuration errors until the signed-in browser has an age private ID. Tracked by [issue #517](https://github.com/willjasen/rawrarr/issues/517).

## Initial state

The coordinator launches Arr settings, library, or Prowlarr requests when these views open, regardless of whether the browser has imported an age ID. TV and Music can therefore display vault/request errors; Discover reports them as Prowlarr errors. Movies also has cached rendering paths that must remain age-ID-gated.

## Actions

- Add a locked state to the four media pages with a clear route to age-ID settings.
- Gate page data requests and clear/hide stale content when the ID is removed.
- Add focused regression coverage for locked and unlocked navigation.
- The Movies page retains its existing age-ID entry. TV, Music, and Discover show an age-ID gate with a Settings button; age-encrypted configuration and library calls are skipped until the browser has an ID.
- No persistent data, database, or protocol compatibility changes were needed.

## Verification

- In the isolated OrbStack Playwright image, `node --test scripts/tests/test_rawrarr_coordinator_sonarr_settings.mjs scripts/tests/test_rawrarr_age_id_cross_tab_browser.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs` passed (48 tests). Coverage includes no media API calls before import, gated/unlocked states, desktop/mobile widths at 1280×800 and 375×812 without horizontal overflow, cross-tab removal, and source-backed guards.
- The first manual invocation of the source-backed workspace tests lacked the root Compose and `deploy/` fixtures; copying the documented fixture inputs resolved that setup issue.
- `node --check` passed for the coordinator module and updated browser test; `git diff --check` passed.

## Recovery

Revert the task commit to restore the previous media-page request behavior. No persistent data or protocol migration is introduced.
