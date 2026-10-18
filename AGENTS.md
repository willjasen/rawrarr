# Rawrarr project instructions

- Keep all Rust code memory-safe. Do not introduce `unsafe` blocks.
- Keep the service node separate from subscriber nodes. Never place private age identities, passkeys, credentials, media, or subscriber app-data in the service-node or relay code paths.
- Treat `/shared` as synchronized application data, `/media` as node-local media, and `/data` plus PostgreSQL as node-local state. Never commit runtime data, local databases, secrets, `.env` files, or age identities.
- Preserve existing user changes. Before editing, inspect `git status --short --branch`; stage explicit files and review the staged diff before committing.
- For substantive implementation or operations work, create a dated task runbook in `runbooks/` and update its index. Keep design decisions in `docs/DESIGN.md`, architecture in the relevant technical document, and mark unimplemented behavior as planned.
- Maintain tests for meaningful behavior changes under `scripts/tests/`; update their index. Run focused checks for the affected behavior and state clearly what was not verified.
- Keep Rawrarr-specific project work in this repository. Do not import unrelated Arr-services scripts, operational runbooks, service configuration, or media files.

- After UI changes, update the persistent `rawrarr-local` web service on port 7297 and check its health. Preserve its existing data, configuration, and the user’s browser windows.
