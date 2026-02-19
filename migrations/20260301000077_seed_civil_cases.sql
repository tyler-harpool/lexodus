-- Realistic CM/ECF civil case seed data for district9 and district12
-- 8 civil cases with parties, docket entries, and clerk queue items
-- All IDs use deterministic prefixes for easy identification and cleanup

DO $$ BEGIN

-- Guard: skip if data already seeded
IF EXISTS (SELECT 1 FROM civil_cases WHERE id = 'd9cv0001-0000-0000-0000-000000000001') THEN
    RAISE NOTICE 'Civil case seed data already exists, skipping';
    RETURN;
END IF;

-- ============================================================
-- CIVIL CASES (8 total: 4 per district)
-- ============================================================

-- district9 civil cases
INSERT INTO civil_cases (id, court_id, case_number, title, description, nature_of_suit, cause_of_action, jurisdiction_basis, jury_demand, class_action, amount_in_controversy, status, priority, assigned_judge_id, district_code, location, is_sealed, pro_se)
VALUES
-- Employment discrimination (Title VII)
('d9cv0001-0000-0000-0000-000000000001', 'district9', '9:26-cv-00301', 'Johnson v. Apex Technologies Inc.',
 'Former employee alleges termination based on race and gender in violation of Title VII. Plaintiff was a senior software engineer for 8 years with exemplary performance reviews before being terminated during a reorganization that disproportionately affected minority employees.',
 '442', '42 U.S.C. 2000e (Title VII)', 'federal_question', 'plaintiff', false, 750000.00,
 'discovery', 'medium', 'd9b00001-0000-0000-0000-000000000001', 'district9', 'Federal City', false, false),

-- Patent infringement
('d9cv0002-0000-0000-0000-000000000002', 'district9', '9:26-cv-00302', 'NovaTech Solutions LLC v. DataStream Corp.',
 'Patent holder alleges infringement of three utility patents (US 11,234,567; US 11,345,678; US 11,456,789) relating to real-time data compression algorithms used in cloud storage systems.',
 '830', '35 U.S.C. 271 (Patent Infringement)', 'federal_question', 'both', false, 25000000.00,
 'pretrial', 'high', 'd9b00001-0000-0000-0000-000000000001', 'district9', 'Federal City', false, false),

-- Diversity personal injury (motor vehicle)
('d9cv0003-0000-0000-0000-000000000003', 'district9', '9:26-cv-00303', 'Martinez v. National Freight Lines Inc.',
 'Plaintiff sustained severe injuries when defendant''s commercial tractor-trailer crossed the center line on Interstate 95, causing a head-on collision. Plaintiff seeks compensatory and punitive damages for medical expenses, lost wages, and pain and suffering.',
 '350', '28 U.S.C. 1332 (Diversity)', 'diversity', 'plaintiff', false, 2500000.00,
 'filed', 'medium', 'd9b00003-0000-0000-0000-000000000003', 'district9', 'Federal City', false, false),

-- Class action consumer protection (TCPA)
('d9cv0004-0000-0000-0000-000000000004', 'district9', '9:26-cv-00304', 'Williams v. QuickLoan Financial Services Inc.',
 'Putative class action alleging defendant made thousands of unsolicited automated telephone calls and text messages to consumers'' cell phones without prior express consent, in violation of the Telephone Consumer Protection Act.',
 '485', '47 U.S.C. 227 (TCPA)', 'federal_question', 'plaintiff', true, 50000000.00,
 'pending', 'high', 'd9b00001-0000-0000-0000-000000000001', 'district9', 'Federal City', false, false)
ON CONFLICT (id) DO NOTHING;

-- district12 civil cases
INSERT INTO civil_cases (id, court_id, case_number, title, description, nature_of_suit, cause_of_action, jurisdiction_basis, jury_demand, class_action, amount_in_controversy, status, priority, assigned_judge_id, district_code, location, is_sealed, pro_se)
VALUES
-- ADA accessibility
('d12v0001-0000-0000-0000-000000000001', 'district12', '12:26-cv-00401', 'Disability Rights Coalition v. Metro Transit Authority',
 'Advocacy organization sues transit authority for systemic ADA violations including inaccessible bus stops, non-functioning wheelchair lifts, and failure to provide reasonable accommodations for visually impaired passengers across the metropolitan transit system.',
 '446', '42 U.S.C. 12132 (ADA Title II)', 'federal_question', 'none', false, NULL,
 'discovery', 'high', 'd12b0001-0000-0000-0000-000000000001', 'district12', 'Metro City', false, false),

