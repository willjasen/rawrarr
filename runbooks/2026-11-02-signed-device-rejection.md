# Signed device rejection

Issue: [#40 Replicate device trust and verify signed shared data](https://github.com/willjasen/rawrarr/issues/40)

## Change

Rejecting a pending enrollment now asks the owner to unlock the age-encrypted instance-root backup and independently saved root fingerprint. The browser signs a versioned `rawrarr-device-rejection-v1` payload binding the request ID, stable node ID, role, display name, and exact ML-DSA device public key. The coordinator verifies the signature against the pinned instance root (or pins it on the first signed decision), stores the signed record, then removes the pending request in the same database transaction. Future enrollment is denied with HTTP 410 when either the node ID or device-key fingerprint matches a rejected identity.

The new `0011_signed_device_rejections.sql` migration is additive and runs through the coordinator's normal SQLx migration startup. Existing node records, tokens, passkeys, volumes, and shared data are unchanged. Old coordinator code does not enforce the new table; keep the coordinator on the new image after migration. For rollback, restore the pre-deployment coordinator database backup together with the prior image. Do not roll back only the code, since that would make rejected identities eligible to reappear.

Mixed operation is safe when only the coordinator is upgraded: subscribers keep their existing protocol and receive HTTP 410 if a rejected identity retries enrollment. Rejection records stay in coordinator PostgreSQL and do not rewrite `/shared` data. No subscriber migration is required.

## Verification and rollout

Verification: focused signing/workspace/enrollment UI checks passed (44/44); the final isolated project run against this source passed all 146 Node tests and all 115 Rust tests. Two unrelated browser checks failed: the Radarr settings direct-route test timed out waiting for its unconfigured-node message, and the signing-topology test could not see its section title. The remaining browser checks passed. The unit suite does not run PostgreSQL-backed enrollment/retry integration; confirm that behavior in the internal environment after deployment by rejecting a test enrollment, verifying the stored signed record, confirming the pending item disappears, and retrying the same device ID and signing key to get HTTP 410. Also verify an un-rejected device can still enroll.

This is coordinator-only runtime code plus an additive coordinator database migration. Deployment target is the internal `roarlay` coordinator on do3; Mac fossil nodes and do3 public services are not targets. Restore the paired database backup and prior coordinator image if rollback is needed.
