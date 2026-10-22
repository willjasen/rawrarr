# Check GitHub UI releases every minute

## Objective

Have the existing UI updater check GitHub once per minute and install a newer signed UI bundle automatically. GitHub issue: [#29](https://github.com/willjasen/rawrarr/issues/29).

## Initial state

The application updater checked the private GitHub release every five minutes. The latest UI bundle mechanism was observed updating the internal coordinator, and Will reported that the milliseconds-only status had appeared. The coordinator's running application image still has the five-minute default until this code change is deployed in a new image.

## Actions

- Changed the updater default to 60 seconds and retained a hard minimum of 60 seconds.
- Added regression coverage for the default, minimum, and a longer requested interval.

## Verification

- `python3 scripts/tests/test_ui_bundle.py -v` passed all 6 tests, including the one-minute default and minimum interval regression.
- `python3 -m py_compile rust-arr/update_ui_bundle.py` passed.
- `git diff --check` passed.

## Deployment and recovery

This is updater code included in the application image, not a UI bundle asset. It takes effect on the internal coordinator after a normal roarlay application image deployment. No Mac mini connection or deployment was performed for this source change. To recover, restore the updater default and minimum interval to the previous five-minute behavior, then deploy the reverted application image.
