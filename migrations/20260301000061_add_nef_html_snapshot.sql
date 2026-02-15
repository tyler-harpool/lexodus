-- Add html_snapshot column to nefs table for persisted NEF rendering.
ALTER TABLE nefs ADD COLUMN IF NOT EXISTS html_snapshot TEXT;
