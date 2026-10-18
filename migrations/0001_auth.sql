CREATE TABLE rawrarr_users (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE rawrarr_passkeys (
    credential_id BYTEA PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    passkey JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX rawrarr_passkeys_user_id_idx ON rawrarr_passkeys(user_id);

CREATE TABLE rawrarr_auth_challenges (
    id UUID PRIMARY KEY,
    flow TEXT NOT NULL CHECK (flow IN ('registration', 'authentication')),
    user_id UUID NOT NULL,
    username TEXT NOT NULL,
    display_name TEXT NOT NULL DEFAULT '',
    state JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX rawrarr_auth_challenges_expiry_idx ON rawrarr_auth_challenges(expires_at);

CREATE TABLE rawrarr_sessions (
    token_hash BYTEA PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX rawrarr_sessions_expiry_idx ON rawrarr_sessions(expires_at);
