ALTER TABLE rules DROP COLUMN IF EXISTS triggers;
DROP INDEX IF EXISTS idx_rules_triggers;
