# Remember the age ID on this device

## Objective and initial state

Reduce repeated age identity entry during local Rawrarr testing. Previously the unlocked vault existed only in process memory, so a restart or logout cleared it. Related context: [age-encrypted config](2026-10-17-rawrarr-age-encrypted-config.md) and [device architecture](../docs/device-architecture.md).

## Actions

- Added an explicit **Keep this ID on this device** choice during age unlock.
- When selected, Rawrarr stores the identity at `/data/age-identity` with owner-only file permissions and automatically loads it at startup. `/data` is node-local; identity material is not written to `/shared`.
- Passkey authentication remains required for browser access. The existing `RAWARR_AGE_IDENTITY_FILE` secret mount still takes precedence.
- Updated the design guide and user guide.

## Verification

Pending.

## Final state and remaining work

Pending. The user must enable the option during a successful unlock; the server cannot recover an identity that was only entered in a previous in-memory session.
