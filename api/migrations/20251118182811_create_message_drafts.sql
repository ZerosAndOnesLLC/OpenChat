-- Create message_drafts table for cross-device draft sync
CREATE TABLE IF NOT EXISTS message_drafts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id UUID REFERENCES channels(id) ON DELETE CASCADE,
    dm_id UUID REFERENCES direct_messages(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Ensure draft is for either channel or DM, not both
    CONSTRAINT check_channel_or_dm CHECK (
        (channel_id IS NOT NULL AND dm_id IS NULL) OR
        (channel_id IS NULL AND dm_id IS NOT NULL)
    ),

    -- Unique constraint: one draft per user per channel/DM
    UNIQUE(user_id, channel_id),
    UNIQUE(user_id, dm_id)
);

-- Indexes for efficient lookups
CREATE INDEX idx_message_drafts_user_id ON message_drafts(user_id);
CREATE INDEX idx_message_drafts_channel_id ON message_drafts(channel_id);
CREATE INDEX idx_message_drafts_dm_id ON message_drafts(dm_id);
CREATE INDEX idx_message_drafts_updated_at ON message_drafts(updated_at);
