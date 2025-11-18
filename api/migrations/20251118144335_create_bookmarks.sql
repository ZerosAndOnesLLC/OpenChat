-- Create bookmarks table
CREATE TABLE IF NOT EXISTS bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    bookmarked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_bookmark UNIQUE (user_id, message_id)
);

-- Create indexes
CREATE INDEX idx_bookmarks_user_id ON bookmarks(user_id, bookmarked_at DESC);
CREATE INDEX idx_bookmarks_message_id ON bookmarks(message_id);

-- Enable RLS
ALTER TABLE bookmarks ENABLE ROW LEVEL SECURITY;

-- RLS Policy: Users can only view their own bookmarks
CREATE POLICY bookmarks_select_policy ON bookmarks
    FOR SELECT
    USING (user_id = current_setting('app.user_id')::UUID);

-- RLS Policy: Users can only insert their own bookmarks
CREATE POLICY bookmarks_insert_policy ON bookmarks
    FOR INSERT
    WITH CHECK (user_id = current_setting('app.user_id')::UUID);

-- RLS Policy: Users can only delete their own bookmarks
CREATE POLICY bookmarks_delete_policy ON bookmarks
    FOR DELETE
    USING (user_id = current_setting('app.user_id')::UUID);
