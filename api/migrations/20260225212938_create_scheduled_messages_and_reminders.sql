-- Scheduled messages
CREATE TABLE scheduled_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    channel_id UUID,
    dm_id UUID,
    content TEXT NOT NULL,
    parent_message_id UUID,
    scheduled_at TIMESTAMPTZ NOT NULL,
    sent BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_scheduled_messages_pending ON scheduled_messages(scheduled_at) WHERE sent = FALSE;
CREATE INDEX idx_scheduled_messages_user ON scheduled_messages(user_id) WHERE sent = FALSE;

-- Reminders
CREATE TABLE reminders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    message_id UUID NOT NULL,
    channel_id UUID,
    dm_id UUID,
    remind_at TIMESTAMPTZ NOT NULL,
    message_preview TEXT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_reminders_pending ON reminders(remind_at) WHERE completed = FALSE;
CREATE INDEX idx_reminders_user ON reminders(user_id) WHERE completed = FALSE;
