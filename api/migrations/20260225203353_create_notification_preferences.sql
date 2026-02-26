CREATE TABLE notification_preferences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    channel_id UUID REFERENCES channels(id) ON DELETE CASCADE,
    dm_id UUID REFERENCES direct_messages(id) ON DELETE CASCADE,
    preference VARCHAR(20) NOT NULL DEFAULT 'all'
        CHECK (preference IN ('all', 'mentions', 'nothing')),
    mute_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (
        (channel_id IS NOT NULL AND dm_id IS NULL)
        OR (channel_id IS NULL AND dm_id IS NOT NULL)
    )
);
CREATE UNIQUE INDEX idx_notif_prefs_user_channel
    ON notification_preferences(user_id, channel_id) WHERE channel_id IS NOT NULL;
CREATE UNIQUE INDEX idx_notif_prefs_user_dm
    ON notification_preferences(user_id, dm_id) WHERE dm_id IS NOT NULL;
CREATE INDEX idx_notif_prefs_user ON notification_preferences(user_id);
