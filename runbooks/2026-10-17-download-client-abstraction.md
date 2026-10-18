# Download client abstraction

## Objective and initial state

Create a provider-neutral interface for download clients so rawrarr can later connect to BitTorrent, Usenet, and other transfer methods. The app currently has a Radarr status integration only; it has no download-client configuration or adapters. Existing client context is documented in [Gamarr's Prowlarr and qBittorrent setup](2026-10-16-gamarr-prowlarr-setup.md) and [archive extraction investigation](2026-10-16-gamarr-automatic-extraction.md).

## Actions

- Added `src/download_clients.rs` with a protocol-neutral adapter trait and normalized descriptors, capabilities, queue states/items, sources, priorities, submission receipts, and sanitized error categories.
- The request model accepts either a URI or an uploaded payload, covering link-based and file-based client APIs without provider-specific fields.
- Kept this change contract-only: no client credentials, settings, network requests, or concrete adapters were added. Updated the dashboard summary and README to make this implementation boundary clear.

## Verification

- `cargo fmt` and `cargo build --release --locked` passed with Rust 1.85.1.
- The local dashboard rendered the updated `Download providers` summary and reports zero configured providers.
- No download-client calls or download jobs were made.

## Final state and remaining work

rawrarr now has a compiled provider-neutral download-client interface. qBittorrent, SABnzbd, and other provider adapters, configuration, operational actions, and related UI remain future work.
