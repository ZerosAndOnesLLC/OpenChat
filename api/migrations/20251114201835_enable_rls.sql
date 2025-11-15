-- Enable Row Level Security on all org-scoped tables
ALTER TABLE organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE channels ENABLE ROW LEVEL SECURITY;
ALTER TABLE channel_members ENABLE ROW LEVEL SECURITY;
ALTER TABLE direct_messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE dm_participants ENABLE ROW LEVEL SECURITY;
ALTER TABLE messages ENABLE ROW LEVEL SECURITY;
ALTER TABLE reactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE attachments ENABLE ROW LEVEL SECURITY;

-- Organizations: Users can only see their own org
CREATE POLICY org_isolation_policy ON organizations
    FOR ALL
    USING (id = current_setting('app.current_org_id', true)::uuid);

-- Users: Only see users in same org
CREATE POLICY org_isolation_policy ON users
    FOR ALL
    USING (org_id = current_setting('app.current_org_id', true)::uuid);

-- Channels: Only see channels in same org
CREATE POLICY org_isolation_policy ON channels
    FOR ALL
    USING (org_id = current_setting('app.current_org_id', true)::uuid);

-- Channel Members: Only see memberships for channels in same org
CREATE POLICY org_isolation_policy ON channel_members
    FOR ALL
    USING (
        channel_id IN (
            SELECT id FROM channels
            WHERE org_id = current_setting('app.current_org_id', true)::uuid
        )
    );

-- Direct Messages: Only see DMs in same org
CREATE POLICY org_isolation_policy ON direct_messages
    FOR ALL
    USING (org_id = current_setting('app.current_org_id', true)::uuid);

-- DM Participants: Only see participants for DMs in same org
CREATE POLICY org_isolation_policy ON dm_participants
    FOR ALL
    USING (
        dm_id IN (
            SELECT id FROM direct_messages
            WHERE org_id = current_setting('app.current_org_id', true)::uuid
        )
    );

-- Messages: Only see messages in channels/DMs in same org
CREATE POLICY org_isolation_policy ON messages
    FOR ALL
    USING (
        (channel_id IN (
            SELECT id FROM channels
            WHERE org_id = current_setting('app.current_org_id', true)::uuid
        ))
        OR
        (dm_id IN (
            SELECT id FROM direct_messages
            WHERE org_id = current_setting('app.current_org_id', true)::uuid
        ))
    );

-- Reactions: Only see reactions on messages in same org
CREATE POLICY org_isolation_policy ON reactions
    FOR ALL
    USING (
        message_id IN (
            SELECT id FROM messages WHERE
                (channel_id IN (
                    SELECT id FROM channels
                    WHERE org_id = current_setting('app.current_org_id', true)::uuid
                ))
                OR
                (dm_id IN (
                    SELECT id FROM direct_messages
                    WHERE org_id = current_setting('app.current_org_id', true)::uuid
                ))
        )
    );

-- Attachments: Only see attachments on messages in same org
CREATE POLICY org_isolation_policy ON attachments
    FOR ALL
    USING (
        message_id IN (
            SELECT id FROM messages WHERE
                (channel_id IN (
                    SELECT id FROM channels
                    WHERE org_id = current_setting('app.current_org_id', true)::uuid
                ))
                OR
                (dm_id IN (
                    SELECT id FROM direct_messages
                    WHERE org_id = current_setting('app.current_org_id', true)::uuid
                ))
        )
    );
