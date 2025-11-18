-- Create roles table for role-based access control
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID REFERENCES organizations(id) ON DELETE CASCADE,  -- NULL for global system roles
    role_name TEXT NOT NULL,
    is_system_role BOOLEAN NOT NULL DEFAULT FALSE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, role_name)
);

-- Create index for faster lookups
CREATE INDEX idx_roles_org_id ON roles(org_id);
CREATE INDEX idx_roles_is_system_role ON roles(org_id, is_system_role);

-- Enable RLS
ALTER TABLE roles ENABLE ROW LEVEL SECURITY;

-- Policy: Users can view roles in their org (including global system roles)
CREATE POLICY roles_select_policy ON roles
    FOR SELECT
    USING (
        org_id = current_setting('app.current_org_id', true)::uuid
        OR org_id IS NULL  -- Global system roles visible to all
    );
