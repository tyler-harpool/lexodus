-- Add triggers column (JSONB array of trigger event strings)
ALTER TABLE rules ADD COLUMN IF NOT EXISTS triggers JSONB NOT NULL DEFAULT '[]';

-- Expand source CHECK to include all rule sources
ALTER TABLE rules DROP CONSTRAINT IF EXISTS rules_source_check;
ALTER TABLE rules ADD CONSTRAINT rules_source_check
    CHECK (source IN (
        'Federal Rules of Civil Procedure',
        'Federal Rules of Criminal Procedure',
        'Federal Rules of Evidence',
        'Federal Rules of Appellate Procedure',
        'Local Rules',
        'Standing Orders',
        'Statutory',
        'Administrative',
        'Custom',
        'General Order'
    ));

-- Expand category CHECK
ALTER TABLE rules DROP CONSTRAINT IF EXISTS rules_category_check;
ALTER TABLE rules ADD CONSTRAINT rules_category_check
    CHECK (category IN (
        'Procedural', 'Evidentiary', 'Deadline', 'Filing', 'Discovery',
        'Sentencing', 'Appeal', 'Administrative', 'Other',
        'Fee', 'Assignment', 'Service', 'Sealing', 'Privacy', 'Format'
    ));

CREATE INDEX IF NOT EXISTS idx_rules_triggers ON rules USING GIN (triggers);

-- Backfill triggers from existing conditions JSON
UPDATE rules
SET triggers = jsonb_build_array(conditions->>'trigger')
WHERE conditions ? 'trigger' AND triggers = '[]';
