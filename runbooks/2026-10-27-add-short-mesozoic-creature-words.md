# Add short Mesozoic creature words

## Objective and initial state

Add the approved first-word candidates and short dinosaur names to the generated account-name pools. The generator combines a first word, a creature word, and an account-ID-derived suffix while respecting the 32-character username limit.

Related issue: [#206](https://github.com/willjasen/rawrarr/issues/206).

## Changes

- Added `alba`, `ankylo`, `cera`, `dilo`, `giga`, `mosa`, `ptera`, `rex`, `stego`, `theri`, and `trike` to the creature pool.
- Added `arctic`, `briny`, `buried`, `cavern`, `chalky`, `coastal`, `dusky`, `earthy`, `eocene`, `fossilized`, `igneous`, and `ironclad` to the first-word pool; `jurassic` was already present.
- Kept existing `dino` and `raptor` entries without duplicating them.
- Retained the existing all-pairs username-length test.

## Verification

- Tests were not run.

## Final state

The first-word pool now contains 41 unique entries and the creature pool contains 39, for 1,599 possible word pairs before generator length filtering. Names remain deterministically generated from the expanded pools; profile labels can therefore change when refreshed after this vocabulary update.
