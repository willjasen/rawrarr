-- Umami 3.4.1 PostgreSQL retention policy: keep visitor/event data for 90 days.
-- Run as a single transaction; child records are deleted before their parents.
BEGIN;

DELETE FROM event_data
WHERE created_at < now() - interval '90 days';

DELETE FROM session_data
WHERE created_at < now() - interval '90 days';

DELETE FROM revenue
WHERE created_at < now() - interval '90 days';

DELETE FROM heatmap_event
WHERE created_at < now() - interval '90 days';

DELETE FROM session_replay
WHERE created_at < now() - interval '90 days';

DELETE FROM session_replay_saved
WHERE created_at < now() - interval '90 days';

DELETE FROM website_event
WHERE created_at < now() - interval '90 days';

DELETE FROM session_link
WHERE created_at < now() - interval '90 days';

DELETE FROM session
WHERE created_at < now() - interval '90 days'
  AND NOT EXISTS (
    SELECT 1
    FROM website_event
    WHERE website_event.session_id = session.session_id
  );

COMMIT;
