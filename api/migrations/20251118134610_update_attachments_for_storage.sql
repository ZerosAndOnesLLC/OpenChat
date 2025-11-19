-- Add storage configuration columns to attachments table
ALTER TABLE attachments
ADD COLUMN storage_type VARCHAR(20) NOT NULL DEFAULT 'local',
ADD COLUMN storage_path TEXT NOT NULL DEFAULT '';

-- Add constraint to validate storage_type
ALTER TABLE attachments
ADD CONSTRAINT attachments_storage_type_check
CHECK (storage_type IN ('local', 's3'));

-- Update existing rows to use 'local' storage
UPDATE attachments SET storage_type = 'local' WHERE storage_type IS NULL;

-- Create index on storage_type for faster queries
CREATE INDEX idx_attachments_storage_type ON attachments(storage_type);
