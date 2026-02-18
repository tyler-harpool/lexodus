-- Realistic CM/ECF seed data for district9 and district12
-- Judges, attorneys, cases, docket entries, calendar events, and more
-- All IDs use deterministic prefixes for easy identification and cleanup

DO $$ BEGIN

-- Guard: skip if data already seeded
IF EXISTS (SELECT 1 FROM criminal_cases WHERE id = 'd9c00001-0000-0000-0000-000000000001') THEN
    RAISE NOTICE 'Realistic seed data already exists, skipping';
    RETURN;
END IF;

-- ============================================================
-- JUDGES (8 total: 4 per district)
-- ============================================================

INSERT INTO judges (id, court_id, name, title, district, status, courtroom, current_caseload, max_caseload, specializations)
VALUES
('d9b00001-0000-0000-0000-000000000001', 'district9', 'Hon. Ronnie Abrams', 'Chief Judge', 'district9', 'Active', 'Courtroom 1A', 142, 150, '{white-collar,fraud,RICO}'),
('d9b00002-0000-0000-0000-000000000002', 'district9', 'Hon. Lance M. Africk', 'Senior Judge', 'district9', 'Senior', 'Courtroom 3B', 45, 75, '{narcotics,firearms}'),
('d9b00003-0000-0000-0000-000000000003', 'district9', 'Hon. Gray M. Borden', 'Magistrate Judge', 'district9', 'Active', 'Courtroom 5C', 88, 200, '{initial-appearances,discovery-disputes}'),
('d9b00004-0000-0000-0000-000000000004', 'district9', 'Hon. Nancy G. Abudu', 'Visiting Judge', 'district9', 'Active', NULL, 12, 50, '{appellate,constitutional-law}')
ON CONFLICT (id) DO NOTHING;

INSERT INTO judges (id, court_id, name, title, district, status, courtroom, current_caseload, max_caseload, specializations)
VALUES
('d12b0001-0000-0000-0000-000000000001', 'district12', 'Hon. Amir H. Ali', 'Judge', 'district12', 'Active', 'Courtroom 2A', 118, 150, '{cybercrime,national-security}'),
('d12b0002-0000-0000-0000-000000000002', 'district12', 'Hon. Georgia N. Alexakis', 'Judge', 'district12', 'Active', 'Courtroom 4B', 95, 150, '{immigration,drug-offenses}'),
('d12b0003-0000-0000-0000-000000000003', 'district12', 'Hon. Sonja F. Bivins', 'Magistrate Judge', 'district12', 'Active', 'Courtroom 6A', 110, 200, '{pretrial,bail-hearings}'),
('d12b0004-0000-0000-0000-000000000004', 'district12', 'Hon. Seth R. Aframe', 'Visiting Judge', 'district12', 'Active', NULL, 8, 50, '{appellate,sentencing-review}')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- ATTORNEYS (8 total: 4 per district)
-- ============================================================

INSERT INTO attorneys (id, court_id, bar_number, first_name, middle_name, last_name, firm_name, email, phone, status, cja_panel_member, address_street1, address_city, address_state, address_zip)
VALUES
('d9a00001-0000-0000-0000-000000000001', 'district9', 'DC-2019-04521', 'Sarah', 'K.', 'Mitchell', 'U.S. Attorney''s Office', 'sarah.mitchell@usdoj.gov', '202-555-0101', 'Active', false, '1 Courthouse Way', 'Federal City', 'DC', '20001'),
('d9a00002-0000-0000-0000-000000000002', 'district9', 'TX-2015-08832', 'Marcus', 'J.', 'Rivera', 'Federal Public Defender''s Office', 'marcus.rivera@fd.org', '202-555-0102', 'Active', true, '200 Defense Plaza', 'Federal City', 'DC', '20002'),
('d9a00003-0000-0000-0000-000000000003', 'district9', 'NY-2012-15567', 'Catherine', 'L.', 'Whitfield', 'Whitfield & Associates LLP', 'cwhitfield@whitfieldlaw.com', '212-555-0103', 'Active', false, '500 Park Avenue', 'New York', 'NY', '10022'),
('d9a00004-0000-0000-0000-000000000004', 'district9', 'CA-2018-22104', 'David', 'R.', 'Okonkwo', 'Law Office of David Okonkwo', 'dokonkwo@okonkwolaw.com', '415-555-0104', 'Active', true, '750 Market Street', 'San Francisco', 'CA', '94103')
ON CONFLICT (id) DO NOTHING;

