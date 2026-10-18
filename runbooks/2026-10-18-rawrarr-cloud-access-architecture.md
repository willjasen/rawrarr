# Plan Rawrarr cloud access

## Objective and initial state

Record the user's architecture analogy: a Rawrarr cloud portal should help a person reach their own nodes while the hosted service itself has no access to node data or configuration. Existing device architecture already required an outbound node tunnel, opaque routing IDs, and TLS that terminates at the node; see [device architecture](../docs/device-architecture.md).

## Actions

- Defined the cloud portal as an account/discovery entry point, Rawrarr nodes as the application and authorization authority, and a relay as an opaque encrypted-stream forwarder.
- Separated portal authentication, node passkeys, age decryption, subscription entitlements, Tailscale connectivity, and Syncthing file synchronization.
- Documented candidate connection flow, data visibility boundaries, security constraints, open design decisions, and implementation sequence in [the access architecture](../docs/cloud-access-architecture.md).

## Verification and final state

Documentation-only architecture planning. No cloud account, relay, node tunnel, identity provider, or service configuration was created or changed. `git diff --check` passed, and relative links in the changed architecture documents resolve. The model does not claim details about UniFi's internal implementation.
