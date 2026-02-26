CREATE TABLE slash_commands (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    command_name VARCHAR(50) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    usage_hint VARCHAR(200),
    handler_type VARCHAR(20) NOT NULL DEFAULT 'builtin',
    webhook_url TEXT,
    response_type VARCHAR(20) NOT NULL DEFAULT 'in_channel',
    created_by UUID NOT NULL REFERENCES users(id),
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX idx_slash_commands_org_name ON slash_commands(org_id, command_name);
