-- Add encryption columns to messages
ALTER TABLE messages ADD COLUMN encrypted_content BYTEA;
ALTER TABLE messages ADD COLUMN encryption_metadata JSONB;

-- Update FTS trigger to skip encrypted messages
CREATE OR REPLACE FUNCTION messages_content_tsv_trigger() RETURNS trigger AS $$
BEGIN
    IF NEW.encrypted_content IS NOT NULL THEN
        NEW.content_tsv := NULL;
    ELSE
        NEW.content_tsv := to_tsvector('english', COALESCE(NEW.content, ''));
    END IF;
    RETURN NEW;
END
$$ LANGUAGE plpgsql;
