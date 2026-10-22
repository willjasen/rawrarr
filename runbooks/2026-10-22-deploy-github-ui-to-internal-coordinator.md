# Deploy GitHub UI updates to the internal coordinator

## Objective

Deploy the private GitHub UI update mechanism to the internal coordinator (`roarlay` in Compose project `rawrarr-internal-mini`) so UI changes published from GitHub can update without a node image rebuild for every release.

## Initial state and blockers

- The implementation is committed as `8c05fe620975c55e346b57f9ed77fae54891ae2d` on local branch `codex/github-ui-updates`; the shared `main` checkout is dirty with unrelated task changes. The implementation has not been pushed or merged, so GitHub Actions has not published the private UI release.
- No `RAWARR_UI_GITHUB_TOKEN_PATH` file exists in the local protected Rawrarr application-support directory or the Mac mini's coordinator secrets directory. No token contents were read or logged.
- GitHub issue search and creation through the connected account both failed (422 and 404). The proposed issue is: "Deploy private GitHub UI updates to the internal coordinator", with acceptance criteria to publish/fetch the private bundle, update only roarlay, verify health and tailnet route, and document rollback/signature follow-up.
- Local SSH to `brandons-mac-mini` succeeds. The mini has existing internal-coordinator source/backups, but its SSH login shell could not resolve the `docker` command. The live service could not be inspected or deployed in this session.
- A local `.container-rebuild-fossil.lock` exists and belongs to another task; it was left untouched. No roarlay deployment marker was found on the mini.

## Required continuation

1. Push/merge the implementation commit to `main` through the repository's normal workflow. Preserve unrelated shared-checkout edits and do not push them.
2. Configure a fine-grained, Contents: Read GitHub token in a protected file on the Mac mini, and set `RAWARR_UI_GITHUB_TOKEN_PATH` to that file in the deployment environment. Never place the token in Git, Compose text, or logs.
3. Confirm the release assets `ui-bundle.zip` and `ui-bundle.zip.sha256` are published and accessible to the node token.
4. Establish Docker CLI availability on the mini's supported OrbStack/Compose environment; inspect existing project/container state and verify the roarlay rebuild marker is absent, then atomically claim it.
5. Deploy only `roarlay` in the existing `rawrarr-internal-mini` Compose project. Preserve PostgreSQL, Tailscale state, the live `Serve` route, and browser windows.
6. Verify roarlay health, `tailscale serve status --json`, HTTPS `/health`, and the UI bundle version/assets. Release the marker after verification.

## Verification and rollback

The implementation checks SHA-256 integrity and keeps `/ui/previous`; it does not yet provide publisher-authenticating signatures or browser-side signature verification. See [the implementation design and tests](2026-10-22-github-ui-distribution.md). No internal coordinator services were changed in this attempt because the release, token, and Docker CLI prerequisites were unavailable.