INSERT INTO attorneys (id, court_id, bar_number, first_name, middle_name, last_name, firm_name, email, phone, status, cja_panel_member, address_street1, address_city, address_state, address_zip)
VALUES
('d12a0001-0000-0000-0000-000000000001', 'district12', 'IL-2017-11893', 'Jennifer', 'M.', 'Huang', 'U.S. Attorney''s Office', 'jennifer.huang@usdoj.gov', '312-555-0201', 'Active', false, '1 Federal Plaza', 'Metro City', 'IL', '60601'),
('d12a0002-0000-0000-0000-000000000002', 'district12', 'GA-2014-07261', 'Robert', 'A.', 'Blackwell', 'Federal Public Defender''s Office', 'robert.blackwell@fd.org', '404-555-0202', 'Active', true, '300 Defender Lane', 'Metro City', 'IL', '60602'),
('d12a0003-0000-0000-0000-000000000003', 'district12', 'FL-2016-19440', 'Elena', 'V.', 'Petrossian', 'Petrossian Legal Services', 'elena@petrossianlegal.com', '305-555-0203', 'Active', false, '1200 Brickell Ave', 'Miami', 'FL', '33131'),
('d12a0004-0000-0000-0000-000000000004', 'district12', 'WA-2013-06178', 'Thomas', 'W.', 'Nakamura', 'Nakamura Law Group', 'tnakamura@nakamuralaw.com', '206-555-0204', 'Active', false, '800 Fifth Avenue', 'Seattle', 'WA', '98104')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- CRIMINAL CASES (15 total: 8 district9, 7 district12)
-- ============================================================

-- district9 cases
INSERT INTO criminal_cases (id, court_id, case_number, title, crime_type, status, priority, district_code, assigned_judge_id, description, is_sealed, location)
VALUES
('d9c00001-0000-0000-0000-000000000001', 'district9', '9:26-cr-00101', 'United States v. Rodriguez', 'drug_offense', 'filed', 'medium', 'district9', 'd9b00003-0000-0000-0000-000000000003', 'Distribution of fentanyl-laced counterfeit oxycodone pills resulting in multiple overdoses across three counties.', false, 'Federal City'),
('d9c00002-0000-0000-0000-000000000002', 'district9', '9:26-cr-00102', 'United States v. Chen', 'cybercrime', 'arraigned', 'medium', 'district9', 'd9b00001-0000-0000-0000-000000000001', 'Unauthorized access to financial institution computer systems and theft of customer personally identifiable information.', false, 'Federal City'),
('d9c00003-0000-0000-0000-000000000003', 'district9', '9:26-cr-00103', 'United States v. Williams et al.', 'racketeering', 'discovery', 'critical', 'district9', 'd9b00001-0000-0000-0000-000000000001', 'RICO enterprise involving extortion, wire fraud, and illegal gambling operations spanning four states.', false, 'Federal City'),
('d9c00004-0000-0000-0000-000000000004', 'district9', '9:26-cr-00104', 'United States v. Petrov', 'money_laundering', 'pretrial_motions', 'high', 'district9', 'd9b00002-0000-0000-0000-000000000002', 'Laundering over $12 million in proceeds from international fraud scheme through shell companies and cryptocurrency exchanges.', false, 'Federal City'),
('d9c00005-0000-0000-0000-000000000005', 'district9', '9:25-cr-00098', 'United States v. Jackson', 'firearms', 'trial_ready', 'critical', 'district9', 'd9b00001-0000-0000-0000-000000000001', 'Illegal possession and trafficking of fully automatic weapons and silencers by a previously convicted felon.', false, 'Federal City'),
('d9c00006-0000-0000-0000-000000000006', 'district9', '9:25-cr-00087', 'United States v. Morrison', 'fraud', 'in_trial', 'high', 'district9', 'd9b00001-0000-0000-0000-000000000001', 'Multi-million dollar healthcare fraud scheme involving fabricated patient records and phantom billing to Medicare.', false, 'Federal City'),
('d9c00007-0000-0000-0000-000000000007', 'district9', '9:24-cr-00042', 'United States v. Ahmed', 'tax_offense', 'sentenced', 'medium', 'district9', 'd9b00001-0000-0000-0000-000000000001', 'Willful failure to file tax returns and tax evasion totaling $2.3 million in unpaid federal income taxes over six years.', false, 'Federal City'),
('d9c00008-0000-0000-0000-000000000008', 'district9', '9:24-cr-00019', 'United States v. Reeves', 'drug_offense', 'on_appeal', 'high', 'district9', 'd9b00004-0000-0000-0000-000000000004', 'Conspiracy to manufacture and distribute methamphetamine; appeal challenges sufficiency of evidence and sentencing guidelines calculation.', false, 'Federal City')
ON CONFLICT (id) DO NOTHING;

