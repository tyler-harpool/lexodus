CREATE TABLE IF NOT EXISTS civil_cases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id TEXT NOT NULL,
    case_number TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    nature_of_suit TEXT NOT NULL,
    cause_of_action TEXT NOT NULL DEFAULT '',
    jurisdiction_basis TEXT NOT NULL CHECK (jurisdiction_basis IN ('federal_question', 'diversity', 'us_government_plaintiff', 'us_government_defendant')),
    jury_demand TEXT NOT NULL DEFAULT 'none' CHECK (jury_demand IN ('none', 'plaintiff', 'defendant', 'both')),
    class_action BOOLEAN NOT NULL DEFAULT false,
    amount_in_controversy NUMERIC(15,2),
    status TEXT NOT NULL DEFAULT 'filed' CHECK (status IN ('filed', 'pending', 'discovery', 'pretrial', 'trial_ready', 'in_trial', 'settled', 'judgment_entered', 'on_appeal', 'closed', 'dismissed', 'transferred')),
    priority TEXT NOT NULL DEFAULT 'medium' CHECK (priority IN ('low', 'medium', 'high', 'critical')),
    assigned_judge_id UUID REFERENCES judges(id),
    district_code TEXT NOT NULL DEFAULT '',
    location TEXT NOT NULL DEFAULT '',
    is_sealed BOOLEAN NOT NULL DEFAULT false,
    sealed_date TIMESTAMPTZ,
    sealed_by TEXT,
    seal_reason TEXT,
    related_case_id UUID,
    consent_to_magistrate BOOLEAN NOT NULL DEFAULT false,
    pro_se BOOLEAN NOT NULL DEFAULT false,
    opened_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    closed_at TIMESTAMPTZ,
    UNIQUE(court_id, case_number)
);

CREATE INDEX idx_civil_cases_court_status ON civil_cases(court_id, status);
CREATE INDEX idx_civil_cases_court_judge ON civil_cases(court_id, assigned_judge_id);
CREATE INDEX idx_civil_cases_nos ON civil_cases(nature_of_suit);
