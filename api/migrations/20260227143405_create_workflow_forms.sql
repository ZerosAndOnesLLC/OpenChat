-- Workflow forms (used by create_form action)
CREATE TABLE IF NOT EXISTS workflow_forms (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    step_id UUID NOT NULL REFERENCES workflow_steps(id) ON DELETE CASCADE,
    execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    fields JSONB NOT NULL DEFAULT '[]',
    target_user_id UUID NOT NULL,
    submitted_by UUID,
    submitted_data JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    submitted_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_workflow_forms_target_pending ON workflow_forms(target_user_id) WHERE status = 'pending';
CREATE INDEX IF NOT EXISTS idx_workflow_forms_execution_id ON workflow_forms(execution_id);
