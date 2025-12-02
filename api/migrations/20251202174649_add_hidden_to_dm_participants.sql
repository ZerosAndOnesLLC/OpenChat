-- Add hidden column to dm_participants table
-- When a user hides a DM, it will not appear in their list but the conversation is preserved
ALTER TABLE dm_participants ADD COLUMN hidden BOOLEAN NOT NULL DEFAULT FALSE;

-- Add index for filtering hidden DMs
CREATE INDEX idx_dm_participants_hidden ON dm_participants(hidden);
