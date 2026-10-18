# Manage browser age ID storage in Settings

## Objective

Show whether the age private ID is retained by this browser and provide explicit save and remove controls. Keep root signing material outside browser persistence and require it to be supplied per approval when signed device approval is implemented.

## Initial state

The coordinator stored the age ID in browser `localStorage` only after opt-in from Movies/TV. Settings had no status or management controls. Existing node approvals use the authenticated enrollment endpoint; the browser ML-DSA signing primitive is not integrated into that approval flow.

## Actions

- Add an age-ID storage card to Settings with stored/not-stored status, entry and save controls, and a remove action.
- Validate entered age IDs through the selected metadata node's direct age-settings endpoint before storing them. Keep the API request's `rememberOnDevice` value false so this browser preference does not persist an identity on the node.
- Removing the browser-stored ID also clears the in-memory ID, decrypted movie list, and summary values.
- Make the Settings panel and existing Movies/TV remember checkbox display the same browser-storage state. Keep browser storage limited to `rawrarr.browser.age-id`; do not save root signing keys.
- Add browser coverage for status, validation, save, remove, and root-key storage exclusion.

## Verification

No device-approval/signature behavior is added: current approval backend does not accept or verify an ML-DSA root signature, so prompting for a key here would falsely suggest that it authorizes approval.

## Recovery and remaining work

The owner can remove the stored age ID from Settings or use Clear private ID on Movies. Integrating one-time root-key entry, local signing, and key clearing into device approval requires a separate backend protocol change and signature verification; the signing key must remain only in memory for that action.

## Verification results

- Focused coordinator browser and static tests passed: 15 tests, 0 failures. The browser flow verified stored/not-stored reporting, direct metadata-node validation before save, password-field clearing, persistence, removal, and metadata lock after removal.
- The UI's browser storage key is dedicated to the age private ID. Static regression coverage rejects root/signing-key localStorage writes. No root signing key is present in the current enrollment approval UI or backend.
- Rebuilt the existing `rawrarr-local` roarlay service. The container is healthy; Tailscale Serve still routes `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`; HTTPS `/health` returned HTTP 200.
- Removed the task-owned roarlay deployment marker after verification.

## Verification results

- Focused Chromium and static coordinator checks passed: 15 tests, 0 failures. Coverage includes storage status, direct metadata-node validation before saving, clearing the entry field, persistence, removal, and metadata locking after removal.
- Settings now reports whether the ID is stored, or only present in the active tab. Removing the stored ID clears in-memory state in this and other open tabs.
- Static regression coverage confirms browser persistence is dedicated to the age private ID and does not write root/signing keys.
- Rebuilt roarlay in Compose project `rawrarr-local`; container health reached `healthy`, HTTPS `/health` returned HTTP 200, and Tailscale Serve routes remained `/` to `http://127.0.0.1:8080` and `/direct-api` to `http://127.0.0.1:8082`. The task-owned deployment marker was released after verification.

## Remaining work

The current device approval flow is still passkey-authenticated token enrollment and does not accept or verify a root signature. Implementing a one-time root-key entry, local signing, backend signature verification, and guaranteed browser-memory cleanup must be completed as an end-to-end approval protocol before the requested per-approval root signing interaction exists. The UI here does not claim to do that.
