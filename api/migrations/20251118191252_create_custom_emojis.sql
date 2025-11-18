-- Custom emojis per organization
CREATE TABLE custom_emojis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    image_url TEXT,
    storage_type VARCHAR(20) NOT NULL DEFAULT 'local',
    storage_path TEXT NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id, name)
);

-- Add constraint to validate storage_type
ALTER TABLE custom_emojis
ADD CONSTRAINT custom_emojis_type_check
CHECK (storage_type IN ('local', 's3'));

-- Add constraint to validate name format (alphanumeric, underscore, hyphen only)
ALTER TABLE custom_emojis
ADD CONSTRAINT custom_emojis_name_check
CHECK (name ~ '^[a-zA-Z0-9_-]+$');

-- Create index for faster org lookups
CREATE INDEX idx_custom_emojis_org_id ON custom_emojis(org_id);

-- Create index for faster name lookups
CREATE INDEX idx_custom_emojis_org_name ON custom_emojis(org_id, name);
