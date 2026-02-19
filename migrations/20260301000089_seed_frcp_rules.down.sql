-- Remove seeded FRCP and local civil rules
DELETE FROM rules WHERE source = 'Federal Rules of Civil Procedure';
DELETE FROM rules WHERE source = 'Local Rules' AND id::text LIKE 'd9ru%';
DELETE FROM rules WHERE source = 'Local Rules' AND id::text LIKE 'd12ru%';
