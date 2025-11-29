-- Migration jobs table for tracking Mattermost imports
CREATE TABLE IF NOT EXISTS migration_jobs (
    id UUID PRIMARY KEY,
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    progress JSONB NOT NULL DEFAULT '{}',
    error TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for querying jobs by org
CREATE INDEX idx_migration_jobs_org_id ON migration_jobs(org_id);

-- Index for querying active jobs
CREATE INDEX idx_migration_jobs_status ON migration_jobs(status) WHERE status IN ('pending', 'running');

-- Add RLS policy
ALTER TABLE migration_jobs ENABLE ROW LEVEL SECURITY;

CREATE POLICY migration_jobs_org_isolation ON migration_jobs
    USING (org_id = current_setting('app.current_org_id', true)::uuid);

COMMENT ON TABLE migration_jobs IS 'Tracks data migration jobs from external platforms (e.g., Mattermost)';
