-- Add archived column to channels table
ALTER TABLE channels ADD COLUMN archived BOOLEAN NOT NULL DEFAULT FALSE;

-- Add index for filtering out archived channels
CREATE INDEX idx_channels_archived ON channels(archived);
