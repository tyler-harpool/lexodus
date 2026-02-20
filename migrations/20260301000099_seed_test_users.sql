-- Seed test users for role-based UI testing
-- All passwords are argon2 hash of 'testuser123'

-- Fix admin: assign district9 admin court role
UPDATE users SET court_roles = '{"district9": "admin"}',
                 preferred_court_id = COALESCE(preferred_court_id, 'district9')
WHERE id = 1;

-- Clerk user
INSERT INTO users (username, display_name, email, password_hash, role, tier, court_roles, email_verified, preferred_court_id)
VALUES ('clerk_test', 'Maria Santos (Clerk)', 'maria.santos@district9.uscourts.gov',
        '$argon2id$v=19$m=65536,t=3,p=4$AmUYK1khMRZEfuBSbJ6Iow$d9j+cFHHRrUIz1/t+G4fjftMHiLsTKLt42CW9cxp0U0',
        'user', 'free', '{"district9": "clerk"}', true, 'district9')
ON CONFLICT (email) DO UPDATE SET
    court_roles = EXCLUDED.court_roles,
    display_name = EXCLUDED.display_name,
    preferred_court_id = EXCLUDED.preferred_court_id;

-- Judge user (linked to Hon. Ronnie Abrams)
INSERT INTO users (username, display_name, email, password_hash, role, tier, court_roles, email_verified, preferred_court_id, linked_judge_id)
VALUES ('judge_test', 'Hon. Ronnie Abrams (Judge)', 'ronnie.abrams@district9.uscourts.gov',
        '$argon2id$v=19$m=65536,t=3,p=4$AmUYK1khMRZEfuBSbJ6Iow$d9j+cFHHRrUIz1/t+G4fjftMHiLsTKLt42CW9cxp0U0',
        'user', 'free', '{"district9": "judge"}', true, 'district9',
        'd9b00001-0000-0000-0000-000000000001')
ON CONFLICT (email) DO UPDATE SET
    court_roles = EXCLUDED.court_roles,
    display_name = EXCLUDED.display_name,
    preferred_court_id = EXCLUDED.preferred_court_id,
    linked_judge_id = EXCLUDED.linked_judge_id;

-- Attorney user (linked to Sarah Mitchell)
INSERT INTO users (username, display_name, email, password_hash, role, tier, court_roles, email_verified, preferred_court_id, linked_attorney_id)
VALUES ('attorney_test', 'Sarah Mitchell (Attorney)', 'sarah.mitchell@usdoj.gov',
        '$argon2id$v=19$m=65536,t=3,p=4$AmUYK1khMRZEfuBSbJ6Iow$d9j+cFHHRrUIz1/t+G4fjftMHiLsTKLt42CW9cxp0U0',
        'user', 'free', '{"district9": "attorney"}', true, 'district9',
        'd9a00001-0000-0000-0000-000000000001')
ON CONFLICT (email) DO UPDATE SET
    court_roles = EXCLUDED.court_roles,
    display_name = EXCLUDED.display_name,
    preferred_court_id = EXCLUDED.preferred_court_id,
    linked_attorney_id = EXCLUDED.linked_attorney_id;

-- Public user (no court roles)
INSERT INTO users (username, display_name, email, password_hash, role, tier, court_roles, email_verified)
VALUES ('public_test', 'John Q. Public', 'john.public@example.com',
        '$argon2id$v=19$m=65536,t=3,p=4$AmUYK1khMRZEfuBSbJ6Iow$d9j+cFHHRrUIz1/t+G4fjftMHiLsTKLt42CW9cxp0U0',
        'user', 'free', '{}', true)
ON CONFLICT (email) DO UPDATE SET
    court_roles = EXCLUDED.court_roles,
    display_name = EXCLUDED.display_name;
