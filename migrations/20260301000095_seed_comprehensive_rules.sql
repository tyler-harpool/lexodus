-- Seed comprehensive Federal Rules (FRCP, FRCrP, FRE, FRAP, Statutory, Local)
-- for district9 and district12 using tagged-enum conditions/actions format.
-- Also backfills triggers on existing rules from migration 000089.

-- ============================================================
-- TIER 1: FRCP Deadline Rules (Civil)
-- ============================================================

-- 1. FRCP 4(m) -- Service of Process (90 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 4(m) — Service of Process',
    'Plaintiff must serve the summons and complaint on the defendant within 90 days after filing. If the plaintiff fails to serve within this period, the court must dismiss the action without prejudice or order service within a specified time.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCP 4(m)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Service of process", "days_from_trigger": 90}]'::jsonb,
    '["case_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 4(m) — Service of Process',
    'Plaintiff must serve the summons and complaint on the defendant within 90 days after filing. If the plaintiff fails to serve within this period, the court must dismiss the action without prejudice or order service within a specified time.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCP 4(m)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Service of process", "days_from_trigger": 90}]'::jsonb,
    '["case_filed"]'::jsonb
);

-- 2. FRCP 12(a)(1) -- Answer to Complaint (21 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 12(a)(1) — Answer to Complaint',
    'A defendant must serve an answer within 21 days after being served with the summons and complaint, unless a federal statute provides otherwise.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCP 12(a)(1)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Answer to complaint", "days_from_trigger": 21}]'::jsonb,
    '["complaint_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 12(a)(1) — Answer to Complaint',
    'A defendant must serve an answer within 21 days after being served with the summons and complaint, unless a federal statute provides otherwise.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCP 12(a)(1)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Answer to complaint", "days_from_trigger": 21}]'::jsonb,
    '["complaint_filed"]'::jsonb
);

-- 3. FRCP 12(a)(4) -- Response to Motion (14 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 12(a)(4) — Response to Motion',
    'Unless the court sets a different time, a party must serve a response to a motion within 14 days after being served with the motion.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCP 12(a)(4)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Response to motion", "days_from_trigger": 14}]'::jsonb,
    '["motion_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 12(a)(4) — Response to Motion',
    'Unless the court sets a different time, a party must serve a response to a motion within 14 days after being served with the motion.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCP 12(a)(4)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Response to motion", "days_from_trigger": 14}]'::jsonb,
    '["motion_filed"]'::jsonb
);

-- 4. FRCP 26(a)(1) -- Initial Disclosures (14 days after 26(f) conference)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 26(a)(1) — Initial Disclosures',
    'A party must provide initial disclosures—including witness identities, document copies, damage computations, and insurance agreements—within 14 days after the parties'' Rule 26(f) conference.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district9', 'FRCP 26(a)(1)',
    '[{"type": "field_contains", "field": "document_type", "value": "26f"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Initial disclosures", "days_from_trigger": 14}]'::jsonb,
    '["document_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 26(a)(1) — Initial Disclosures',
    'A party must provide initial disclosures—including witness identities, document copies, damage computations, and insurance agreements—within 14 days after the parties'' Rule 26(f) conference.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district12', 'FRCP 26(a)(1)',
    '[{"type": "field_contains", "field": "document_type", "value": "26f"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Initial disclosures", "days_from_trigger": 14}]'::jsonb,
    '["document_filed"]'::jsonb
);

-- 5. FRCP 26(f) -- Discovery Planning Conference (90 days from filing to complete)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 26(f) — Discovery Planning Conference',
    'Parties must confer as soon as practicable—and at least 21 days before a scheduling conference—to consider claims, defenses, the discovery plan, and settlement possibilities. Deadline set at 90 days from case filing to allow scheduling.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district9', 'FRCP 26(f)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Rule 26(f) conference", "days_from_trigger": 90}]'::jsonb,
    '["case_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 26(f) — Discovery Planning Conference',
    'Parties must confer as soon as practicable—and at least 21 days before a scheduling conference—to consider claims, defenses, the discovery plan, and settlement possibilities. Deadline set at 90 days from case filing to allow scheduling.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district12', 'FRCP 26(f)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Rule 26(f) conference", "days_from_trigger": 90}]'::jsonb,
    '["case_filed"]'::jsonb
);

