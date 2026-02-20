-- Seed lifecycle automation rules: status advancement + speedy trial
-- These rules auto-advance case status and start the Speedy Trial clock
-- when specific docket entries are filed, making the case lifecycle self-driving.

-- ═══════════════════════════════════════════════════════════════════
-- District 9 — Lifecycle Rules
-- ═══════════════════════════════════════════════════════════════════

-- Indictment/Arraignment → advance to "arraigned" + start Speedy Trial
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Arraignment: Advance Status on Indictment',
    'When an indictment is filed, advance case status to arraigned and start the Speedy Trial Act clock (70 days).',
    'Federal Rules of Criminal Procedure',
    'Procedural',
    20,
    'Active',
    'district9',
    'FRCrP 10',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "arraigned"}, {"type": "start_speedy_trial"}, {"type": "generate_deadline", "description": "Speedy Trial Act deadline (70 days)", "days_from_trigger": 70}]'::jsonb,
    '["complaint_filed"]'::jsonb
);

-- Answer filed → advance to "discovery"
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Discovery: Advance Status on Answer',
    'When an answer is filed, advance case status to discovery phase.',
    'Federal Rules of Civil Procedure',
    'Procedural',
    20,
    'Active',
    'district9',
    'FRCP 26',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "discovery"}]'::jsonb,
    '["answer_filed"]'::jsonb
);

-- Judgment/Verdict entered → advance to "awaiting_sentencing"
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Sentencing: Advance Status on Verdict',
    'When a verdict or judgment is entered, advance case to awaiting sentencing.',
    'Federal Rules of Criminal Procedure',
    'Procedural',
    20,
    'Active',
    'district9',
    'FRCrP 32',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "awaiting_sentencing"}, {"type": "generate_deadline", "description": "Sentencing hearing must be scheduled", "days_from_trigger": 90}]'::jsonb,
    '["judgment_entered"]'::jsonb
);

-- Sentence docket entry → advance to "sentenced" + appeal deadline
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Sentenced: Advance Status and Create Appeal Deadline',
    'When a sentence is entered on the docket, advance to sentenced and create the 14-day appeal deadline per FRAP 4(b).',
    'Federal Rules of Appellate Procedure',
    'Procedural',
    20,
    'Active',
    'district9',
    'FRAP 4(b)',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "sentenced"}, {"type": "generate_deadline", "description": "Notice of appeal deadline (criminal)", "days_from_trigger": 14}]'::jsonb,
    '["sentencing_scheduled"]'::jsonb
);

-- Notice of appeal → advance to "on_appeal"
INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district9',
    'Appeal: Advance Status on Notice of Appeal',
    'When a notice of appeal is filed, advance case status to on_appeal.',
    'Federal Rules of Appellate Procedure',
    'Procedural',
    20,
    'Active',
    'district9',
    'FRAP 3',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "on_appeal"}]'::jsonb,
    '["document_filed"]'::jsonb
);

-- ═══════════════════════════════════════════════════════════════════
-- District 12 — Same lifecycle rules
-- ═══════════════════════════════════════════════════════════════════

INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Arraignment: Advance Status on Indictment',
    'When an indictment is filed, advance case status to arraigned and start the Speedy Trial Act clock (70 days).',
    'Federal Rules of Criminal Procedure',
    'Procedural',
    20,
    'Active',
    'district12',
    'FRCrP 10',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "arraigned"}, {"type": "start_speedy_trial"}, {"type": "generate_deadline", "description": "Speedy Trial Act deadline (70 days)", "days_from_trigger": 70}]'::jsonb,
    '["complaint_filed"]'::jsonb
);

INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Discovery: Advance Status on Answer',
    'When an answer is filed, advance case status to discovery phase.',
    'Federal Rules of Civil Procedure',
    'Procedural',
    20,
    'Active',
    'district12',
    'FRCP 26',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "discovery"}]'::jsonb,
    '["answer_filed"]'::jsonb
);

INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Sentencing: Advance Status on Verdict',
    'When a verdict or judgment is entered, advance case to awaiting sentencing.',
    'Federal Rules of Criminal Procedure',
    'Procedural',
    20,
    'Active',
    'district12',
    'FRCrP 32',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "awaiting_sentencing"}, {"type": "generate_deadline", "description": "Sentencing hearing must be scheduled", "days_from_trigger": 90}]'::jsonb,
    '["judgment_entered"]'::jsonb
);

INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Sentenced: Advance Status and Create Appeal Deadline',
    'When a sentence is entered on the docket, advance to sentenced and create the 14-day appeal deadline per FRAP 4(b).',
    'Federal Rules of Appellate Procedure',
    'Procedural',
    20,
    'Active',
    'district12',
    'FRAP 4(b)',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "sentenced"}, {"type": "generate_deadline", "description": "Notice of appeal deadline (criminal)", "days_from_trigger": 14}]'::jsonb,
    '["document_filed"]'::jsonb
);

INSERT INTO rules (court_id, name, description, source, category, priority, status, jurisdiction, citation, conditions, actions, triggers)
VALUES (
    'district12',
    'Appeal: Advance Status on Notice of Appeal',
    'When a notice of appeal is filed, advance case status to on_appeal.',
    'Federal Rules of Appellate Procedure',
    'Procedural',
    20,
    'Active',
    'district12',
    'FRAP 3',
    '[{"type": "field_equals", "field": "case_type", "value": "criminal"}]'::jsonb,
    '[{"type": "advance_status", "new_status": "on_appeal"}]'::jsonb,
    '["document_filed"]'::jsonb
);
