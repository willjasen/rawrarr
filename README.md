# rawrarr

rawrarr is an early self-hosted device-to-device file-sharing project, with media automation as its first focused use case. The repository contains a Rust web application and local Docker Compose setup. See [ROADMAP.md](ROADMAP.md) for completed work, outstanding work, and the recommended next milestone.

## Run locally

See the [user guide](docs/user-guide.md) for Docker Compose setup, passkey onboarding, age identity setup, Radarr configuration, and local scanning.

## Development workflow

Code changes are made and verified before updating the persistent `rawrarr-local` environment. After UI changes, rebuild that environment so hands-on review at http://localhost:7297 uses the current implementation. The application image uses BuildKit Cargo caches to reuse downloaded crates and Rust build artifacts across source changes.

## Test

Run the Node and Rust suites in a uniquely named container cloned from the reusable dependency image:

```sh
python3 scripts/tests/run_rawrarr_tests_in_docker.py
```

See [TESTING.md](TESTING.md) for the test-base lifecycle and [the test index](scripts/tests/README.md) for suite coverage.

## Project documentation

- [Roadmap](ROADMAP.md)
- [Documentation index](docs/README.md)
- [Device and service architecture](docs/device-architecture.md)
- [Product and interface design](docs/DESIGN.md)
- [User guide](docs/user-guide.md)
- [Technical boundaries](docs/technical-status.md)
- [Runbooks](runbooks/README.md)
