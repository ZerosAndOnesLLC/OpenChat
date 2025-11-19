-- Retention policies table for data retention management
CREATE TABLE IF NOT EXISTS retention_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    policy_type VARCHAR(50) NOT NULL, -- 'messages' or 'files'
    retention_days INTEGER NOT NULL CHECK (retention_days > 0),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,

    -- Ensure one policy per org per type
    UNIQUE(org_id, policy_type)
);

-- Index for querying policies by org
CREATE INDEX idx_retention_policies_org_id ON retention_policies(org_id);
CREATE INDEX idx_retention_policies_enabled ON retention_policies(enabled) WHERE enabled = TRUE;

-- Legal hold table for freezing deletion of specific channels
CREATE TABLE IF NOT EXISTS legal_holds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    channel_id UUID REFERENCES channels(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    disabled_at TIMESTAMP WITH TIME ZONE,
    disabled_by UUID REFERENCES users(id)
);

-- Ensure one active legal hold per channel using partial unique index
CREATE UNIQUE INDEX idx_legal_holds_channel_active ON legal_holds(channel_id) WHERE enabled = TRUE;

-- Indexes for legal holds
CREATE INDEX idx_legal_holds_org_id ON legal_holds(org_id);
CREATE INDEX idx_legal_holds_channel_id ON legal_holds(channel_id);
CREATE INDEX idx_legal_holds_enabled ON legal_holds(enabled) WHERE enabled = TRUE;
