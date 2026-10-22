# Authenticate Rawrarr UI bundle updates

## Objective and issue

Require the coordinator to verify a publisher signature before activating downloaded UI bundles, retain the last verified UI when an update fails verification, and expose verification state to the signed-in owner. Subscriber nodes do not retrieve GitHub UI releases. GitHub issue [#22](https://github.com/willjasen/rawrarr/issues/22) tracks this work. Acceptance criteria: releases carry a detached signature produced from a protected Actions secret; the coordinator verifies it against a pinned public key before extraction/activation; invalid or unsigned bundles are rejected without disturbing current UI; a trusted server status is visible to the owner.

## Initial state

The updater downloads a ZIP and SHA-256 sidecar from the latest private GitHub release, checks the digest, then atomically switches `/ui/current` and retains `/ui/previous`. A checksum does not authenticate the publisher. Coordinator workspace already has a profile footer. See the existing [UI distribution runbook](2026-10-22-github-ui-distribution.md). The shared checkout has unrelated staged/unstaged work; this task uses a dedicated worktree.

## Progress

Created the Ed25519 publisher keypair. The private key is stored locally at `~/Library/Application Support/Rawrarr/ui-publisher-signing-key.pem` with mode 600 and is also configured as the GitHub repository secret `RAWRARR_UI_SIGNING_KEY`. Only the public key is committed as `rust-arr/ui-publisher-public-key.pem` and copied into the application image. The private key is never written into the repository or output.

The release workflow signs the exact ZIP bytes and publishes `ui-bundle.zip.sig`. The coordinator updater (gated by `RAWARR_ROLE=coordinator`) requires the ZIP, checksum, and detached signature; it verifies using the pinned public key before archive validation/extraction and only atomically swaps after verification. Subscriber roles do not start the GitHub updater. An installed marker is served by an authenticated Rust endpoint; the profile sidebar displays green “Signed” only when that endpoint confirms a verified active bundle and successful updater status. A rejected latest release appears in red while the previous verified UI remains active; missing status appears unverified.

Focused Python updater tests, JavaScript syntax validation, and `git diff --check` passed. Rust `cargo check --locked` could not start because host LLVM cannot load `libz3.5.1.dylib`. The GitHub secret is configured and its public key was verified against the locally protected private key. No coordinator deployment was requested or performed; a signed release will be published by Actions after this workflow change reaches `main`.
