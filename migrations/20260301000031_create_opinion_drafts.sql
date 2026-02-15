CREATE TABLE IF NOT EXISTS opinion_drafts (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id   TEXT NOT NULL REFERENCES courts(id),
    opinion_id UUID NOT NULL REFERENCES judicial_opinions(id) ON DELETE CASCADE,
    version    INT NOT NULL,
    content    TEXT NOT NULL,
    status     TEXT NOT NULL DEFAULT 'Draft'
        CHECK (status IN ('Draft','Under Review','Approved','Rejected','Superseded')),
    author_id  TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, opinion_id, version)
);
CREATE INDEX idx_opinion_drafts_court ON opinion_drafts(court_id);
CREATE INDEX idx_opinion_drafts_court_opinion ON opinion_drafts(court_id, opinion_id);
CREATE INDEX idx_opinion_drafts_court_status ON opinion_drafts(court_id, status);
