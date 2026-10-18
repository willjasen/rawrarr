ALTER TABLE rawrarr_auth_challenges
    DROP CONSTRAINT rawrarr_auth_challenges_flow_check,
    ADD CONSTRAINT rawrarr_auth_challenges_flow_check
        CHECK (flow IN ('registration', 'authentication', 'add_passkey')),
    ADD COLUMN link_token_sha256 TEXT;

CREATE TABLE rawrarr_passkey_link_tokens (
    token_sha256 TEXT PRIMARY KEY CHECK (length(token_sha256) = 64),
    user_id UUID NOT NULL REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    source_node_id TEXT NOT NULL REFERENCES rawrarr_nodes(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ
);

ALTER TABLE rawrarr_auth_challenges
    ADD CONSTRAINT rawrarr_auth_challenges_link_token_fkey
        FOREIGN KEY (link_token_sha256)
        REFERENCES rawrarr_passkey_link_tokens(token_sha256)
        ON DELETE CASCADE;

CREATE INDEX rawrarr_passkey_link_tokens_expiry_idx
    ON rawrarr_passkey_link_tokens(expires_at);
