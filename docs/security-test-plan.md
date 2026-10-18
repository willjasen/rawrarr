# Browser security test plan

## Goal

Give interested reviewers repeatable evidence that metadata and storage nodes can serve Rawrarr content through the coordinator while subscriber content remains opaque at the coordinator boundary. This is a regression test plan, not an independent security audit or proof that the entire product is secure.

The test suite should use Playwright with a real Chromium browser and disposable local services. It should exercise actual browser code and coordinator/node HTTP handlers, rather than infer behavior from source text alone. Keep the existing fast source/unit tests; add this as a separate opt-in integration tier in the established Docker test runner.

## Trust boundary and evidence

Use a fixture coordinator, metadata node, storage node, and browser. The coordinator is the observation point. Instrument its request handler and outbound relay boundary to retain only synthetic test bytes and assert what entered and left the coordinator. Collect its disposable PostgreSQL rows and captured logs after each scenario. Tests may bypass passkey ceremonies through a test-only authentication fixture, generate a temporary age ID, and generate a temporary root signing key. These credentials must be created per run (or per isolated test), used only by disposable test services, and destroyed during cleanup. They must never be copied from production, printed, checked into source, persisted in shared/runtime volumes, or included in retained reports. Do not use live account data, real age identities, API credentials, media paths, or production endpoints.

Generate unique recognizable markers for each run: the temporary age private identity and root signing key, movie/series title, provider ID, media path, API key, configuration value, and artwork bytes. Populate fixture nodes with representative plaintext source data and expected age-encrypted objects addressed to the public recipient derived from that run's temporary age ID. Assertions should search coordinator ingress, egress, logs, and database values for every private-key and content marker, while confirming intended public coordination fields and opaque ciphertext are present. The test-only passkey bypass must be limited to the disposable harness and must not add a production bypass or alter production authentication behavior.

This test establishes what the application sends to and receives from the coordinator in the tested scenarios. HTTPS packet capture alone cannot show plaintext after TLS termination, and a test cannot establish the absence of every possible leak. A malicious coordinator can also serve modified browser code. Report these limits next to results.

## Scenarios

### Metadata node

1. Start a disposable coordinator and metadata node with a synthetic Radarr/Sonarr fixture and the public age recipient derived from the run's temporary age ID; open the coordinator workspace in Chromium and authenticate through the harness's test-only passkey bypass.
2. Load library metadata and artwork through the coordinator relay. Verify the browser renders the expected fixture after local decryption.
3. Assert request and response bodies observed at the coordinator contain no title, provider ID, media path, upstream API key, decrypted config value, artwork plaintext, or private identity marker. Assert the forwarded content is ciphertext and opaque identifiers do not contain fixture values.
4. Verify the coordinator logs and disposable database contain no content markers or payload copies. Allow documented account/node identifiers, authorization state, request route, sizes, timing, and online status.
5. Alter ciphertext, nonce, node key, object reference, and encrypted payload length in separate cases. The browser must reject invalid content without rendering it; the coordinator must reject unknown routes, roles, and oversized bodies without echoing payloads into errors or logs.

### Storage node

1. Start a disposable storage node with a synthetic read-only media tree and no private age identity. Give it only the public age recipient derived from the temporary test age ID, needed to create encrypted sidecars. Generate and use a temporary root signing key only if the flow under test needs device signing; keep it within the disposable harness and assert its private bytes never reach the coordinator.
2. Trigger the supported scan/publish flow through the browser and coordinator. Verify the storage node discovers the fixture locally, emits age ciphertext with opaque references, and the browser decrypts and renders the expected content.
3. Apply the same coordinator ingress/egress, logs, database, tampering, authorization, route allowlist, and size assertions as the metadata scenario. Confirm the coordinator never receives media bytes or paths and the storage node never needs a private identity.
4. Verify scan failures, unavailable drives, and empty inventories do not turn into deletion signals or disclose local paths through coordinator responses.

### Negative and regression cases

- Run with a wrong temporary age identity and with a missing/invalid identity; no content should render and no private identity should be sent to the coordinator.
- Confirm the temporary root signing key is used only for the intended local signing operation, and that only explicitly public verification material or signatures required by the protocol cross the coordinator boundary.
- Deny an unapproved node, wrong owner, wrong role, expired session, and guessed object reference. Confirm fail-closed responses and absence of fixture data.
- Exercise coordinator errors and upstream timeouts. Browser-visible errors should be generic, with no upstream bodies, secrets, paths, or content markers.
- Record browser requests with Playwright and assert the private age identity, decrypted object bytes, and upstream service credentials never appear in any request URL, header, or body directed to the coordinator.
- Repeat the successful scenarios with both metadata and storage roles and preserve the captured evidence as a sanitized test report: test names, pass/fail, versions, and marker-free assertions only.

### Authentication, session, and request integrity

- Use the passkey bypass only in the disposable harness. Add a negative check that production builds/configuration reject or ignore the test bypass, so it cannot create a production authentication path.
- Verify session cookie attributes for the test HTTPS setup, then test expiry, logout invalidation, session rotation where applicable, and rejection of malformed or replayed session/node credentials.
- Test CSRF protections and cross-origin behavior for state-changing coordinator and node routes. Keep browser-level checks (origin, cookies, CORS behavior) alongside server-side request/authorization tests; CORS alone is not a CSRF defense.
- Attempt cross-account, cross-node, and cross-role access, including using a valid credential against another node's endpoints. Verify revocation takes effect and no route falls back to broader access.
- For signed requests or device approvals, test replayed signatures, altered payloads, wrong instance/role/node bindings, unknown signers, revoked keys, and stale revisions. Verify only public keys/signatures needed by the protocol cross the coordinator; private root signing bytes remain local and transient.

