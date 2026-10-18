-- Enrollment ownership belongs to the account on this coordinator instance.
ALTER TABLE rawrarr_nodes
    ADD COLUMN user_id UUID REFERENCES rawrarr_users(id) ON DELETE CASCADE;

ALTER TABLE rawrarr_node_enrollments
    ADD COLUMN user_id UUID;

-- This deployment supports one account per coordinator. Backfill existing
-- nodes and pending requests to that account; discard only unowned transient
-- requests that could not be associated with an account.
UPDATE rawrarr_nodes
SET user_id = (SELECT id FROM rawrarr_users ORDER BY created_at LIMIT 1)
WHERE user_id IS NULL
  AND (SELECT count(*) FROM rawrarr_users) = 1;

UPDATE rawrarr_node_enrollments
SET user_id = (SELECT id FROM rawrarr_users ORDER BY created_at LIMIT 1)
WHERE user_id IS NULL
  AND (SELECT count(*) FROM rawrarr_users) = 1;

DELETE FROM rawrarr_node_enrollments WHERE user_id IS NULL;

ALTER TABLE rawrarr_node_enrollments
    ALTER COLUMN user_id SET NOT NULL,
    ADD CONSTRAINT rawrarr_node_enrollments_user_id_fkey
        FOREIGN KEY (user_id) REFERENCES rawrarr_users(id) ON DELETE CASCADE;

CREATE INDEX rawrarr_node_enrollments_user_pending_idx
    ON rawrarr_node_enrollments(user_id, requested_at)
    WHERE approved_at IS NULL;
