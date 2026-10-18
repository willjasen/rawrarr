-- Questions are private coordinator-local state, never Syncthing data.
CREATE TABLE rawrarr_ai_requests (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES rawrarr_users(id) ON DELETE CASCADE,
    channel TEXT NOT NULL CHECK (channel IN ('text', 'voice')),
    question TEXT NOT NULL,
    intent JSONB,
    status TEXT NOT NULL CHECK (status IN ('pending', 'completed', 'needs_review', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    finished_at TIMESTAMPTZ
);

CREATE INDEX rawrarr_ai_requests_user_time_idx
    ON rawrarr_ai_requests(user_id, created_at DESC);
