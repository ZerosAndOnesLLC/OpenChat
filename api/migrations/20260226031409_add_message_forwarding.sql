-- Add forwarding columns to messages table
ALTER TABLE messages
    ADD COLUMN forwarded_from_message_id UUID REFERENCES messages(id),
    ADD COLUMN forwarded_from_channel_id UUID REFERENCES channels(id);

-- Partial index for efficient lookups on forwarded messages
CREATE INDEX idx_messages_forwarded_from_message_id
    ON messages (forwarded_from_message_id)
    WHERE forwarded_from_message_id IS NOT NULL;
