# Node and app age identity design exploration

## Objective

Implement automatic creation and persistent reuse of a separate node age identity at subscriber startup, and record remaining node-to-node and Arr identity design questions.

## Initial state

- The current product has a user age identity for unlocking shared encrypted configuration and metadata.
- `/shared/config.aged` is encrypted using the shared instance recipient; nodes have local `/data` and synchronized `/shared` domains.
- Arr API configuration is handled through Rawrarr's existing encrypted config and browser-to-node flow. There is no established Radarr/Sonarr age identity protocol.
- The working tree contains unrelated edits in coordinator UI/source/tests and existing runbooks. Those were left untouched.

## Design recorded

Subscriber startup now checks `/data/node-age-identity`; if absent, it creates and persists a post-quantum age identity with private permissions, including in existing environments. If present, it validates and reuses it. The existing user identity at `/data/age-identity` is unchanged. Creation installs the completed file atomically; invalid existing content is preserved and causes startup failure instead of silent replacement. The coordinator role does not generate a subscriber node identity. Updated [device signing design](../docs/device-signing-design.md) to distinguish user, node, and optional integration age identities. A public node recipient must be authenticated through owner-approved enrollment; age encryption does not authenticate senders, so node-to-node messages also need the separate device-signing trust mechanism. Integration identities remain conditional on a concrete workflow that needs independent encryption principals; existing API settings alone do not justify additional keys.

## Verification

- Focused Rust tests passed (2/2): generation, repeat-start reuse, preservation of a separate user identity, private file permissions on Unix, and refusal to overwrite an invalid identity.
- Full Rust suite compiled; 65/66 tests passed. The existing coordinator direct-route test failed in `coordinator::direct_route_tests::direct_routes_must_be_safe_paths_on_a_separate_tailnet_origin`; that file already had unrelated working-tree edits before this task.
- The isolated Docker runner's browser test passed. Its Node suite had 21 failures because the runner copied only `rust-arr`, tests, and Compose rather than the full current source tree; its Rust stage could not resolve a missing cached `mime_guess` crate with networking disabled. A direct host run with the pinned Rust 1.85.1 toolchain and cached dependencies succeeded for the focused tests and ran the full Rust suite.
- `git diff --check` passed. A full rustfmt check reports existing formatting differences across the crate and included test modules, so no broad reformat was applied.
- Confirmed implementation uses the existing `age-keygen -pq` utility already shipped in the Rawrarr image.

## Remaining design work

- Define the enrollment/rotation/recovery protocol and node recipient publication.
- Define an authenticated node-recipient registry, message envelope, replay protection, and behavior while peers are offline or revoked.
- Identify a concrete Arr use case before assigning Radarr or Sonarr an independent identity; specify which process owns each private key and how credentials/config are isolated.
- Update current-state/technical docs and implementation plans when these decisions are accepted for implementation.
