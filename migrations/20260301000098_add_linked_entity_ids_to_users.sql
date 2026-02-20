-- Add linked judge/attorney entity columns to users table
-- These directly link a user account to their judge or attorney record
-- for role-based dashboard features (My Items, pending rulings, etc.)
ALTER TABLE users ADD COLUMN IF NOT EXISTS linked_judge_id UUID;
ALTER TABLE users ADD COLUMN IF NOT EXISTS linked_attorney_id UUID;
