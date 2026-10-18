# rawrarr

rawrarr is an early self-hosted application for device-to-device sharing of encrypted app data, with media automation as its first use case. The current Rust/Axum prototype provides local passkey sign-in, an age-encrypted settings vault, a Radarr integration, local media scanning, and Syncthing configuration. It is not yet a production multi-node service: remote node claiming, shared passkey accounts, hosted subscriptions, and the blind relay remain planned.

## What works today

- Register and sign in to a local account with a passkey stored in the node's PostgreSQL database.
- Create a post-quantum age identity or paste one into the masked setup field. For a new identity, copy it from the pop-up, confirm the copy, enter three requested characters, and continue to the dashboard. The app keeps the private identity in browser memory only; it is not persisted or synchronized.
- Store Radarr settings as age-encrypted `/shared/config.aged`, and display movie inventory and connection status.
- Scan an explicitly mounted media directory and create encrypted metadata and artwork. Media stays on the storage host.
- Configure Syncthing peers for the shared app-data folder. The web app streams dashboard updates over `/api/events` using server-sent events.
- Run the Rust service and PostgreSQL with Docker Compose. `/health` supports container health checks.

The current Compose setup is a local development/self-hosting example, not a hosted deployment portal or generated subscriber install. See the [user guide](docs/user-guide.md) for setup and operations.

## Run and test

For local development and deployment, follow the [user guide](docs/user-guide.md). The repository's test runner creates a uniquely named container from `rawrarr-testing-base:local`, copies in the current source, and prints results to its logs:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py
```

The base image caches external tools and Rust dependencies. Source changes do not rebuild it; use `--rebuild-base` after changing the toolchain or dependencies. See [TESTING.md](TESTING.md) for details and the [test index](scripts/tests/README.md) for coverage.

## Project documentation

- [Documentation index](docs/README.md)
- [Device and service architecture](docs/device-architecture.md)
- [Product and interface design](docs/DESIGN.md)
- [Technical status](docs/technical-status.md)
- [Product direction](docs/product-direction.md)
- [Runbooks](runbooks/README.md)
