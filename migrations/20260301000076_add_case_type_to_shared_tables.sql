-- Add case_type column to shared tables so they can reference either
-- criminal_cases or civil_cases. Drop the hard FK to criminal_cases(id)
-- and replace with a case_type discriminator + plain case_id UUID.

-- 1. docket_entries
ALTER TABLE docket_entries DROP CONSTRAINT IF EXISTS docket_entries_case_id_fkey;
ALTER TABLE docket_entries ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_docket_entries_case_type ON docket_entries(court_id, case_type);

-- 2. calendar_events
ALTER TABLE calendar_events DROP CONSTRAINT IF EXISTS calendar_events_case_id_fkey;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_calendar_events_case_type ON calendar_events(court_id, case_type);

-- 3. deadlines
ALTER TABLE deadlines DROP CONSTRAINT IF EXISTS deadlines_case_id_fkey;
ALTER TABLE deadlines ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_deadlines_case_type ON deadlines(court_id, case_type);

-- 4. parties
ALTER TABLE parties DROP CONSTRAINT IF EXISTS parties_case_id_fkey;
ALTER TABLE parties ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_parties_case_type ON parties(court_id, case_type);

-- 5. judicial_orders
ALTER TABLE judicial_orders DROP CONSTRAINT IF EXISTS judicial_orders_case_id_fkey;
ALTER TABLE judicial_orders ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_judicial_orders_case_type ON judicial_orders(court_id, case_type);

-- 6. judicial_opinions
ALTER TABLE judicial_opinions DROP CONSTRAINT IF EXISTS judicial_opinions_case_id_fkey;
ALTER TABLE judicial_opinions ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_judicial_opinions_case_type ON judicial_opinions(court_id, case_type);

-- 7. documents
ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_case_id_fkey;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_documents_case_type ON documents(court_id, case_type);

-- 8. filings
ALTER TABLE filings DROP CONSTRAINT IF EXISTS filings_case_id_fkey;
ALTER TABLE filings ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_filings_case_type ON filings(court_id, case_type);

-- 9. representations
ALTER TABLE representations DROP CONSTRAINT IF EXISTS representations_case_id_fkey;
ALTER TABLE representations ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_representations_case_type ON representations(court_id, case_type);

-- 10. motions
ALTER TABLE motions DROP CONSTRAINT IF EXISTS motions_case_id_fkey;
ALTER TABLE motions ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_motions_case_type ON motions(court_id, case_type);

-- 11. case_notes
ALTER TABLE case_notes DROP CONSTRAINT IF EXISTS case_notes_case_id_fkey;
ALTER TABLE case_notes ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_case_notes_case_type ON case_notes(court_id, case_type);

-- 12. clerk_queue
ALTER TABLE clerk_queue DROP CONSTRAINT IF EXISTS clerk_queue_case_id_fkey;
ALTER TABLE clerk_queue ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_clerk_queue_case_type ON clerk_queue(court_id, case_type);

-- 13. case_assignments
ALTER TABLE case_assignments DROP CONSTRAINT IF EXISTS case_assignments_case_id_fkey;
ALTER TABLE case_assignments ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_case_assignments_case_type ON case_assignments(court_id, case_type);

-- 14. conflict_checks
ALTER TABLE conflict_checks DROP CONSTRAINT IF EXISTS conflict_checks_case_id_fkey;
ALTER TABLE conflict_checks ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_conflict_checks_case_type ON conflict_checks(court_id, case_type);

-- 15. attorney_pro_hac_vice
ALTER TABLE attorney_pro_hac_vice DROP CONSTRAINT IF EXISTS attorney_pro_hac_vice_case_id_fkey;
ALTER TABLE attorney_pro_hac_vice ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_phv_case_type ON attorney_pro_hac_vice(court_id, case_type);

-- 16. attorney_cja_appointments
ALTER TABLE attorney_cja_appointments DROP CONSTRAINT IF EXISTS attorney_cja_appointments_case_id_fkey;
ALTER TABLE attorney_cja_appointments ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
CREATE INDEX IF NOT EXISTS idx_cja_appt_case_type ON attorney_cja_appointments(court_id, case_type);
