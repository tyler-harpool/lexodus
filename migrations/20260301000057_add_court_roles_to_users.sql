-- Per-court role memberships: store which courts a user belongs to and their role in each.
-- Format: {"arwd": "clerk", "sdny": "judge", "cdca": "attorney"}
ALTER TABLE users ADD COLUMN IF NOT EXISTS court_roles JSONB NOT NULL DEFAULT '{}';