-- 6. FRCP 33 -- Interrogatory Responses (30 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 33 — Interrogatory Responses',
    'The responding party must serve its answers and any objections to interrogatories within 30 days after being served. No more than 25 interrogatories (including subparts) without leave of court.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district9', 'FRCP 33',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Interrogatory responses", "days_from_trigger": 30}]'::jsonb,
    '["discovery_request_served"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 33 — Interrogatory Responses',
    'The responding party must serve its answers and any objections to interrogatories within 30 days after being served. No more than 25 interrogatories (including subparts) without leave of court.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district12', 'FRCP 33',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Interrogatory responses", "days_from_trigger": 30}]'::jsonb,
    '["discovery_request_served"]'::jsonb
);

-- 7. FRCP 34 -- Document Production (30 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 34 — Document Production',
    'A party must respond to a request for production within 30 days of service, stating whether inspection will be permitted and describing any withheld documents.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district9', 'FRCP 34',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Document production response", "days_from_trigger": 30}]'::jsonb,
    '["discovery_request_served"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 34 — Document Production',
    'A party must respond to a request for production within 30 days of service, stating whether inspection will be permitted and describing any withheld documents.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district12', 'FRCP 34',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Document production response", "days_from_trigger": 30}]'::jsonb,
    '["discovery_request_served"]'::jsonb
);

-- 8. FRCP 36 -- Admission Requests (30 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 36 — Admission Requests',
    'A matter is admitted unless the party to whom the request is directed serves a written answer or objection within 30 days after being served.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district9', 'FRCP 36',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Response to admission requests", "days_from_trigger": 30}]'::jsonb,
    '["discovery_request_served"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 36 — Admission Requests',
    'A matter is admitted unless the party to whom the request is directed serves a written answer or objection within 30 days after being served.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district12', 'FRCP 36',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Response to admission requests", "days_from_trigger": 30}]'::jsonb,
    '["discovery_request_served"]'::jsonb
);

-- 9. FRCP 56 -- Summary Judgment Response (21 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 56 — Summary Judgment Response',
    'If a motion for summary judgment is properly supported, the opposing party must respond within 21 days showing a genuine dispute of material fact. The court shall grant if no genuine dispute exists.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCP 56',
    '[{"type": "field_contains", "field": "document_type", "value": "summary_judgment"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Response to summary judgment", "days_from_trigger": 21}]'::jsonb,
    '["motion_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 56 — Summary Judgment Response',
    'If a motion for summary judgment is properly supported, the opposing party must respond within 21 days showing a genuine dispute of material fact. The court shall grant if no genuine dispute exists.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCP 56',
    '[{"type": "field_contains", "field": "document_type", "value": "summary_judgment"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Response to summary judgment", "days_from_trigger": 21}]'::jsonb,
    '["motion_filed"]'::jsonb
);

-- 10. FRCP 59 -- Motion for New Trial (28 days after judgment)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 59 — Motion for New Trial',
    'A motion for a new trial must be filed no later than 28 days after the entry of judgment. The court may grant a new trial for any reason that would support one at common law.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCP 59',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Motion for new trial", "days_from_trigger": 28}]'::jsonb,
    '["judgment_entered"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 59 — Motion for New Trial',
    'A motion for a new trial must be filed no later than 28 days after the entry of judgment. The court may grant a new trial for any reason that would support one at common law.',
    'Federal Rules of Civil Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCP 59',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Motion for new trial", "days_from_trigger": 28}]'::jsonb,
    '["judgment_entered"]'::jsonb
);

-- ============================================================
-- TIER 2: FRCrP Rules (Criminal)
-- ============================================================

-- 11. FRCrP 5(a) -- Initial Appearance (flag for review)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCrP 5(a) — Initial Appearance',
    'A person making an arrest must take the defendant without unnecessary delay before a magistrate judge. Appearance required within 48 hours of arrest.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCrP 5(a)',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "flag_for_review", "reason": "Initial appearance required within 48 hours"}]'::jsonb,
    '["case_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCrP 5(a) — Initial Appearance',
    'A person making an arrest must take the defendant without unnecessary delay before a magistrate judge. Appearance required within 48 hours of arrest.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCrP 5(a)',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "flag_for_review", "reason": "Initial appearance required within 48 hours"}]'::jsonb,
    '["case_filed"]'::jsonb
);