-- Securities fraud
('d12v0002-0000-0000-0000-000000000002', 'district12', '12:26-cv-00402', 'SEC v. Pinnacle Growth Partners LLC',
 'SEC enforcement action alleging defendants operated a Ponzi scheme disguised as a hedge fund, misappropriating over $180 million from approximately 3,400 investors through material misrepresentations about investment returns and fund performance.',
 '491', '15 U.S.C. 78j(b) (Securities Exchange Act)', 'federal_question', 'none', false, 180000000.00,
 'pretrial', 'critical', 'd12b0001-0000-0000-0000-000000000001', 'district12', 'Metro City', false, false),

-- FOIA request
('d12v0003-0000-0000-0000-000000000003', 'district12', '12:26-cv-00403', 'Investigative Press Foundation v. Department of Homeland Security',
 'Nonprofit news organization seeks court order compelling DHS to produce records related to surveillance technology procurement and deployment at the southern border, after agency failed to respond to FOIA request within statutory deadline.',
 '895', '5 U.S.C. 552 (FOIA)', 'us_government_defendant', 'none', false, NULL,
 'filed', 'medium', 'd12b0002-0000-0000-0000-000000000002', 'district12', 'Metro City', false, false),

-- Fair Labor Standards Act (collective action)
('d12v0004-0000-0000-0000-000000000004', 'district12', '12:26-cv-00404', 'Ramirez et al. v. GigWorx Delivery Inc.',
 'Delivery drivers allege misclassification as independent contractors rather than employees, seeking unpaid overtime wages, minimum wage violations, and unreimbursed expenses under FLSA and state labor law. Conditional certification of collective action sought.',
 '710', '29 U.S.C. 216(b) (FLSA)', 'federal_question', 'plaintiff', false, 15000000.00,
 'pending', 'medium', 'd12b0002-0000-0000-0000-000000000002', 'district12', 'Metro City', false, false)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- PARTIES (civil cases use parties table, not defendants)
-- ============================================================

-- district9 parties
INSERT INTO parties (id, court_id, case_id, case_type, party_type, party_role, name, entity_type, first_name, last_name, represented, pro_se, status)
VALUES
-- Case 1: Johnson v. Apex Technologies
('d9pv0001-0000-0000-0000-000000000001', 'district9', 'd9cv0001-0000-0000-0000-000000000001', 'civil', 'Plaintiff', 'Lead', 'Terri Johnson', 'Individual', 'Terri', 'Johnson', true, false, 'Active'),
('d9pv0002-0000-0000-0000-000000000002', 'district9', 'd9cv0001-0000-0000-0000-000000000001', 'civil', 'Defendant', 'Lead', 'Apex Technologies Inc.', 'Corporation', NULL, NULL, true, false, 'Active'),

-- Case 2: NovaTech v. DataStream
('d9pv0003-0000-0000-0000-000000000003', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 'Plaintiff', 'Lead', 'NovaTech Solutions LLC', 'LLC', NULL, NULL, true, false, 'Active'),
('d9pv0004-0000-0000-0000-000000000004', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 'Defendant', 'Lead', 'DataStream Corp.', 'Corporation', NULL, NULL, true, false, 'Active'),

-- Case 3: Martinez v. National Freight
('d9pv0005-0000-0000-0000-000000000005', 'district9', 'd9cv0003-0000-0000-0000-000000000003', 'civil', 'Plaintiff', 'Lead', 'Rosa Martinez', 'Individual', 'Rosa', 'Martinez', true, false, 'Active'),
('d9pv0006-0000-0000-0000-000000000006', 'district9', 'd9cv0003-0000-0000-0000-000000000003', 'civil', 'Defendant', 'Lead', 'National Freight Lines Inc.', 'Corporation', NULL, NULL, true, false, 'Active'),

