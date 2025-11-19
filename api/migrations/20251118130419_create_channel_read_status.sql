-- Channel read status tracking
CREATE TABLE channel_read_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    last_read_message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    last_read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unread_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, channel_id)
);

-- Indexes for performance
CREATE INDEX idx_channel_read_status_user ON channel_read_status(user_id);
CREATE INDEX idx_channel_read_status_channel ON channel_read_status(channel_id);
CREATE INDEX idx_channel_read_status_user_channel ON channel_read_status(user_id, channel_id);
