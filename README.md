# rust-arr

Initial Rust service skeleton for a Radarr-inspired application. It exposes `GET /health` and is packaged for Docker. No media paths, database, or credentials are configured yet.

## Local development

Requires Rust 1.85 or newer.

```sh
cargo run --manifest-path rust-arr/Cargo.toml
curl http://127.0.0.1:8080/health
```

Set `RUST_ARR_LISTEN_ADDR` to change the listen address.

## Docker

From the repository root:

```sh
docker compose up --build
curl http://127.0.0.1:8080/health
```

The image uses a multi-stage build and runs the application as UID 10001. Persistent storage and application configuration will be added with later features.

## Memory safety policy

The application crate has `#![forbid(unsafe_code)]`, and its Cargo lint configuration rejects unsafe blocks and declarations. This enforces the policy in code written in this crate. Rust dependencies are separate code and may use `unsafe` internally; review their safety notes and keep the dependency set small before expanding the application. A dependency audit is not a proof that all transitive code is memory safe.
