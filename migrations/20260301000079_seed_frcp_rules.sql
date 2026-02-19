-- Seed Federal Rules of Civil Procedure for district9 and district12
-- These are the most commonly applied FRCP rules in civil litigation

DO $$ BEGIN

-- Guard: skip if data already seeded
IF EXISTS (SELECT 1 FROM rules WHERE citation = 'Fed. R. Civ. P. 4' AND court_id = 'district9') THEN
    RAISE NOTICE 'FRCP rules already seeded, skipping';
    RETURN;
END IF;

-- district9 FRCP rules
INSERT INTO rules (id, court_id, name, description, source, category, priority, status, citation)
VALUES
-- Service and Filing
('d9ru0001-0000-0000-0000-000000000001', 'district9', 'FRCP 4 — Summons',
 'Requires service of summons and complaint within 90 days of filing. Failure to serve results in dismissal without prejudice unless good cause shown.',
 'Federal Rules of Civil Procedure', 'Filing', 1, 'Active', 'Fed. R. Civ. P. 4'),

('d9ru0002-0000-0000-0000-000000000002', 'district9', 'FRCP 5 — Service and Filing',
 'After the original complaint, every pleading, written motion, and similar paper must be served on every party. Electronic filing satisfies the service requirement for registered users.',
 'Federal Rules of Civil Procedure', 'Filing', 2, 'Active', 'Fed. R. Civ. P. 5'),

-- Pleading Rules
('d9ru0003-0000-0000-0000-000000000003', 'district9', 'FRCP 8 — General Rules of Pleading',
 'A complaint must contain: (1) a short plain statement of grounds for jurisdiction, (2) a short plain statement of the claim, and (3) a demand for relief sought.',
 'Federal Rules of Civil Procedure', 'Procedural', 2, 'Active', 'Fed. R. Civ. P. 8'),

('d9ru0004-0000-0000-0000-000000000004', 'district9', 'FRCP 12 — Defenses and Objections',
 'Provides motions to dismiss (12(b)(6)), motion for judgment on the pleadings (12(c)), and motion for more definite statement (12(e)). Response to complaint due within 21 days of service.',
 'Federal Rules of Civil Procedure', 'Procedural', 1, 'Active', 'Fed. R. Civ. P. 12'),

-- Answer and Response
('d9ru0005-0000-0000-0000-000000000005', 'district9', 'FRCP 15 — Amended and Supplemental Pleadings',
 'A party may amend its pleading once as a matter of course within 21 days of serving it, or within 21 days of service of a responsive pleading or motion under Rule 12(b), (e), or (f).',
 'Federal Rules of Civil Procedure', 'Procedural', 3, 'Active', 'Fed. R. Civ. P. 15'),

-- Scheduling and Case Management
('d9ru0006-0000-0000-0000-000000000006', 'district9', 'FRCP 16 — Pretrial Conferences and Scheduling',
 'Court must issue scheduling order after receiving parties'' Rule 26(f) report. Sets deadlines for joinder, amendment, motions, and discovery completion.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 16'),

-- Discovery Rules
('d9ru0007-0000-0000-0000-000000000007', 'district9', 'FRCP 26 — Duty to Disclose; General Provisions',
 'Parties must make initial disclosures within 14 days of the Rule 26(f) conference. Includes identification of witnesses, documents, damages computation, and insurance agreements.',
 'Federal Rules of Civil Procedure', 'Discovery', 1, 'Active', 'Fed. R. Civ. P. 26'),

('d9ru0008-0000-0000-0000-000000000008', 'district9', 'FRCP 30 — Depositions by Oral Examination',
 'A party may depose any person without leave of court unless the deponent is in prison. Maximum 10 depositions per side, each limited to 7 hours in 1 day, unless court orders otherwise.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 30'),

('d9ru0009-0000-0000-0000-000000000009', 'district9', 'FRCP 33 — Interrogatories to Parties',
 'A party may serve no more than 25 written interrogatories (including subparts) without court leave. Responding party has 30 days to serve answers or objections.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 33'),

