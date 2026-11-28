# Temporarily hide the Roarlay assistant

Issue: [#531](https://github.com/willjasen/rawrarr/issues/531)

## Objective

Hide `https://roarlay.rawrarr.net/#assistant` for now while retaining the assistant implementation for a later restoration.

## Change

- Removed the assistant from the workspace navigation.
- Marked the assistant view hidden and inert, and made the workspace router fall back to Overview when `#assistant` is requested.
- This UI-only change has no persistent data or protocol migration requirement. Restoring assistant access requires re-adding the navigation entry and removing the router restriction and inert marker.

## Verification

- `node --test scripts/tests/test_rawrarr_coordinator_workspace.mjs`
- `git diff --check`

## Deployment

The change takes effect when its published `main` UI bundle is deployed to the public Roarlay service. No deployment was performed as part of this source change.
