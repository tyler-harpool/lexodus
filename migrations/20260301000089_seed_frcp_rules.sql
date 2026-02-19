-- Seed Federal Rules of Civil Procedure and local civil rules
-- for district9 and district12

DO $$ BEGIN

-- Guard: skip if data already seeded
IF EXISTS (SELECT 1 FROM rules WHERE source = 'Federal Rules of Civil Procedure' AND court_id = 'district9') THEN
    RAISE NOTICE 'FRCP rules already seeded, skipping';
    RETURN;
END IF;

-- district9 FRCP rules
INSERT INTO rules (id, court_id, name, description, source, category, priority, status, citation, conditions, actions)
VALUES
-- Service
('d9000001-0000-0000-0000-000000000001', 'district9',
 'FRCP 4(m) — Service of Process',
 'Plaintiff must serve defendant within 90 days of filing complaint. Court must dismiss without prejudice or order service within specified time if good cause shown.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 4(m)',
 '{"trigger": "case_filed", "case_type": "civil"}',
 '{"create_deadline": {"days": 90, "title": "Service of process deadline"}}'),

-- Answer
('d9000002-0000-0000-0000-000000000002', 'district9',
 'FRCP 12(a)(1) — Answer',
 'Defendant must serve answer within 21 days after being served with the summons and complaint.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 12(a)(1)',
 '{"trigger": "service_completed", "case_type": "civil"}',
 '{"create_deadline": {"days": 21, "title": "Answer due"}}'),

-- Discovery Conference
('d9000003-0000-0000-0000-000000000003', 'district9',
 'FRCP 26(f) — Discovery Conference',
 'Parties must confer to consider claims, defenses, discovery plan, and settlement at least 21 days before scheduling conference or scheduling order due date.',
 'Federal Rules of Civil Procedure', 'Discovery', 1, 'Active', 'Fed. R. Civ. P. 26(f)',
 '{"trigger": "answer_filed", "case_type": "civil"}',
 '{"create_deadline": {"days": 21, "title": "Rule 26(f) discovery conference"}}'),

-- Scheduling Order
('d9000004-0000-0000-0000-000000000004', 'district9',
 'FRCP 16(b) — Scheduling Order',
 'Court must issue scheduling order within 8 weeks after defendant has been served (or 90 days after complaint served). Order limits time for joinder, amendment, motions, and discovery.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 16(b)',
 '{"trigger": "service_completed", "case_type": "civil"}',
 '{"create_deadline": {"days": 56, "title": "Scheduling order due"}}'),

-- Initial Disclosures
('d9000005-0000-0000-0000-000000000005', 'district9',
 'FRCP 26(a)(1) — Initial Disclosures',
 'Each party must disclose names of witnesses, copies of documents, damages computation, and insurance agreements within 14 days after the Rule 26(f) conference.',
 'Federal Rules of Civil Procedure', 'Discovery', 1, 'Active', 'Fed. R. Civ. P. 26(a)(1)',
 '{"trigger": "discovery_conference", "case_type": "civil"}',
 '{"create_deadline": {"days": 14, "title": "Initial disclosures due"}}'),

-- Expert Disclosures
('d9000006-0000-0000-0000-000000000006', 'district9',
 'FRCP 26(a)(2) — Expert Disclosures',
 'Party must disclose identity of expert witnesses and expert reports at least 90 days before trial date. Rebuttal experts due 30 days after opposing expert disclosure.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 26(a)(2)',
 '{"trigger": "trial_date_set", "case_type": "civil"}',
 '{"create_deadline": {"days_before_trial": 90, "title": "Expert disclosures due"}}'),

-- Pretrial Disclosures
('d9000007-0000-0000-0000-000000000007', 'district9',
 'FRCP 26(a)(3) — Pretrial Disclosures',
 'Must disclose witness list, designations of deposition testimony, and exhibit list at least 30 days before trial. Objections due within 14 days of disclosure.',
 'Federal Rules of Civil Procedure', 'Deadline', 2, 'Active', 'Fed. R. Civ. P. 26(a)(3)',
 '{"trigger": "trial_date_set", "case_type": "civil"}',
 '{"create_deadline": {"days_before_trial": 30, "title": "Pretrial disclosures due"}}'),

-- Summary Judgment
('d9000008-0000-0000-0000-000000000008', 'district9',
 'FRCP 56 — Summary Judgment',
 'Party may file motion for summary judgment at any time until 30 days after close of discovery. Court may grant if no genuine dispute of material fact exists.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 56',
 '{"trigger": "discovery_closed", "case_type": "civil"}',
 '{"create_deadline": {"days": 30, "title": "Summary judgment motion deadline"}}'),

-- Interrogatories
('d9000009-0000-0000-0000-000000000009', 'district9',
 'FRCP 33 — Interrogatories',
 'Maximum 25 interrogatories (including subparts) without court leave. Responding party has 30 days to serve answers or objections.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 33',
 '{"case_type": "civil"}',
 '{"response_deadline_days": 30}'),

