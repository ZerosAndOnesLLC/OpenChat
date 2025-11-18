-- Add performance indexes for optimizing common query patterns
-- These composite indexes improve performance for high-traffic queries

-- Optimize queries that fetch messages for a specific channel ordered by time
-- This is the most common query pattern: "get latest messages in channel X"
CREATE INDEX IF NOT EXISTS idx_messages_channel_created ON messages(channel_id, created_at DESC)
    WHERE deleted_at IS NULL AND channel_id IS NOT NULL;

-- Optimize queries that fetch messages for a specific DM ordered by time
-- Common query: "get latest messages in DM X"
CREATE INDEX IF NOT EXISTS idx_messages_dm_created ON messages(dm_id, created_at DESC)
    WHERE deleted_at IS NULL AND dm_id IS NOT NULL;

-- Optimize queries that fetch all messages from a specific user
-- Used for: user message history, user search, moderation tools
CREATE INDEX IF NOT EXISTS idx_messages_user_created ON messages(user_id, created_at DESC)
    WHERE deleted_at IS NULL;

-- Optimize queries that find all channels for a specific user
-- Common query: "get all channels where user X is a member"
CREATE INDEX IF NOT EXISTS idx_channel_members_user_channel ON channel_members(user_id, channel_id);

-- Optimize queries for thread replies (finding all replies to a parent message)
-- Common query: "get all thread replies for message X"
CREATE INDEX IF NOT EXISTS idx_messages_parent_created ON messages(parent_message_id, created_at ASC)
    WHERE deleted_at IS NULL AND parent_message_id IS NOT NULL;

-- Optimize DM participant lookups for finding DMs between users
-- Used when: checking if DM exists between users, finding DMs for a user
CREATE INDEX IF NOT EXISTS idx_dm_participants_user_dm ON dm_participants(user_id, dm_id);

COMMENT ON INDEX idx_messages_channel_created IS 'Optimizes channel message queries with time ordering';
COMMENT ON INDEX idx_messages_dm_created IS 'Optimizes DM message queries with time ordering';
COMMENT ON INDEX idx_messages_user_created IS 'Optimizes user message history queries';
COMMENT ON INDEX idx_channel_members_user_channel IS 'Optimizes user channel membership lookups';
COMMENT ON INDEX idx_messages_parent_created IS 'Optimizes thread reply queries';
COMMENT ON INDEX idx_dm_participants_user_dm IS 'Optimizes DM participant lookups';
