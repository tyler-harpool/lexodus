CREATE TABLE IF NOT EXISTS opinion_headnotes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id        TEXT NOT NULL REFERENCES courts(id),
    opinion_id      UUID NOT NULL REFERENCES judicial_opinions(id) ON DELETE CASCADE,
    headnote_number INT NOT NULL,
    topic           TEXT NOT NULL,
    text            TEXT NOT NULL,
    key_number      TEXT,
    UNIQUE(court_id, opinion_id, headnote_number)
);
CREATE INDEX idx_opinion_headnotes_court ON opinion_headnotes(court_id);
CREATE INDEX idx_opinion_headnotes_court_opinion ON opinion_headnotes(court_id, opinion_id);
CREATE INDEX idx_opinion_headnotes_court_topic ON opinion_headnotes(court_id, topic);