-- Document Production
('d900000a-0000-0000-0000-000000000001', 'district9',
 'FRCP 34 — Document Production',
 'Response to requests for production due within 30 days of service. Must state whether inspection will be permitted and describe withheld documents.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 34',
 '{"case_type": "civil"}',
 '{"response_deadline_days": 30}'),

-- Jury Demand
('d900000b-0000-0000-0000-000000000002', 'district9',
 'FRCP 38 — Right to Jury Trial',
 'Jury trial demand must be served no later than 14 days after service of the last pleading directed to the triable issue. Failure to demand waives the right.',
 'Federal Rules of Civil Procedure', 'Filing', 1, 'Active', 'Fed. R. Civ. P. 38',
 '{"trigger": "last_pleading_served", "case_type": "civil"}',
 '{"create_deadline": {"days": 14, "title": "Jury demand deadline"}}'),

-- Local: Motion Response
('d900000c-0000-0000-0000-000000000003', 'district9',
 'Local Rule — Motion Response',
 'Opposition to any motion must be filed within 14 days after service of the motion.',
 'Local Rules', 'Filing', 2, 'Active', 'L.R. 7.1(b)',
 '{"trigger": "motion_filed", "case_type": "civil"}',
 '{"create_deadline": {"days": 14, "title": "Motion response due"}}'),

-- Local: Reply Brief
('d900000d-0000-0000-0000-000000000004', 'district9',
 'Local Rule — Reply Brief',
 'Reply in support of motion may be filed within 7 days after service of the opposition.',
 'Local Rules', 'Filing', 3, 'Active', 'L.R. 7.1(c)',
 '{"trigger": "response_filed", "case_type": "civil"}',
 '{"create_deadline": {"days": 7, "title": "Reply brief due"}}'),

-- Local: Dismiss for Want of Prosecution
('d900000e-0000-0000-0000-000000000005', 'district9',
 'Local Rule — Dismiss for Failure to Prosecute',
 'Cases with no docket activity for 6 months may be dismissed for failure to prosecute. Clerk issues show cause order before dismissal.',
 'Local Rules', 'Administrative', 1, 'Active', 'L.R. 41.1',
 '{"trigger": "no_activity", "inactivity_days": 180, "case_type": "civil"}',
 '{"action": "show_cause_order"}'),

-- Class Action
('d900000f-0000-0000-0000-000000000006', 'district9',
 'FRCP 23 — Class Actions',
 'Court must determine class certification at an early practicable time. Requires numerosity, commonality, typicality, and adequacy of representation.',
 'Federal Rules of Civil Procedure', 'Procedural', 1, 'Active', 'Fed. R. Civ. P. 23',
 '{"trigger": "class_action_filed", "case_type": "civil"}',
 '{"create_deadline": {"title": "Class certification determination"}}')
ON CONFLICT (id) DO NOTHING;

-- district12 FRCP rules (same rules, different court)
INSERT INTO rules (id, court_id, name, description, source, category, priority, status, citation, conditions, actions)
VALUES
('d1200001-0000-0000-0000-000000000001', 'district12',
 'FRCP 4(m) — Service of Process',
 'Plaintiff must serve defendant within 90 days of filing complaint. Court must dismiss without prejudice or order service within specified time if good cause shown.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 4(m)',
 '{"trigger": "case_filed", "case_type": "civil"}',
 '{"create_deadline": {"days": 90, "title": "Service of process deadline"}}'),

('d1200002-0000-0000-0000-000000000002', 'district12',
 'FRCP 12(a)(1) — Answer',
 'Defendant must serve answer within 21 days after being served with the summons and complaint.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 12(a)(1)',
 '{"trigger": "service_completed", "case_type": "civil"}',
 '{"create_deadline": {"days": 21, "title": "Answer due"}}'),

('d1200003-0000-0000-0000-000000000003', 'district12',
 'FRCP 26(f) — Discovery Conference',
 'Parties must confer to consider claims, defenses, discovery plan, and settlement at least 21 days before scheduling conference or scheduling order due date.',
 'Federal Rules of Civil Procedure', 'Discovery', 1, 'Active', 'Fed. R. Civ. P. 26(f)',
 '{"trigger": "answer_filed", "case_type": "civil"}',
 '{"create_deadline": {"days": 21, "title": "Rule 26(f) discovery conference"}}'),

('d1200004-0000-0000-0000-000000000004', 'district12',
 'FRCP 16(b) — Scheduling Order',
 'Court must issue scheduling order within 8 weeks after defendant has been served (or 90 days after complaint served). Order limits time for joinder, amendment, motions, and discovery.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 16(b)',
 '{"trigger": "service_completed", "case_type": "civil"}',
 '{"create_deadline": {"days": 56, "title": "Scheduling order due"}}'),

