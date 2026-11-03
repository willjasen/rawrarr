# Signed device rejection

Issue: [#40 Replicate device trust and verify signed shared data](https://github.com/willjasen/rawrarr/issues/40)

## Change

Rejecting a pending enrollment now asks the owner to unlock the age-encrypted instance-root backup and independently saved root fingerprint. The browser signs a versioned `rawrarr-device-rejection-v1` payload binding the request ID, stable node ID, role, display name, and exact ML-DSA device public key. The coordinator verifies the signature against the pinned instance root (or pins it on the first signed decision), stores the signed record, then removes the pending request in the same database transaction. Future enrollment is denied with HTTP 410 when either the node ID or device-key fingerprint matches a rejected identity.

The new `0011_signed_device_rejections.sql` migration is additive and runs through the coordinator's normal SQLx migration startup. Existing node records, tokens, passkeys, volumes, and shared data are unchanged. Old coordinator code does not enforce the new table; keep the coordinator on the new image after migration. For rollback, restore the pre-deployment coordinator database backup together with the prior image. Do not roll back only the code, since that would make rejected identities eligible to reappear.

Mixed operation is safe when only the coordinator is upgraded: existing subscribers keep their protocol, receive HTTP 410 if a rejected identity retries enrollment, and cannot recreate a pending owner prompt. Rejection records stay in coordinator PostgreSQL and do not rewrite `/shared` data. No subscriber upgrade or migration is required.

## Verification and rollout

Verification: focused signing/workspace/enrollment UI checks passed (44/44); the final isolated project run against this source passed all 146 Node tests and all 115 Rust tests. Two unrelated browser checks failed: the Radarr settings direct-route test timed out waiting for its unconfigured-node message, and the signing-topology test could not see its section title. The remaining browser checks passed. The unit suite does not run PostgreSQL-backed enrollment/retry integration; confirm that behavior in the internal environment after deployment by rejecting a test enrollment, verifying the stored signed record, confirming the pending item disappears, and retrying the same device ID and signing key to get HTTP 410. Also verify an un-rejected device can still enroll.

This is coordinator-only runtime code plus an additive coordinator database migration. The deployment classifier maps coordinator SQLx migrations and the coordinator-served signing asset to the coordinator target; subscriber Rust code does not change. No Mac fossil node update is required. Restore each coordinator database backup together with its prior image if rollback is needed; do not roll back only the application code.

### Deployment record

On 2026-11-02, Will requested deployment to both internal and live environments. Revision `16943d61cefc75da596811b98aa476e9ba08435f` was built on do3 for native `linux/amd64` as `rawrarr:16943d6-amd64` (image ID `sha256:15957a59149038520c356941748c5d19f1c172a888a9e05173987d9f87d98c2e`) and promoted to the host-local `rawrarr:internal-base-amd64` tag. The current classifier selected coordinator only, so no Mac subscriber was rebuilt.

Both databases were backed up before migration and the custom-format archives passed `pg_restore --list`:

- Internal: `/opt/rawrarr-internal-do3/backups/pre-signed-device-rejection-16943d6.dump`, SHA-256 `294f37d04842d69655def004b406daa57b9415ed4436b54298abec1e50544ec8`.
- Public: `/opt/rawrarr-roarlay-prod/backups/pre-signed-device-rejection-16943d6.dump`, SHA-256 `9fbd2cbbb93d42bba0ec7c7939f2736ccda350c30bac7c61b9c06a7bd4c38b18`.

Only `roarlay` was recreated in each Compose project, with `--no-deps --pull never`. PostgreSQL, Tailscale, Caddy, Cloudflare Tunnel, analytics, and other services were not recreated. Both databases advanced from migration 10 to 11. Both coordinator containers report healthy with zero restarts and use the image labeled with the published revision. The internal Tailscale Serve handler remains `https://rawrarr.risk-mermaid.ts.net/` → `http://127.0.0.1:8080`. Internal `/health` and public `https://roarlay.rawrarr.net/health` both returned HTTP 200. Both deployment locks were released after verification.

The signed rejection and retry flow was not exercised with an owner-signed test enrollment in production. Complete that behavioral check in the internal environment using a disposable test device before treating end-to-end rejection as verified; confirm the signed row persists, the prompt disappears, the rejected identity receives HTTP 410 on retry, and a fresh identity can still enroll.
