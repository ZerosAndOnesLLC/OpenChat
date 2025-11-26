-- Sprint 5: Performance Optimization Indexes
-- Add indexes to optimize frequently used queries

-- Messages table: Optimize channel message queries
CREATE INDEX IF NOT EXISTS idx_messages_channel_created
ON messages(channel_id, created_at DESC)
WHERE channel_id IS NOT NULL;

-- Messages table: Optimize DM message queries
CREATE INDEX IF NOT EXISTS idx_messages_dm_created
ON messages(dm_id, created_at DESC)
WHERE dm_id IS NOT NULL;

-- Messages table: Optimize parent message lookup for threads
CREATE INDEX IF NOT EXISTS idx_messages_parent_id
ON messages(parent_message_id)
WHERE parent_message_id IS NOT NULL;

-- Channel read status: Optimize unread count queries
CREATE INDEX IF NOT EXISTS idx_channel_read_status_user_channel
ON channel_read_status(user_id, channel_id, last_read_at DESC);

-- Channel read status: Optimize last read message lookup
CREATE INDEX IF NOT EXISTS idx_channel_read_status_last_read_message
ON channel_read_status(last_read_message_id)
WHERE last_read_message_id IS NOT NULL;

-- Pinned messages: Optimize channel pins lookup
CREATE INDEX IF NOT EXISTS idx_pinned_messages_channel
ON pinned_messages(channel_id, pinned_at DESC);

-- Channel members: Optimize member lookup by channel
CREATE INDEX IF NOT EXISTS idx_channel_members_channel_joined
ON channel_members(channel_id, joined_at DESC);

-- Direct message participants: Optimize participant lookup
CREATE INDEX IF NOT EXISTS idx_dm_participants_dm_user
ON dm_participants(dm_id, user_id);

-- Direct message participants: Optimize user's DMs lookup
CREATE INDEX IF NOT EXISTS idx_dm_participants_user_dm
ON dm_participants(user_id, dm_id);

-- Messages: Optimize user's message lookup
CREATE INDEX IF NOT EXISTS idx_messages_user_created
ON messages(user_id, created_at DESC);

-- Reactions: Optimize message reactions lookup
CREATE INDEX IF NOT EXISTS idx_reactions_message
ON reactions(message_id, created_at DESC);

-- Notifications: Optimize user notifications query
CREATE INDEX IF NOT EXISTS idx_notifications_user_created
ON notifications(user_id, created_at DESC);

-- Notifications: Optimize unread notifications query
CREATE INDEX IF NOT EXISTS idx_notifications_user_unread
ON notifications(user_id, read, created_at DESC);

-- User status: Optimize active users lookup
CREATE INDEX IF NOT EXISTS idx_user_status_status_updated
ON user_status(status, updated_at DESC)
WHERE status != 'offline';

-- ANALYZE tables to update statistics for query planner
ANALYZE messages;
ANALYZE channel_read_status;
ANALYZE channel_members;
ANALYZE pinned_messages;
ANALYZE dm_participants;
ANALYZE reactions;
ANALYZE notifications;
ANALYZE user_status;
