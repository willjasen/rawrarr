# Disable the public Funnel on do3

## Objective and issue

Disable public Tailscale Funnel access to the coordinator on do3 while retaining HTTPS Serve for tailnet devices and the existing TCP routes. Tracked by [issue #307](https://github.com/willjasen/rawrarr/issues/307).

## Initial state

- Connected to the host at `100.91.193.80` over Tailscale SSH inside the host's `ai` tmux session.
- `tailscale funnel status` showed public HTTPS on `do3.risk-mermaid.ts.net`, proxying to `127.0.0.1:18787`.
- TCP forwards on 17300 and 17301 were tailnet-only. An existing HTTPS Serve alias for `rawrarr-roarlay.risk-mermaid.ts.net` also pointed to the same backend.

## Actions

- Ran `tailscale funnel --https=443 off` on do3 to disable public Funnel for HTTPS.
- Restored the current host name as tailnet-only Serve with `tailscale serve --https=443 --bg http://127.0.0.1:18787`.
- Left TCP forwards 17300 and 17301, the old tailnet-only HTTPS alias, host tags, ACL policy, Docker, Caddy, and service data unchanged.

## Verification

- `tailscale funnel status` no longer lists a public HTTPS route; it lists TCP 17300 and 17301 as tailnet-only.
- `tailscale serve status` reports `https://do3.risk-mermaid.ts.net` as tailnet-only with `/` proxying to `http://127.0.0.1:18787`; the old alias remains configured tailnet-only.
- The backend returned HTTP 200 locally, and `https://do3.risk-mermaid.ts.net/` returned HTTP 200 from this tailnet-connected workstation.
- A non-tailnet external request was not available in this environment, so public inaccessibility was not independently tested from outside the tailnet. Tailscale's live Funnel status confirms Funnel is off.

## Rollback

To restore public HTTPS exposure if requested, run `tailscale funnel --https=443 on` on do3 and verify `tailscale funnel status`. The current Serve backend and ACL/tag configuration were preserved.
