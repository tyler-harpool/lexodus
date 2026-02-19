-- Realistic CM/ECF civil case seed data for district9 and district12
-- 8 civil cases with parties, docket entries, and clerk queue items
-- Uses valid UUIDs with deterministic hex prefixes for identification

DO $$ BEGIN

-- Guard: skip if data already seeded
IF EXISTS (SELECT 1 FROM civil_cases WHERE case_number = '9:26-cv-00301') THEN
    RAISE NOTICE 'Civil case seed data already exists, skipping';
    RETURN;
END IF;

-- Ensure test judges exist (seed judges if needed for FK references)
INSERT INTO judges (id, court_id, name, title, district)
VALUES
    ('a9b00001-0000-0000-0000-000000000001', 'district9', 'Hon. Ronnie Abrams', 'Judge', 'district9'),
    ('a9b00003-0000-0000-0000-000000000003', 'district9', 'Hon. Mary Kay Vyskocil', 'Judge', 'district9'),
    ('a12b0001-0000-0000-0000-000000000001', 'district12', 'Hon. Amir H. Ali', 'Judge', 'district12'),
    ('a12b0002-0000-0000-0000-000000000002', 'district12', 'Hon. Sarah Chen', 'Judge', 'district12')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- CIVIL CASES (8 total: 4 per district)
-- ============================================================

-- district9 civil cases
INSERT INTO civil_cases (id, court_id, case_number, title, description, nature_of_suit, cause_of_action, jurisdiction_basis, jury_demand, class_action, amount_in_controversy, status, priority, assigned_judge_id, district_code, location, is_sealed, pro_se)
VALUES
-- Employment discrimination (Title VII)
('a9c10001-0000-0000-0000-000000000001', 'district9', '9:26-cv-00301', 'Johnson v. Apex Technologies Inc.',
 'Former employee alleges termination based on race and gender in violation of Title VII.',
 '442', '42 U.S.C. 2000e (Title VII)', 'federal_question', 'plaintiff', false, 750000.00,
 'discovery', 'medium', 'a9b00001-0000-0000-0000-000000000001', 'district9', 'Federal City', false, false),

-- Patent infringement
('a9c10002-0000-0000-0000-000000000002', 'district9', '9:26-cv-00302', 'NovaTech Solutions LLC v. DataStream Corp.',
 'Patent holder alleges infringement of three utility patents relating to real-time data compression algorithms.',
 '830', '35 U.S.C. 271 (Patent Infringement)', 'federal_question', 'both', false, 25000000.00,
 'pretrial', 'high', 'a9b00001-0000-0000-0000-000000000001', 'district9', 'Federal City', false, false),

-- Diversity personal injury (motor vehicle)
('a9c10003-0000-0000-0000-000000000003', 'district9', '9:26-cv-00303', 'Martinez v. National Freight Lines Inc.',
 'Plaintiff sustained severe injuries when defendant''s commercial tractor-trailer crossed the center line.',
 '350', '28 U.S.C. 1332 (Diversity)', 'diversity', 'plaintiff', false, 2500000.00,
 'filed', 'medium', 'a9b00003-0000-0000-0000-000000000003', 'district9', 'Federal City', false, false),

-- Class action consumer protection (TCPA)
('a9c10004-0000-0000-0000-000000000004', 'district9', '9:26-cv-00304', 'Williams v. QuickLoan Financial Services Inc.',
 'Putative class action alleging unsolicited automated telephone calls in violation of the TCPA.',
 '485', '47 U.S.C. 227 (TCPA)', 'federal_question', 'plaintiff', true, 50000000.00,
 'pending', 'high', 'a9b00001-0000-0000-0000-000000000001', 'district9', 'Federal City', false, false)
ON CONFLICT (id) DO NOTHING;

-- district12 civil cases
INSERT INTO civil_cases (id, court_id, case_number, title, description, nature_of_suit, cause_of_action, jurisdiction_basis, jury_demand, class_action, amount_in_controversy, status, priority, assigned_judge_id, district_code, location, is_sealed, pro_se)
VALUES
-- ADA accessibility
('a12c0001-0000-0000-0000-000000000001', 'district12', '12:26-cv-00401', 'Disability Rights Coalition v. Metro Transit Authority',
 'Advocacy organization sues transit authority for systemic ADA violations.',
 '446', '42 U.S.C. 12132 (ADA Title II)', 'federal_question', 'none', false, NULL,
 'discovery', 'high', 'a12b0001-0000-0000-0000-000000000001', 'district12', 'Metro City', false, false),

-- Securities fraud
('a12c0002-0000-0000-0000-000000000002', 'district12', '12:26-cv-00402', 'SEC v. Pinnacle Growth Partners LLC',
 'SEC enforcement action alleging defendants operated a Ponzi scheme.',
 '491', '15 U.S.C. 78j(b) (Securities Exchange Act)', 'federal_question', 'none', false, 180000000.00,
 'pretrial', 'critical', 'a12b0001-0000-0000-0000-000000000001', 'district12', 'Metro City', false, false),

