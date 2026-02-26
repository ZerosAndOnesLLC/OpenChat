CREATE TABLE user_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    handle VARCHAR(50) NOT NULL,
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX idx_user_groups_org_handle ON user_groups(org_id, handle);
CREATE INDEX idx_user_groups_org ON user_groups(org_id);

CREATE TABLE user_group_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    added_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX idx_user_group_members_group_user ON user_group_members(group_id, user_id);
CREATE INDEX idx_user_group_members_user ON user_group_members(user_id);

-- Expand mentions to support group mentions
ALTER TABLE mentions ADD COLUMN mentioned_group_id UUID REFERENCES user_groups(id) ON DELETE SET NULL;
CREATE INDEX idx_mentions_group ON mentions(mentioned_group_id) WHERE mentioned_group_id IS NOT NULL;

-- Drop existing inline check constraint and recreate with 'group' added
ALTER TABLE mentions DROP CONSTRAINT mentions_mention_type_check;
ALTER TABLE mentions ADD CONSTRAINT mentions_mention_type_check
    CHECK (mention_type IN ('user', 'channel', 'here', 'everyone', 'group'));
