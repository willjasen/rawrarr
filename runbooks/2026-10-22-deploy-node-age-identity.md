# Deploy persistent node age identity

## Objective and initial state

Deploy commit `0ac33ce` to the internal MacBook Pro subscriber (`rawrarr-node-internal` / `fossil`) so subscriber startup creates and reuses `/data/node-age-identity`. The coordinator returns before this startup code, so the Mac mini `roarlay` does not need an update.

Related work: [node and app age identity design](2026-10-22-node-and-app-age-identities.md) and [main deployment to the internal Macs](2026-10-22-deploy-main-to-internal-macs.md).

GitHub issue tracking could not be completed before deployment: searching `willjasen/rawrarr` failed with API 422 (repository unavailable to the integration), and issue creation failed with API 404. Proposed issue title: “Deploy persistent node age identity to internal subscriber.” Proposed scope: “Deploy the committed persistent node age identity behavior to the internal MacBook subscriber (fossil). Confirm identity persistence and service health while preserving the user age identity and coordinator role boundary.”

## Actions

- Applied the two-file implementation from commit `0ac33ce` to the MacBook's existing `github-source` checkout; both files passed `git apply --check` before application, and `git diff --check` passed. The other `source` package was left untouched.
- Confirmed the `fossil` deployment marker was free, atomically claimed it, built `rawrarr:local` from `github-source/rust-arr`, and recreated only service `fossil` in Compose project `rawrarr-node-internal` with `--no-deps`. The first recreate encountered a stale `Created` fossil container from an earlier interrupted operation. Removed only that non-running container and retried successfully; the prior healthy fossil remained running until replacement.
- Preserved the fossil `/data` and `/shared` volumes and the Tailscale sidecar. Released and confirmed removal of the fossil marker after verification.

## Verification

- The image build succeeded. Fossil became healthy with zero restarts.
- Confirmed `/data/node-age-identity` exists with mode `600` without reading or printing its contents. Confirmed the existing `/data/age-identity` remains present.
- The MacBook HTTPS `/health` route returned HTTP 200. Tailscale Serve still maps `rawrarr-macbook-pro.risk-mermaid.ts.net:443` `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`.

## Final state and recovery

- The persistent identity behavior is deployed on internal subscriber `fossil`; coordinator `roarlay` was not changed because coordinator startup bypasses this code.
- GitHub issue search returned API 422 and issue creation returned API 404; the proposed issue title and scope are recorded above.
- No rollback was needed. Recovery is to rebuild the prior image from the prior committed source and recreate only `fossil` in `rawrarr-node-internal`, preserving its named volumes and sidecar.
