-- Keep deleted preconfigured devices from being restored from the startup registry.
CREATE TABLE rawrarr_revoked_nodes (
    node_id TEXT PRIMARY KEY,
    token_sha256 TEXT NOT NULL,
    revoked_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