-- FOIA request
('a12c0003-0000-0000-0000-000000000003', 'district12', '12:26-cv-00403', 'Investigative Press Foundation v. Department of Homeland Security',
 'Nonprofit news organization seeks court order compelling DHS to produce records under FOIA.',
 '895', '5 U.S.C. 552 (FOIA)', 'us_government_defendant', 'none', false, NULL,
 'filed', 'medium', 'a12b0002-0000-0000-0000-000000000002', 'district12', 'Metro City', false, false),

-- Fair Labor Standards Act (collective action)
('a12c0004-0000-0000-0000-000000000004', 'district12', '12:26-cv-00404', 'Ramirez et al. v. GigWorx Delivery Inc.',
 'Delivery drivers allege misclassification as independent contractors under FLSA.',
 '710', '29 U.S.C. 216(b) (FLSA)', 'federal_question', 'plaintiff', false, 15000000.00,
 'pending', 'medium', 'a12b0002-0000-0000-0000-000000000002', 'district12', 'Metro City', false, false)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- PARTIES (civil cases use parties table, not defendants)
-- ============================================================

-- district9 parties
INSERT INTO parties (id, court_id, case_id, case_type, party_type, party_role, name, entity_type, represented, pro_se, status)
VALUES
-- Case 1: Johnson v. Apex Technologies
('a9ca0001-0000-0000-0000-000000000001', 'district9', 'a9c10001-0000-0000-0000-000000000001', 'civil', 'Plaintiff', 'Lead', 'Terri Johnson', 'Individual', true, false, 'Active'),
('a9ca0002-0000-0000-0000-000000000002', 'district9', 'a9c10001-0000-0000-0000-000000000001', 'civil', 'Defendant', 'Lead', 'Apex Technologies Inc.', 'Corporation', true, false, 'Active'),
-- Case 2: NovaTech v. DataStream
('a9ca0003-0000-0000-0000-000000000003', 'district9', 'a9c10002-0000-0000-0000-000000000002', 'civil', 'Plaintiff', 'Lead', 'NovaTech Solutions LLC', 'LLC', true, false, 'Active'),
('a9ca0004-0000-0000-0000-000000000004', 'district9', 'a9c10002-0000-0000-0000-000000000002', 'civil', 'Defendant', 'Lead', 'DataStream Corp.', 'Corporation', true, false, 'Active'),
-- Case 3: Martinez v. National Freight
('a9ca0005-0000-0000-0000-000000000005', 'district9', 'a9c10003-0000-0000-0000-000000000003', 'civil', 'Plaintiff', 'Lead', 'Rosa Martinez', 'Individual', true, false, 'Active'),
('a9ca0006-0000-0000-0000-000000000006', 'district9', 'a9c10003-0000-0000-0000-000000000003', 'civil', 'Defendant', 'Lead', 'National Freight Lines Inc.', 'Corporation', true, false, 'Active'),
-- Case 4: Williams v. QuickLoan (class action)
('a9ca0007-0000-0000-0000-000000000007', 'district9', 'a9c10004-0000-0000-0000-000000000004', 'civil', 'Plaintiff', 'Lead', 'Denise Williams', 'Individual', true, false, 'Active'),
('a9ca0008-0000-0000-0000-000000000008', 'district9', 'a9c10004-0000-0000-0000-000000000004', 'civil', 'Defendant', 'Lead', 'QuickLoan Financial Services Inc.', 'Corporation', true, false, 'Active')
ON CONFLICT (id) DO NOTHING;

