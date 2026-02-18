-- Down: remove realistic seed data
-- Order matters: children first due to foreign key constraints
DELETE FROM clerk_queue WHERE id::text LIKE 'd9e%' OR id::text LIKE 'd12e%';
DELETE FROM charges WHERE id::text LIKE 'd9cf%' OR id::text LIKE 'd12cf%';
DELETE FROM defendants WHERE id::text LIKE 'd9de%' OR id::text LIKE 'd12de%';
DELETE FROM criminal_cases WHERE id::text LIKE 'd9c%' OR id::text LIKE 'd12c%';
DELETE FROM attorneys WHERE id::text LIKE 'd9a%' OR id::text LIKE 'd12a%';
DELETE FROM judges WHERE id::text LIKE 'd9b%' OR id::text LIKE 'd12b%';
