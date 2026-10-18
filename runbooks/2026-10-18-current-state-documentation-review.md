# Review current Rawrarr documentation

## Objective and initial state

Review project entry points, architecture, deployment, user procedures, and status claims against the current checkout. The `main` working tree was clean at task start (`1475a80`), with one worktree and no configured remote. Related history: [original project documentation](2026-10-18-rawrarr-project-documentation.md), [full coordinator Movies view](2026-10-18-full-coordinator-movies-page.md), [movie export in Settings](2026-10-18-settings-metadata-export.md), and [automatic storage scanning](2026-10-18-storage-automatic-scanning.md).

## Findings and changes

- The user guide named the removed `coordinator` Compose service and implied base Compose published 7297. Current files name `roarlay`; the MacBook Tailscale overlay publishes 7297 and 7298 to roarlay, while metadata listens on internal loopback 8081. Read-only container labels confirmed the persistent services record different Compose file sets; the local 7297 health page identified roarlay. Updated the guide and deployment notes to require the recorded file set and per-service deployment lock for future rebuilds.
- Roadmap and architecture pages still described the former three-record coordinator view and movie export. Current source lists all valid movie records for export, builds a browser ZIP in memory, and renders full Movies and TV libraries with browser-encrypted responses. Updated completed work, remaining scope, and the browser privacy boundary.
- Storage source and Compose default to a 24-hour interval and wait that interval before the first unattended scan. A later Mini update rebuilt storage, and a bounded manual scan produced three synchronized encrypted sidecars; an unattended scan has not been observed. The dated Sonarr backfill runbook provides live evidence at that time, so the roadmap no longer treats all live Sonarr use as unverified.
- Corrected local Tailscale route documentation for roarlay HTTPS, narrow direct APIs, and the separate Syncthing TCP forward when configured. Live Serve handler state was not changed or verified by this documentation task.
- Corrected the root README and direct-node plan: local port 7297 is a roarlay diagnostic path in the MacBook overlay, and the Mini has a narrow direct API route, not a general browser UI listener.

## Verification and final state

Checked the edited claims against Compose, Rust and browser source, relevant dated runbooks, current Docker container labels, and the loopback health page. The protected-password-file MacBook Compose merge passed `config --quiet`; all relative links in the edited Markdown files resolve; `git diff --check` passed. No service, configuration, secret, or runtime data was changed. The review does not establish current authenticated library behavior, an unattended Mac mini scan, or the current tailnet Serve configuration; those require their own live checks.

## Recovery

Revert only this task's documentation commit if the wording needs revision. No runtime rollback is needed.
