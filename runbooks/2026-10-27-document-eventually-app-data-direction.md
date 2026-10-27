# Document the “eventually” app-data direction

## Objective and initial state

Make the product idea of “eventually” explicit in Rawrarr's high-level documentation: useful app data should be designed to become portable and sync between approved nodes over time, while documenting that current sync applies only to `/shared` and does not replicate databases or all node-local state. The existing `.ev` folder suffix already means a planned or future placeholder, and the product direction discussed portable records, but the meaning was not stated consistently across the top-level design and data-boundary documents.

Reviewed [product direction](../docs/product-direction.md), [product design](../docs/DESIGN.md), [data domains](../docs/data-domains.md), [technical status](../docs/technical-status.md), and prior [data-domain runbook](2026-10-18-rawrarr-data-domains.md).

## Actions

- Added the “eventually” idea and its current-state caveat to the product direction, product design, data-domain registry, and technical status.
- Clarified that portability applies to explicitly designed records with defined encryption, authorization, conflict, and recovery behavior; it does not mean arbitrary database replication.
- Added the product-direction document to the documentation index and this task to the runbook index.

## Verification

- Documentation-only change; no runtime behavior or synchronization configuration changed.
- `git diff --check` passed.
- Relative links in the added runbook and documentation index resolve.

## Final state and remaining work

The high-level docs now describe “eventually” as a future capability for portable app records between approved peers, and distinguish that direction from current `/shared` Syncthing replication. No sync behavior was implemented or enabled by this task.
