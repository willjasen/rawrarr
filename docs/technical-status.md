# Rawrarr technical status

> Updated container architecture (2026-10-18): central **coordinator** owns passkeys/authentication and node coordination; **storage** and **metadata** serve the app, Radarr, and shared data. Only storage reads media. Metadata replaces shard; the MacBook app is metadata. See [deployment and Mac mini setup](container-deployment.md), which supersedes older role/startup statements below. Owner-approved pending enrollment is implemented; browser device-code linking remains planned.

Last reviewed: 2026-10-18.

The [roadmap](../ROADMAP.md) is the source of truth for completed and outstanding work. This page records the architectural constraints that apply across current and future features. See the [user guide](user-guide.md) for current usage and the [runbook index](../runbooks/README.md) for dated implementation evidence.

## Data and trust boundaries

- `/media` is a node-local, read-only media mount. Rawrarr, Syncthing, Tailscale, and the relay must not copy or proxy those media bytes as part of app-data operations.
- `/shared` is the Syncthing app-data root. `/metadata` aliases `/shared/metadata`. Metadata is grouped by type and content kind, such as `/metadata/movies/text/` and `/metadata/movies/artwork/`; generic library sidecars are under `/metadata/library/text/`. `config.aged` and metadata ciphertext sync, while logs are compressed and allowlisted but not age-encrypted individually. Local Compose can maintain a mode-`0600` Radarr settings mirror in node-local `/data`; production disables and removes that mirror.
- `/data` and the live PostgreSQL database are local to each container/node. A Syncthing copy of files is not database replication or quorum.
- Production Syncthing peer and folder setup is gated on explicit owner approval of the joining Rawrarr node. The Mac mini relay pairing was an operational test; it does not imply automatic sharing for later nodes. Public Syncthing relays are the fallback when a direct connection is unavailable.
- The operator's relay/service node may store account credentials, public age recipients, billing/license status, and opaque instance/device IDs. It must never receive private age identities or subscriber app-data plaintext.

## Current-source and runtime status

These descriptions reflect the current checkout and the local runtime deployment verified on 2026-10-18. Historical runbook tests and deployment results apply only to their recorded task.

The coordinator root shows passkey sign-in while anonymous and the owner workspace after sign-in. It lists owner-authorized nodes and renders a bounded movie metadata view by relaying only age ciphertext to the browser for local decryption. General node-page and plaintext API relay requests are rejected. The browser keeps the age private ID in tab-scoped `sessionStorage` and clears it on sign-out. The three-record view remains a proof of concept; see [blind coordinator content flow](blind-coordinator-content.md). `/health` remains a separate HTML status page and checks web-process responsiveness only. A successful response does not establish database, integration, or peer readiness.

The local `rawrarr-local` coordinator and metadata containers were rebuilt and recreated for the workspace change. Verification passed for local and Tailscale-root sign-in pages, `/health`, coordinator-to-metadata network reachability, and a recent metadata heartbeat. A disposable Docker/PostgreSQL/Chromium integration verified coordinator passkey sign-in, automatic metadata selection and page display, account-linked passkey registration, node-scoped sessions, enrollment approval, and fail-closed behavior. A real interactive passkey ceremony at the live Tailscale origin remains a user action.

## Verification inventory

Automated checks are indexed in [scripts/tests/README.md](../scripts/tests/README.md). Recent documented checks include Rust unit tests, a Node DOM test for age setup behavior, disposable PostgreSQL API tests, and isolated two-node Syncthing tests with OrbStack or local Syncthing processes. Not every test runs on every change. Check each test's prerequisites and its runbook before treating historical results as proof of current runtime health.

### Sonarr initial integration

Implemented authenticated Sonarr status, encrypted connection settings (read/save/clear), and read-only series inventory using the v3 API (Sonarr 3/4). The coordinator Settings form sends the age identity and Sonarr credentials directly from the browser to the selected metadata node; the coordinator relay does not accept or forward Sonarr settings. Shared library columns serve Radarr and Sonarr, and both pages use one configurable four-card summary renderer with service-specific labels and detail text. The coordinator TV page reads the configured Sonarr series inventory directly from the selected metadata node and displays series, episode counts, TVDB ID, monitoring, year, quality profile, disk size, and the size of the encrypted TV metadata subtree alongside total shared metadata size. Metadata nodes scrape uncached Sonarr series metadata and posters into separate HMAC-named age-encrypted `tv/text/` and `tv/artwork/` folders sequentially after first connection and every six hours. The TV page hides completed-backfill and successful-load messages once every eligible poster is saved, even while the worker checks for newly added series. Metadata search and episode mutations remain planned.
