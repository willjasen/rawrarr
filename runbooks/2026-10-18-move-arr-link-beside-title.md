# Move Arr detail link beside title

## Objective and initial state

Move the external Arr detail icon from the Year column into the first Movies/TV column, after the title and year. The repository has extensive pre-existing changes; this task is scoped to the shared library renderer and its focused regression check.

## Actions

- Append the displayed year to the title text in the artwork/title cell.
- Place the accessible external-link icon after that title/year content while preserving the title/poster link to Arr.

## Verification

- `node --test scripts/tests/test_rawrarr_library_detail_link_layout.mjs` passed (1/1).
- `git diff --check` reported a pre-existing/mixed-file README EOF formatting issue; normalized the test index EOF and rechecked scoped files.
- No deployment was performed.

## Recovery

Revert only the renderer and focused test changes from this task. The additional year text and icon are both produced in `renderArrLibrary`.
