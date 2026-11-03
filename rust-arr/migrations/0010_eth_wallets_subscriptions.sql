CREATE TABLE IF NOT EXISTS rawrarr_eth_wallets (
    address TEXT PRIMARY KEY CHECK (address ~ '^0x[0-9a-f]{40}$'),
    user_id UUID NOT NULL REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    ens_name TEXT NOT NULL,
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_auth_at TIMESTAMPTZ,
    UNIQUE(user_id)
);

CREATE TABLE IF NOT EXISTS rawrarr_eth_auth_challenges (
    id UUID PRIMARY KEY,
    flow TEXT NOT NULL CHECK (flow IN ('wallet_login', 'wallet_link')),
    address TEXT NOT NULL CHECK (address ~ '^0x[0-9a-f]{40}$'),
    ens_name TEXT NOT NULL,
    message TEXT NOT NULL,
    nonce TEXT NOT NULL UNIQUE,
    user_id UUID REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    CHECK ((flow = 'wallet_link') = (user_id IS NOT NULL))
);

CREATE INDEX IF NOT EXISTS rawrarr_eth_auth_challenges_expiry
    ON rawrarr_eth_auth_challenges(expires_at);

CREATE TABLE IF NOT EXISTS rawrarr_eth_payments (
    tx_hash TEXT PRIMARY KEY CHECK (tx_hash ~ '^0x[0-9a-f]{64}$'),
    account_id UUID NOT NULL REFERENCES rawrarr_users(id) ON DELETE RESTRICT,
    payer_address TEXT NOT NULL CHECK (payer_address ~ '^0x[0-9a-f]{40}$'),
    plan TEXT NOT NULL CHECK (plan IN ('monthly', 'annual')),
    amount_wei NUMERIC(78,0) NOT NULL CHECK (amount_wei > 0),
    recipient_address TEXT NOT NULL CHECK (recipient_address ~ '^0x[0-9a-f]{40}$'),
    block_number NUMERIC(78,0) NOT NULL CHECK (block_number >= 0),
    block_hash TEXT NOT NULL CHECK (block_hash ~ '^0x[0-9a-f]{64}$'),
    verified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS rawrarr_eth_payments_account
    ON rawrarr_eth_payments(account_id, verified_at DESC);

CREATE TABLE IF NOT EXISTS rawrarr_eth_subscriptions (
    account_id UUID PRIMARY KEY REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    paid_through TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
