-- Trial timestamps are account-local billing state in PostgreSQL. Legacy
-- accounts remain unset so deploying this migration cannot expire them.
ALTER TABLE rawrarr_users
    ADD COLUMN trial_started_at TIMESTAMPTZ;
