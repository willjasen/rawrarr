# Blind coordinator content flow

## Requirement

The coordinator may handle passkeys, account/node identifiers, presence, routing metadata, and opaque ciphertext. It must not receive plaintext subscriber content such as movie titles, TMDb/provider IDs, descriptions, artwork, library inventories, or decrypted configuration. HTTPS termination at the coordinator does not satisfy this boundary: application payloads must remain encrypted through the coordinator and be decrypted only in the end user's browser.

Metadata and storage nodes encrypt publishable content to the instance's public age recipient before upload. The coordinator stores or forwards the ciphertext unchanged, using opaque object identifiers that do not reveal provider IDs or content. The browser fetches authorized ciphertext and decrypts it locally with the user's age identity. Existing HMAC-derived metadata filenames are useful for shared storage but must not be treated as hiding plaintext returned by node APIs.

## Current implementation

The coordinator workspace now renders a bounded movie metadata view in the browser. Its relay accepts only `GET` requests for the metadata export index, encrypted shared config, and validated encrypted movie text/artwork objects. It rejects other node-page/API paths, so the coordinator does not proxy subscriber HTML or plaintext API responses. The metadata node serves existing age ciphertext; the browser decrypts it locally. The current index remains a three-record proof of concept; storage-node publishing, full-library indexing, and other metadata classes are not implemented yet.

## Target flow

1. Metadata/storage nodes create content records locally and encrypt each publishable record with the shared instance's age recipient. A node serves bounded ciphertext objects and an opaque inventory to the authenticated coordinator relay.
2. The coordinator authorizes node ownership and browser access, and forwards ciphertext without parsing, transforming, indexing, or logging its contents. Object IDs and inventory fields are opaque and must not encode titles, provider IDs, paths, or other content.
3. The browser fetches ciphertext and the minimum opaque inventory needed to render, decrypts in memory using the user's age identity, validates the decrypted schema, then displays titles, IDs, artwork, and other content locally.
4. Node pages that still make direct requests must use a browser-to-node encrypted channel or a ciphertext API plus a browser renderer. Do not proxy plaintext node HTML/API responses through the coordinator.

Transport encryption remains required. Age encryption protects content end-to-end; it does not hide timing, object sizes, node/account association, presence, or network metadata. Coordinator authorization and age recipient membership are separate controls. Never send a private age identity or decrypted payload to the coordinator.

## Migration and acceptance criteria

- Replace page relay for subscriber content with ciphertext publication/retrieval and browser decryption before claiming the coordinator is blind.
- Cover movie text, provider/TMDb IDs, artwork, library inventory, and future metadata classes; keep coordinator controls limited to opaque identifiers and operational status.
- Test that node uploads contain ciphertext, coordinator persistence/logging contains no known plaintext fixture markers, and browser rendering recovers the expected fixture values with the authorized age identity.
- Test that wrong identities, malformed ciphertext, unauthorized users/nodes, and invalid opaque object IDs fail closed.
- Inspect browser network traffic and coordinator request/response paths to confirm no plaintext content traverses the coordinator.
- Decide separately whether published ciphertext lives in coordinator storage or is relayed from an online node. Either design must preserve the same ciphertext-only boundary.

The coordinator page-relay boundary and browser decryption are implemented for a three-record movie view. Broader node publication, scalable opaque inventory, all content classes, and comprehensive browser regression coverage remain.
