# Guide new ID creation in a pop-up

## Objective

After a user creates a new age ID, guide them through copying it, confirming they copied and saved it, entering three requested characters, and opening the main site.

## Initial state

The setup page already supported ID generation, clipboard copy, backup acknowledgement, and a random three-character challenge, but these controls shared the page and required a separate unlock click. Related context: [private key copy verification](2026-10-18-rawrarr-private-key-copy-verification.md) and [age identity copy screen](2026-10-18-rawrarr-age-identity-copy-screen.md).

## Changes

- Open a modal after successful ID generation and move its copy, acknowledgement, and challenge controls into that guided flow.
- Reveal the acknowledgement only after clipboard copy succeeds. Reveal the character prompt only after acknowledgement.
- Automatically submit a generated identity and navigate to `/` after the correct three characters. Keep the explicit **Unlock vault** action for pasted identities.
- Extend the DOM harness to cover the sequence, navigation, and generated/pasted identity behavior; update the product design guide and runbook index.

## Verification

- An intermediate `python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack` run against the dependency-only image exposed two runner issues: Docker nested copied source directories, and Cargo reused the placeholder application's test binary. The runner now copies directory contents into the clone and clears only the Rust package artifacts before testing.
- Final verification with `python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack --rebuild-base` rebuilt the reusable base once with dependency artifacts only, then ran a uniquely named container from that image. All six Node tests and all nine Rust unit tests passed; the PostgreSQL and peer-sync integration checks remain separate and were reported as skipped by the runner.
- `git diff --check` passed.

## Final state

Generated IDs use the guided pop-up and automatically continue after the challenge succeeds. Each uniquely named runner is cloned from `rawrarr-testing-base:local`; the image stores dependencies, while current project files and the test entrypoint are copied into each runner's writable layer. The base rebuilds only when missing or when `--rebuild-base` is requested.
