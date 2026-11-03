-- Keep owner-rejected enrollment identities from returning to the pending queue.
CREATE TABLE rawrarr_rejected_nodes (
    node_id TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES rawrarr_users(id),
    request_id UUID NOT NULL,
    role TEXT NOT NULL,
    signing_public_key BYTEA NOT NULL,
    signing_fingerprint TEXT NOT NULL CHECK (length(signing_fingerprint) = 64),
    root_fingerprint TEXT NOT NULL CHECK (length(root_fingerprint) = 64),
    signed_payload JSONB NOT NULL,
    signature BYTEA NOT NULL,
    rejected_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX rawrarr_rejected_nodes_signing_fingerprint_idx
    ON rawrarr_rejected_nodes(signing_fingerprint);
