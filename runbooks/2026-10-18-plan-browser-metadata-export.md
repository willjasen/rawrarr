# Plan browser-side metadata export

## Objective and initial state

Record a safe product and architecture plan for a user-downloadable plaintext copy of Rawrarr's metadata tree. The current shared tree contains age-encrypted metadata and artwork, with movie filenames derived from an encrypted HMAC key. The repository already has extensive user changes; they were preserved.

## Actions

- Reviewed the current product design, metadata data-domain rules, repository instructions, and existing metadata runbook.
- Added [the metadata export plan](../docs/metadata-export-plan.md), scoped to metadata only and explicitly excluding broader Rawrchive contents and media.
- Chose browser-held age decryption and local ZIP construction as the target. The metadata service returns ciphertext only; compatibility with Rawrarr's active post-quantum age format is a prerequisite to implementation.
- Proposed stable identifier-based paths and a manifest to make exports useful despite opaque on-disk filenames.

## Verification

- Documentation links and scope were checked against the existing `docs/data-domains.md` and metadata filename design.
- No code, service, or data was changed; no runtime test was applicable.

## Final state and remaining work

The feature is documented as planned, not implemented. Next work is to inventory formats and prototype browser decryption with sanitized fixtures before designing or exposing an export endpoint.
