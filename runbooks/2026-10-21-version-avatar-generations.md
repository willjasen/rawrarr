# Version and compare avatar generations

## Objective

Define a repeatable path for improving Rawrarr's generated dinosaur profile art through named renderer generations that can be compared over time without changing account identity.

## Initial state

`rust-arr/src/coordinator_avatar.rs` contains one deterministic SVG composition in `svg_for_digest`. The public age recipient is SHA-256 hashed in the browser; the resulting digest drives the illustration and is passed to authenticated PNG/JPEG XL endpoints. Existing tests cover deterministic output and some trait variation, but there is no fixed-seed visual contact sheet or renderer version selector.

The checkout already contains unrelated changes in coordinator, onboarding, tests, and runbooks. Those files were left untouched.

## Design direction

- Keep the account's digest as the stable identity key; renderer versions transform the same digest into different art.
- Give each renderer an explicit generation identifier (for example `classic-v1`, `polished-v2`) and preserve old renderers while comparison is useful.
- Keep high-level identity traits stable across versions when feasible: palette family and major species/silhouette traits can derive from a shared trait mapping, while line art, anatomy, shading, scene composition, and resolution may improve.
- Build a local comparison preview/contact sheet that uses a fixed set of sanitized digests and renders each digest under each available generation. Do not expose preview controls in the signed-in UI until a generation has been reviewed and selected for production.
- Make output dimensions and encoding quality explicit per generation. Higher pixel dimensions alone do not improve drawing quality; the silhouette and face should be designed to read clearly at the 38 px sidebar size as well as the 124 px profile size.
- Promotion is an intentional release decision. Do not call it automatic evolution, and do not rotate the identity seed to make an avatar appear to evolve.

## Visual exploration

Created three local, untracked preview sheets outside the repository. The first explored five broad illustration styles; the user selected the premium 3D direction. The second explored five refinements within that style; the user preferred the cinematic violet-blue lighting of option 2 and the sculpted dinosaur treatments of options 4 and 5. The third explored five prehistoric forest backgrounds; the user liked the flowering ginkgo grove of option 4 and asked for more varied tree heights. These sheets were previews only, not production renderers, and were kept in `/tmp` rather than committed.

Future generations may evolve dinosaur anatomy and species, head and snout shape, crest/horns, tail, wings, back plates/spikes, scales/markings, facial expression, pose, palette, eye color, lighting, tree species/heights/placement, undergrowth, rocks/water/mountains, weather/time of day, texture/shading, camera angle, and framing. Compare small controlled mutations against fixed seed sets, then retain human-selected improvements as the next parent generation.

## Suggested implementation sequence

1. Extract the current renderer as the immutable baseline generation.
2. Add a generation dispatch API internally and retain the current endpoint as the selected production generation.
3. Add a CLI or script for fixed-seed PNG/contact-sheet generation; keep outputs under ignored `logs/` or a dedicated ignored preview directory.
4. Implement a second SVG composition focused on stronger dinosaur anatomy, clearer head/eye expression, cleaner silhouette, balanced framing, and richer but restrained shading.
5. Review comparisons at both small and large display sizes; select the preferred generation, update docs, and add behavioral tests for determinism and renderer dispatch.

## Verification and remaining work

This task recorded design direction and completed visual concept comparisons only. No renderer, UI, preview utility, or deployment changed. Implementation, a reusable comparison tool, focused checks, and production promotion remain outstanding.
