CREATE TABLE IF NOT EXISTS draft_comments (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id    TEXT NOT NULL REFERENCES courts(id),
    draft_id    UUID NOT NULL REFERENCES opinion_drafts(id) ON DELETE CASCADE,
    author      TEXT NOT NULL,
    content     TEXT NOT NULL,
    resolved    BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_draft_comments_court ON draft_comments(court_id);
CREATE INDEX idx_draft_comments_court_draft ON draft_comments(court_id, draft_id);
CREATE INDEX idx_draft_comments_court_resolved ON draft_comments(court_id, resolved);
