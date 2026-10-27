# Move rawrs Talk to a floating voice control

Issue: [#218](https://github.com/willjasen/rawrarr/issues/218)

## Change

- Added a persistent bottom-left “Talk to rawrs” launcher across coordinator workspace pages.
- Moved microphone controls, transcript, and spoken answer into a compact dismissible panel; the rawrs page continues to provide text chat.
- The launcher and panel adapt their position for the sidebar and small screens. Escape and the close button dismiss the panel.

## Verification

- `node --test scripts/tests/test_rawrarr_ai_chat.mjs scripts/tests/test_rawrarr_coordinator_workspace.mjs` — passed (39 tests).
- Live coordinator/browser behavior was not exercised.
