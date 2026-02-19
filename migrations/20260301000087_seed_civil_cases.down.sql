-- Remove civil case seed data by deterministic ID prefixes
DELETE FROM clerk_queue WHERE id::text LIKE 'd9qv%' OR id::text LIKE 'd12qv%';
DELETE FROM docket_entries WHERE id::text LIKE 'd9dv%' OR id::text LIKE 'd12dv%';
DELETE FROM parties WHERE id::text LIKE 'd9pv%' OR id::text LIKE 'd12pv%';
DELETE FROM civil_cases WHERE id::text LIKE 'd9cv%' OR id::text LIKE 'd12v%';
