# Recover group identity envelope browser loading

Issue [#260](https://github.com/willjasen/rawrarr/issues/260).

## Cause

Movies and Cluster configuration both resolve the same user-wrapped group identity envelope in `metadata-export.js`. The decrypt helper always wraps its result in `Uint8Array`; the caller requested text, so JavaScript converted the string result into an empty byte array. The resulting empty plaintext triggered “The group identity envelope is invalid.” The browser also assumed the decrypted identity started immediately with `AGE-SECRET-KEY-PQ-1`, although `age-keygen -pq` emits comment lines before the identity.

## Change

- Decrypt the user envelope as bytes, decode it as UTF-8, accept age-keygen comment lines, and pass the normalized identity line to browser decryption and node provisioning.
- Derive the group identity's public recipient in the browser and compare it with the published shared recipient before using the identity. A mismatch reports a separate error.
- Keep the original envelope and `config.aged` unchanged. Existing state has no migration requirement; the fix only reads the existing format correctly.
- Update the embedded browser asset and add a real-ciphertext browser regression covering the commented identity, recipient match, config decryption, and device rewrapping.

## Verification

- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --remove-after` — passed on OrbStack arm64; 137 Node tests, all Playwright browser tests, and 112 Rust tests passed. Python integration tests were skipped by the repository test runner because they require disposable PostgreSQL, age, or Syncthing fixtures.
- The focused mini-export browser fixture used generated age identities and ciphertext only. No live `/shared` files, private identities, or encrypted config were changed.

## Deployment and rollback

This changes browser code served by the internal coordinator only. After the task branch is merged and published on GitHub `main`, deploy the exact published revision to the internal do3 `roarlay` service using the host-local linked-clone workflow. Do not update the public roarlay or Mac fossil services. Verify coordinator health, the internal Tailscale Serve route, and `/health`. No deployment has been performed for this task yet.

If the published coordinator build fails verification, restore the prior coordinator image using its saved rollback tag and repeat route and health checks. Since this change does not rewrite shared data, rollback requires no data migration.
