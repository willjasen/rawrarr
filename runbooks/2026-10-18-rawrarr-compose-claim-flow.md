# Generated Docker Compose and node claim flow

## Objective and initial state

Capture the planned self-service deployment flow: a private or subscribed user gets a Rawrarr Docker Compose YAML file from the site, starts it locally, and the node contacts the operator's main service node. The user then claims the pending node through passkey-authenticated portal access. Existing node/cloud architecture docs describe service boundaries but did not specify this Compose onboarding path.

## Actions

- Added the planned deployment and claim lifecycle to `docs/node-types.md` and `docs/cloud-access-architecture.md`.
- Specified outbound node registration, locally generated device keys, explicit passkey-authenticated claim, private/subscribed modes, and no reusable credential in Compose or logs.
- Recorded that claiming shares only minimal routing/account metadata with the service node; app data, media, private age identity, and node-local databases remain subscriber-controlled.
- Kept this as a design plan; did not implement portal, enrollment protocol, or Compose generation.
- The new `Codebox/rawrarr` repository is intended to contain Rawrarr-specific app, packaging, tests, and documentation only. Older non-Rawrarr Arr-services work should stay in the existing repository during extraction.

## Verification and remaining work

Review the documentation diff and run `git diff --check`. Implementation still needs a threat model, enrollment-token/key-continuity protocol, authenticated claim APIs, generated role-specific Compose output, and integration tests for expired/replayed claims and service outages.