('d9ru000a-0000-0000-0000-000000000001', 'district9', 'FRCP 34 — Producing Documents and ESI',
 'A party may serve requests for production of documents, electronically stored information, or tangible things. Response due within 30 days of service.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 34'),

('d9ru000b-0000-0000-0000-000000000002', 'district9', 'FRCP 37 — Failure to Make Disclosures or Cooperate in Discovery',
 'Court may order sanctions including fees, adverse inference, or default judgment for failure to comply with discovery obligations.',
 'Federal Rules of Civil Procedure', 'Discovery', 1, 'Active', 'Fed. R. Civ. P. 37'),

-- Summary Judgment
('d9ru000c-0000-0000-0000-000000000003', 'district9', 'FRCP 56 — Summary Judgment',
 'A party may move for summary judgment on all or part of a claim. Must be filed at least 30 days before trial unless court orders otherwise. No genuine dispute of material fact required.',
 'Federal Rules of Civil Procedure', 'Procedural', 1, 'Active', 'Fed. R. Civ. P. 56'),

-- Trial Rules
('d9ru000d-0000-0000-0000-000000000004', 'district9', 'FRCP 38 — Right to a Jury Trial',
 'Jury trial demand must be served within 14 days after service of the last pleading addressing the issue. Failure to demand waives the right.',
 'Federal Rules of Civil Procedure', 'Procedural', 2, 'Active', 'Fed. R. Civ. P. 38'),

-- Class Actions
('d9ru000e-0000-0000-0000-000000000005', 'district9', 'FRCP 23 — Class Actions',
 'Class certification requires: (1) numerosity, (2) commonality, (3) typicality, (4) adequacy of representation. Court must determine certification at an early practicable time.',
 'Federal Rules of Civil Procedure', 'Procedural', 1, 'Active', 'Fed. R. Civ. P. 23'),

-- Judgment
('d9ru000f-0000-0000-0000-000000000006', 'district9', 'FRCP 58 — Entering Judgment',
 'Every judgment must be set out in a separate document unless the court otherwise orders. Judgment is effective when entered under Rule 79(a).',
 'Federal Rules of Civil Procedure', 'Procedural', 2, 'Active', 'Fed. R. Civ. P. 58')
ON CONFLICT (id) DO NOTHING;

-- district12 FRCP rules (same rules, different court)
INSERT INTO rules (id, court_id, name, description, source, category, priority, status, citation)
VALUES
('d12ru001-0000-0000-0000-000000000001', 'district12', 'FRCP 4 — Summons',
 'Requires service of summons and complaint within 90 days of filing. Failure to serve results in dismissal without prejudice unless good cause shown.',
 'Federal Rules of Civil Procedure', 'Filing', 1, 'Active', 'Fed. R. Civ. P. 4'),

('d12ru002-0000-0000-0000-000000000002', 'district12', 'FRCP 5 — Service and Filing',
 'After the original complaint, every pleading, written motion, and similar paper must be served on every party. Electronic filing satisfies the service requirement for registered users.',
 'Federal Rules of Civil Procedure', 'Filing', 2, 'Active', 'Fed. R. Civ. P. 5'),

('d12ru003-0000-0000-0000-000000000003', 'district12', 'FRCP 8 — General Rules of Pleading',
 'A complaint must contain: (1) a short plain statement of grounds for jurisdiction, (2) a short plain statement of the claim, and (3) a demand for relief sought.',
 'Federal Rules of Civil Procedure', 'Procedural', 2, 'Active', 'Fed. R. Civ. P. 8'),

('d12ru004-0000-0000-0000-000000000004', 'district12', 'FRCP 12 — Defenses and Objections',
 'Provides motions to dismiss (12(b)(6)), motion for judgment on the pleadings (12(c)), and motion for more definite statement (12(e)). Response to complaint due within 21 days of service.',
 'Federal Rules of Civil Procedure', 'Procedural', 1, 'Active', 'Fed. R. Civ. P. 12'),

