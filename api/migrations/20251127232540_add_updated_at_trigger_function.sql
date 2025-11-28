-- Create the update_updated_at_column trigger function
-- This is used by tables that need automatic updated_at timestamp management

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
