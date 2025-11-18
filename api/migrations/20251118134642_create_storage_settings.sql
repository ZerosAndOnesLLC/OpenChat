-- Storage settings per organization
CREATE TABLE storage_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    storage_type VARCHAR(20) NOT NULL DEFAULT 'local',
    s3_bucket VARCHAR(255),
    s3_region VARCHAR(50),
    s3_access_key_id TEXT,
    s3_secret_key_encrypted TEXT,
    s3_endpoint TEXT, -- Optional: for S3-compatible storage
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(org_id)
);

-- Add constraint to validate storage_type
ALTER TABLE storage_settings
ADD CONSTRAINT storage_settings_type_check
CHECK (storage_type IN ('local', 's3'));

-- Add constraint: if storage_type is 's3', then s3_bucket and s3_region must be set
ALTER TABLE storage_settings
ADD CONSTRAINT storage_settings_s3_check
CHECK (
    (storage_type = 'local') OR
    (storage_type = 's3' AND s3_bucket IS NOT NULL AND s3_region IS NOT NULL)
);

-- Create index for faster org lookups
CREATE INDEX idx_storage_settings_org_id ON storage_settings(org_id);
