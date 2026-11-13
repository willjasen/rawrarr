# Public subscriber image pulls

Issue: [#34](https://github.com/willjasen/rawrarr/issues/34)

## Goal

Allow user-operated fossil and boneyard nodes to pull `ghcr.io/willjasen/rawrarr:main` without a GitHub account, token, or prior `docker login`. Coordinator-side GitHub credentials and behavior are outside this change.

## Changes

- Marked subscriber Compose guidance as an anonymous pull from the public GHCR package.
- Updated the focused Compose regression test, deployment documentation, node distribution documentation, and roadmap.
- Renamed the image publishing workflow to reflect public distribution. GitHub Actions still uses its scoped `GITHUB_TOKEN` to publish the package.

## Adoption and compatibility

This changes image distribution only. It does not change application state, database schemas, encrypted configuration, or node/coordinator protocols, so no migration is required. Existing subscriber volumes and Compose files remain compatible. The package is public independently of repository visibility, and newly generated Compose files can pull its current `main` image anonymously.

## Completed and verified

- Changed `ghcr.io/willjasen/rawrarr` from private to public in GitHub package settings. Coordinator-side GitHub token behavior remains unchanged.
- The anonymous GHCR token, `main` multi-architecture index, both architecture manifests, and 20 config/layer blobs were retrieved or checked without credentials. The index digest is `sha256:e1434b6b764c58a71ad6d9c94833dd0273a54badfe3ce990cbef0391d2e9ccf9`; platforms are `linux/amd64` and `linux/arm64`.
- `node --test scripts/tests/test_rawrarr_subscriber_compose.mjs` passed 4/4 in the cached isolated Node test image. `git diff --check` passed.
