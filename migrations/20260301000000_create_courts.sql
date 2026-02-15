CREATE TABLE IF NOT EXISTS courts (
    id         TEXT PRIMARY KEY,
    name       TEXT NOT NULL,
    court_type TEXT NOT NULL DEFAULT 'district',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

INSERT INTO courts (id, name, court_type) VALUES
    ('district9', 'District 9 (Test)', 'test'),
    ('district12', 'District 12 (Test)', 'test')
ON CONFLICT (id) DO NOTHING;
