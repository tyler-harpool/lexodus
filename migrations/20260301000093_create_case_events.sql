CREATE TABLE IF NOT EXISTS case_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    case_id         UUID NOT NULL,
    case_type       TEXT NOT NULL CHECK (case_type IN ('criminal', 'civil')),
    trigger_event   TEXT NOT NULL,
    actor_id        UUID,
    payload         JSONB NOT NULL DEFAULT '{}',
    compliance_report JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_case_events_court_case ON case_events(court_id, case_id);
CREATE INDEX idx_case_events_trigger ON case_events(trigger_event);
CREATE INDEX idx_case_events_created ON case_events(created_at DESC);
