-- Additive enrollment trust material. Existing nodes remain present with NULL
-- signing keys until they complete a separately approved upgrade.
ALTER TABLE rawrarr_nodes
    ADD COLUMN signing_public_key BYTEA,
    ADD COLUMN signing_fingerprint TEXT,
    ADD COLUMN signing_approval_payload JSONB,
    ADD COLUMN signing_approval_signature BYTEA;

ALTER TABLE rawrarr_node_enrollments
    ADD COLUMN signing_public_key BYTEA,
    ADD COLUMN key_proof_signature BYTEA;

CREATE TABLE rawrarr_instance_signing_root (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    algorithm TEXT NOT NULL CHECK (algorithm = 'ML-DSA-65'),
    public_key BYTEA NOT NULL,
    fingerprint TEXT NOT NULL CHECK (length(fingerprint) = 64),
    owner_user_id UUID NOT NULL REFERENCES rawrarr_users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE rawrarr_node_request_nonces (
    node_id TEXT NOT NULL REFERENCES rawrarr_nodes(id) ON DELETE CASCADE,
    nonce UUID NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY(node_id, nonce)
);
CREATE INDEX rawrarr_node_request_nonces_expiry_idx ON rawrarr_node_request_nonces(expires_at);
