# Move age ID import into a pop-up

## Objective and initial state

Change the existing identity action to **Import ID** and move the masked private-key field off the setup page into a dialog opened by that action. Previously the paste field was always visible beside ID creation. Related context: [direct pasted-ID unlock](2026-10-18-pasted-age-id-direct-unlock.md).

## Actions

- Replaced the inline field/action with **Import ID**.
- Added a focused dialog with a masked post-quantum ID field and explicit **Unlock vault** action; close it only after successful unlock.
- Kept the generated-ID copy and backup flow separate.
- Updated the browser regression test and user/design documentation.

## Verification

Pending.

## Final state and remaining work

Pending.
