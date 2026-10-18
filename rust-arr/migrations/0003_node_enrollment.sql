-- Store approved node credentials and short-lived node enrollment requests.
ALTER TABLE rawrarr_nodes ADD COLUMN token_sha256 TEXT;
ALTER TABLE rawrarr_nodes ADD COLUMN origin TEXT;
ALTER TABLE rawrarr_nodes ADD COLUMN rp_id TEXT;
CREATE UNIQUE INDEX rawrarr_nodes_token_sha256_idx ON rawrarr_nodes(token_sha256) WHERE token_sha256 IS NOT NULL;

CREATE TABLE rawrarr_node_enrollments (
    request_id UUID PRIMARY KEY,
    node_id TEXT NOT NULL UNIQUE,
    role TEXT NOT NULL CHECK (role IN ('storage', 'metadata')),
    display_name TEXT NOT NULL,
    token_sha256 TEXT NOT NULL CHECK (length(token_sha256) = 64),
    origin TEXT NOT NULL,
    rp_id TEXT NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    approved_at TIMESTAMPTZ
);
CREATE INDEX rawrarr_node_enrollments_pending_idx ON rawrarr_node_enrollments(requested_at)
    WHERE approved_at IS NULL;