-- district12 parties
INSERT INTO parties (id, court_id, case_id, case_type, party_type, party_role, name, entity_type, represented, pro_se, status)
VALUES
-- Case 5: DRC v. Metro Transit
('a12a0001-0000-0000-0000-000000000001', 'district12', 'a12c0001-0000-0000-0000-000000000001', 'civil', 'Plaintiff', 'Lead', 'Disability Rights Coalition', 'Non-Profit', true, false, 'Active'),
('a12a0002-0000-0000-0000-000000000002', 'district12', 'a12c0001-0000-0000-0000-000000000001', 'civil', 'Defendant', 'Lead', 'Metro Transit Authority', 'Government', true, false, 'Active'),
-- Case 6: SEC v. Pinnacle Growth
('a12a0003-0000-0000-0000-000000000003', 'district12', 'a12c0002-0000-0000-0000-000000000002', 'civil', 'Plaintiff', 'Lead', 'Securities and Exchange Commission', 'Government', true, false, 'Active'),
('a12a0004-0000-0000-0000-000000000004', 'district12', 'a12c0002-0000-0000-0000-000000000002', 'civil', 'Defendant', 'Lead', 'Pinnacle Growth Partners LLC', 'LLC', true, false, 'Active'),
('a12a0005-0000-0000-0000-000000000005', 'district12', 'a12c0002-0000-0000-0000-000000000002', 'civil', 'Defendant', 'Co-Defendant', 'Marcus A. Sterling', 'Individual', true, false, 'Active'),
-- Case 7: IPF v. DHS
('a12a0006-0000-0000-0000-000000000006', 'district12', 'a12c0003-0000-0000-0000-000000000003', 'civil', 'Plaintiff', 'Lead', 'Investigative Press Foundation', 'Non-Profit', true, false, 'Active'),
('a12a0007-0000-0000-0000-000000000007', 'district12', 'a12c0003-0000-0000-0000-000000000003', 'civil', 'Defendant', 'Lead', 'Department of Homeland Security', 'Government', true, false, 'Active'),
-- Case 8: Ramirez v. GigWorx
('a12a0008-0000-0000-0000-000000000008', 'district12', 'a12c0004-0000-0000-0000-000000000004', 'civil', 'Plaintiff', 'Lead', 'Carlos Ramirez', 'Individual', true, false, 'Active'),
('a12a0009-0000-0000-0000-000000000009', 'district12', 'a12c0004-0000-0000-0000-000000000004', 'civil', 'Plaintiff', 'Co-Plaintiff', 'Maria Santos', 'Individual', true, false, 'Active'),
('a12a000a-0000-0000-0000-000000000001', 'district12', 'a12c0004-0000-0000-0000-000000000004', 'civil', 'Defendant', 'Lead', 'GigWorx Delivery Inc.', 'Corporation', true, false, 'Active')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- DOCKET ENTRIES for civil cases (representative sample)
-- ============================================================

-- Case 1: Johnson v. Apex Technologies (discovery)
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('a9de0001-0000-0000-0000-000000000001', 'district9', 'a9c10001-0000-0000-0000-000000000001', 'civil', 1, 'complaint', 'COMPLAINT filed by Terri Johnson against Apex Technologies Inc. for employment discrimination under Title VII.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '60 days'),
('a9de0002-0000-0000-0000-000000000002', 'district9', 'a9c10001-0000-0000-0000-000000000001', 'civil', 2, 'summons', 'SUMMONS issued as to Apex Technologies Inc.', 'Clerk', false, false, NOW() - INTERVAL '60 days'),
('a9de0003-0000-0000-0000-000000000003', 'district9', 'a9c10001-0000-0000-0000-000000000001', 'civil', 3, 'answer', 'ANSWER to Complaint filed by Apex Technologies Inc.', 'Defense counsel', false, false, NOW() - INTERVAL '38 days'),
('a9de0004-0000-0000-0000-000000000004', 'district9', 'a9c10001-0000-0000-0000-000000000001', 'civil', 4, 'scheduling_order', 'SCHEDULING ORDER: Discovery closes 07/15/2026. Trial set for 11/02/2026.', 'Court', false, false, NOW() - INTERVAL '30 days')
ON CONFLICT (id) DO NOTHING;

-- Case 5: DRC v. Metro Transit (discovery)
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('a12e0001-0000-0000-0000-000000000001', 'district12', 'a12c0001-0000-0000-0000-000000000001', 'civil', 1, 'complaint', 'COMPLAINT filed by Disability Rights Coalition against Metro Transit Authority for ADA violations.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '75 days'),
('a12e0002-0000-0000-0000-000000000002', 'district12', 'a12c0001-0000-0000-0000-000000000001', 'civil', 2, 'answer', 'ANSWER filed by Metro Transit Authority.', 'Defense counsel', false, false, NOW() - INTERVAL '53 days'),
('a12e0003-0000-0000-0000-000000000003', 'district12', 'a12c0001-0000-0000-0000-000000000001', 'civil', 3, 'scheduling_order', 'SCHEDULING ORDER: Fact discovery closes 06/30/2026.', 'Court', false, false, NOW() - INTERVAL '45 days')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- CLERK QUEUE ITEMS for civil cases
-- ============================================================

INSERT INTO clerk_queue (id, court_id, queue_type, priority, status, title, description, source_type, source_id, case_id, case_type, case_number, current_step)
VALUES
('a9ce0001-0000-0000-0000-000000000001', 'district9', 'filing', 3, 'pending', 'New Civil Complaint — Martinez v. National Freight Lines',
 'Process initial complaint filing. Verify filing fee payment and issue summons.',
 'filing', 'a9de0001-0000-0000-0000-000000000001', 'a9c10003-0000-0000-0000-000000000003', 'civil', '9:26-cv-00303', 'review'),

('a12e0004-0000-0000-0000-000000000004', 'district12', 'filing', 3, 'pending', 'New FOIA Complaint — IPF v. DHS',
 'Process new FOIA complaint. Ensure proper service per FRCP 4(i).',
 'filing', 'a12e0001-0000-0000-0000-000000000001', 'a12c0003-0000-0000-0000-000000000003', 'civil', '12:26-cv-00403', 'review')
ON CONFLICT (id) DO NOTHING;

END $$;