-- 12. FRCrP 10 -- Arraignment (14 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCrP 10 — Arraignment',
    'Arraignment must be conducted in open court. The defendant is informed of the charges and asked to enter a plea. Must occur within a reasonable time after initial appearance.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCrP 10',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Arraignment", "days_from_trigger": 14}]'::jsonb,
    '["case_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCrP 10 — Arraignment',
    'Arraignment must be conducted in open court. The defendant is informed of the charges and asked to enter a plea. Must occur within a reasonable time after initial appearance.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCrP 10',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Arraignment", "days_from_trigger": 14}]'::jsonb,
    '["case_filed"]'::jsonb
);

-- 13. FRCrP 12(b) -- Pretrial Motions (14 days before trial)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCrP 12(b) — Pretrial Motions',
    'Certain defenses, objections, and requests must be raised by pretrial motion. The court may set a deadline for pretrial motions; 14-day default before trial.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCrP 12(b)',
    '[{"type": "field_equals", "field": "new_status", "value": "trial_ready"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Pretrial motions deadline", "days_from_trigger": 14}]'::jsonb,
    '["status_changed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCrP 12(b) — Pretrial Motions',
    'Certain defenses, objections, and requests must be raised by pretrial motion. The court may set a deadline for pretrial motions; 14-day default before trial.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCrP 12(b)',
    '[{"type": "field_equals", "field": "new_status", "value": "trial_ready"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Pretrial motions deadline", "days_from_trigger": 14}]'::jsonb,
    '["status_changed"]'::jsonb
);

-- 14. FRCrP 29 -- Motion for Judgment of Acquittal (14 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCrP 29 — Motion for Judgment of Acquittal',
    'A defendant may move for a judgment of acquittal after the government closes its evidence or after the close of all evidence. Post-verdict motion must be filed within 14 days of guilty verdict or jury discharge.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCrP 29',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Motion for judgment of acquittal", "days_from_trigger": 14}]'::jsonb,
    '["judgment_entered"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCrP 29 — Motion for Judgment of Acquittal',
    'A defendant may move for a judgment of acquittal after the government closes its evidence or after the close of all evidence. Post-verdict motion must be filed within 14 days of guilty verdict or jury discharge.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCrP 29',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Motion for judgment of acquittal", "days_from_trigger": 14}]'::jsonb,
    '["judgment_entered"]'::jsonb
);

-- 15. FRCrP 32 -- Presentence Report (35 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCrP 32 — Presentence Report',
    'The probation officer must conduct a presentence investigation and submit a report to the court before sentencing. Report due at least 35 days before sentencing.',
    'Federal Rules of Criminal Procedure', 'Sentencing', 20, 'Active', 'district9', 'FRCrP 32',
    '[{"type": "field_equals", "field": "new_status", "value": "awaiting_sentencing"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Presentence report", "days_from_trigger": 35}]'::jsonb,
    '["status_changed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCrP 32 — Presentence Report',
    'The probation officer must conduct a presentence investigation and submit a report to the court before sentencing. Report due at least 35 days before sentencing.',
    'Federal Rules of Criminal Procedure', 'Sentencing', 20, 'Active', 'district12', 'FRCrP 32',
    '[{"type": "field_equals", "field": "new_status", "value": "awaiting_sentencing"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Presentence report", "days_from_trigger": 35}]'::jsonb,
    '["status_changed"]'::jsonb
);

