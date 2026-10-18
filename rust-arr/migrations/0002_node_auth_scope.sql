-- Preserve the existing local browser sessions under its approved metadata node ID.
ALTER TABLE rawrarr_auth_challenges ADD COLUMN node_id TEXT NOT NULL DEFAULT 'macbook-metadata';
ALTER TABLE rawrarr_sessions ADD COLUMN node_id TEXT NOT NULL DEFAULT 'macbook-metadata';
CREATE TABLE rawrarr_nodes (
    id TEXT PRIMARY KEY,
    role TEXT NOT NULL CHECK (role IN ('storage', 'metadata')),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now()
);
