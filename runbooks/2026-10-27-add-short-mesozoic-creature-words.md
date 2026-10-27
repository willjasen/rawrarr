# Add short Mesozoic creature words

## Objective and initial state

Add the approved short dinosaur names and familiar short forms to the generated account-name creature pool. The generator combines a first word, a creature word, and an account-ID suffix, and all combinations must remain within the 32-character username limit.

Related issue: [#206](https://github.com/willjasen/rawrarr/issues/206).

## Changes

- Added `alba`, `ankylo`, `cera`, `dilo`, `giga`, `mosa`, `ptera`, `rex`, `stego`, `theri`, and `trike` to the creature pool.
- Kept existing `dino` and `raptor` entries without duplicating them.
- Retained the existing all-pairs username-length test.

## Verification

- Tests were not run.

## Final state

The creature pool now contains 39 unique entries. Names remain deterministically generated from the expanded pool; profile labels can therefore change when refreshed after this vocabulary update.