-- 16. FRCrP 33 -- Motion for New Trial (14 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCrP 33 — Motion for New Trial',
    'A motion for new trial based on newly discovered evidence must be filed within 3 years after the verdict; any other motion for new trial must be filed within 14 days of the verdict.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district9', 'FRCrP 33',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Motion for new trial", "days_from_trigger": 14}]'::jsonb,
    '["judgment_entered"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCrP 33 — Motion for New Trial',
    'A motion for new trial based on newly discovered evidence must be filed within 3 years after the verdict; any other motion for new trial must be filed within 14 days of the verdict.',
    'Federal Rules of Criminal Procedure', 'Deadline', 20, 'Active', 'district12', 'FRCrP 33',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Motion for new trial", "days_from_trigger": 14}]'::jsonb,
    '["judgment_entered"]'::jsonb
);

-- 17. FRCrP 35 -- Sentence Correction (14 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCrP 35 — Sentence Correction',
    'Within 14 days after sentencing, the court may correct a sentence that resulted from arithmetical, technical, or other clear error. Government may also move to reduce for substantial assistance.',
    'Federal Rules of Criminal Procedure', 'Sentencing', 20, 'Active', 'district9', 'FRCrP 35',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Motion to correct sentence", "days_from_trigger": 14}]'::jsonb,
    '["sentencing_scheduled"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCrP 35 — Sentence Correction',
    'Within 14 days after sentencing, the court may correct a sentence that resulted from arithmetical, technical, or other clear error. Government may also move to reduce for substantial assistance.',
    'Federal Rules of Criminal Procedure', 'Sentencing', 20, 'Active', 'district12', 'FRCrP 35',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Motion to correct sentence", "days_from_trigger": 14}]'::jsonb,
    '["sentencing_scheduled"]'::jsonb
);

-- ============================================================
-- TIER 3: Filing & Service Rules
-- ============================================================

-- 18. FRCP 5(b) -- Service Requirements (log compliance)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 5(b) — Service Requirements',
    'Every pleading filed after the original complaint, every discovery paper, every written motion, and every written notice must be served on each party. Service may be made electronically through CM/ECF.',
    'Federal Rules of Civil Procedure', 'Service', 20, 'Active', 'district9', 'FRCP 5(b)',
    '[{"type": "always"}]'::jsonb,
    '[{"type": "log_compliance", "message": "Service required per FRCP 5(b)"}]'::jsonb,
    '["document_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 5(b) — Service Requirements',
    'Every pleading filed after the original complaint, every discovery paper, every written motion, and every written notice must be served on each party. Service may be made electronically through CM/ECF.',
    'Federal Rules of Civil Procedure', 'Service', 20, 'Active', 'district12', 'FRCP 5(b)',
    '[{"type": "always"}]'::jsonb,
    '[{"type": "log_compliance", "message": "Service required per FRCP 5(b)"}]'::jsonb,
    '["document_filed"]'::jsonb
);

-- 19. FRCP 5.2 -- Privacy Protection (flag for review)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 5.2 — Privacy Protection',
    'Filings containing SSNs, taxpayer IDs, birth dates, financial account numbers, or minor names must redact to partial identifiers. Full documents may be filed under seal.',
    'Federal Rules of Civil Procedure', 'Privacy', 20, 'Active', 'district9', 'FRCP 5.2',
    '[{"type": "always"}]'::jsonb,
    '[{"type": "flag_for_review", "reason": "Verify PII redactions per FRCP 5.2"}]'::jsonb,
    '["document_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 5.2 — Privacy Protection',
    'Filings containing SSNs, taxpayer IDs, birth dates, financial account numbers, or minor names must redact to partial identifiers. Full documents may be filed under seal.',
    'Federal Rules of Civil Procedure', 'Privacy', 20, 'Active', 'district12', 'FRCP 5.2',
    '[{"type": "always"}]'::jsonb,
    '[{"type": "flag_for_review", "reason": "Verify PII redactions per FRCP 5.2"}]'::jsonb,
    '["document_filed"]'::jsonb
);

