-- Keep only the current signed grant per app instance and approved fossil.
-- The JSON bundle contains public metadata and age-encrypted identity bytes.
CREATE TABLE rawrarr_app_key_grants (
    app_instance_id UUID NOT NULL,
    device_id TEXT NOT NULL REFERENCES rawrarr_nodes(id) ON DELETE CASCADE,
    device_recipient TEXT NOT NULL,
    key_epoch BIGINT NOT NULL CHECK (key_epoch > 0),
    bundle BYTEA NOT NULL CHECK (octet_length(bundle) BETWEEN 1 AND 65536),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (app_instance_id, device_id)
);
