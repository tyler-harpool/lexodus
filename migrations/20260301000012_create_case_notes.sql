CREATE TABLE IF NOT EXISTS case_notes (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id   TEXT NOT NULL REFERENCES courts(id),
    case_id    UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    author     TEXT NOT NULL,
    content    TEXT NOT NULL,
    note_type  TEXT NOT NULL DEFAULT 'General'
        CHECK (note_type IN ('General','Legal Research','Procedural','Confidential','Bench Note','Clerk Note','Other')),
    is_private BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_case_notes_court ON case_notes(court_id);
CREATE INDEX idx_case_notes_court_case ON case_notes(court_id, case_id);
CREATE INDEX idx_case_notes_court_type ON case_notes(court_id, note_type);
CREATE INDEX idx_case_notes_court_private ON case_notes(court_id, is_private);
