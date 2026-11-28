# Remove obsolete UI bundle releases

## Objective

Remove the historical standalone UI bundle releases now that Rawrarr serves its UI from the application container. Tracked by [issue #505](https://github.com/willjasen/rawrarr/issues/505).

## Initial state

- The repository is at the current `origin/main` revision in a dedicated worktree.
- GitHub reports 191 releases with `ui-*` tags, each containing release assets. The latest is `ui-27d611fba46de0cbdcf143d59d9eaaf3eb031f36`.
- No current GitHub Actions workflow publishes UI bundle releases. `scripts/tests/test_image_ui_authority.py` checks that container startup and Compose no longer configure the GitHub UI updater.
- Non-UI releases are outside this cleanup scope.

## Actions

- Deleted all 191 releases whose tag began with `ui-`, including assets and release tags. The inventory contained no non-UI releases.

## Verification

- Re-queried every GitHub release page; zero release tags remain. The initial inventory also confirmed there were zero non-UI releases to preserve.
- `python3 -m unittest scripts.tests.test_image_ui_authority -v`: passed 4/4.
- `git diff --check`: passed.

## Final state

The obsolete standalone UI releases and tags are removed. UI assets continue to ship through the application container image. No application deployment or persistent-data migration was needed.
