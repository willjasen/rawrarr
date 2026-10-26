# Check coordinator UI releases every five minutes

## Objective

Return the coordinator UI updater to a five-minute polling interval. The updater remains restricted to coordinator-role containers. GitHub issue [#169](https://github.com/willjasen/rawrarr/issues/169).

## Changes

- Set the updater default and minimum poll interval to 300 seconds.
- Added focused regression coverage for the default, floor, and a longer requested interval.
- Updated the script test index.

## Verification

- `python3 scripts/tests/test_ui_bundle.py -v` passed all 6 tests, including the five-minute default/floor and longer custom interval.
- `git diff --check` passed.

## Deployment

No deployment was requested or performed. The change must reach GitHub `main` before deployment under the project instructions.
