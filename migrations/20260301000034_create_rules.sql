CREATE TABLE IF NOT EXISTS rules (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id          TEXT NOT NULL REFERENCES courts(id),
    name              TEXT NOT NULL,
    description       TEXT,
    source            TEXT NOT NULL
        CHECK (source IN ('Federal Rules of Criminal Procedure','Federal Rules of Evidence','Federal Rules of Appellate Procedure','Local Rules','Standing Orders','Statutory','Administrative','Custom')),
    category          TEXT NOT NULL
        CHECK (category IN ('Procedural','Evidentiary','Deadline','Filing','Discovery','Sentencing','Appeal','Administrative','Other')),
    priority          INT NOT NULL DEFAULT 0,
    status            TEXT NOT NULL DEFAULT 'Active'
        CHECK (status IN ('Active','Inactive','Draft','Superseded','Expired')),
    jurisdiction      TEXT,
    citation          TEXT,
    effective_date    TIMESTAMPTZ,
    expiration_date   TIMESTAMPTZ,
    supersedes_rule_id UUID REFERENCES rules(id),
    conditions        JSONB NOT NULL DEFAULT '{}',
    actions           JSONB NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_rules_court ON rules(court_id);
CREATE INDEX idx_rules_court_source ON rules(court_id, source);
CREATE INDEX idx_rules_court_category ON rules(court_id, category);
CREATE INDEX idx_rules_court_status ON rules(court_id, status);
CREATE INDEX idx_rules_court_priority ON rules(court_id, priority);
CREATE INDEX idx_rules_court_effective ON rules(court_id, effective_date);
