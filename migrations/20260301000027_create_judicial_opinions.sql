CREATE TABLE IF NOT EXISTS judicial_opinions (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id            TEXT NOT NULL REFERENCES courts(id),
    case_id             UUID NOT NULL REFERENCES criminal_cases(id) ON DELETE CASCADE,
    case_name           TEXT NOT NULL,
    docket_number       TEXT NOT NULL,
    author_judge_id     UUID NOT NULL REFERENCES judges(id),
    author_judge_name   TEXT NOT NULL,
    opinion_type        TEXT NOT NULL
        CHECK (opinion_type IN ('Majority','Concurrence','Dissent','Per Curiam','Memorandum','En Banc','Summary','Other')),
    disposition         TEXT
        CHECK (disposition IS NULL OR disposition IN ('Affirmed','Reversed','Remanded','Vacated','Dismissed','Modified','Certified')),
    title               TEXT NOT NULL,
    syllabus            TEXT,
    content             TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'Draft'
        CHECK (status IN ('Draft','Under Review','Circulated','Filed','Published','Withdrawn','Superseded')),
    is_published        BOOLEAN NOT NULL DEFAULT FALSE,
    is_precedential     BOOLEAN NOT NULL DEFAULT FALSE,
    citation_volume     TEXT,
    citation_reporter   TEXT,
    citation_page       TEXT,
    filed_at            TIMESTAMPTZ,
    published_at        TIMESTAMPTZ,
    keywords            TEXT[] NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_judicial_opinions_court ON judicial_opinions(court_id);
CREATE INDEX idx_judicial_opinions_court_case ON judicial_opinions(court_id, case_id);
CREATE INDEX idx_judicial_opinions_court_author ON judicial_opinions(court_id, author_judge_id);
CREATE INDEX idx_judicial_opinions_court_type ON judicial_opinions(court_id, opinion_type);
CREATE INDEX idx_judicial_opinions_court_status ON judicial_opinions(court_id, status);
CREATE INDEX idx_judicial_opinions_court_published ON judicial_opinions(court_id, is_published);
CREATE INDEX idx_judicial_opinions_court_precedential ON judicial_opinions(court_id, is_precedential);
CREATE INDEX idx_judicial_opinions_court_filed ON judicial_opinions(court_id, filed_at);
