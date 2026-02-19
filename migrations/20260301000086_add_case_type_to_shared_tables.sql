-- Add case_type column to 12 shared tables so they can reference either
-- criminal_cases or civil_cases. Drop the hard FK to criminal_cases(id)
-- and add a case_type discriminator. The application layer handles routing.

-- 1. docket_entries
ALTER TABLE docket_entries DROP CONSTRAINT IF EXISTS docket_entries_case_id_fkey;
ALTER TABLE docket_entries ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 2. parties
ALTER TABLE parties DROP CONSTRAINT IF EXISTS parties_case_id_fkey;
ALTER TABLE parties ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 3. calendar_events
ALTER TABLE calendar_events DROP CONSTRAINT IF EXISTS calendar_events_case_id_fkey;
ALTER TABLE calendar_events ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 4. deadlines
ALTER TABLE deadlines DROP CONSTRAINT IF EXISTS deadlines_case_id_fkey;
ALTER TABLE deadlines ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 5. documents
ALTER TABLE documents DROP CONSTRAINT IF EXISTS documents_case_id_fkey;
ALTER TABLE documents ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 6. filings
ALTER TABLE filings DROP CONSTRAINT IF EXISTS filings_case_id_fkey;
ALTER TABLE filings ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 7. motions
ALTER TABLE motions DROP CONSTRAINT IF EXISTS motions_case_id_fkey;
ALTER TABLE motions ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 8. evidence
ALTER TABLE evidence DROP CONSTRAINT IF EXISTS evidence_case_id_fkey;
ALTER TABLE evidence ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 9. judicial_orders
ALTER TABLE judicial_orders DROP CONSTRAINT IF EXISTS judicial_orders_case_id_fkey;
ALTER TABLE judicial_orders ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 10. clerk_queue
ALTER TABLE clerk_queue DROP CONSTRAINT IF EXISTS clerk_queue_case_id_fkey;
ALTER TABLE clerk_queue ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 11. victims
ALTER TABLE victims DROP CONSTRAINT IF EXISTS victims_case_id_fkey;
ALTER TABLE victims ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));

-- 12. representations
ALTER TABLE representations DROP CONSTRAINT IF EXISTS representations_case_id_fkey;
ALTER TABLE representations ADD COLUMN IF NOT EXISTS case_type TEXT NOT NULL DEFAULT 'criminal'
    CHECK (case_type IN ('criminal', 'civil'));
