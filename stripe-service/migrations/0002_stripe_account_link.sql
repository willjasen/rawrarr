CREATE TABLE rawrarr_stripe_customers (
    account_id UUID PRIMARY KEY REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    stripe_customer_id TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE rawrarr_stripe_subscriptions (
    stripe_subscription_id TEXT PRIMARY KEY,
    account_id UUID NOT NULL REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    stripe_customer_id TEXT NOT NULL REFERENCES rawrarr_stripe_customers(stripe_customer_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    current_period_end TIMESTAMPTZ,
    cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,
    stripe_event_created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX rawrarr_stripe_subscriptions_account_id_idx
    ON rawrarr_stripe_subscriptions(account_id);
