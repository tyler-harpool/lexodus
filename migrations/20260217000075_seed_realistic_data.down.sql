-- Down: remove realistic seed data
-- Order matters: children first due to foreign key constraints
DELETE FROM docket_entries WHERE id::text LIKE 'd9d0%' OR id::text LIKE 'd12d0%';
DELETE FROM representations WHERE id::text LIKE 'd9ae%' OR id::text LIKE 'd12ae%';
DELETE FROM parties WHERE id::text LIKE 'd9ab%' OR id::text LIKE 'd12ab%';
DELETE FROM clerk_queue WHERE id::text LIKE 'd9e%' OR id::text LIKE 'd12e%';
DELETE FROM charges WHERE id::text LIKE 'd9cf%' OR id::text LIKE 'd12cf%';
DELETE FROM defendants WHERE id::text LIKE 'd9de%' OR id::text LIKE 'd12de%';
DELETE FROM criminal_cases WHERE id::text LIKE 'd9c%' OR id::text LIKE 'd12c%';
DELETE FROM attorneys WHERE id::text LIKE 'd9a%' OR id::text LIKE 'd12a%';
DELETE FROM judges WHERE id::text LIKE 'd9b%' OR id::text LIKE 'd12b%';