-- district12 cases
INSERT INTO criminal_cases (id, court_id, case_number, title, crime_type, status, priority, district_code, assigned_judge_id, description, is_sealed, location)
VALUES
('d12c0001-0000-0000-0000-000000000001', 'district12', '12:26-cr-00201', 'United States v. Gonzalez', 'immigration', 'plea_negotiations', 'medium', 'district12', 'd12b0002-0000-0000-0000-000000000002', 'Alien smuggling operation bringing undocumented individuals across the border using fraudulent travel documents.', false, 'Metro City'),
('d12c0002-0000-0000-0000-000000000002', 'district12', '12:25-cr-00178', 'United States v. Park', 'cybercrime', 'awaiting_sentencing', 'high', 'district12', 'd12b0001-0000-0000-0000-000000000001', 'State-sponsored hacking campaign targeting defense contractors and exfiltrating classified weapons system specifications.', false, 'Metro City'),
('d12c0003-0000-0000-0000-000000000003', 'district12', '12:25-cr-00165', 'United States v. Thompson', 'firearms', 'dismissed', 'low', 'district12', 'd12b0001-0000-0000-0000-000000000001', 'Possession of an unregistered short-barreled rifle; case dismissed after successful suppression motion on Fourth Amendment grounds.', false, 'Metro City'),
('d12c0004-0000-0000-0000-000000000004', 'district12', '12:26-cr-00210', 'United States v. Volkov & Sokolov', 'money_laundering', 'pretrial_motions', 'critical', 'district12', 'd12b0001-0000-0000-0000-000000000001', 'International money laundering conspiracy funneling illicit funds through real estate holdings and offshore bank accounts.', true, 'Metro City'),
('d12c0005-0000-0000-0000-000000000005', 'district12', '12:26-cr-00215', 'United States v. Davis', 'fraud', 'arraigned', 'medium', 'district12', 'd12b0002-0000-0000-0000-000000000002', 'Wire fraud and identity theft scheme targeting elderly victims through fraudulent investment advisory services.', false, 'Metro City'),
('d12c0006-0000-0000-0000-000000000006', 'district12', '12:26-cr-00220', 'United States v. Hernandez', 'drug_offense', 'discovery', 'medium', 'district12', 'd12b0002-0000-0000-0000-000000000002', 'Conspiracy to distribute cocaine and heroin through a network of stash houses in the metropolitan area.', false, 'Metro City'),
('d12c0007-0000-0000-0000-000000000007', 'district12', '12:26-cr-00225', 'United States v. Carter', 'racketeering', 'filed', 'high', 'district12', 'd12b0003-0000-0000-0000-000000000003', 'RICO conspiracy involving a criminal enterprise engaged in wire fraud, money laundering, and obstruction of justice.', false, 'Metro City')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- DEFENDANTS (22 total)
-- ============================================================

