-- Encrypted channels and encryption sessions for E2E encryption
CREATE TABLE encrypted_channels (
    channel_id UUID PRIMARY KEY REFERENCES channels(id) ON DELETE CASCADE,
    encryption_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    algorithm VARCHAR(50) NOT NULL DEFAULT 'megolm.v1',
    rotation_period_msgs INTEGER NOT NULL DEFAULT 100,
    rotation_period_ms BIGINT NOT NULL DEFAULT 604800000,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE encryption_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(255) NOT NULL,
    channel_id UUID REFERENCES channels(id) ON DELETE CASCADE,
    dm_id UUID REFERENCES direct_messages(id) ON DELETE CASCADE,
    sender_device_id VARCHAR(64) NOT NULL,
    sender_user_id UUID NOT NULL REFERENCES users(id),
    algorithm VARCHAR(50) NOT NULL,
    session_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMPTZ,
    CHECK ((channel_id IS NOT NULL AND dm_id IS NULL) OR (channel_id IS NULL AND dm_id IS NOT NULL) OR (channel_id IS NULL AND dm_id IS NULL))
);
CREATE INDEX idx_encryption_sessions_channel ON encryption_sessions(channel_id);
CREATE INDEX idx_encryption_sessions_dm ON encryption_sessions(dm_id);
