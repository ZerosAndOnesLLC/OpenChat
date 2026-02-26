CREATE TABLE channel_sections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    collapsed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX idx_channel_sections_user_org_name ON channel_sections(user_id, org_id, name);
CREATE INDEX idx_channel_sections_user_org_position ON channel_sections(user_id, org_id, position);

CREATE TABLE channel_section_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    section_id UUID NOT NULL REFERENCES channel_sections(id) ON DELETE CASCADE,
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX idx_channel_section_items_section_channel ON channel_section_items(section_id, channel_id);
CREATE INDEX idx_channel_section_items_section_position ON channel_section_items(section_id, position);