-- district9 defendants
INSERT INTO defendants (id, court_id, case_id, name, custody_status, bail_type, bail_amount, citizenship_status)
VALUES
-- Case 1: Rodriguez (drug_offense, filed)
('d9de0001-0000-0000-0000-000000000001', 'district9', 'd9c00001-0000-0000-0000-000000000001', 'Carlos Rodriguez', 'Released', 'Personal Recognizance', NULL, 'Citizen'),
-- Case 2: Chen (cybercrime, arraigned)
('d9de0002-0000-0000-0000-000000000002', 'district9', 'd9c00002-0000-0000-0000-000000000002', 'Wei Chen', 'Bail', 'Cash', 100000.00, 'Visa Holder'),
-- Case 3: Williams RICO (discovery) — 4 defendants
('d9de0003-0000-0000-0000-000000000003', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'James Williams', 'In Custody', 'Denied', NULL, 'Citizen'),
('d9de0004-0000-0000-0000-000000000004', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'Tyrone Brooks', 'Bond', 'Surety', 500000.00, 'Citizen'),
('d9de0005-0000-0000-0000-000000000005', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'Keisha Watts', 'Released', 'Personal Recognizance', NULL, 'Citizen'),
('d9de0006-0000-0000-0000-000000000006', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'Derek Simmons', 'Bond', 'Surety', 250000.00, 'Citizen'),
-- Case 4: Petrov (money_laundering, pretrial_motions)
('d9de0007-0000-0000-0000-000000000007', 'district9', 'd9c00004-0000-0000-0000-000000000004', 'Aleksandr Petrov', 'In Custody', 'Denied', NULL, 'Permanent Resident'),
-- Case 5: Jackson (firearms, trial_ready)
('d9de0008-0000-0000-0000-000000000008', 'district9', 'd9c00005-0000-0000-0000-000000000005', 'Marcus Jackson', 'Bond', 'Surety', 250000.00, 'Citizen'),
-- Case 6: Morrison (fraud, in_trial)
('d9de0009-0000-0000-0000-000000000009', 'district9', 'd9c00006-0000-0000-0000-000000000006', 'Linda Morrison', 'Bond', 'Surety', 500000.00, 'Citizen'),
-- Case 7: Ahmed (tax_offense, sentenced)
('d9de000a-0000-0000-0000-000000000001', 'district9', 'd9c00007-0000-0000-0000-000000000007', 'Farooq Ahmed', 'In Custody', NULL, NULL, 'Citizen'),
-- Case 8: Reeves (drug_offense, on_appeal)
('d9de000b-0000-0000-0000-000000000002', 'district9', 'd9c00008-0000-0000-0000-000000000008', 'Darnell Reeves', 'In Custody', NULL, NULL, 'Citizen')
ON CONFLICT (id) DO NOTHING;

-- district12 defendants
INSERT INTO defendants (id, court_id, case_id, name, custody_status, bail_type, bail_amount, citizenship_status)
VALUES
-- Case 9: Gonzalez (immigration, plea_negotiations)
('d12de001-0000-0000-0000-000000000001', 'district12', 'd12c0001-0000-0000-0000-000000000001', 'Rafael Gonzalez', 'Released', 'Personal Recognizance', NULL, 'Permanent Resident'),
-- Case 10: Park (cybercrime, awaiting_sentencing)
('d12de002-0000-0000-0000-000000000002', 'district12', 'd12c0002-0000-0000-0000-000000000002', 'Sung-Ho Park', 'In Custody', NULL, NULL, 'Visa Holder'),
-- Case 11: Thompson (firearms, dismissed)
('d12de003-0000-0000-0000-000000000003', 'district12', 'd12c0003-0000-0000-0000-000000000003', 'Brian Thompson', 'Released', NULL, NULL, 'Citizen'),
-- Case 12: Volkov & Sokolov (money_laundering, pretrial_motions, sealed) — 2 defendants
('d12de004-0000-0000-0000-000000000004', 'district12', 'd12c0004-0000-0000-0000-000000000004', 'Viktor Volkov', 'In Custody', 'Denied', NULL, 'Visa Holder'),
('d12de005-0000-0000-0000-000000000005', 'district12', 'd12c0004-0000-0000-0000-000000000004', 'Dmitri Sokolov', 'In Custody', 'Denied', NULL, 'Visa Holder'),
-- Case 13: Davis (fraud, arraigned, pro se)
('d12de006-0000-0000-0000-000000000006', 'district12', 'd12c0005-0000-0000-0000-000000000005', 'Raymond Davis', 'Released', 'Personal Recognizance', NULL, 'Citizen'),
-- Case 14: Hernandez (drug_offense, discovery)
('d12de007-0000-0000-0000-000000000007', 'district12', 'd12c0006-0000-0000-0000-000000000006', 'Miguel Hernandez', 'Bond', 'Surety', 150000.00, 'Citizen'),
-- Case 15: Carter (racketeering, filed)
('d12de008-0000-0000-0000-000000000008', 'district12', 'd12c0007-0000-0000-0000-000000000007', 'Terrence Carter', 'Released', 'Personal Recognizance', NULL, 'Citizen')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- CHARGES (~30 total)
-- ============================================================

