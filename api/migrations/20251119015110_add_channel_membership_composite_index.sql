-- Add composite index to optimize channel membership existence checks
-- The query pattern: SELECT EXISTS(SELECT 1 FROM channel_members WHERE channel_id = $1 AND user_id = $2)
-- This index allows the query to be satisfied entirely from the index without touching the table

CREATE INDEX IF NOT EXISTS idx_channel_members_composite ON channel_members(channel_id, user_id);

COMMENT ON INDEX idx_channel_members_composite IS 'Optimizes channel membership existence checks for authorization';
