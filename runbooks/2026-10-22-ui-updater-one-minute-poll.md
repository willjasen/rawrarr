# Check GitHub UI releases every minute

## Objective

Have the internal coordinator check GitHub once per minute and install a newer signed UI bundle automatically. GitHub issue: [#29](https://github.com/willjasen/rawrarr/issues/29). Related design: [Separate UI updates from node images](2026-10-22-github-ui-distribution.md).

## Initial state

The updater in roarlay's application image checked the private GitHub release every five minutes. The image was from before the one-minute updater change. The coordinator already had the protected GitHub token mounted and a persistent `/ui` volume; its installed bundle was `270d77c`. The Mac mini `github-source` checkout was clean at main commit `742f75d`, which includes the one-minute change.

## Actions

- Changed the updater default to 60 seconds and retained a hard minimum of 60 seconds. Added regression coverage for the default, floor, and longer intervals.
- Rebuilt and recreated only `roarlay` in the existing `rawrarr-internal-mini` Compose project, using the clean `github-source` checkout at `742f75d`. Used `--no-deps` so PostgreSQL and Tailscale were not recreated. Compose received the existing protected secret file paths without reading their contents.
- The first build stopped before changing a container because SSH's PATH omitted `/usr/local/bin/docker-credential-osxkeychain`. Retried with `/usr/local/bin` added to PATH; the build and roarlay recreation completed.
- New image ID: `sha256:ee906f216a42de9e2ca52c6c5e8af363acb7e7e370a42f411996c33f021ce333`. The previous image remains available as `sha256:ed75ed36b8e2de4bc25a9f8c9d2a16d7476205c203baacaecb93ea06475d23eb` for rollback.
- The deployment lock was atomically claimed, held through verification, then removed and confirmed absent.

## Verification

- `python3 scripts/tests/test_ui_bundle.py -v` passed all 6 tests; Python compilation and `git diff --check` passed before deployment. The main-branch GitHub Actions test and signed UI bundle publication also succeeded.
- Roarlay is healthy with zero restarts. The running updater process is `python3 /usr/local/bin/rawrarr-update-ui --apply --watch`; its image contains the 60-second default.
- The updater reports signed status. Installed bundle version `742f75d037ab1b26b27e6ae02a69d9c0ae7967cd` and SHA-256 `b3ac9e4528ae0bf54539daa5440a57ae068b1e8eb0cb363d182adf7a7ca008b8` match the latest signed GitHub release. OpenSSL verified the release signature with the pinned publisher key.
- Installed `coordinator_workspace.html` and `coordinator-workspace.mjs` hashes match the files in the signed GitHub archive. The live coordinator JS asset hash also matches the archive, and it is served with `Cache-Control: no-store`. The HTML includes the coordinator styles, so the installed HTML/CSS and JS are from the verified GitHub bundle.
- `https://rawrarr.risk-mermaid.ts.net/health` returned HTTP 200 and identified roarlay. Tailscale Serve read-back confirms `rawrarr.risk-mermaid.ts.net:443` `/` proxies to `http://127.0.0.1:8080`.
- PostgreSQL and Tailscale stayed running with zero restarts; their start times were unchanged. Their volumes and the UI volume were preserved. The MacBook fossil node was not touched.

## Recovery

If the new updater causes a problem, retag the retained previous image as `rawrarr:internal-mini` and recreate only roarlay with the same `rawrarr-internal-mini` Compose project and `--no-deps`. Keep the existing `/ui` volume and protected secrets. The UI updater can be disabled with `RAWARR_UI_UPDATES_ENABLED=false` if rollback requires serving the embedded bundle.
