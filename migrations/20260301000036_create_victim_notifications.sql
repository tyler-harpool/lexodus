CREATE TABLE IF NOT EXISTS victim_notifications (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id          TEXT NOT NULL REFERENCES courts(id),
    victim_id         UUID NOT NULL REFERENCES victims(id) ON DELETE CASCADE,
    notification_type TEXT NOT NULL
        CHECK (notification_type IN ('Case Filed','Hearing Scheduled','Plea Agreement','Sentencing','Release','Restitution','Appeal','Status Change','Other')),
    sent_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    method            TEXT NOT NULL
        CHECK (method IN ('Email','Mail','Phone','In-App','Fax')),
    content_summary   TEXT,
    acknowledged      BOOLEAN NOT NULL DEFAULT FALSE,
    acknowledged_at   TIMESTAMPTZ
);
CREATE INDEX idx_victim_notifications_court ON victim_notifications(court_id);
CREATE INDEX idx_victim_notifications_court_victim ON victim_notifications(court_id, victim_id);
CREATE INDEX idx_victim_notifications_court_type ON victim_notifications(court_id, notification_type);
CREATE INDEX idx_victim_notifications_court_acknowledged ON victim_notifications(court_id, acknowledged);
