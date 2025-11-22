-- Add device pairing tables for desktop quick login feature

-- Table for temporary pairing codes used to authenticate desktop apps
CREATE TABLE device_pairing_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code VARCHAR(6) NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id UUID NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    used BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for efficient lookups
CREATE INDEX idx_device_pairing_codes_code ON device_pairing_codes(code);
CREATE INDEX idx_device_pairing_codes_expires ON device_pairing_codes(expires_at);
CREATE INDEX idx_device_pairing_codes_user ON device_pairing_codes(user_id);

-- Table for tracking desktop device sessions
CREATE TABLE device_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    org_id UUID NOT NULL,
    device_type VARCHAR(50) NOT NULL, -- 'desktop', 'mobile', 'web'
    device_name VARCHAR(255),
    device_fingerprint TEXT,
    last_active_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_device_sessions_user ON device_sessions(user_id);
CREATE INDEX idx_device_sessions_org ON device_sessions(org_id);
CREATE INDEX idx_device_sessions_last_active ON device_sessions(last_active_at);
