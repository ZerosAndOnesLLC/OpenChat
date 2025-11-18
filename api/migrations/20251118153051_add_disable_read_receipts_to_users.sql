-- Add privacy setting for disabling read receipts
-- Users can choose to not send read receipts (similar to WhatsApp/iMessage settings)
ALTER TABLE users
ADD COLUMN disable_read_receipts BOOLEAN NOT NULL DEFAULT FALSE;

-- Index for filtering users by read receipt preference (if needed for analytics)
CREATE INDEX idx_users_disable_read_receipts ON users(disable_read_receipts) WHERE disable_read_receipts = TRUE;
