# ETH donations, ENS sign-in, and subscription tiers

Issue: [#208](https://github.com/willjasen/rawrarr/issues/208)

## Decisions

- Accept ETH on Ethereum mainnet first. Subscription renewals are manual; automatic recurring charging is a later smart-contract feature that needs separate audit and deployment work.
- ENS names are account handles only after a wallet signs a one-time SIWE challenge. The existing passkey flow creates an account; an authenticated passkey session links one ENS wallet.
- Donations are available to visitors, accept a user-selected amount, and never create or change account entitlement.
- Public accounts have a full-access three-day trial, then a permanent free tier with one fossil (metadata) node. A paid subscription removes that cap. Internal deployments bypass public billing enforcement.
- Keep the payment UI disabled until operators set a treasury address and at least one positive fixed plan amount. Prices and the paid-tier's eventual node limits require product decisions before launch.

## Implementation

The coordinator database stores one linked public address and ENS name per account, expiring SIWE challenges, verified transaction records, and `paid_through`. Wallet login and payment routes run only inside the coordinator's browser auth context. Subscriber nodes receive no payment secrets or wallet credentials. Subscription claims require the linked wallet's direct native ETH transfer to the configured recipient for the exact configured amount, a successful receipt, Ethereum mainnet, and finalized canonical inclusion. Payments are consumed idempotently by transaction hash. Account-scoped PostgreSQL advisory locks serialize free-tier fossil enrollment and approval checks.

ENS resolution uses the mainnet Universal Resolver through `eth_call`. The current implementation does not follow CCIP Read gateway responses and only accepts EOA `personal_sign` signatures; it fails closed for those unsupported cases. The default public RPC is configurable and is not an availability promise.

## Configuration

- `RAWARR_BILLING_AUDIENCE=public` enables public billing behavior; keep internal deployments at `internal`.
- `RAWARR_ETH_TREASURY_ADDRESS` enables voluntary donations.
- `RAWARR_ETH_MONTHLY_PRICE_WEI` and `RAWARR_ETH_ANNUAL_PRICE_WEI` enable their respective exact-amount manual subscription plans.
- `RAWARR_ETH_RPC_URL` may point at a trusted HTTPS Ethereum mainnet JSON-RPC endpoint without credentials/query/fragment.

Do not configure a real treasury or price until the operator wallet, recovery procedure, plan values, public disclosures, and paid node limits are confirmed. Automated tests use mocked wallet and RPC calls; they must never broadcast a transaction.

## Verification status

- Passed: `cargo test --locked coordinator_crypto::tests` in the Rust 1.85 OrbStack container (6 tests). This compiles the coordinator binary and test target; the focused crypto tests cover ENS name normalization/namehash/resolver ABI, EIP-191 recovery, SIWE fields, and crypto input validation.
- Passed: `node --experimental-global-webcrypto --test scripts/tests/test_rawrarr_crypto_ui.mjs` in the isolated Playwright image (12 tests, including browser-driven donations, manual subscription, mainnet and linked-wallet preflight, ENS sign-in and wrong-account rejection, plus free-tier enforcement assertions).
- Passed: `git diff --check`.
- Broader Rust suite attempted: `cargo test --locked` compiled successfully; 90 of 103 tests passed and 13 unrelated environment-dependent tests failed because this minimal Rust container lacks `age-keygen`, image encoders, and some test fixture files. The six focused crypto tests pass.
- Not run: the complete Rawrarr regression and environment stacks. An earlier full runner attempt also reported existing failures in branding, Stripe files absent from this task branch, and browser timing/layout cases; the ETH/ENS Playwright suite passes independently.
- No live coordinator deployment, live RPC/ENS request, or real payment was performed.
