-- Create pinned_messages table
CREATE TABLE IF NOT EXISTS pinned_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    pinned_by UUID NOT NULL REFERENCES users(id),
    pinned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_pinned_message UNIQUE (channel_id, message_id)
);

-- Create indexes
CREATE INDEX idx_pinned_messages_channel_id ON pinned_messages(channel_id, pinned_at DESC);
CREATE INDEX idx_pinned_messages_message_id ON pinned_messages(message_id);
CREATE INDEX idx_pinned_messages_pinned_by ON pinned_messages(pinned_by);

-- Enable RLS
ALTER TABLE pinned_messages ENABLE ROW LEVEL SECURITY;

-- RLS Policy: Users can view pins for channels they're members of
CREATE POLICY pinned_messages_select_policy ON pinned_messages
    FOR SELECT
    USING (
        channel_id IN (
            SELECT channel_id FROM channel_members WHERE user_id = current_setting('app.user_id')::UUID
        )
    );

-- RLS Policy: Users with appropriate permissions can insert pins
CREATE POLICY pinned_messages_insert_policy ON pinned_messages
    FOR INSERT
    WITH CHECK (
        pinned_by = current_setting('app.user_id')::UUID
        AND channel_id IN (
            SELECT channel_id FROM channel_members
            WHERE user_id = current_setting('app.user_id')::UUID
        )
    );

-- RLS Policy: Users with appropriate permissions can delete pins
CREATE POLICY pinned_messages_delete_policy ON pinned_messages
    FOR DELETE
    USING (
        channel_id IN (
            SELECT channel_id FROM channel_members
            WHERE user_id = current_setting('app.user_id')::UUID
        )
    );