-- Case 4: Williams v. QuickLoan (class action)
('d9pv0007-0000-0000-0000-000000000007', 'district9', 'd9cv0004-0000-0000-0000-000000000004', 'civil', 'Plaintiff', 'Lead', 'Denise Williams', 'Individual', 'Denise', 'Williams', true, false, 'Active'),
('d9pv0008-0000-0000-0000-000000000008', 'district9', 'd9cv0004-0000-0000-0000-000000000004', 'civil', 'Defendant', 'Lead', 'QuickLoan Financial Services Inc.', 'Corporation', NULL, NULL, true, false, 'Active')
ON CONFLICT (id) DO NOTHING;

-- district12 parties
INSERT INTO parties (id, court_id, case_id, case_type, party_type, party_role, name, entity_type, first_name, last_name, organization_name, represented, pro_se, status)
VALUES
-- Case 5: DRC v. Metro Transit
('d12pv001-0000-0000-0000-000000000001', 'district12', 'd12v0001-0000-0000-0000-000000000001', 'civil', 'Plaintiff', 'Lead', 'Disability Rights Coalition', 'Non-Profit', NULL, NULL, 'Disability Rights Coalition', true, false, 'Active'),
('d12pv002-0000-0000-0000-000000000002', 'district12', 'd12v0001-0000-0000-0000-000000000001', 'civil', 'Defendant', 'Lead', 'Metro Transit Authority', 'Government', NULL, NULL, 'Metro Transit Authority', true, false, 'Active'),

-- Case 6: SEC v. Pinnacle Growth
('d12pv003-0000-0000-0000-000000000003', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 'Plaintiff', 'Lead', 'Securities and Exchange Commission', 'Government', NULL, NULL, 'SEC', true, false, 'Active'),
('d12pv004-0000-0000-0000-000000000004', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 'Defendant', 'Lead', 'Pinnacle Growth Partners LLC', 'LLC', NULL, NULL, 'Pinnacle Growth Partners LLC', true, false, 'Active'),
('d12pv005-0000-0000-0000-000000000005', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 'Defendant', 'Co-Defendant', 'Marcus A. Sterling', 'Individual', 'Marcus', 'Sterling', NULL, true, false, 'Active'),

-- Case 7: IPF v. DHS
('d12pv006-0000-0000-0000-000000000006', 'district12', 'd12v0003-0000-0000-0000-000000000003', 'civil', 'Plaintiff', 'Lead', 'Investigative Press Foundation', 'Non-Profit', NULL, NULL, 'Investigative Press Foundation', true, false, 'Active'),
('d12pv007-0000-0000-0000-000000000007', 'district12', 'd12v0003-0000-0000-0000-000000000003', 'civil', 'Defendant', 'Lead', 'Department of Homeland Security', 'Government', NULL, NULL, 'DHS', true, false, 'Active'),

-- Case 8: Ramirez v. GigWorx
('d12pv008-0000-0000-0000-000000000008', 'district12', 'd12v0004-0000-0000-0000-000000000004', 'civil', 'Plaintiff', 'Lead', 'Carlos Ramirez', 'Individual', 'Carlos', 'Ramirez', NULL, true, false, 'Active'),
('d12pv009-0000-0000-0000-000000000009', 'district12', 'd12v0004-0000-0000-0000-000000000004', 'civil', 'Plaintiff', 'Co-Plaintiff', 'Maria Santos', 'Individual', 'Maria', 'Santos', NULL, true, false, 'Active'),
('d12pv00a-0000-0000-0000-000000000001', 'district12', 'd12v0004-0000-0000-0000-000000000004', 'civil', 'Defendant', 'Lead', 'GigWorx Delivery Inc.', 'Corporation', NULL, NULL, 'GigWorx Delivery Inc.', true, false, 'Active')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- DOCKET ENTRIES for civil cases
-- ============================================================

