-- Create message_edits table to track edit history
CREATE TABLE message_edits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    old_content TEXT NOT NULL,
    edited_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    edited_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for efficient lookups
CREATE INDEX idx_message_edits_message_id_edited_at ON message_edits(message_id, edited_at DESC);

-- Enable RLS on message_edits table
ALTER TABLE message_edits ENABLE ROW LEVEL SECURITY;

-- Policy: Users can view edit history for messages they have access to
-- For now, allow all authenticated users to view edit history
-- TODO: Restrict based on channel/DM membership
CREATE POLICY message_edits_select_policy ON message_edits
    FOR SELECT
    USING (true);
