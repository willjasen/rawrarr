# Keep the relay blind to subscriber app data

## Objective and initial state

Record the trust boundary for the planned Rawrarr relay/subscription service. The user clarified that “main server” refers to this future cloud service, not a subscriber's self-hosted Rawrarr node. This repository contains the self-hosted node and planning documents; it does not contain a relay/subscription server implementation. Related references: [cloud access architecture](../docs/cloud-access-architecture.md), [subscription model](../docs/subscription-model.md), and [device architecture](../docs/device-architecture.md).

## Decisions recorded

- The relay/subscription account service may store passkey account records and public age recipients associated with the passkey-authenticated account in PostgreSQL.
- The planned cloud database models accounts, passkey credentials, public age recipient records, and an explicit passkey-to-recipient association; none of these records contain a private identity.
- Public recipients are public encryption keys, never login credentials. The matching private age identity must never be sent to or held by the relay/subscription service.
- Subscriber Rawrarr app data remains on subscriber-controlled nodes and their Syncthing folder. The relay must not age-encrypt, decrypt, or persist app-data payloads; remote relay mode forwards the end-to-end encrypted TLS stream unchanged. This service must not make an encrypted-at-rest copy of subscriber app data because it must not store that payload in the first place.
- A subscriber's own Rawrarr node is a distinct trust boundary and remains responsible for its local app behavior. Do not confuse it with the cloud relay/subscription service.

## Verification plan for future relay implementation

- Database integration: create an account with passkey credentials, associate one or more valid public post-quantum recipients with it, and confirm another account cannot enumerate or change them.
- API/input audit: reject private identity labels/formats from recipient-registration fields; allow only validated public recipients. Never log request bodies.
- Payload pass-through: use a known encrypted byte fixture and assert the relay forwards it byte-for-byte without decrypting, wrapping, or storing a payload copy.
- Secret absence: scan service config, environment, logs, database schema, and process inputs for private-identity fields or age decryption keys; assert none are accepted or retained.
- Failure/privacy: verify that subscription outages and route errors expose no app-data content and that logs retain only minimal opaque routing identifiers.

## Current state and remaining work

Architecture and subscription documents now state this boundary. `git diff --check` passed, and the existing passkey/age UI Node tests passed (3 tests). The local Rawrarr health endpoint still returned `ok`. No relay implementation exists here, so runtime enforcement and the relay-specific tests above remain future work. The current local Rawrarr node was not rebuilt or restarted for this documentation-only clarification; its code is separate from the future cloud service.
