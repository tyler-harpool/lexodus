-- Reverse: drop case_type column from all 12 shared tables
ALTER TABLE docket_entries DROP COLUMN IF EXISTS case_type;
ALTER TABLE parties DROP COLUMN IF EXISTS case_type;
ALTER TABLE calendar_events DROP COLUMN IF EXISTS case_type;
ALTER TABLE deadlines DROP COLUMN IF EXISTS case_type;
ALTER TABLE documents DROP COLUMN IF EXISTS case_type;
ALTER TABLE filings DROP COLUMN IF EXISTS case_type;
ALTER TABLE motions DROP COLUMN IF EXISTS case_type;
ALTER TABLE evidence DROP COLUMN IF EXISTS case_type;
ALTER TABLE judicial_orders DROP COLUMN IF EXISTS case_type;
ALTER TABLE clerk_queue DROP COLUMN IF EXISTS case_type;
ALTER TABLE victims DROP COLUMN IF EXISTS case_type;
ALTER TABLE representations DROP COLUMN IF EXISTS case_type;