('d12ru005-0000-0000-0000-000000000005', 'district12', 'FRCP 15 — Amended and Supplemental Pleadings',
 'A party may amend its pleading once as a matter of course within 21 days of serving it, or within 21 days of service of a responsive pleading or motion under Rule 12(b), (e), or (f).',
 'Federal Rules of Civil Procedure', 'Procedural', 3, 'Active', 'Fed. R. Civ. P. 15'),

('d12ru006-0000-0000-0000-000000000006', 'district12', 'FRCP 16 — Pretrial Conferences and Scheduling',
 'Court must issue scheduling order after receiving parties'' Rule 26(f) report. Sets deadlines for joinder, amendment, motions, and discovery completion.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 16'),

('d12ru007-0000-0000-0000-000000000007', 'district12', 'FRCP 26 — Duty to Disclose; General Provisions',
 'Parties must make initial disclosures within 14 days of the Rule 26(f) conference. Includes identification of witnesses, documents, damages computation, and insurance agreements.',
 'Federal Rules of Civil Procedure', 'Discovery', 1, 'Active', 'Fed. R. Civ. P. 26'),

('d12ru008-0000-0000-0000-000000000008', 'district12', 'FRCP 30 — Depositions by Oral Examination',
 'A party may depose any person without leave of court unless the deponent is in prison. Maximum 10 depositions per side, each limited to 7 hours in 1 day, unless court orders otherwise.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 30'),

('d12ru009-0000-0000-0000-000000000009', 'district12', 'FRCP 33 — Interrogatories to Parties',
 'A party may serve no more than 25 written interrogatories (including subparts) without court leave. Responding party has 30 days to serve answers or objections.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 33'),

('d12ru00a-0000-0000-0000-000000000001', 'district12', 'FRCP 34 — Producing Documents and ESI',
 'A party may serve requests for production of documents, electronically stored information, or tangible things. Response due within 30 days of service.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 34'),

('d12ru00b-0000-0000-0000-000000000002', 'district12', 'FRCP 37 — Failure to Make Disclosures or Cooperate in Discovery',
 'Court may order sanctions including fees, adverse inference, or default judgment for failure to comply with discovery obligations.',
 'Federal Rules of Civil Procedure', 'Discovery', 1, 'Active', 'Fed. R. Civ. P. 37'),

('d12ru00c-0000-0000-0000-000000000003', 'district12', 'FRCP 56 — Summary Judgment',
 'A party may move for summary judgment on all or part of a claim. Must be filed at least 30 days before trial unless court orders otherwise. No genuine dispute of material fact required.',
 'Federal Rules of Civil Procedure', 'Procedural', 1, 'Active', 'Fed. R. Civ. P. 56'),

('d12ru00d-0000-0000-0000-000000000004', 'district12', 'FRCP 38 — Right to a Jury Trial',
 'Jury trial demand must be served within 14 days after service of the last pleading addressing the issue. Failure to demand waives the right.',
 'Federal Rules of Civil Procedure', 'Procedural', 2, 'Active', 'Fed. R. Civ. P. 38'),

('d12ru00e-0000-0000-0000-000000000005', 'district12', 'FRCP 23 — Class Actions',
 'Class certification requires: (1) numerosity, (2) commonality, (3) typicality, (4) adequacy of representation. Court must determine certification at an early practicable time.',
 'Federal Rules of Civil Procedure', 'Procedural', 1, 'Active', 'Fed. R. Civ. P. 23'),

('d12ru00f-0000-0000-0000-000000000006', 'district12', 'FRCP 58 — Entering Judgment',
 'Every judgment must be set out in a separate document unless the court otherwise orders. Judgment is effective when entered under Rule 79(a).',
 'Federal Rules of Civil Procedure', 'Procedural', 2, 'Active', 'Fed. R. Civ. P. 58')
ON CONFLICT (id) DO NOTHING;

END $$;
