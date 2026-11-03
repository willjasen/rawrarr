-- Provision explicitly from an authenticated account and an online fossil.
-- Do not infer an encryption recipient from the instance signing root.
CREATE TABLE rawrarr_environment_owner (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    owner_user_id UUID NOT NULL REFERENCES rawrarr_users(id),
    age_recipient TEXT NOT NULL CHECK (
        length(age_recipient) BETWEEN 20 AND 4096
        AND age_recipient ~ '^age1pq1[a-z0-9]+$'
    ),
    source_node_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
