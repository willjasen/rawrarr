# Fix Movies group age unlock

This recovery fix is tracked by existing [issue #109](https://github.com/willjasen/rawrarr/issues/109), which defines owner-approved group-key distribution to each approved node. No duplicate issue was created.

## Findings

The earlier diagnosis recorded in commit `4425bdb` found both internal fossils without node-local `/data/group-age-identity` and without the synchronized `group-age-user-envelope.aged`. Current `AgeVault` initialization only creates a group key for a completely fresh shared directory. If a prior `age-recipient` and `config.aged` remain but the group state is absent, startup does not prove that the supplied identity can decrypt the old config before proceeding. On newer group-key state, a public `group-age-recipient` alone cannot reconstruct the private key.

## Change

- For legacy shared data with no group recipient, bootstrap the group key only after the submitted identity decrypts a valid `PersistedConfig` (when one exists). The existing config is then migrated by the normal `load` path.
- If a group recipient already exists but the local group key is absent, require the matching user-wrapped group envelope. Never mint a replacement key over existing group state.
- Keep device-specific group-key envelopes encrypted to the selected fossil's node recipient and install them directly on that node; never publish a private identity or assume one device's envelope can provision another device.
- Existing deployments need no destructive data migration. Legacy config migration is repeatable through the existing read-and-rewrite path. A missing group private key plus missing user envelope remains unrecoverable from a public recipient/ciphertext alone and requires restoring a valid owner backup; this change preserves all existing data in that state.

## Verification

- Added Rust regressions for legacy recovery, per-device envelope installation, and fail-closed preservation of an existing group recipient.
- `python3 scripts/tests/run_rawrarr_tests_in_docker.py --context orbstack` — passed: 146 Node tests, all Playwright browser tests, and 115 Rust tests.
- The runner skipped Python integration tests requiring disposable PostgreSQL, age, or Syncthing fixtures; the separate OrbStack peer-container sync test was not run.
- Not run: internal environment state inspection or deployment. The change has no deployed effects.
