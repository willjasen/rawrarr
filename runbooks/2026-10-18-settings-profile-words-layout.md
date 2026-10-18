# Give the profile name room for a two by two layout

## Objective

Narrow the browser Age ID card on Settings and let the four profile words display in two columns beside the avatar.

## Initial state

Settings used a two-column identity grid weighted toward the Age ID card. The repository contains extensive pre-existing changes, so this task is limited to the scoped CSS and this runbook/index entry.

## Actions

- Shifted the identity grid from a 0.8/1.2 split to a 1.2/0.8 split, with a minimum 380px profile column on wide layouts.
- Set the profile name to a two-column grid so four generated words flow into two rows.
- Kept the existing single-column layout below 900px.

## Verification

- Rebuilt and recreated only `rawrarr-local-roarlay-1` using `docker-compose.yml`, `docker-compose.coordinator.yml`, and `docker-compose.tailscale-macbook.yml` with project `rawrarr-local`.
- The roarlay container reports healthy. Local `http://127.0.0.1:7297/health` and tailnet `https://rawrarr.risk-mermaid.ts.net/health` both returned HTTP 200 and the Roarlay health page.
- Read back Tailscale Serve status: `rawrarr.risk-mermaid.ts.net:443` `/` still proxies to `http://127.0.0.1:8080`; the existing `/direct-api` mapping to `http://127.0.0.1:8082` remains present.
- The roarlay deployment marker was removed after verification. `git diff --check` passed for the task runbook, index, and UI file.

## Final state

- The Settings UI with the wider profile card, narrower Age ID card, and two-column profile-name words is deployed to roarlay.