('d1200005-0000-0000-0000-000000000005', 'district12',
 'FRCP 26(a)(1) — Initial Disclosures',
 'Each party must disclose names of witnesses, copies of documents, damages computation, and insurance agreements within 14 days after the Rule 26(f) conference.',
 'Federal Rules of Civil Procedure', 'Discovery', 1, 'Active', 'Fed. R. Civ. P. 26(a)(1)',
 '{"trigger": "discovery_conference", "case_type": "civil"}',
 '{"create_deadline": {"days": 14, "title": "Initial disclosures due"}}'),

('d1200006-0000-0000-0000-000000000006', 'district12',
 'FRCP 26(a)(2) — Expert Disclosures',
 'Party must disclose identity of expert witnesses and expert reports at least 90 days before trial date. Rebuttal experts due 30 days after opposing expert disclosure.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 26(a)(2)',
 '{"trigger": "trial_date_set", "case_type": "civil"}',
 '{"create_deadline": {"days_before_trial": 90, "title": "Expert disclosures due"}}'),

('d1200007-0000-0000-0000-000000000007', 'district12',
 'FRCP 26(a)(3) — Pretrial Disclosures',
 'Must disclose witness list, designations of deposition testimony, and exhibit list at least 30 days before trial. Objections due within 14 days of disclosure.',
 'Federal Rules of Civil Procedure', 'Deadline', 2, 'Active', 'Fed. R. Civ. P. 26(a)(3)',
 '{"trigger": "trial_date_set", "case_type": "civil"}',
 '{"create_deadline": {"days_before_trial": 30, "title": "Pretrial disclosures due"}}'),

('d1200008-0000-0000-0000-000000000008', 'district12',
 'FRCP 56 — Summary Judgment',
 'Party may file motion for summary judgment at any time until 30 days after close of discovery. Court may grant if no genuine dispute of material fact exists.',
 'Federal Rules of Civil Procedure', 'Deadline', 1, 'Active', 'Fed. R. Civ. P. 56',
 '{"trigger": "discovery_closed", "case_type": "civil"}',
 '{"create_deadline": {"days": 30, "title": "Summary judgment motion deadline"}}'),

('d1200009-0000-0000-0000-000000000009', 'district12',
 'FRCP 33 — Interrogatories',
 'Maximum 25 interrogatories (including subparts) without court leave. Responding party has 30 days to serve answers or objections.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 33',
 '{"case_type": "civil"}',
 '{"response_deadline_days": 30}'),

('d120000a-0000-0000-0000-000000000001', 'district12',
 'FRCP 34 — Document Production',
 'Response to requests for production due within 30 days of service. Must state whether inspection will be permitted and describe withheld documents.',
 'Federal Rules of Civil Procedure', 'Discovery', 2, 'Active', 'Fed. R. Civ. P. 34',
 '{"case_type": "civil"}',
 '{"response_deadline_days": 30}'),

('d120000b-0000-0000-0000-000000000002', 'district12',
 'FRCP 38 — Right to Jury Trial',
 'Jury trial demand must be served no later than 14 days after service of the last pleading directed to the triable issue. Failure to demand waives the right.',
 'Federal Rules of Civil Procedure', 'Filing', 1, 'Active', 'Fed. R. Civ. P. 38',
 '{"trigger": "last_pleading_served", "case_type": "civil"}',
 '{"create_deadline": {"days": 14, "title": "Jury demand deadline"}}'),

('d120000c-0000-0000-0000-000000000003', 'district12',
 'Local Rule — Motion Response',
 'Opposition to any motion must be filed within 14 days after service of the motion.',
 'Local Rules', 'Filing', 2, 'Active', 'L.R. 7.1(b)',
 '{"trigger": "motion_filed", "case_type": "civil"}',
 '{"create_deadline": {"days": 14, "title": "Motion response due"}}'),

('d120000d-0000-0000-0000-000000000004', 'district12',
 'Local Rule — Reply Brief',
 'Reply in support of motion may be filed within 7 days after service of the opposition.',
 'Local Rules', 'Filing', 3, 'Active', 'L.R. 7.1(c)',
 '{"trigger": "response_filed", "case_type": "civil"}',
 '{"create_deadline": {"days": 7, "title": "Reply brief due"}}'),

('d120000e-0000-0000-0000-000000000005', 'district12',
 'Local Rule — Dismiss for Failure to Prosecute',
 'Cases with no docket activity for 6 months may be dismissed for failure to prosecute. Clerk issues show cause order before dismissal.',
 'Local Rules', 'Administrative', 1, 'Active', 'L.R. 41.1',
 '{"trigger": "no_activity", "inactivity_days": 180, "case_type": "civil"}',
 '{"action": "show_cause_order"}'),

('d120000f-0000-0000-0000-000000000006', 'district12',
 'FRCP 23 — Class Actions',
 'Court must determine class certification at an early practicable time. Requires numerosity, commonality, typicality, and adequacy of representation.',
 'Federal Rules of Civil Procedure', 'Procedural', 1, 'Active', 'Fed. R. Civ. P. 23',
 '{"trigger": "class_action_filed", "case_type": "civil"}',
 '{"create_deadline": {"title": "Class certification determination"}}')
ON CONFLICT (id) DO NOTHING;

END $$;
