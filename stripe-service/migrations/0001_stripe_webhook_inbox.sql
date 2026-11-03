CREATE TABLE stripe_webhook_events (
    event_id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    account_id UUID REFERENCES rawrarr_users(id) ON DELETE SET NULL,
    stripe_created_at TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX stripe_webhook_events_received_at_idx
    ON stripe_webhook_events(received_at);
