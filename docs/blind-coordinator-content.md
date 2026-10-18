# Blind coordinator content flow

## Requirement

The coordinator may handle passkeys, account/node identifiers, presence, routing metadata, and opaque ciphertext. It must not receive plaintext subscriber content such as movie titles, TMDb/provider IDs, descriptions, artwork, library inventories, or decrypted configuration. HTTPS termination at the coordinator does not satisfy this boundary: application payloads must remain encrypted through the coordinator and be decrypted only in the end user's browser.

Metadata and storage nodes encrypt publishable content to the instance's public age recipient before upload. The coordinator stores or forwards the ciphertext unchanged, using opaque object identifiers that do not reveal provider IDs or content. The browser fetches authorized ciphertext and decrypts it locally with the user's age identity. Existing HMAC-derived metadata filenames are useful for shared storage but must not be treated as hiding plaintext returned by node APIs.

## Current implementation

The roarlay workspace renders full Movies and TV inventories from selected metadata nodes. Library responses are encrypted for a one-request browser session, so roarlay forwards opaque bytes; the browser decrypts them. The movie export reads validated age ciphertext and decrypts all indexed movie text records and matching available artwork in the browser. The relay is limited to configured nodes and scoped paths. Storage-node content publication, export of other metadata classes, and a production-scale streaming inventory are not implemented yet.

## Target flow

1. Metadata/storage nodes create content records locally and encrypt each publishable record with the shared instance's age recipient. A node serves bounded ciphertext objects and an opaque inventory to the authenticated coordinator relay.
2. The coordinator authorizes node ownership and browser access, and forwards ciphertext without parsing, transforming, indexing, or logging its contents. Object IDs and inventory fields are opaque and must not encode titles, provider IDs, paths, or other content.
3. The browser fetches ciphertext and the minimum opaque inventory needed to render, decrypts in memory using the user's age identity, validates the decrypted schema, then displays titles, IDs, artwork, and other content locally.
4. Node pages that still make direct requests must use a browser-to-node encrypted channel or a ciphertext API plus a browser renderer. Do not proxy plaintext node HTML/API responses through the coordinator.

Transport encryption remains required. Age encryption protects content end-to-end; it does not hide timing, object sizes, node/account association, presence, or network metadata. Coordinator authorization and age recipient membership are separate controls. Never send a private age identity or decrypted payload to the coordinator.

## Migration and acceptance criteria

- Keep every subscriber-content route within browser-encrypted responses or age ciphertext, and test new relay paths before claiming the coordinator is blind.
- Extend the ciphertext boundary to future metadata classes and storage-node content; keep coordinator controls limited to opaque identifiers and operational status.
- Test that node uploads contain ciphertext, coordinator persistence/logging contains no known plaintext fixture markers, and browser rendering recovers the expected fixture values with the authorized age identity.
- Test that wrong identities, malformed ciphertext, unauthorized users/nodes, and invalid opaque object IDs fail closed.
- Inspect browser network traffic and coordinator request/response paths to confirm no plaintext content traverses the coordinator.
- Decide separately whether published ciphertext lives in coordinator storage or is relayed from an online node. Either design must preserve the same ciphertext-only boundary.

Browser decryption is implemented for full Movies and TV inventory responses and all indexed movie export records. Broader node publication, other content classes, scalable streaming export, and comprehensive browser regression coverage remain.
