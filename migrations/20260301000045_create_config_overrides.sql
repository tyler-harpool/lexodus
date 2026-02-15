CREATE TABLE IF NOT EXISTS config_overrides (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id     TEXT NOT NULL REFERENCES courts(id),
    scope        TEXT NOT NULL
        CHECK (scope IN ('district','judge')),
    scope_id     TEXT NOT NULL,
    config_key   TEXT NOT NULL,
    config_value JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, scope, scope_id, config_key)
);
CREATE INDEX idx_config_overrides_court ON config_overrides(court_id);
CREATE INDEX idx_config_overrides_court_scope ON config_overrides(court_id, scope, scope_id);
CREATE INDEX idx_config_overrides_court_key ON config_overrides(court_id, config_key);