-- Case 1: Johnson v. Apex Technologies (discovery) — 6 entries
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9dv0001-0000-0000-0000-000000000001', 'district9', 'd9cv0001-0000-0000-0000-000000000001', 'civil', 1, 'complaint', 'COMPLAINT filed by Terri Johnson against Apex Technologies Inc. for employment discrimination under Title VII. (Filing fee $402.00 paid.) Jury trial demanded.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '60 days'),
('d9dv0002-0000-0000-0000-000000000002', 'district9', 'd9cv0001-0000-0000-0000-000000000001', 'civil', 2, 'summons', 'SUMMONS issued as to Apex Technologies Inc. Service due within 90 days.', 'Clerk', false, false, NOW() - INTERVAL '60 days'),
('d9dv0003-0000-0000-0000-000000000003', 'district9', 'd9cv0001-0000-0000-0000-000000000001', 'civil', 3, 'service_return', 'RETURN OF SERVICE executed on Apex Technologies Inc. via registered agent on 01/20/2026.', 'Process Server', false, false, NOW() - INTERVAL '52 days'),
('d9dv0004-0000-0000-0000-000000000004', 'district9', 'd9cv0001-0000-0000-0000-000000000001', 'civil', 4, 'answer', 'ANSWER to Complaint filed by Apex Technologies Inc. Affirmative defenses raised including failure to exhaust administrative remedies.', 'Defense counsel', false, false, NOW() - INTERVAL '38 days'),
('d9dv0005-0000-0000-0000-000000000005', 'district9', 'd9cv0001-0000-0000-0000-000000000001', 'civil', 5, 'scheduling_order', 'SCHEDULING ORDER: Initial disclosures due 03/01/2026. Discovery closes 07/15/2026. Dispositive motions due 08/15/2026. Trial set for 11/02/2026. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '30 days'),
('d9dv0006-0000-0000-0000-000000000006', 'district9', 'd9cv0001-0000-0000-0000-000000000001', 'civil', 6, 'discovery_request', 'PLAINTIFF''S FIRST SET OF INTERROGATORIES AND REQUESTS FOR PRODUCTION served on defendant (25 interrogatories, 30 document requests).', 'Plaintiff counsel', false, false, NOW() - INTERVAL '20 days')
ON CONFLICT (id) DO NOTHING;

-- Case 2: NovaTech v. DataStream (pretrial) — 8 entries
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9dv0007-0000-0000-0000-000000000007', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 1, 'complaint', 'COMPLAINT for Patent Infringement filed by NovaTech Solutions LLC against DataStream Corp. Three patents at issue: US 11,234,567; US 11,345,678; US 11,456,789.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '120 days'),
('d9dv0008-0000-0000-0000-000000000008', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 2, 'answer', 'ANSWER and COUNTERCLAIM for Declaratory Judgment of Non-Infringement and Invalidity filed by DataStream Corp.', 'Defense counsel', false, false, NOW() - INTERVAL '99 days'),
('d9dv0009-0000-0000-0000-000000000009', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 3, 'scheduling_order', 'SCHEDULING ORDER: Claim construction briefing due 03/01/2026. Markman hearing set 04/15/2026. Fact discovery closes 08/01/2026. Expert reports due 09/15/2026.', 'Court', false, false, NOW() - INTERVAL '90 days'),
('d9dv000a-0000-0000-0000-000000000001', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 4, 'notice', 'JOINT CLAIM CONSTRUCTION STATEMENT identifying disputed claim terms for US Patents 11,234,567 and 11,345,678.', 'Joint filing', false, false, NOW() - INTERVAL '60 days'),
('d9dv000b-0000-0000-0000-000000000002', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 5, 'motion', 'PLAINTIFF''S OPENING CLAIM CONSTRUCTION BRIEF with proposed constructions for 12 disputed claim terms.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '45 days'),
('d9dv000c-0000-0000-0000-000000000003', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 6, 'response', 'DEFENDANT''S RESPONSIVE CLAIM CONSTRUCTION BRIEF opposing plaintiff''s proposed constructions.', 'Defense counsel', false, false, NOW() - INTERVAL '30 days'),
('d9dv000d-0000-0000-0000-000000000004', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 7, 'expert_report', 'DECLARATION OF DR. JAMES WHITAKER in support of plaintiff''s claim construction. (Expert in data compression algorithms)', 'Plaintiff counsel', false, false, NOW() - INTERVAL '28 days'),
('d9dv000e-0000-0000-0000-000000000005', 'district9', 'd9cv0002-0000-0000-0000-000000000002', 'civil', 8, 'hearing_notice', 'NOTICE of Markman hearing set for 04/15/2026 at 10:00 AM before Hon. Ronnie Abrams, Courtroom 1A.', 'Court', false, false, NOW() - INTERVAL '14 days')
ON CONFLICT (id) DO NOTHING;

-- Case 3: Martinez v. National Freight (filed) — 3 entries
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9dv000f-0000-0000-0000-000000000006', 'district9', 'd9cv0003-0000-0000-0000-000000000003', 'civil', 1, 'complaint', 'COMPLAINT filed by Rosa Martinez against National Freight Lines Inc. for negligence resulting in personal injury. Diversity jurisdiction; amount in controversy exceeds $75,000. Jury trial demanded.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '7 days'),
('d9dv0010-0000-0000-0000-000000000007', 'district9', 'd9cv0003-0000-0000-0000-000000000003', 'civil', 2, 'summons', 'SUMMONS issued as to National Freight Lines Inc.', 'Clerk', false, false, NOW() - INTERVAL '7 days'),
('d9dv0011-0000-0000-0000-000000000008', 'district9', 'd9cv0003-0000-0000-0000-000000000003', 'civil', 3, 'notice', 'NOTICE of related case. Plaintiff identifies two pending personal injury actions against same defendant in other districts.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '5 days')
ON CONFLICT (id) DO NOTHING;

-- Case 4: Williams v. QuickLoan (pending) — 4 entries
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9dv0012-0000-0000-0000-000000000009', 'district9', 'd9cv0004-0000-0000-0000-000000000004', 'civil', 1, 'complaint', 'CLASS ACTION COMPLAINT filed by Denise Williams on behalf of all persons similarly situated against QuickLoan Financial Services Inc. for violations of the Telephone Consumer Protection Act, 47 U.S.C. 227.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '21 days'),
('d9dv0013-0000-0000-0000-000000000001', 'district9', 'd9cv0004-0000-0000-0000-000000000004', 'civil', 2, 'summons', 'SUMMONS issued as to QuickLoan Financial Services Inc.', 'Clerk', false, false, NOW() - INTERVAL '21 days'),
('d9dv0014-0000-0000-0000-000000000002', 'district9', 'd9cv0004-0000-0000-0000-000000000004', 'civil', 3, 'notice', 'CORPORATE DISCLOSURE STATEMENT filed by defendant QuickLoan Financial Services Inc. pursuant to Fed. R. Civ. P. 7.1.', 'Defense counsel', false, false, NOW() - INTERVAL '10 days'),
('d9dv0015-0000-0000-0000-000000000003', 'district9', 'd9cv0004-0000-0000-0000-000000000004', 'civil', 4, 'motion', 'MOTION to Dismiss for Failure to State a Claim pursuant to Fed. R. Civ. P. 12(b)(6) filed by defendant QuickLoan Financial Services Inc.', 'Defense counsel', false, false, NOW() - INTERVAL '8 days')
ON CONFLICT (id) DO NOTHING;