### Browser content and origin protections

- Exercise hostile titles, descriptions, artwork metadata, error strings, and node-provided display names containing HTML/script payloads. Confirm they render as inert text and cannot execute same-origin script.
- Check the deployed page's Content Security Policy and related security headers against the intended production policy, including whether inline scripts and unexpected origins are permitted. Treat this as policy regression coverage, not proof against a malicious coordinator that can replace the page and its policy.
- Verify browser storage cleanup for sign-out and explicit age-ID removal, and that private IDs/root keys do not appear in URLs, browser history, caches, service-worker state, or retained test artifacts. Keep memory-only versus explicit browser retention behavior distinct in expected results.

### Storage-node filesystem boundaries

- Build a synthetic, read-only media tree containing normal files, unreadable files, symlinks, traversal-like names, and paths outside the configured root. Confirm scans do not follow symlinks or traversal outside the allowed root and never modify media files.
- Test missing/unavailable media mounts, empty libraries, permission failures, and interrupted scans. Confirm these states do not become deletion instructions or expose full local paths in coordinator responses, logs, or database records.
- Verify a storage node can produce encrypted sidecars using only the public age recipient and has no private age identity, coordinator database, or writable access to the media fixture.

### Abuse resistance and metadata leakage

- Send oversized and malformed request bodies, ciphertext, object references, and metadata; verify bounded parsing, size limits, generic errors, and no partial plaintext rendering.
- Exercise concurrent and repeated scans/requests, slow or stalled upstreams, and upstream disconnects. Confirm work is bounded/cancellable, failure paths do not leak payloads, and logs contain no content markers.
- Check filenames, URLs, headers, and object references for title/provider-ID/path leakage. Explicitly record allowed metadata leakage—such as account/node association, online status, timing, route, and approximate request/object sizes—because content encryption does not hide it.
- Where supported by the test harness, check that node-to-coordinator transport requires the configured HTTPS origin and rejects downgrade or unexpected-origin configurations. Do not treat this as a substitute for application-layer encryption.

### Harness isolation and cleanup

- Make tests refuse production hostnames, persistent Compose projects, non-test databases, and non-test volumes. Use isolated ports/networks, labels, and disposable storage; test that the fixtures cannot accidentally target the live deployment.
- Verify passkey bypass, temporary age-ID generation, and root signing-key generation are harness-only. Run cleanup assertions after both success and forced failures, checking containers, networks, databases, fixture media, generated keys, logs, and temporary files are removed.
- Preserve only sanitized test summaries: commit and dependency versions, scenario names, results, and marker-absence outcomes. Never retain generated private keys, raw request captures, fixture payloads, or credential-bearing logs.

## Security qualification

The current browser-to-node P-256 ECDH exchange does not independently authenticate the node key. A coordinator that actively substitutes public keys may establish separate sessions and read or modify the content. Playwright tests with an honest fixture coordinator can verify the unchanged-relay case, but cannot justify a claim of protection from a malicious coordinator. Before making that stronger claim, implement node-key authentication pinned or provisioned independently of the coordinator, bind node identity and request context into the authenticated transcript, and add substitution/replay tests that demonstrate rejection.

Do not claim this suite proves confidentiality against compromised nodes, browsers, browser extensions, same-origin script injection, malicious updates, traffic analysis, or a malicious coordinator that can replace the served JavaScript. It demonstrates tested application behavior at explicit boundaries.

## Implementation sequence and acceptance

1. Confirm which content flows are implemented and reachable through the coordinator; exclude planned or disconnected flows from claims.
2. Add deterministic fixture generation and a disposable coordinator/metadata/storage integration topology to `scripts/tests/`, isolated from persistent Compose projects and production ports/volumes. Add harness-only passkey bypass, temporary age-ID and root-signing-key generation, and failure-safe cleanup; never add a production authentication bypass.
3. Add a small Playwright test for metadata first, with coordinator-boundary capture and marker assertions; integrate it as a clearly named security integration tier.
4. Add storage scan/publish coverage after its reachable flow and fixture semantics are confirmed.
5. Add session/authorization lifecycle, signature/replay, browser rendering/origin, filesystem-boundary, malformed input, oversize, timeout, and log/database absence cases; ensure cleanup runs even after failures.
6. Add harness isolation checks proving no live deployment, persistent volumes, or non-test database can be targeted and that generated test credentials and artifacts are removed on success and failure.
7. Document exact invocation, prerequisites, evidence collected, limits, and known untested paths in `TESTING.md` and `scripts/tests/README.md` when implemented.

Acceptance requires successful browser-side rendering of known fixture content; no plaintext markers in coordinator-observed requests/responses, logs, or database; no private identity, root signing key, decrypted object bytes, or upstream credential in coordinator-directed browser traffic; ciphertext and opaque references forwarded as expected; fail-closed behavior for tampering and unauthorized access; bounded behavior on malformed/oversized/slow requests; storage scans confined to the read-only fixture root; and reliable cleanup of generated credentials and disposable infrastructure. Every report must identify the tested commit and state that the results cover only these scenarios.

## Current status

This document is a plan only. Test-only passkey bypass and per-run temporary age-ID/root-signing-key generation are authorized for the isolated harness; implementation must keep them out of production behavior and retained artifacts. No integration tests, runtime changes, dependency changes, or deployments were made for it. The current security documentation records that the full Movies POST flow is not yet admitted by the coordinator export relay and that integrated end-to-end decryption is unverified. Storage-node publishing coverage depends on confirming its current reachable path before implementation.
