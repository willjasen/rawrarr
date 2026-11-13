# rawrarr

<p align="center">
  <img src="rust-arr/src/assets/rawrarr-logo.png" alt="Rawrarr logo" width="360">
</p>

rawrarr is an early self-hosted device-to-device file-sharing project, with media automation as its first focused use case. The repository contains a Rust web application and local Docker Compose setup. See [ROADMAP.md](ROADMAP.md) for completed work, outstanding work, and the recommended next milestone.

## License

Rawrarr is licensed under the GNU Affero General Public License v3.0 only. See [LICENSE](LICENSE) for the full text. Third-party components remain subject to their own licenses.

## Run locally

See the [user guide](docs/user-guide.md) for Docker Compose setup, passkey onboarding, age identity setup, Radarr configuration, and local scanning.

## Development workflow

For local development, the user guide's `rawrarr-local` commands describe an existing generic/legacy local Compose installation; they are not instructions for deploying to the current internal environment. Internal operators should use the current environment names and deployment procedures in [AGENTS.md](AGENTS.md). The internal coordinator UI is available at `https://rawrarr.risk-mermaid.ts.net/`; follow `AGENTS.md` for its deployment and verification workflow.

## Test

Run the Node, Playwright browser, and Rust suites in a uniquely named container cloned from the reusable dependency image:

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
