-- Message Read Receipts Table
-- Tracks who has read which messages (Slack-style "seen by" feature)
CREATE TABLE IF NOT EXISTS message_read_receipts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_message_user_receipt UNIQUE (message_id, user_id)
);

-- Create indexes for efficient queries
-- Index for getting all receipts for a message (ordered by read time)
CREATE INDEX idx_message_read_receipts_message_id ON message_read_receipts(message_id, read_at DESC);

-- Index for getting all receipts by a user
CREATE INDEX idx_message_read_receipts_user_id ON message_read_receipts(user_id, read_at DESC);

-- Composite index for checking if a specific user read a specific message
CREATE INDEX idx_message_read_receipts_message_user ON message_read_receipts(message_id, user_id);

-- Enable RLS
ALTER TABLE message_read_receipts ENABLE ROW LEVEL SECURITY;

-- RLS Policy: Users can view receipts for messages in channels they're members of or DMs they're part of
CREATE POLICY message_read_receipts_select_policy ON message_read_receipts
    FOR SELECT
    USING (
        message_id IN (
            -- Messages in channels the user is a member of
            SELECT m.id FROM messages m
            INNER JOIN channel_members cm ON m.channel_id = cm.channel_id
            WHERE cm.user_id = current_setting('app.user_id')::UUID

            UNION

            -- Messages in DMs the user is part of
            SELECT m.id FROM messages m
            INNER JOIN dm_participants dp ON m.dm_id = dp.dm_id
            WHERE dp.user_id = current_setting('app.user_id')::UUID
        )
    );

-- RLS Policy: Users can only insert their own read receipts
CREATE POLICY message_read_receipts_insert_policy ON message_read_receipts
    FOR INSERT
    WITH CHECK (
        user_id = current_setting('app.user_id')::UUID
        AND message_id IN (
            -- Messages in channels the user is a member of
            SELECT m.id FROM messages m
            INNER JOIN channel_members cm ON m.channel_id = cm.channel_id
            WHERE cm.user_id = current_setting('app.user_id')::UUID

            UNION

            -- Messages in DMs the user is part of
            SELECT m.id FROM messages m
            INNER JOIN dm_participants dp ON m.dm_id = dp.dm_id
            WHERE dp.user_id = current_setting('app.user_id')::UUID
        )
    );

-- RLS Policy: Users can delete their own read receipts
CREATE POLICY message_read_receipts_delete_policy ON message_read_receipts
    FOR DELETE
    USING (user_id = current_setting('app.user_id')::UUID);