-- 20. FRCP 11 -- Certification of Filings (log compliance)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 11 — Certification of Filings',
    'By presenting a filing to the court, an attorney certifies that: (1) it is not for improper purpose, (2) legal contentions are warranted, (3) factual contentions have evidentiary support, and (4) denials are warranted on evidence.',
    'Federal Rules of Civil Procedure', 'Filing', 20, 'Active', 'district9', 'FRCP 11',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "log_compliance", "message": "Filing certified under FRCP 11"}]'::jsonb,
    '["document_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 11 — Certification of Filings',
    'By presenting a filing to the court, an attorney certifies that: (1) it is not for improper purpose, (2) legal contentions are warranted, (3) factual contentions have evidentiary support, and (4) denials are warranted on evidence.',
    'Federal Rules of Civil Procedure', 'Filing', 20, 'Active', 'district12', 'FRCP 11',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "log_compliance", "message": "Filing certified under FRCP 11"}]'::jsonb,
    '["document_filed"]'::jsonb
);

-- ============================================================
-- TIER 4: Discovery Rules
-- ============================================================

-- 21. FRCP 37 -- Discovery Sanctions (log compliance)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 37 — Discovery Sanctions',
    'If a party fails to comply with discovery obligations, the court may order sanctions including prohibiting the introduction of evidence, striking pleadings, or entering default judgment.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district9', 'FRCP 37',
    '[{"type": "always"}]'::jsonb,
    '[{"type": "log_compliance", "message": "Check for outstanding discovery obligations"}]'::jsonb,
    '["status_changed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 37 — Discovery Sanctions',
    'If a party fails to comply with discovery obligations, the court may order sanctions including prohibiting the introduction of evidence, striking pleadings, or entering default judgment.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district12', 'FRCP 37',
    '[{"type": "always"}]'::jsonb,
    '[{"type": "log_compliance", "message": "Check for outstanding discovery obligations"}]'::jsonb,
    '["status_changed"]'::jsonb
);

-- 22. FRCP 30(a) -- Deposition Limit (flag if >10)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRCP 30(a) — Deposition Limit',
    'A party must obtain leave of court to take more than 10 depositions. Each deposition is limited to 1 day of 7 hours unless otherwise agreed or ordered.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district9', 'FRCP 30(a)',
    '[{"type": "field_equals", "field": "document_type", "value": "deposition"}]'::jsonb,
    '[{"type": "flag_for_review", "reason": "Check deposition limit (10 per party)"}]'::jsonb,
    '["document_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRCP 30(a) — Deposition Limit',
    'A party must obtain leave of court to take more than 10 depositions. Each deposition is limited to 1 day of 7 hours unless otherwise agreed or ordered.',
    'Federal Rules of Civil Procedure', 'Discovery', 20, 'Active', 'district12', 'FRCP 30(a)',
    '[{"type": "field_equals", "field": "document_type", "value": "deposition"}]'::jsonb,
    '[{"type": "flag_for_review", "reason": "Check deposition limit (10 per party)"}]'::jsonb,
    '["document_filed"]'::jsonb
);

-- ============================================================
-- TIER 5: Appeal Rules
-- ============================================================

-- 23. FRAP 4(a) -- Civil Appeal Notice (30 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRAP 4(a) — Civil Appeal Notice',
    'In a civil case, a notice of appeal must be filed within 30 days after entry of the judgment or order appealed from. If the United States is a party, the time is 60 days.',
    'Federal Rules of Appellate Procedure', 'Appeal', 20, 'Active', 'district9', 'FRAP 4(a)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Notice of appeal", "days_from_trigger": 30}]'::jsonb,
    '["judgment_entered"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRAP 4(a) — Civil Appeal Notice',
    'In a civil case, a notice of appeal must be filed within 30 days after entry of the judgment or order appealed from. If the United States is a party, the time is 60 days.',
    'Federal Rules of Appellate Procedure', 'Appeal', 20, 'Active', 'district12', 'FRAP 4(a)',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Notice of appeal", "days_from_trigger": 30}]'::jsonb,
    '["judgment_entered"]'::jsonb
);

-- 24. FRAP 4(b) -- Criminal Appeal Notice (14 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'FRAP 4(b) — Criminal Appeal Notice',
    'In a criminal case, a defendant''s notice of appeal must be filed within 14 days after entry of the judgment or order being appealed. The government must also file within 30 days.',
    'Federal Rules of Appellate Procedure', 'Appeal', 20, 'Active', 'district9', 'FRAP 4(b)',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Notice of appeal", "days_from_trigger": 14}]'::jsonb,
    '["judgment_entered"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'FRAP 4(b) — Criminal Appeal Notice',
    'In a criminal case, a defendant''s notice of appeal must be filed within 14 days after entry of the judgment or order being appealed. The government must also file within 30 days.',
    'Federal Rules of Appellate Procedure', 'Appeal', 20, 'Active', 'district12', 'FRAP 4(b)',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Notice of appeal", "days_from_trigger": 14}]'::jsonb,
    '["judgment_entered"]'::jsonb
);

