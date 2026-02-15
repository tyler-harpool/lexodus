CREATE TABLE IF NOT EXISTS opinion_votes (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id   TEXT NOT NULL REFERENCES courts(id),
    opinion_id UUID NOT NULL REFERENCES judicial_opinions(id) ON DELETE CASCADE,
    judge_id   UUID NOT NULL REFERENCES judges(id),
    vote_type  TEXT NOT NULL
        CHECK (vote_type IN ('Join','Concur','Concur in Part','Dissent','Dissent in Part','Recused','Not Participating')),
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notes      TEXT,
    UNIQUE(court_id, opinion_id, judge_id)
);
CREATE INDEX idx_opinion_votes_court ON opinion_votes(court_id);
CREATE INDEX idx_opinion_votes_court_opinion ON opinion_votes(court_id, opinion_id);
CREATE INDEX idx_opinion_votes_court_judge ON opinion_votes(court_id, judge_id);
CREATE INDEX idx_opinion_votes_court_type ON opinion_votes(court_id, vote_type);
