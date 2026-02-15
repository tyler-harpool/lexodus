CREATE TABLE IF NOT EXISTS opinion_citations (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id         TEXT NOT NULL REFERENCES courts(id),
    opinion_id       UUID NOT NULL REFERENCES judicial_opinions(id) ON DELETE CASCADE,
    cited_opinion_id UUID,
    citation_text    TEXT NOT NULL,
    citation_type    TEXT NOT NULL
        CHECK (citation_type IN ('Followed','Distinguished','Overruled','Cited','Discussed','Criticized','Questioned','Harmonized','Parallel','Other')),
    context          TEXT,
    pinpoint_cite    TEXT
);
CREATE INDEX idx_opinion_citations_court ON opinion_citations(court_id);
CREATE INDEX idx_opinion_citations_court_opinion ON opinion_citations(court_id, opinion_id);
CREATE INDEX idx_opinion_citations_court_cited ON opinion_citations(court_id, cited_opinion_id);
CREATE INDEX idx_opinion_citations_court_type ON opinion_citations(court_id, citation_type);
