-- Down: remove realistic seed data
DELETE FROM clerk_queue WHERE id::text LIKE 'd9e%' OR id::text LIKE 'd12e%';
DELETE FROM criminal_cases WHERE id::text LIKE 'd9c%' OR id::text LIKE 'd12c%';
DELETE FROM attorneys WHERE id::text LIKE 'd9a%' OR id::text LIKE 'd12a%';
DELETE FROM judges WHERE id::text LIKE 'd9b%' OR id::text LIKE 'd12b%';
