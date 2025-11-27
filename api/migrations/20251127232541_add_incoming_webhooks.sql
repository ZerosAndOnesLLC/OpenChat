-- Incoming webhooks allow external services to post messages to channels
-- Similar to Mattermost/Slack incoming webhooks

CREATE TABLE incoming_webhooks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    -- Unique token for the webhook URL (POST /api/hooks/:token)
    token VARCHAR(64) NOT NULL UNIQUE,
    -- Display name shown when webhook posts messages
    display_name VARCHAR(100) NOT NULL,
    -- Description of what this webhook is for
    description TEXT,
    -- Optional icon URL for the webhook avatar
    icon_url TEXT,
    -- Optional username override (defaults to display_name)
    username VARCHAR(100),
    -- Whether this webhook is enabled
    enabled BOOLEAN NOT NULL DEFAULT true,
    -- Who created this webhook
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fast token lookup (public endpoint)
CREATE INDEX idx_incoming_webhooks_token ON incoming_webhooks(token) WHERE enabled = true;

-- Index for listing webhooks by org
CREATE INDEX idx_incoming_webhooks_org ON incoming_webhooks(org_id);

-- Index for listing webhooks by channel
CREATE INDEX idx_incoming_webhooks_channel ON incoming_webhooks(channel_id);

-- Trigger to update updated_at
CREATE TRIGGER update_incoming_webhooks_updated_at
    BEFORE UPDATE ON incoming_webhooks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
