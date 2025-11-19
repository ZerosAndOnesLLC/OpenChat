-- Add tsvector column for full-text search
ALTER TABLE messages ADD COLUMN content_tsv tsvector;

-- Create function to update tsvector on insert/update
CREATE OR REPLACE FUNCTION messages_content_tsv_trigger() RETURNS trigger AS $$
BEGIN
    NEW.content_tsv := to_tsvector('english', COALESCE(NEW.content, ''));
    RETURN NEW;
END
$$ LANGUAGE plpgsql;

-- Create trigger to auto-update tsvector on insert/update
CREATE TRIGGER messages_content_tsv_update
    BEFORE INSERT OR UPDATE OF content
    ON messages
    FOR EACH ROW
    EXECUTE FUNCTION messages_content_tsv_trigger();

-- Create GIN index for fast full-text search
CREATE INDEX idx_messages_content_tsv ON messages USING GIN (content_tsv);

-- Backfill existing messages with tsvector data
UPDATE messages SET content_tsv = to_tsvector('english', COALESCE(content, ''))
