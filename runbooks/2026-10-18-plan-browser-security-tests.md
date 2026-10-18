# Plan browser security tests

## Objective and initial state

Plan a reviewer-facing Playwright integration suite that exercises metadata and storage nodes through the coordinator and checks that private content is not exposed there. The repository already has a Chromium/Playwright Docker test harness, but the security documentation says the full Movies flow is not integrated end to end and storage publishing coverage needs confirmation. Relevant design: [security test plan](../docs/security-test-plan.md), [Rawrarr security](../docs/SECURITY.md), and [blind coordinator content flow](../docs/blind-coordinator-content.md).

## Plan recorded

- Use Chromium against disposable coordinator, metadata, and storage services, with synthetic fixture data and recognizable per-run markers.
- The user authorizes a harness-only passkey bypass and per-run temporary age ID and root signing key; keep generated credentials isolated to disposable services and remove them on cleanup, without changing production authentication.
- Expanded the plan with session/CSRF and authorization lifecycle, signature/replay, browser content/origin protections, storage filesystem boundaries, abuse resistance, metadata leakage, and harness isolation/cleanup checks.
- Observe coordinator ingress/egress at the application boundary and inspect disposable logs/database, rather than relying on TLS packet capture or source assertions alone.
- Cover local browser decryption, ciphertext/opaque reference forwarding, no marker leakage, private identity exclusion, tampering, authorization failures, bounds, timeouts, and storage scans without a private age identity.
- State explicitly that this is regression evidence, not an audit; current key exchange does not prevent an actively malicious coordinator from substituting node keys.
- Keep implementation separate until requested; no tests or services were run or changed.

## Verification and remaining work

Documentation-only review against current `docs/SECURITY.md`, `docs/blind-coordinator-content.md`, and the existing Playwright test harness. Implementation and runtime verification remain future work.
