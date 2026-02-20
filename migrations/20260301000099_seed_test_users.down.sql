DELETE FROM users WHERE username IN ('clerk_test', 'judge_test', 'attorney_test', 'public_test');
UPDATE users SET court_roles = '{}' WHERE id = 1;
