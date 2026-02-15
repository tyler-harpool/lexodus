CREATE TABLE IF NOT EXISTS device_authorizations (
    id           BIGSERIAL    PRIMARY KEY,
    device_code  TEXT         NOT NULL UNIQUE,  -- SHA-256 hash (secret)
    user_code    TEXT         NOT NULL UNIQUE,  -- plain text (displayed to user)
    user_id      BIGINT       REFERENCES users(id) ON DELETE CASCADE,  -- NULL until approved
    status       TEXT         NOT NULL DEFAULT 'pending',  -- pending/approved/expired
    client_info  TEXT,
    expires_at   TIMESTAMPTZ  NOT NULL,
    approved_at  TIMESTAMPTZ,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_device_auth_device_code ON device_authorizations(device_code);
CREATE INDEX idx_device_auth_user_code ON device_authorizations(user_code);
