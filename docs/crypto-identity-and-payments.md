# ENS identity and ETH payments

## Scope

The coordinator supports one optional Ethereum wallet per Rawrarr account. A user must first create and recover an account through the existing passkey flow, then link the wallet by signing a short-lived SIWE challenge. ENS names are resolved on Ethereum mainnet through the ENS Universal Resolver. The stored ENS name is a display/login handle; it is not an authenticator by itself. Rawrarr never requests private keys or seed phrases.

The current resolver call is an on-chain `eth_call` against the mainnet Universal Resolver. It fails closed when resolution requires off-chain CCIP Read because no gateway trust/allowlist policy has been selected. Wallet account abstraction signatures (EIP-1271) are not accepted; EOA `personal_sign` signatures are supported. These limits should be revisited before broad ENS wallet support is advertised.

## Donations

Donations are optional and independent of login and account entitlement. A visitor chooses a positive ETH amount; the coordinator validates the configured recipient, amount syntax, public billing audience, and Ethereum mainnet RPC availability, then returns the transfer details. The browser submits a direct native ETH transfer from the donor's wallet. The coordinator does not create a payment or subscription record for a donation, and donation transaction hashes cannot be used as subscription claims.

## Manual subscription

The browser can submit a configured monthly or annual exact wei amount to the configured treasury address. The account must have a linked ENS wallet, and the transfer sender must equal that wallet. The coordinator verifies chain ID 1, recipient, value, empty transaction input, successful receipt, transaction/receipt block agreement, inclusion in the `finalized` block, and canonical block hash before recording the payment and advancing `paid_through`. A transaction may be claimed only once. Renewals are explicitly user initiated. No recurring contract is deployed or used.

## Account tiers

Public accounts get the existing three-day trial. The trial grants full access. After it ends, the free tier retains the account and allows one fossil metadata node. A paid subscription removes the free fossil cap while paid. Donations do not grant or extend access. Internal deployments bypass this public entitlement system. Storage nodes are not counted in the fossil limit. Deleting nodes or subscriber data is never an automatic consequence of tier changes.

This initial paid tier does not yet add a signed offline license to subscriber nodes. The coordinator is the entitlement authority. Final subscription prices, product periods, paid node limits, treasury ownership/recovery, and the free tier's other feature limits remain launch decisions. Keep the payment UI disabled until `RAWARR_ETH_TREASURY_ADDRESS` and a positive `RAWARR_ETH_MONTHLY_PRICE_WEI` and/or `RAWARR_ETH_ANNUAL_PRICE_WEI` are deliberately configured. `RAWARR_ETH_RPC_URL` may select a trusted Ethereum mainnet JSON-RPC endpoint; its URL cannot contain credentials or query parameters. The default endpoint is public and is not an availability guarantee.

## Data and service boundary

The coordinator database stores linked public addresses and ENS names, short-lived SIWE challenges, transaction hashes, and verified public payment metadata. It never stores wallet signing keys. The routes live only in the coordinator browser router. Subscriber nodes receive no billing or wallet credentials and do not independently interpret a donation as entitlement. Do not put these records or RPC credentials into `/shared`, media directories, or subscriber configuration.

## Configuration

- `RAWARR_BILLING_AUDIENCE=public` enables public-account trial, free-tier enforcement, wallet auth, and ETH payment routes. Internal deployments keep `internal`.
- `RAWARR_ETH_TREASURY_ADDRESS` enables donations; an unset recipient leaves donation controls disabled.
- `RAWARR_ETH_MONTHLY_PRICE_WEI` and `RAWARR_ETH_ANNUAL_PRICE_WEI` configure the optional exact amounts. Either may be omitted.
- `RAWARR_ETH_RPC_URL` optionally selects the HTTPS mainnet RPC URL. Credentials, query strings, and fragments are rejected; HTTP loopback is allowed for local tests.

No real payment has been made as part of development or testing. Use a local mocked wallet and mocked JSON-RPC responses for automated checks.