-- Case 5: DRC v. Metro Transit (discovery) — 5 entries
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12dv001-0000-0000-0000-000000000001', 'district12', 'd12v0001-0000-0000-0000-000000000001', 'civil', 1, 'complaint', 'COMPLAINT filed by Disability Rights Coalition against Metro Transit Authority for violations of the Americans with Disabilities Act, Title II. Injunctive relief and compensatory damages sought.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '75 days'),
('d12dv002-0000-0000-0000-000000000002', 'district12', 'd12v0001-0000-0000-0000-000000000001', 'civil', 2, 'answer', 'ANSWER filed by Metro Transit Authority. Defendant denies material allegations and asserts undue burden defense.', 'Defense counsel', false, false, NOW() - INTERVAL '53 days'),
('d12dv003-0000-0000-0000-000000000003', 'district12', 'd12v0001-0000-0000-0000-000000000001', 'civil', 3, 'scheduling_order', 'SCHEDULING ORDER: Initial disclosures complete. Fact discovery closes 06/30/2026. Expert reports due 08/01/2026. Mediation to be completed by 09/15/2026.', 'Court', false, false, NOW() - INTERVAL '45 days'),
('d12dv004-0000-0000-0000-000000000004', 'district12', 'd12v0001-0000-0000-0000-000000000001', 'civil', 4, 'discovery_request', 'PLAINTIFF''S FIRST SET OF REQUESTS FOR PRODUCTION: route accessibility data, maintenance logs, complaint records, and ADA compliance audit reports for all 147 bus routes.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '30 days'),
('d12dv005-0000-0000-0000-000000000005', 'district12', 'd12v0001-0000-0000-0000-000000000001', 'civil', 5, 'motion', 'MOTION to Compel Discovery Responses filed by plaintiff. Defendant failed to respond to 18 of 25 document requests within 30-day deadline.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '10 days')
ON CONFLICT (id) DO NOTHING;

-- Case 6: SEC v. Pinnacle Growth (pretrial) — 7 entries
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12dv006-0000-0000-0000-000000000006', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 1, 'complaint', 'COMPLAINT filed by Securities and Exchange Commission against Pinnacle Growth Partners LLC and Marcus A. Sterling for securities fraud under Section 10(b) and Rule 10b-5. Emergency TRO and asset freeze sought.', 'SEC Enforcement', false, false, NOW() - INTERVAL '90 days'),
('d12dv007-0000-0000-0000-000000000007', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 2, 'order', 'TEMPORARY RESTRAINING ORDER freezing assets of defendants and appointing receiver over Pinnacle Growth Partners LLC. Signed by Hon. Amir H. Ali.', 'Court', false, false, NOW() - INTERVAL '89 days'),
('d12dv008-0000-0000-0000-000000000008', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 3, 'answer', 'ANSWER filed by Marcus A. Sterling denying all allegations. Defendant asserts good faith reliance on professional advisors.', 'Defense counsel', false, false, NOW() - INTERVAL '68 days'),
('d12dv009-0000-0000-0000-000000000009', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 4, 'motion', 'MOTION for Summary Judgment filed by SEC. Undisputed evidence shows defendant commingled investor funds with personal accounts.', 'SEC Enforcement', false, false, NOW() - INTERVAL '35 days'),
('d12dv00a-0000-0000-0000-000000000001', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 5, 'response', 'RESPONSE in Opposition to Motion for Summary Judgment filed by defendant Sterling. Argues genuine disputes of material fact exist.', 'Defense counsel', false, false, NOW() - INTERVAL '21 days'),
('d12dv00b-0000-0000-0000-000000000002', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 6, 'reply', 'REPLY in Support of Motion for Summary Judgment filed by SEC.', 'SEC Enforcement', false, false, NOW() - INTERVAL '14 days'),
('d12dv00c-0000-0000-0000-000000000003', 'district12', 'd12v0002-0000-0000-0000-000000000002', 'civil', 7, 'hearing_notice', 'NOTICE of oral argument on Motion for Summary Judgment set for 03/20/2026 at 2:00 PM before Hon. Amir H. Ali.', 'Court', false, false, NOW() - INTERVAL '7 days')
ON CONFLICT (id) DO NOTHING;

-- Case 7: IPF v. DHS (filed) — 2 entries
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12dv00d-0000-0000-0000-000000000004', 'district12', 'd12v0003-0000-0000-0000-000000000003', 'civil', 1, 'complaint', 'COMPLAINT filed by Investigative Press Foundation against Department of Homeland Security for wrongful withholding of agency records under the Freedom of Information Act, 5 U.S.C. 552.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '5 days'),
('d12dv00e-0000-0000-0000-000000000005', 'district12', 'd12v0003-0000-0000-0000-000000000003', 'civil', 2, 'summons', 'SUMMONS issued to Department of Homeland Security. Service to be made on U.S. Attorney and Attorney General pursuant to Fed. R. Civ. P. 4(i).', 'Clerk', false, false, NOW() - INTERVAL '5 days')
ON CONFLICT (id) DO NOTHING;

-- Case 8: Ramirez v. GigWorx (pending) — 3 entries
INSERT INTO docket_entries (id, court_id, case_id, case_type, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12dv00f-0000-0000-0000-000000000006', 'district12', 'd12v0004-0000-0000-0000-000000000004', 'civil', 1, 'complaint', 'COLLECTIVE ACTION COMPLAINT filed by Carlos Ramirez and Maria Santos on behalf of all similarly situated delivery drivers against GigWorx Delivery Inc. for FLSA violations including unpaid overtime and minimum wage.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '14 days'),
('d12dv010-0000-0000-0000-000000000007', 'district12', 'd12v0004-0000-0000-0000-000000000004', 'civil', 2, 'summons', 'SUMMONS issued as to GigWorx Delivery Inc.', 'Clerk', false, false, NOW() - INTERVAL '14 days'),
('d12dv011-0000-0000-0000-000000000008', 'district12', 'd12v0004-0000-0000-0000-000000000004', 'civil', 3, 'motion', 'MOTION for Conditional Certification of FLSA Collective Action filed by plaintiffs. Declarations of 47 current and former delivery drivers attached.', 'Plaintiff counsel', false, false, NOW() - INTERVAL '7 days')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- CLERK QUEUE ITEMS for civil cases
-- ============================================================

INSERT INTO clerk_queue (id, court_id, queue_type, priority, status, title, description, source_type, source_id, case_id, case_type, case_number, current_step)
VALUES
-- New filing: Martinez complaint needs processing
('d9qv0001-0000-0000-0000-000000000001', 'district9', 'filing', 3, 'pending', 'New Civil Complaint — Martinez v. National Freight Lines',
 'Process initial complaint filing. Verify filing fee payment, assign case number, and issue summons.',
 'filing', 'd9dv000f-0000-0000-0000-000000000006', 'd9cv0003-0000-0000-0000-000000000003', 'civil', '9:26-cv-00303', 'review'),

-- Motion pending: QuickLoan MTD needs routing to judge
('d9qv0002-0000-0000-0000-000000000002', 'district9', 'motion', 2, 'pending', 'Motion to Dismiss — Williams v. QuickLoan',
 'Route defendant''s 12(b)(6) motion to dismiss to assigned judge for consideration. Set briefing schedule.',
 'motion', 'd9dv0015-0000-0000-0000-000000000003', 'd9cv0004-0000-0000-0000-000000000004', 'civil', '9:26-cv-00304', 'route_judge'),

-- Motion to compel needs review
('d12qv001-0000-0000-0000-000000000001', 'district12', 'motion', 2, 'pending', 'Motion to Compel — DRC v. Metro Transit',
 'Review plaintiff''s motion to compel discovery responses. Route to judge for ruling.',
 'motion', 'd12dv005-0000-0000-0000-000000000005', 'd12v0001-0000-0000-0000-000000000001', 'civil', '12:26-cv-00401', 'route_judge'),

-- Summary judgment hearing prep
('d12qv002-0000-0000-0000-000000000002', 'district12', 'motion', 1, 'in_review', 'MSJ Hearing Prep — SEC v. Pinnacle Growth',
 'Prepare courtroom and docket for oral argument on SEC''s motion for summary judgment scheduled 03/20/2026.',
 'motion', 'd12dv00c-0000-0000-0000-000000000003', 'd12v0002-0000-0000-0000-000000000002', 'civil', '12:26-cv-00402', 'docket'),

-- FOIA complaint needs processing
('d12qv003-0000-0000-0000-000000000003', 'district12', 'filing', 3, 'pending', 'New FOIA Complaint — IPF v. DHS',
 'Process new FOIA complaint. Ensure proper service on U.S. Attorney and Attorney General per FRCP 4(i).',
 'filing', 'd12dv00d-0000-0000-0000-000000000004', 'd12v0003-0000-0000-0000-000000000003', 'civil', '12:26-cv-00403', 'review'),

-- Collective action certification motion
('d12qv004-0000-0000-0000-000000000004', 'district12', 'motion', 2, 'pending', 'Conditional Certification Motion — Ramirez v. GigWorx',
 'Route motion for conditional certification of FLSA collective action to assigned judge. Set response deadline.',
 'motion', 'd12dv00f-0000-0000-0000-000000000006', 'd12v0004-0000-0000-0000-000000000004', 'civil', '12:26-cv-00404', 'route_judge')
ON CONFLICT (id) DO NOTHING;

END $$;
