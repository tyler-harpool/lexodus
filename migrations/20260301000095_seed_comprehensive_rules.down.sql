-- Remove comprehensive rules seeded by migration 000095.
-- Uses citation to target only the new tagged-enum format rules (priority >= 10).
-- Also resets triggers backfill on legacy 000089 rules.

DELETE FROM rules
WHERE citation IN (
    'FRCP 4(m)', 'FRCP 12(a)(1)', 'FRCP 12(a)(4)', 'FRCP 26(a)(1)', 'FRCP 26(f)',
    'FRCP 33', 'FRCP 34', 'FRCP 36', 'FRCP 56', 'FRCP 59',
    'FRCrP 5(a)', 'FRCrP 10', 'FRCrP 12(b)', 'FRCrP 29', 'FRCrP 32', 'FRCrP 33', 'FRCrP 35',
    'FRCP 5(b)', 'FRCP 5.2', 'FRCP 11',
    'FRCP 37', 'FRCP 30(a)',
    'FRAP 4(a)', 'FRAP 4(b)',
    '28 U.S.C. 1914', '28 U.S.C. 1917',
    'L.R. 7.1', 'L.R. 16.1',
    '18 U.S.C. 3161(b)', '18 U.S.C. 3161(c)'
)
AND priority >= 10;

-- Reset triggers on legacy 000089 rules that were backfilled
UPDATE rules
SET triggers = '[]'::jsonb
WHERE source = 'Federal Rules of Civil Procedure'
  AND category = 'Discovery'
  AND name IN ('FRCP 33 — Interrogatories', 'FRCP 34 — Document Production')
  AND priority < 10;