-- district9 charges
INSERT INTO charges (id, court_id, defendant_id, count_number, statute, offense_description, statutory_max_months, statutory_min_months, plea, plea_date, verdict, verdict_date)
VALUES
-- Case 1: Rodriguez (filed) — plea not yet entered
('d9cf0001-0000-0000-0000-000000000001', 'district9', 'd9de0001-0000-0000-0000-000000000001', 1, '21 U.S.C. § 841(a)(1)', 'Distribution of a controlled substance (fentanyl)', 240, 60, 'Not Yet Entered', NULL, NULL, NULL),
('d9cf0002-0000-0000-0000-000000000002', 'district9', 'd9de0001-0000-0000-0000-000000000001', 2, '21 U.S.C. § 846', 'Conspiracy to distribute controlled substances', 240, 60, 'Not Yet Entered', NULL, NULL, NULL),

-- Case 2: Chen (arraigned) — plea not yet entered
('d9cf0003-0000-0000-0000-000000000003', 'district9', 'd9de0002-0000-0000-0000-000000000002', 1, '18 U.S.C. § 1030(a)(2)', 'Unauthorized access to protected computer systems', 60, 0, 'Not Yet Entered', NULL, NULL, NULL),
('d9cf0004-0000-0000-0000-000000000004', 'district9', 'd9de0002-0000-0000-0000-000000000002', 2, '18 U.S.C. § 1028A', 'Aggravated identity theft', 24, 24, 'Not Yet Entered', NULL, NULL, NULL),

-- Case 3: Williams RICO (discovery) — not guilty pleas
('d9cf0005-0000-0000-0000-000000000005', 'district9', 'd9de0003-0000-0000-0000-000000000003', 1, '18 U.S.C. § 1962(c)', 'RICO — conducting affairs of enterprise through pattern of racketeering activity', 240, 0, 'Not Guilty', '2026-01-15 10:00:00-05', NULL, NULL),
('d9cf0006-0000-0000-0000-000000000006', 'district9', 'd9de0003-0000-0000-0000-000000000003', 2, '18 U.S.C. § 1962(d)', 'RICO conspiracy', 240, 0, 'Not Guilty', '2026-01-15 10:00:00-05', NULL, NULL),
('d9cf0007-0000-0000-0000-000000000007', 'district9', 'd9de0004-0000-0000-0000-000000000004', 1, '18 U.S.C. § 1962(c)', 'RICO — conducting affairs of enterprise through pattern of racketeering activity', 240, 0, 'Not Guilty', '2026-01-15 10:30:00-05', NULL, NULL),
('d9cf0008-0000-0000-0000-000000000008', 'district9', 'd9de0005-0000-0000-0000-000000000005', 1, '18 U.S.C. § 1962(d)', 'RICO conspiracy', 240, 0, 'Not Guilty', '2026-01-15 11:00:00-05', NULL, NULL),
('d9cf0009-0000-0000-0000-000000000009', 'district9', 'd9de0006-0000-0000-0000-000000000006', 1, '18 U.S.C. § 1962(d)', 'RICO conspiracy', 240, 0, 'Not Guilty', '2026-01-15 11:30:00-05', NULL, NULL),
('d9cf000a-0000-0000-0000-000000000001', 'district9', 'd9de0006-0000-0000-0000-000000000006', 2, '18 U.S.C. § 1955', 'Illegal gambling business', 60, 0, 'Not Guilty', '2026-01-15 11:30:00-05', NULL, NULL),

-- Case 4: Petrov (pretrial_motions) — not guilty
('d9cf000b-0000-0000-0000-000000000002', 'district9', 'd9de0007-0000-0000-0000-000000000007', 1, '18 U.S.C. § 1956(a)(1)', 'Money laundering — financial transactions with proceeds of unlawful activity', 240, 0, 'Not Guilty', '2026-01-20 09:00:00-05', NULL, NULL),
('d9cf000c-0000-0000-0000-000000000003', 'district9', 'd9de0007-0000-0000-0000-000000000007', 2, '18 U.S.C. § 1957', 'Engaging in monetary transactions in property derived from specified unlawful activity', 120, 0, 'Not Guilty', '2026-01-20 09:00:00-05', NULL, NULL),

