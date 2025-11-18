-- DM read status tracking
CREATE TABLE dm_read_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    dm_id UUID NOT NULL REFERENCES direct_messages(id) ON DELETE CASCADE,
    last_read_message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    last_read_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    unread_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, dm_id)
);

-- Indexes for performance
CREATE INDEX idx_dm_read_status_user ON dm_read_status(user_id);
CREATE INDEX idx_dm_read_status_dm ON dm_read_status(dm_id);
CREATE INDEX idx_dm_read_status_user_dm ON dm_read_status(user_id, dm_id);
