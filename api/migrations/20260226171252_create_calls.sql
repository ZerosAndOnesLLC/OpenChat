CREATE TABLE calls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    channel_id UUID REFERENCES channels(id) ON DELETE SET NULL,
    dm_id UUID REFERENCES direct_messages(id) ON DELETE SET NULL,
    call_type VARCHAR(20) NOT NULL DEFAULT 'audio',
    status VARCHAR(20) NOT NULL DEFAULT 'ringing',
    started_by UUID NOT NULL REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    livekit_room_name VARCHAR(255) NOT NULL,
    is_huddle BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT chk_calls_target CHECK (
        (channel_id IS NOT NULL AND dm_id IS NULL) OR
        (channel_id IS NULL AND dm_id IS NOT NULL)
    )
);

CREATE INDEX idx_calls_channel_active ON calls(channel_id) WHERE status != 'ended';
CREATE INDEX idx_calls_dm_active ON calls(dm_id) WHERE status != 'ended';
CREATE INDEX idx_calls_org_active ON calls(org_id) WHERE status IN ('ringing', 'active');

CREATE TABLE call_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    call_id UUID NOT NULL REFERENCES calls(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at TIMESTAMPTZ,
    muted BOOLEAN NOT NULL DEFAULT false,
    video_off BOOLEAN NOT NULL DEFAULT true
);

CREATE INDEX idx_call_participants_call ON call_participants(call_id);
CREATE INDEX idx_call_participants_active ON call_participants(call_id) WHERE left_at IS NULL;
CREATE UNIQUE INDEX idx_call_participant_unique ON call_participants(call_id, user_id) WHERE left_at IS NULL;
