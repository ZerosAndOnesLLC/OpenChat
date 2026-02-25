CREATE TABLE job_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    job_type VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for worker polling: pending/retryable jobs due now
CREATE INDEX idx_job_queue_pending ON job_queue(scheduled_at)
    WHERE status IN ('pending', 'retry');

-- Index for job type filtering
CREATE INDEX idx_job_queue_type_status ON job_queue(job_type, status);

-- Index for org-scoped queries
CREATE INDEX idx_job_queue_org ON job_queue(org_id);

-- Cleanup index: find completed/failed jobs older than N days
CREATE INDEX idx_job_queue_completed ON job_queue(completed_at)
    WHERE status IN ('completed', 'failed');
