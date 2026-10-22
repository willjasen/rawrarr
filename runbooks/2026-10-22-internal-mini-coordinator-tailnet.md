# Internal Mac mini coordinator on Tailscale

## Objective and starting point

Clarify and implement the internal deployment path where the Mac mini hosts Rawrarr coordinator and deployed subscriber nodes reach it through the owner's Tailscale tailnet. The current deployment guide describes the MacBook as coordinator and the mini as boneyard storage; no live role migration is included here. The prior tailnet-provider change added mode selection but used the onboarding page's origin as coordinator URL for both modes.

## Changes

- Internal Tailscale onboarding now asks for the Mac mini coordinator's HTTPS Tailscale Serve URL.
- Generated internal subscriber Compose sets the supplied URL as `RAWARR_COORDINATOR_URL` and `RAWARR_DIRECT_ALLOWED_ORIGIN`. Headscale mode retains the hosting page origin.
- Documented the intended mini coordinator topology and distinguished it from the current boneyard-on-mini deployment.

## Verification

`node --test scripts/tests/test_rawrarr_subscriber_compose.mjs` passed all three tests. The onboarding browser bundle rebuilt successfully. Docker Compose parsed generated internal-Tailscale and Headscale configurations, and `git diff --check` passed. The browser test and live tailnet deployment were not run.

## Remaining work and limits

No deployment services or live devices were changed. The Mac mini role/data migration, private HTTPS Serve configuration, coordinator health, subscriber reachability, and end-to-end node approval must be planned and verified separately before treating this as deployed.
