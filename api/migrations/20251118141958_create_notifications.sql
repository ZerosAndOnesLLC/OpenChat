-- Create notifications table for tracking mentions, DM notifications, and thread replies
CREATE TABLE notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    notification_type VARCHAR(30) NOT NULL CHECK (notification_type IN ('mention', 'dm', 'thread_reply', 'channel_invite')),
    message_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    channel_id UUID REFERENCES channels(id) ON DELETE CASCADE,
    dm_id UUID REFERENCES direct_messages(id) ON DELETE CASCADE,
    read BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for querying user's unread notifications
CREATE INDEX idx_notifications_user_read_created ON notifications(user_id, read, created_at DESC);

-- Index for querying by message (for cleanup)
CREATE INDEX idx_notifications_message_id ON notifications(message_id);

-- Enable RLS
ALTER TABLE notifications ENABLE ROW LEVEL SECURITY;

-- Users can only see their own notifications
CREATE POLICY notifications_select ON notifications
    FOR SELECT
    USING (user_id = current_setting('app.user_id', true)::UUID);

-- Users can only update their own notifications (for marking as read)
CREATE POLICY notifications_update ON notifications
    FOR UPDATE
    USING (user_id = current_setting('app.user_id', true)::UUID);

-- System can create notifications for users (this will be done via service role)
CREATE POLICY notifications_insert ON notifications
    FOR INSERT
    WITH CHECK (true);
