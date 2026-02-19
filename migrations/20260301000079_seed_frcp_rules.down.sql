-- Remove FRCP rule seed data by deterministic ID prefixes
DELETE FROM rules WHERE id::text LIKE 'd9ru%' OR id::text LIKE 'd12ru%';
