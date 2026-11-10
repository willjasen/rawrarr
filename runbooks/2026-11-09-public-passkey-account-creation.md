# Public passkey account creation

Issue: [#448](https://github.com/willjasen/rawrarr/issues/448)

## Scope and behavior

The public `/auth` page offers **Sign in with passkey** and **Create Passkey**.
Passkey sign-in presents credentials registered to any account on the
coordinator; neither a username nor a password is part of the browser sign-in
flow. ENS wallet sign-in remains available for accounts that have linked a
wallet. Creating an account uses the existing WebAuthn registration ceremony
and creates an independent user/session.

The first account still claims preconfigured unowned nodes during initial
setup. Additional accounts do not inherit nodes or existing users' data.
Coordinator node enrollment continues to reject zero-account and multi-account
instances; owner-issued invitations are a separate future feature.

## Data compatibility and rollout

No database migration is needed. The existing user, passkey, session, and
registration/authentication-challenge tables already represent multiple
accounts. Existing users and credentials remain unchanged. After the new
coordinator application is deployed, all accounts can use passkey sign-in.
An older application revision still signs in its original first account, but
does not support registering or signing into additional accounts; deploy the
new coordinator revision before enabling public account creation. Rollback
does not delete added accounts, but an older revision cannot authenticate
them until the compatible application is restored.

## Verification

Passed:

```sh
node --test scripts/tests/test_rawrarr_passkey_ui.mjs
RUSTC="$HOME/.rustup/toolchains/1.85.1-x86_64-apple-darwin/bin/rustc" "$HOME/.rustup/toolchains/1.85.1-x86_64-apple-darwin/bin/cargo" check --locked --manifest-path rust-arr/Cargo.toml
RUSTC="$HOME/.rustup/toolchains/1.85.1-x86_64-apple-darwin/bin/rustc" "$HOME/.rustup/toolchains/1.85.1-x86_64-apple-darwin/bin/cargo" test --locked --manifest-path rust-arr/Cargo.toml auth::tests:: -- --nocapture
node --test /workspace/scripts/tests/test_rawrarr_passkey_ui_browser.mjs
```

The browser test ran offline in a linked OrbStack clone of the current
worktree. It verified the registration-only initial state, both actions for an
existing account, ENS wallet availability, and desktop (1280×800) and mobile
(375×812) layout without horizontal overflow.

The PostgreSQL-backed endpoint regression remains to be run in a disposable
database ending in `_test`:

```sh
python3 scripts/tests/test_rawrarr_passkey.py --binary <release-binary> --database-url <disposable-database-ending-in-_test>
```

It verifies that an existing account no longer blocks another registration
challenge, username-based passkey selection is rejected, and malformed
registration still creates neither an account nor a session. A complete valid
second WebAuthn credential ceremony and multi-account browser login are not
covered by this focused endpoint test.
