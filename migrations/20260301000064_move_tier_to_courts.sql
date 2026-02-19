-- Move subscription tier from users to courts.
-- The court organization subscribes; all members inherit its tier.
-- users.tier stays as a vestigial column for backward compat.

ALTER TABLE courts ADD COLUMN IF NOT EXISTS tier TEXT NOT NULL DEFAULT 'free';

ALTER TABLE subscriptions ADD COLUMN IF NOT EXISTS court_id TEXT REFERENCES courts(id);

CREATE INDEX IF NOT EXISTS idx_subscriptions_court_id ON subscriptions(court_id);
