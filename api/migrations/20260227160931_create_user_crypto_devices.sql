-- User crypto devices for E2E encryption
CREATE TABLE user_crypto_devices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id VARCHAR(64) NOT NULL,
    display_name VARCHAR(255),
    identity_key VARCHAR(255) NOT NULL,
    signing_key VARCHAR(255) NOT NULL,
    one_time_keys JSONB NOT NULL DEFAULT '{}',
    fallback_key JSONB,
    verified BOOLEAN NOT NULL DEFAULT FALSE,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, device_id)
);
CREATE INDEX idx_user_crypto_devices_user_id ON user_crypto_devices(user_id);
