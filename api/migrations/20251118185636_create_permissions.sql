-- Create permissions table for defining available permissions
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    permission_name TEXT NOT NULL UNIQUE,
    resource_type TEXT NOT NULL,  -- channel, org, dm
    action TEXT NOT NULL,  -- read, write, delete, manage_members, etc.
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for faster lookups
CREATE INDEX idx_permissions_resource_type ON permissions(resource_type);
CREATE INDEX idx_permissions_permission_name ON permissions(permission_name);

-- Enable RLS
ALTER TABLE permissions ENABLE ROW LEVEL SECURITY;

-- Policy: All users can view permissions (they are global)
CREATE POLICY permissions_select_policy ON permissions
    FOR SELECT
    USING (true);

-- Note: Permissions are seeded during migration and shouldn't be modified by users
-- Only system migrations can insert/update/delete permissions