-- ============================================================
-- TIER 6: Fee Rules
-- ============================================================

-- 25. 28 USC 1914 -- Civil Filing Fee
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    '28 USC 1914 — Civil Filing Fee',
    'The clerk of each district court shall require the parties instituting any civil action to pay a filing fee of $405.00 (as of 2024). Fee waivers available under 28 USC 1915 for in forma pauperis.',
    'Statutory', 'Fee', 10, 'Active', 'district9', '28 U.S.C. 1914',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "require_fee", "amount_cents": 40500, "description": "Civil case filing fee"}]'::jsonb,
    '["case_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    '28 USC 1914 — Civil Filing Fee',
    'The clerk of each district court shall require the parties instituting any civil action to pay a filing fee of $405.00 (as of 2024). Fee waivers available under 28 USC 1915 for in forma pauperis.',
    'Statutory', 'Fee', 10, 'Active', 'district12', '28 U.S.C. 1914',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "require_fee", "amount_cents": 40500, "description": "Civil case filing fee"}]'::jsonb,
    '["case_filed"]'::jsonb
);

-- 26. 28 USC 1917 -- Appeal Filing Fee
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    '28 USC 1917 — Appeal Filing Fee',
    'The clerk of each district court shall require a fee of $605.00 for filing a notice of appeal in any civil or criminal case to a court of appeals.',
    'Statutory', 'Fee', 10, 'Active', 'district9', '28 U.S.C. 1917',
    '[{"type": "field_equals", "field": "document_type", "value": "notice_of_appeal"}]'::jsonb,
    '[{"type": "require_fee", "amount_cents": 60500, "description": "Appeal filing fee"}]'::jsonb,
    '["document_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    '28 USC 1917 — Appeal Filing Fee',
    'The clerk of each district court shall require a fee of $605.00 for filing a notice of appeal in any civil or criminal case to a court of appeals.',
    'Statutory', 'Fee', 10, 'Active', 'district12', '28 U.S.C. 1917',
    '[{"type": "field_equals", "field": "document_type", "value": "notice_of_appeal"}]'::jsonb,
    '[{"type": "require_fee", "amount_cents": 60500, "description": "Appeal filing fee"}]'::jsonb,
    '["document_filed"]'::jsonb
);

-- ============================================================
-- TIER 7: Administrative / Local / Speedy Trial
-- ============================================================

-- 27. Local Rule 7.1 -- Corporate Disclosure (7 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Local Rule 7.1 — Corporate Disclosure',
    'A nongovernmental corporate party must file a disclosure statement identifying any parent corporation and any publicly held corporation owning 10% or more of its stock within 7 days of case filing.',
    'Local Rules', 'Filing', 40, 'Active', 'district9', 'L.R. 7.1',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Corporate disclosure statement", "days_from_trigger": 7}]'::jsonb,
    '["case_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Local Rule 7.1 — Corporate Disclosure',
    'A nongovernmental corporate party must file a disclosure statement identifying any parent corporation and any publicly held corporation owning 10% or more of its stock within 7 days of case filing.',
    'Local Rules', 'Filing', 40, 'Active', 'district12', 'L.R. 7.1',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Corporate disclosure statement", "days_from_trigger": 7}]'::jsonb,
    '["case_filed"]'::jsonb
);

