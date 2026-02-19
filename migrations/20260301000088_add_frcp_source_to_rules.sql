-- Add 'Federal Rules of Civil Procedure' to the rules source CHECK constraint
ALTER TABLE rules DROP CONSTRAINT IF EXISTS rules_source_check;
ALTER TABLE rules ADD CONSTRAINT rules_source_check
    CHECK (source IN (
        'Federal Rules of Criminal Procedure',
        'Federal Rules of Civil Procedure',
        'Federal Rules of Evidence',
        'Federal Rules of Appellate Procedure',
        'Local Rules',
        'Standing Orders',
        'Statutory',
        'Administrative',
        'Custom'
    ));
