-- User Status Table
-- Tracks user presence and custom status messages
CREATE TABLE user_status (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'offline' CHECK (status IN ('online', 'away', 'dnd', 'offline')),
    custom_message TEXT,
    emoji VARCHAR(10),
    clear_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for querying status by update time
CREATE INDEX idx_user_status_updated_at ON user_status(updated_at DESC);

-- Index for querying users by status type
CREATE INDEX idx_user_status_status ON user_status(status);

-- Enable RLS
ALTER TABLE user_status ENABLE ROW LEVEL SECURITY;

-- Policy: Users can view status of users in their organization
CREATE POLICY user_status_select_policy ON user_status
    FOR SELECT
    USING (
        user_id IN (
            SELECT u.id FROM users u
            WHERE u.org_id = (SELECT org_id FROM users WHERE id = current_setting('app.user_id')::UUID)
        )
    );

-- Policy: Users can update their own status
CREATE POLICY user_status_update_policy ON user_status
    FOR UPDATE
    USING (user_id = current_setting('app.user_id')::UUID);

-- Policy: Users can insert their own status
CREATE POLICY user_status_insert_policy ON user_status
    FOR INSERT
    WITH CHECK (user_id = current_setting('app.user_id')::UUID);

-- Policy: Users can delete their own status
CREATE POLICY user_status_delete_policy ON user_status
    FOR DELETE
    USING (user_id = current_setting('app.user_id')::UUID);

-- Function to auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_user_status_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
CREATE TRIGGER user_status_updated_at_trigger
    BEFORE UPDATE ON user_status
    FOR EACH ROW
    EXECUTE FUNCTION update_user_status_updated_at();
