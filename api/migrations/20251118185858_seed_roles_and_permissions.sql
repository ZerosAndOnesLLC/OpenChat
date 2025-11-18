-- Seed permissions
-- Note: Permissions are global and define what actions can be performed
-- Roles come from SSO provider and are mapped to these permissions

-- Channel permissions
INSERT INTO permissions (permission_name, resource_type, action, description) VALUES
    ('channel.read', 'channel', 'read', 'Read messages and view channel details'),
    ('channel.write', 'channel', 'write', 'Send messages in channel'),
    ('channel.delete', 'channel', 'delete', 'Delete channel'),
    ('channel.invite_users', 'channel', 'invite', 'Invite users to channel'),
    ('channel.manage_members', 'channel', 'manage_members', 'Add/remove channel members'),
    ('channel.delete_messages', 'channel', 'delete_messages', 'Delete any message in channel'),
    ('channel.pin_messages', 'channel', 'pin', 'Pin messages in channel'),
    ('channel.edit_details', 'channel', 'edit', 'Edit channel name, description, etc.');

-- Organization permissions
INSERT INTO permissions (permission_name, resource_type, action, description) VALUES
    ('org.create_channels', 'org', 'create_channels', 'Create new channels'),
    ('org.manage_users', 'org', 'manage_users', 'Manage organization users'),
    ('org.manage_roles', 'org', 'manage_roles', 'Manage roles and permissions'),
    ('org.view_audit_logs', 'org', 'view_audit_logs', 'View organization audit logs'),
    ('org.manage_settings', 'org', 'manage_settings', 'Manage organization settings'),
    ('org.manage_integrations', 'org', 'manage_integrations', 'Manage webhooks and integrations');

-- DM permissions
INSERT INTO permissions (permission_name, resource_type, action, description) VALUES
    ('dm.read', 'dm', 'read', 'Read direct messages'),
    ('dm.write', 'dm', 'write', 'Send direct messages'),
    ('dm.delete_own_messages', 'dm', 'delete_own', 'Delete own messages in DM');

-- Create system roles that map to SSO roles
-- These are global roles (org_id = NULL) that match SSO provider role names

-- SSO Role: openchat-admin
INSERT INTO roles (org_id, role_name, is_system_role, description)
VALUES (NULL, 'openchat-admin', TRUE, 'Administrator role from SSO - full access to all features');

-- SSO Role: openchat
INSERT INTO roles (org_id, role_name, is_system_role, description)
VALUES (NULL, 'openchat', TRUE, 'Standard user role from SSO - basic channel and DM access');

-- Map openchat-admin role to all permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.role_name = 'openchat-admin';

-- Map openchat role to member permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.role_name = 'openchat'
AND p.permission_name IN (
    'channel.read',
    'channel.write',
    'channel.invite_users',
    'org.create_channels',
    'dm.read',
    'dm.write',
    'dm.delete_own_messages'
);

-- The permission checking middleware will:
-- 1. Extract roles from SSO claims (JWT or session)
-- 2. Match SSO role name to system role in database
-- 3. Check if the role has the required permission via role_permissions
-- 4. Cache role-permission mappings in Redis for performance
