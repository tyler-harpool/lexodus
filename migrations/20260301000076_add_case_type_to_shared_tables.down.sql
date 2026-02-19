-- Reverse: drop case_type columns and restore FK constraints to criminal_cases

-- 1. docket_entries
DROP INDEX IF EXISTS idx_docket_entries_case_type;
ALTER TABLE docket_entries DROP COLUMN IF EXISTS case_type;
ALTER TABLE docket_entries ADD CONSTRAINT docket_entries_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 2. calendar_events
DROP INDEX IF EXISTS idx_calendar_events_case_type;
ALTER TABLE calendar_events DROP COLUMN IF EXISTS case_type;
ALTER TABLE calendar_events ADD CONSTRAINT calendar_events_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 3. deadlines
DROP INDEX IF EXISTS idx_deadlines_case_type;
ALTER TABLE deadlines DROP COLUMN IF EXISTS case_type;
ALTER TABLE deadlines ADD CONSTRAINT deadlines_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE SET NULL;

-- 4. parties
DROP INDEX IF EXISTS idx_parties_case_type;
ALTER TABLE parties DROP COLUMN IF EXISTS case_type;
ALTER TABLE parties ADD CONSTRAINT parties_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 5. judicial_orders
DROP INDEX IF EXISTS idx_judicial_orders_case_type;
ALTER TABLE judicial_orders DROP COLUMN IF EXISTS case_type;
ALTER TABLE judicial_orders ADD CONSTRAINT judicial_orders_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 6. judicial_opinions
DROP INDEX IF EXISTS idx_judicial_opinions_case_type;
ALTER TABLE judicial_opinions DROP COLUMN IF EXISTS case_type;
ALTER TABLE judicial_opinions ADD CONSTRAINT judicial_opinions_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 7. documents
DROP INDEX IF EXISTS idx_documents_case_type;
ALTER TABLE documents DROP COLUMN IF EXISTS case_type;
ALTER TABLE documents ADD CONSTRAINT documents_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 8. filings
DROP INDEX IF EXISTS idx_filings_case_type;
ALTER TABLE filings DROP COLUMN IF EXISTS case_type;
ALTER TABLE filings ADD CONSTRAINT filings_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 9. representations
DROP INDEX IF EXISTS idx_representations_case_type;
ALTER TABLE representations DROP COLUMN IF EXISTS case_type;
ALTER TABLE representations ADD CONSTRAINT representations_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 10. motions
DROP INDEX IF EXISTS idx_motions_case_type;
ALTER TABLE motions DROP COLUMN IF EXISTS case_type;
ALTER TABLE motions ADD CONSTRAINT motions_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 11. case_notes
DROP INDEX IF EXISTS idx_case_notes_case_type;
ALTER TABLE case_notes DROP COLUMN IF EXISTS case_type;
ALTER TABLE case_notes ADD CONSTRAINT case_notes_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 12. clerk_queue
DROP INDEX IF EXISTS idx_clerk_queue_case_type;
ALTER TABLE clerk_queue DROP COLUMN IF EXISTS case_type;
ALTER TABLE clerk_queue ADD CONSTRAINT clerk_queue_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE SET NULL;

-- 13. case_assignments
DROP INDEX IF EXISTS idx_case_assignments_case_type;
ALTER TABLE case_assignments DROP COLUMN IF EXISTS case_type;
ALTER TABLE case_assignments ADD CONSTRAINT case_assignments_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 14. conflict_checks
DROP INDEX IF EXISTS idx_conflict_checks_case_type;
ALTER TABLE conflict_checks DROP COLUMN IF EXISTS case_type;
ALTER TABLE conflict_checks ADD CONSTRAINT conflict_checks_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE SET NULL;

-- 15. attorney_pro_hac_vice
DROP INDEX IF EXISTS idx_phv_case_type;
ALTER TABLE attorney_pro_hac_vice DROP COLUMN IF EXISTS case_type;
ALTER TABLE attorney_pro_hac_vice ADD CONSTRAINT attorney_pro_hac_vice_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE CASCADE;

-- 16. attorney_cja_appointments
DROP INDEX IF EXISTS idx_cja_appt_case_type;
ALTER TABLE attorney_cja_appointments DROP COLUMN IF EXISTS case_type;
ALTER TABLE attorney_cja_appointments ADD CONSTRAINT attorney_cja_appointments_case_id_fkey
    FOREIGN KEY (case_id) REFERENCES criminal_cases(id) ON DELETE SET NULL;
