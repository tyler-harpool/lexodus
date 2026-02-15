CREATE TABLE IF NOT EXISTS deadline_reminders (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    deadline_id     UUID NOT NULL REFERENCES deadlines(id) ON DELETE CASCADE,
    recipient       TEXT NOT NULL,
    reminder_type   TEXT NOT NULL
        CHECK (reminder_type IN ('Email','SMS','In-App','Push','Fax')),
    sent_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged    BOOLEAN NOT NULL DEFAULT FALSE,
    acknowledged_at TIMESTAMPTZ
);
CREATE INDEX idx_deadline_reminders_court ON deadline_reminders(court_id);
CREATE INDEX idx_deadline_reminders_court_deadline ON deadline_reminders(court_id, deadline_id);
CREATE INDEX idx_deadline_reminders_court_acknowledged ON deadline_reminders(court_id, acknowledged);
