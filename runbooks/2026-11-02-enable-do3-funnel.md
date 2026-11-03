# Enable Tailscale Funnel on do3

## Objective and issue

Expose the existing HTTPS service on the public Rawrarr host `do3` through Tailscale Funnel. Tracked by [issue #304](https://github.com/willjasen/rawrarr/issues/304).

## Initial state

- The willnet ACL policy commit `95f6ddebe09978bed97352476a57bf8f4ea436ef` was pushed to `main`; GitHub Actions run `41558594182` applied it successfully.
- Tailscale policy now grants the Funnel node attribute only to `tag:funnel`; `tag:ssh` already permits `willjasen@github` to Tailscale SSH as root or non-root.
- The user renamed the device from `rawrarr-roarlay` to `do3` in Tailscale. On the host, Tailscale 1.104.4 is online at `100.91.193.80`, and the MagicDNS name is `do3.risk-mermaid.ts.net`.
- Root access is through Tailscale SSH inside the existing `ai` tmux session. The device currently has no tags; include both `tag:funnel` and `tag:ssh` if adding tags so the existing SSH policy continues to match.
- Existing Tailscale Serve configuration has HTTPS 443 `/` to `http://127.0.0.1:18787`; local HTTP on 18787 returned 200. TCP 17300 and 17301 are separate tailnet-only forwards and must remain private.
- The shared Rawrarr checkout has unrelated staged/conflicted changes, so this runbook is being kept in a dedicated worktree based on the refreshed `origin/main`.

## Actions

- In the Tailscale Machines page, assigned `tag:funnel` and `tag:ssh` to `do3`. This changed its device identity from `willjasen@github` to those tags; `tag:ssh` preserves the existing user-to-server Tailscale SSH rule.
- From an isolated `tmux` server/socket with session `ai` (the shared host `ai` session had concurrent activity), ran `tailscale funnel --bg 18787`. It enabled public HTTPS on the default Funnel port 443 and proxies to `http://127.0.0.1:18787`.
- An initial command with `--https=443` and an explicit URL target returned `invalid argument format` without changing Funnel state. The documented port-target form succeeded.
- No Docker containers, databases, volumes, or Caddy routes were changed. The older `rawrarr-roarlay.risk-mermaid.ts.net` Serve alias remains tailnet-only. TCP 17300 and 17301 remain tailnet-only.

## Verification and remaining work

- GitHub Actions run `41558594182` successfully deployed the willnet ACL policy.
- The Tailscale admin page shows `do3` online with `tag:funnel` and `tag:ssh`. A new Tailscale SSH connection as root succeeded after tagging.
- `tailscale funnel status` reports `https://do3.risk-mermaid.ts.net` as Funnel on and confirms TCP 17300/17301 remain tailnet-only. Read-back shows HTTPS `/` proxies to `http://127.0.0.1:18787` and local backend HTTP returns 200.
- The first public HTTPS probe ran while Tailscale was still obtaining the certificate for `do3.risk-mermaid.ts.net` and failed during TLS negotiation. Tailscaled logs later confirmed certificate issuance. Afterward, direct requests forced to both public A records (`208.111.34.11` and `208.111.35.209`) returned HTTP 200 with TLS verification result 0. Local backend HTTP on port 18787 also returned 200.
- If Funnel must be rolled back, run `tailscale funnel --https=443 off`, then restore the prior private handler with `tailscale serve --https=443 127.0.0.1:18787`. Do not reset the complete Serve configuration because TCP 17300 and 17301 are still in use.
