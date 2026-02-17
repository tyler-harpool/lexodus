-- Store the user's last-selected court district for cross-platform persistence.
ALTER TABLE users ADD COLUMN IF NOT EXISTS preferred_court_id TEXT;
