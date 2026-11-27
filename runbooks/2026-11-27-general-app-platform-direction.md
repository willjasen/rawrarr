# General app platform direction

## Objective

Define the product direction for a RunTipi-like catalog and lifecycle for general self-hosted apps, using Duplicati as a non-media example. Tracked in [issue #482](https://github.com/willjasen/rawrarr/issues/482).

## Initial state

The repository was clean on `main` at `ec88b64`, matching `origin/main`. Existing product direction already described file sharing beyond media, while managed-app work in #96, #112, and #116 concerned Arr provisioning, migration, and UI routes. No general app catalog or Duplicati integration was found.

## Decisions recorded

- General app catalog and lifecycle are planned and distinct from Rawrarr's node roles and media catalog.
- The owner reviews a versioned app definition and chooses an approved node before install or update. Catalog provenance and review are required before execution.
- Duplicati's database, settings, cache, and credentials are node-local by default. Source paths and destinations require explicit owner selection and narrowly scoped access.
- UI routing targets only the selected app instance and retains its authentication. Removal retains app data unless deletion is separately confirmed.
- Definitions need to describe architecture/node requirements, networks/ports, persistent paths, secrets, health, backup, upgrade, and rollback behavior.
- The upstream catalog, manifest format, signing/review process, secret provider, and node execution API remain undecided.
- Existing issues #96, #112, and #116 retain their current scopes; no implementation commitment was added to them.

## Actions

- Read the product direction, node type, Compose layout, design, and data-domain docs plus open issues #96, #112, and #116.
- Created issue #482 with scope, boundaries, acceptance criteria, and out-of-scope items.
- Updated `docs/product-direction.md` and `docs/DESIGN.md` with the planned catalog/lifecycle direction and Duplicati example.

## Verification

- Reviewed the diff and searched docs/source/runbooks for existing Duplicati or general app catalog concepts before editing.
- Documentation-only task; no runtime, data, schema, protocol, or deployment changes. No automated tests required.

## Adoption and rollback

No existing deployment adoption or data/protocol migration is needed. This change records product direction only. Reverting the documentation commit removes the direction text; issue #482 remains the external work tracker and should be updated if the direction changes.

## Final state

The docs describe a RunTipi-like general app catalog as planned, not implemented, and identify Duplicati as an example rather than a supported integration. Implementation remains future work under focused issue acceptance criteria.