-- Case 5: Jackson (trial_ready) — not guilty
('d9cf000d-0000-0000-0000-000000000004', 'district9', 'd9de0008-0000-0000-0000-000000000008', 1, '18 U.S.C. § 922(g)(1)', 'Felon in possession of a firearm', 120, 180, 'Not Guilty', '2025-11-10 10:00:00-05', NULL, NULL),
('d9cf000e-0000-0000-0000-000000000005', 'district9', 'd9de0008-0000-0000-0000-000000000008', 2, '26 U.S.C. § 5861(d)', 'Possession of unregistered firearm (machine gun)', 120, 0, 'Not Guilty', '2025-11-10 10:00:00-05', NULL, NULL),

-- Case 6: Morrison (in_trial) — not guilty
('d9cf000f-0000-0000-0000-000000000006', 'district9', 'd9de0009-0000-0000-0000-000000000009', 1, '18 U.S.C. § 1347', 'Healthcare fraud', 120, 0, 'Not Guilty', '2025-09-05 09:30:00-05', NULL, NULL),
('d9cf0010-0000-0000-0000-000000000007', 'district9', 'd9de0009-0000-0000-0000-000000000009', 2, '18 U.S.C. § 1341', 'Mail fraud', 240, 0, 'Not Guilty', '2025-09-05 09:30:00-05', NULL, NULL),

-- Case 7: Ahmed (sentenced) — guilty plea and verdict
('d9cf0011-0000-0000-0000-000000000008', 'district9', 'd9de000a-0000-0000-0000-000000000001', 1, '26 U.S.C. § 7201', 'Tax evasion', 60, 0, 'Guilty', '2025-06-12 10:00:00-05', 'Guilty', '2025-06-12 10:00:00-05'),
('d9cf0012-0000-0000-0000-000000000009', 'district9', 'd9de000a-0000-0000-0000-000000000001', 2, '26 U.S.C. § 7206(1)', 'Filing false tax returns', 36, 0, 'Guilty', '2025-06-12 10:00:00-05', 'Guilty', '2025-06-12 10:00:00-05'),

-- Case 8: Reeves (on_appeal) — not guilty plea, guilty verdict
('d9cf0013-0000-0000-0000-000000000001', 'district9', 'd9de000b-0000-0000-0000-000000000002', 1, '21 U.S.C. § 841(a)(1)', 'Manufacture of methamphetamine', 480, 120, 'Not Guilty', '2024-08-15 09:00:00-05', 'Guilty', '2025-01-22 15:30:00-05'),
('d9cf0014-0000-0000-0000-000000000002', 'district9', 'd9de000b-0000-0000-0000-000000000002', 2, '21 U.S.C. § 846', 'Conspiracy to manufacture and distribute methamphetamine', 480, 120, 'Not Guilty', '2024-08-15 09:00:00-05', 'Guilty', '2025-01-22 15:30:00-05')
ON CONFLICT (id) DO NOTHING;

-- district12 charges
INSERT INTO charges (id, court_id, defendant_id, count_number, statute, offense_description, statutory_max_months, statutory_min_months, plea, plea_date, verdict, verdict_date)
VALUES
-- Case 9: Gonzalez (plea_negotiations) — plea not yet entered
('d12cf001-0000-0000-0000-000000000001', 'district12', 'd12de001-0000-0000-0000-000000000001', 1, '8 U.S.C. § 1324(a)(1)(A)', 'Alien smuggling — bringing in unauthorized aliens', 120, 0, 'Not Yet Entered', NULL, NULL, NULL),
('d12cf002-0000-0000-0000-000000000002', 'district12', 'd12de001-0000-0000-0000-000000000001', 2, '18 U.S.C. § 1546(a)', 'Fraud and misuse of visas and travel documents', 120, 0, 'Not Yet Entered', NULL, NULL, NULL),