-- 28. Local Rule 16.1 -- Scheduling Order Proposal (14 days)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Local Rule 16.1 — Scheduling Order Proposal',
    'Within 14 days of filing, parties must submit a proposed scheduling order addressing discovery deadlines, motion deadlines, and trial date. Parties must confer before submission.',
    'Local Rules', 'Filing', 40, 'Active', 'district9', 'L.R. 16.1',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Proposed scheduling order", "days_from_trigger": 14}]'::jsonb,
    '["case_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Local Rule 16.1 — Scheduling Order Proposal',
    'Within 14 days of filing, parties must submit a proposed scheduling order addressing discovery deadlines, motion deadlines, and trial date. Parties must confer before submission.',
    'Local Rules', 'Filing', 40, 'Active', 'district12', 'L.R. 16.1',
    '[{"type": "field_equals", "field": "case_type", "value": "civil"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Proposed scheduling order", "days_from_trigger": 14}]'::jsonb,
    '["case_filed"]'::jsonb
);

-- 29. Speedy Trial Act 18 USC 3161(b) -- Indictment (30 days from arrest)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Speedy Trial Act 18 USC 3161(b) — Indictment Deadline',
    'An indictment or information must be filed within 30 days from the date of arrest or service of summons. Certain delays (competency hearings, continuances) are excludable.',
    'Statutory', 'Deadline', 10, 'Active', 'district9', '18 U.S.C. 3161(b)',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Indictment deadline (Speedy Trial Act)", "days_from_trigger": 30}]'::jsonb,
    '["case_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Speedy Trial Act 18 USC 3161(b) — Indictment Deadline',
    'An indictment or information must be filed within 30 days from the date of arrest or service of summons. Certain delays (competency hearings, continuances) are excludable.',
    'Statutory', 'Deadline', 10, 'Active', 'district12', '18 U.S.C. 3161(b)',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Indictment deadline (Speedy Trial Act)", "days_from_trigger": 30}]'::jsonb,
    '["case_filed"]'::jsonb
);

-- 30. Speedy Trial Act 18 USC 3161(c) -- Trial (70 days from indictment)
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Speedy Trial Act 18 USC 3161(c) — Trial Deadline',
    'Trial must commence within 70 days from the filing date of the indictment or information, or from the date the defendant first appears before a judicial officer, whichever is later.',
    'Statutory', 'Deadline', 10, 'Active', 'district9', '18 U.S.C. 3161(c)',
    '[{"type": "and", "conditions": [{"type": "field_equals", "field": "case_type", "value": "criminal"}, {"type": "field_equals", "field": "document_type", "value": "indictment"}]}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Trial deadline (Speedy Trial Act)", "days_from_trigger": 70}]'::jsonb,
    '["document_filed"]'::jsonb
);
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Speedy Trial Act 18 USC 3161(c) — Trial Deadline',
    'Trial must commence within 70 days from the filing date of the indictment or information, or from the date the defendant first appears before a judicial officer, whichever is later.',
    'Statutory', 'Deadline', 10, 'Active', 'district12', '18 U.S.C. 3161(c)',
    '[{"type": "and", "conditions": [{"type": "field_equals", "field": "case_type", "value": "criminal"}, {"type": "field_equals", "field": "document_type", "value": "indictment"}]}]'::jsonb,
    '[{"type": "generate_deadline", "description": "Trial deadline (Speedy Trial Act)", "days_from_trigger": 70}]'::jsonb,
    '["document_filed"]'::jsonb
);

-- ============================================================
-- BACKFILL: Update existing rules from migration 000089
-- to have proper triggers column values
-- ============================================================

-- The backfill in migration 092 already ran:
--   UPDATE rules SET triggers = jsonb_build_array(conditions->>'trigger')
--   WHERE conditions ? 'trigger' AND triggers = '[]';
-- But catch any that were missed (e.g. rules without a 'trigger' key in conditions)
UPDATE rules
SET triggers = jsonb_build_array(conditions->>'trigger')
WHERE conditions ? 'trigger'
  AND (triggers = '[]'::jsonb OR triggers IS NULL);

-- For rules that have no trigger key but have case_type (FRCP 33, FRCP 34 from 000089),
-- set a reasonable default trigger based on their category
UPDATE rules
SET triggers = '["discovery_request_served"]'::jsonb
WHERE triggers = '[]'::jsonb
  AND source = 'Federal Rules of Civil Procedure'
  AND category = 'Discovery'
  AND name IN ('FRCP 33 — Interrogatories', 'FRCP 34 — Document Production');
