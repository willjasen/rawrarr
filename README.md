# rawrarr

rawrarr is an early Radarr-focused Rust application. The root route (`/`) serves a responsive dashboard, `/health` is a service health check, and `/api/radarr/status` is a read-only Radarr connection status endpoint. Movie inventory, media paths, and a database are not implemented yet.

## Local development

Requires Rust 1.85 or newer.

```sh
cargo run --manifest-path rust-arr/Cargo.toml
open http://127.0.0.1:8080/
curl http://127.0.0.1:8080/health
```

Set `RUST_ARR_LISTEN_ADDR` to change the listen address.

Configure the optional Radarr integration with `RADARR_URL` (for example, `http://radarr:7878`) and `RADARR_API_KEY`. The key is read from the environment and never returned in responses or error messages. With Compose, set these in an untracked `.env` file in the repository root or in the shell environment. When configured, `GET /api/radarr/status` reads Radarr's `/api/v3/system/status` endpoint. The route returns `503` when Radarr is not configured and `502` if Radarr does not return a successful response.

## Download client architecture

`src/download_clients.rs` defines the provider-neutral `DownloadClient` contract and normalized queue, source, receipt, capability, and error types. It is intended to support different transfer protocols, including BitTorrent and Usenet, without exposing provider-specific API models to the rest of rawrarr. The contract currently covers connection checks, queue reads, and submission. It has no concrete adapters and does not contact download clients. Provider configuration, pause/resume/remove actions, and UI integration will be added with their respective implementations. Source URIs may contain credentials or signed parameters and must never be logged.

## Docker

From the repository root:

```sh
docker compose up --build
curl http://127.0.0.1:8080/health
```

The image uses a multi-stage build and runs the application as UID 10001. Provide `RADARR_URL` and `RADARR_API_KEY` through the deployment environment. Persistent storage and application configuration will be added with later features.

## Memory safety policy

The application crate has `#![forbid(unsafe_code)]`, and its Cargo lint configuration rejects unsafe blocks and declarations. This enforces the policy in code written in this crate. Rust dependencies are separate code and may use `unsafe` internally; review their safety notes and keep the dependency set small before expanding the application. A dependency audit is not a proof that all transitive code is memory safe.