-- Case 10: Park (awaiting_sentencing) — guilty verdict after trial
('d12cf003-0000-0000-0000-000000000003', 'district12', 'd12de002-0000-0000-0000-000000000002', 1, '18 U.S.C. § 1030(a)(1)', 'Computer fraud — accessing classified national defense information', 120, 0, 'Not Guilty', '2025-05-20 10:00:00-05', 'Guilty', '2025-12-10 14:00:00-05'),
('d12cf004-0000-0000-0000-000000000004', 'district12', 'd12de002-0000-0000-0000-000000000002', 2, '18 U.S.C. § 793(e)', 'Unauthorized retention of national defense information', 120, 0, 'Not Guilty', '2025-05-20 10:00:00-05', 'Guilty', '2025-12-10 14:00:00-05'),

-- Case 11: Thompson (dismissed)
('d12cf005-0000-0000-0000-000000000005', 'district12', 'd12de003-0000-0000-0000-000000000003', 1, '26 U.S.C. § 5861(d)', 'Possession of unregistered short-barreled rifle', 120, 0, NULL, NULL, 'Dismissed', '2025-10-15 11:00:00-05'),

-- Case 12: Volkov & Sokolov (pretrial_motions, sealed) — not guilty
('d12cf006-0000-0000-0000-000000000006', 'district12', 'd12de004-0000-0000-0000-000000000004', 1, '18 U.S.C. § 1956(a)(2)', 'Money laundering — international transportation of monetary instruments', 240, 0, 'Not Guilty', '2026-02-01 09:00:00-05', NULL, NULL),
('d12cf007-0000-0000-0000-000000000007', 'district12', 'd12de004-0000-0000-0000-000000000004', 2, '18 U.S.C. § 1956(h)', 'Conspiracy to commit money laundering', 240, 0, 'Not Guilty', '2026-02-01 09:00:00-05', NULL, NULL),
('d12cf008-0000-0000-0000-000000000008', 'district12', 'd12de005-0000-0000-0000-000000000005', 1, '18 U.S.C. § 1956(a)(2)', 'Money laundering — international transportation of monetary instruments', 240, 0, 'Not Guilty', '2026-02-01 09:30:00-05', NULL, NULL),
('d12cf009-0000-0000-0000-000000000009', 'district12', 'd12de005-0000-0000-0000-000000000005', 2, '18 U.S.C. § 1956(h)', 'Conspiracy to commit money laundering', 240, 0, 'Not Guilty', '2026-02-01 09:30:00-05', NULL, NULL),

-- Case 13: Davis (arraigned) — not yet entered
('d12cf00a-0000-0000-0000-000000000001', 'district12', 'd12de006-0000-0000-0000-000000000006', 1, '18 U.S.C. § 1343', 'Wire fraud', 240, 0, 'Not Yet Entered', NULL, NULL, NULL),
('d12cf00b-0000-0000-0000-000000000002', 'district12', 'd12de006-0000-0000-0000-000000000006', 2, '18 U.S.C. § 1028(a)(7)', 'Identity theft', 60, 0, 'Not Yet Entered', NULL, NULL, NULL),

-- Case 14: Hernandez (discovery) — not guilty
('d12cf00c-0000-0000-0000-000000000003', 'district12', 'd12de007-0000-0000-0000-000000000007', 1, '21 U.S.C. § 846', 'Conspiracy to distribute cocaine and heroin', 240, 60, 'Not Guilty', '2026-01-25 10:00:00-05', NULL, NULL),
('d12cf00d-0000-0000-0000-000000000004', 'district12', 'd12de007-0000-0000-0000-000000000007', 2, '21 U.S.C. § 856(a)(1)', 'Maintaining drug-involved premises', 240, 0, 'Not Guilty', '2026-01-25 10:00:00-05', NULL, NULL),

-- Case 15: Carter (filed) — not yet entered
('d12cf00e-0000-0000-0000-000000000005', 'district12', 'd12de008-0000-0000-0000-000000000008', 1, '18 U.S.C. § 1962(d)', 'RICO conspiracy', 240, 0, 'Not Yet Entered', NULL, NULL, NULL),
('d12cf00f-0000-0000-0000-000000000006', 'district12', 'd12de008-0000-0000-0000-000000000008', 2, '18 U.S.C. § 1956(h)', 'Conspiracy to commit money laundering', 240, 0, 'Not Yet Entered', NULL, NULL, NULL)
ON CONFLICT (id) DO NOTHING;

END $$;
