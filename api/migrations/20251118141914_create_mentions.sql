-- Create mentions table for tracking @username and @channel mentions in messages
CREATE TABLE mentions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    mentioned_user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    mention_type VARCHAR(20) NOT NULL CHECK (mention_type IN ('user', 'channel', 'here', 'everyone')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for querying mentions by user (for showing "you were mentioned")
CREATE INDEX idx_mentions_mentioned_user_created ON mentions(mentioned_user_id, created_at DESC) WHERE mentioned_user_id IS NOT NULL;

-- Index for querying mentions by message
CREATE INDEX idx_mentions_message_id ON mentions(message_id);

-- Enable RLS
ALTER TABLE mentions ENABLE ROW LEVEL SECURITY;

-- Users can see mentions where they are the mentioned user or they have access to the message's channel/DM
CREATE POLICY mentions_select ON mentions
    FOR SELECT
    USING (
        mentioned_user_id = current_setting('app.user_id', true)::UUID
        OR EXISTS (
            SELECT 1 FROM messages m
            LEFT JOIN channel_members cm ON m.channel_id = cm.channel_id
            LEFT JOIN dm_participants dp ON m.dm_id = dp.dm_id
            WHERE m.id = mentions.message_id
            AND (
                (m.channel_id IS NOT NULL AND cm.user_id = current_setting('app.user_id', true)::UUID)
                OR (m.dm_id IS NOT NULL AND dp.user_id = current_setting('app.user_id', true)::UUID)
            )
        )
    );

-- Users can create mentions when sending messages
CREATE POLICY mentions_insert ON mentions
    FOR INSERT
    WITH CHECK (
        EXISTS (
            SELECT 1 FROM messages m
            WHERE m.id = message_id
            AND m.user_id = current_setting('app.user_id', true)::UUID
        )
    );
