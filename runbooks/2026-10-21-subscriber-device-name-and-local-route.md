# Name subscriber devices before Compose generation

## Objective and initial state

Let the owner name a fossil or boneyard node before downloading its subscriber-only Compose file. While checking why the approved Mac node looked offline, its enrollment status returned `approved` and coordinator heartbeats were current. The Devices page showed `Offline` because it had no browser route: this coordinator had no relay/direct target configured for the Mac node, even though its approved origin was `http://localhost:7297`.

## Changes

- Add an editable device-name field to subscriber setup, with role-aware defaults and a 64-character limit. Pass the selected name into `RAWARR_DEVICE_NAME` in generated Compose.
- Add the direct API listener on container port 8082, publish it only on host loopback port 7298, and add `RAWARR_DIRECT_ALLOWED_ORIGIN` so the coordinator-origin browser can call it with CORS.
- Preserve the enrollment display name in the coordinator Devices response. Use an approved node's localhost port 7297 origin to derive its direct API URL at port 7298 when no explicit coordinator route exists; explicit direct targets still take precedence.

## Verification

- `node --test scripts/tests/test_rawrarr_subscriber_compose.mjs scripts/tests/test_rawrarr_node_approval_ui.mjs` passed all six checks. The subscriber onboarding Chromium test passed in `rawrarr-testing-base:local`; `git diff --check` passed.
- Rebuilt only the OrbStack test `roarlay` service and recreated only the Mac `fossil` service after updating `/Users/willjasen/rawrarr-test/compose.yml`. Their named data volumes remain attached.
- The Compose tests verify custom names, the coordinator CORS origin, direct listener port 8082, and loopback mapping 7298. The live Mac node's direct health endpoint returns HTTP 200 with the expected `Access-Control-Allow-Origin`.
- The authenticated coordinator Devices view shows `My fossil node` as `Direct · 5 ms`; PostgreSQL confirms a heartbeat within the 90-second online window. The two test nodes remain visible as relay-routed.
- Local coordinator health, tailnet HTTPS health, and local direct-node health all return HTTP 200.

## Recovery

Revert the task-owned UI, generator, and coordinator changes and rebuild the affected services. Keep the existing subscriber node's `/data` and `/shared` volumes; the name and direct-origin setting contain no credentials.
