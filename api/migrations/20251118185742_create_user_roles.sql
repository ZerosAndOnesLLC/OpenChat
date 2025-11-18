-- Create user_roles table to track user role assignments
-- Note: Primary source of roles is SSO provider. This table is for caching and overrides.
CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    channel_id UUID REFERENCES channels(id) ON DELETE CASCADE,  -- NULL for org-level roles
    source TEXT NOT NULL DEFAULT 'sso',  -- 'sso', 'manual', 'system'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, role_id, channel_id)
);

-- Create indexes for faster lookups
CREATE INDEX idx_user_roles_user_id ON user_roles(user_id);
CREATE INDEX idx_user_roles_role_id ON user_roles(role_id);
CREATE INDEX idx_user_roles_channel_id ON user_roles(channel_id) WHERE channel_id IS NOT NULL;
CREATE INDEX idx_user_roles_source ON user_roles(source);

-- Enable RLS
ALTER TABLE user_roles ENABLE ROW LEVEL SECURITY;

-- Policy: Users can view roles of users in their org
CREATE POLICY user_roles_select_policy ON user_roles
    FOR SELECT
    USING (
        user_id IN (
            SELECT id FROM users WHERE org_id = current_setting('app.current_org_id', true)::uuid
        )
    );
