# Metadata export plan

## Goal

Let an owner download a usable copy of `/shared/metadata` for personal backup, inspection, or reuse. The exported files are plaintext because the user deliberately decrypts them with their age secret in their browser. This is a metadata export, not a full Rawrchive export: it excludes encrypted configuration, logs, account data, node-local state, databases, and all media bytes.

## User experience

- Add **Export metadata** to the metadata or privacy controls. Explain that artwork and metadata records will be decrypted in this browser and saved as a ZIP.
- Require an unlocked age identity in the browser. Show the estimated item count/size, progress, cancellation, and a clear error for missing, corrupt, unsupported, or undecryptable files. Do not silently omit failures; offer a complete export only when every selected file succeeds, with an explicit option to download a report for a partial export.
- Keep private key material in browser memory for the operation only; clear references and temporary plaintext buffers where the runtime permits after completion/cancel. Never put it in a request, URL, local/session storage, IndexedDB, logs, analytics, or error reports.
- Build the ZIP locally in the browser. Do not upload decrypted bytes or write them to server-side temporary storage. The user chooses where the browser saves the resulting archive.

## Proposed archive

Keep the existing category structure (`movies`, `tv`, `music`, `books`, `library`) and content-kind subdirectories (`text`, `artwork`). Decrypt each `.aged` object and write only plaintext entries. For movie records and artwork, use stable TMDb IDs in exported names (for example `movies/text/12345.json` and `movies/artwork/12345.jpg`) instead of opaque HMAC filenames. Include a small `manifest.json` mapping each exported path to its original encrypted relative path, media identifier when known, and content type. Preserve unknown/legacy entries under their relative path with the encryption suffix removed and record their status in the manifest; do not invent titles or discard unrecognized content. Never include the HMAC filename key or age identity in the archive.

This layout should be documented as an export format version so later schema changes can remain compatible. It is intentionally distinct from the synchronized on-disk ciphertext tree and is not suitable for syncing as shared Rawrarr app data.

## Trust and architecture boundaries

- The authenticated coordinator authorizes the export and provides an inventory of metadata paths plus opaque file identifiers. A metadata/storage service may read the files, but it returns ciphertext bytes only; it never receives the user's age secret and never decrypts for this feature.
- Validate every requested path against the metadata root and current user's authorized shared instance. Do not accept arbitrary filesystem paths from the browser. Stream ciphertext with bounded concurrency and size limits; avoid logging file contents or sensitive plaintext metadata.
- Decryption and archive construction happen in the browser. Use a maintained, reviewed age implementation that supports the exact recipient/ciphertext formats Rawrarr writes, including the current post-quantum identity format. Confirm this compatibility before selecting the browser library or promising support. Do not invent a custom age parser or cryptographic format.
- Keep passkey authentication separate from age decryption. A signed-in session or approved node is not a decryption key. Require HTTPS for remote coordinator access.
- If the browser implementation cannot safely and reliably handle the active age format, keep this feature planned until it can; do not fall back to sending the private identity to the server.

## Implementation sequence

1. Inventory every current metadata class, schema, age recipient format, file size range, and legacy naming case. Define which content is included and test compatibility against real-format sanitized fixtures.
2. Prototype browser decryption of one small text record and one artwork file using a maintained implementation. Verify plaintext matches the source and that wrong keys, truncation, tampering, and unsupported formats fail closed.
3. Add an authenticated, read-only ciphertext inventory/download API scoped to the owner's shared metadata tree, with path containment, pagination or streaming, bounded concurrency, and no plaintext code path.
4. Implement a browser export worker for decryption, manifest generation, ZIP streaming, progress, cancellation, and robust partial-failure reporting. Keep sensitive key and plaintext handling in memory.
5. Add focused tests for API authorization/path traversal and browser export naming, completeness, failure cases, and archive contents. Verify with the browser's actual save flow and inspect network requests to confirm only ciphertext is returned.

## Open design questions

- Should exports include only successfully decrypted recognized records, or permit a partial archive plus a mandatory failure report? Recommended: full export by default; partial export only after an explicit choice and visibly marked in the manifest.
- How should future TV/music/book schemas map provider IDs and artwork extensions into stable export names?
- Which browsers and maximum archive sizes are in scope? Large libraries may need a file-streaming API such as the File System Access API where supported, with a portable ZIP-download fallback.
- Should users optionally include encrypted `config.aged` in a separate ciphertext backup workflow? It must remain outside this plaintext metadata export.

## Current movie export

The current implementation exports every indexed movie text ciphertext and its matching available artwork. It uses the pinned `age-encryption` 0.3.1 TypeScript package; the export is an interoperability proof, not a formal security audit or production-scale review. The browser fetches encrypted `config.aged`, decrypts it with the post-quantum age ID, reads `metadata_filename_key`, and independently recomputes each text and artwork HMAC filename from the decrypted TMDb ID. It checks that the text filename matches its decrypted record, decrypts matched artwork, and saves a ZIP assembled in browser memory. The authenticated API exposes validated ciphertext filenames and bytes; it never receives the age ID or HMAC key.

The Settings action uses an age identity already held in browser memory or asks for one in a one-use dialog; the dialog input is cleared after the attempt. Wrong-key errors are deliberately generic so crypto-library diagnostics cannot echo the supplied identity. JavaScript cannot guarantee zeroization of internal crypto-library strings or garbage-collected copies; byte buffers are cleared on completion where practical.

## Status

The all-indexed-movies export is implemented, but the ZIP is assembled in browser memory. Export of other media classes, cancellation, bounded streaming ZIP output, and complete/partial failure-report workflow remain planned.
