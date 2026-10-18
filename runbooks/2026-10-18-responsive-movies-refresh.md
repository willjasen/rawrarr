# Responsive movies refresh

## Objective and initial state

Show the Movies list sooner after refreshing. Existing UI awaited a settings request and browser reachability probe (each with a three-second timeout) before rendering already-fetched movies. The worktree contains substantial unrelated changes, preserved separately. Related: [artwork timing](2026-10-18-radarr-artwork-load-time.md).

## Actions

Render rows and posters immediately after the movie API response. Enhance titles with links asynchronously after the existing credential-free probe succeeds; ignore obsolete probe results after another refresh. Keep rows during updates, prevent overlapping list requests, and label retained data on failure. No browser metadata persistence is introduced.

## Verification

- Nine focused Node/Chromium tests passed: responsiveness, links, navigation, and artwork source.
- Diff whitespace check and release container build passed.
- Verified the existing app owned port 7297; rebuilt the same Compose project preserving its volumes. Container healthy; `/health` returned HTTP 200.
- Live authenticated Radarr timing was not measured; browser checks use controlled fixtures.

## Final state

Deployed locally. Initial movie API latency remains; the optional connection check no longer blocks rendering. UI and existing link-test edits depend on pre-existing uncommitted implementation and remain uncommitted to avoid absorbing other work. New regression coverage and documentation are committed separately. No remote is configured.

## Recovery

Reverse only this task's UI changes and rebuild the same Compose project with existing volumes preserved.
