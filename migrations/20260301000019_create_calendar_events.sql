CREATE TABLE IF NOT EXISTS calendar_events (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id         TEXT NOT NULL REFERENCES courts(id),
    case_id          UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    judge_id         UUID NOT NULL REFERENCES judges(id),
    event_type       TEXT NOT NULL
        CHECK (event_type IN ('Hearing','Trial','Sentencing','Arraignment','Status Conference','Motion Hearing','Pretrial Conference','Plea Hearing','Bond Hearing','Grand Jury','Other')),
    scheduled_date   TIMESTAMPTZ NOT NULL,
    duration_minutes INT NOT NULL DEFAULT 60,
    courtroom        TEXT,
    description      TEXT,
    participants     TEXT[] NOT NULL DEFAULT '{}',
    court_reporter   TEXT,
    is_public        BOOLEAN NOT NULL DEFAULT TRUE,
    status           TEXT NOT NULL DEFAULT 'Scheduled'
        CHECK (status IN ('Scheduled','Confirmed','In Progress','Completed','Cancelled','Continued','Postponed')),
    notes            TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_calendar_events_court ON calendar_events(court_id);
CREATE INDEX idx_calendar_events_court_case ON calendar_events(court_id, case_id);
CREATE INDEX idx_calendar_events_court_judge ON calendar_events(court_id, judge_id);
CREATE INDEX idx_calendar_events_court_date ON calendar_events(court_id, scheduled_date);
CREATE INDEX idx_calendar_events_court_type ON calendar_events(court_id, event_type);
CREATE INDEX idx_calendar_events_court_status ON calendar_events(court_id, status);
CREATE INDEX idx_calendar_events_court_courtroom ON calendar_events(court_id, courtroom);
