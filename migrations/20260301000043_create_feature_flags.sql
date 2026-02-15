CREATE TABLE IF NOT EXISTS feature_flags (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    feature_path TEXT NOT NULL UNIQUE,
    enabled      BOOLEAN NOT NULL DEFAULT FALSE,
    description  TEXT,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_feature_flags_path ON feature_flags(feature_path);
CREATE INDEX idx_feature_flags_enabled ON feature_flags(enabled);
