-- Add back_at column to user_status table
-- This stores when the user expects to return (for away/dnd status)
ALTER TABLE user_status ADD COLUMN back_at TIMESTAMPTZ;
