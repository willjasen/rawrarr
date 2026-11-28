# Update vulnerable Rust URL dependency

Issue: [#508](https://github.com/willjasen/rawrarr/issues/508)

## Objective and initial state

The repository scanner reported a Rust `url` dependency issue: `Url::origin()` can recursively parse nested `blob:` URLs and overflow the stack. Both `rust-arr/Cargo.lock` and `stripe-service/Cargo.lock` selected `url` 2.5.8.

## Actions

Updated `url` to 2.5.9 in both lockfiles using Cargo's precise update. Cargo also advanced `form_urlencoded` from 1.2.2 to 1.2.3 as required by the new URL release. No manifest or application logic changed.

## Compatibility and rollout

This is a dependency-only change. It changes no stored data, schema, encrypted configuration, or node/coordinator protocol, so no migration is needed. Existing deployments adopt it through the normal published application image update for the affected Rust component(s).

## Verification

- `cargo tree -p url --locked` in `rust-arr/` resolved `url v2.5.9`.
- `cargo tree --manifest-path stripe-service/Cargo.toml -p url --locked` resolved `url v2.5.9`.
- Lockfile diff was limited to `url` 2.5.8 → 2.5.9 and `form_urlencoded` 1.2.2 → 1.2.3 in each lockfile.
- Full compile and test suites were not run.

## Final state

Both Rust application lockfiles select the patched URL parser. No deployment was performed.
