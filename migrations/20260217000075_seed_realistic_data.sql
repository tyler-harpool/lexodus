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

-- ============================================================
-- PARTIES (~35 total: government + defendant per case)
-- ============================================================

-- district9 parties
INSERT INTO parties (id, court_id, case_id, party_type, party_role, name, entity_type, represented, pro_se, service_method, status, joined_date)
VALUES
-- Case 1: Rodriguez (filed)
('d9ab0001-0000-0000-0000-000000000001', 'district9', 'd9c00001-0000-0000-0000-000000000001', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '14 days'),
('d9ab0002-0000-0000-0000-000000000002', 'district9', 'd9c00001-0000-0000-0000-000000000001', 'Defendant', 'Lead', 'Carlos Rodriguez', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '14 days'),
-- Case 2: Chen (arraigned)
('d9ab0003-0000-0000-0000-000000000003', 'district9', 'd9c00002-0000-0000-0000-000000000002', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '30 days'),
('d9ab0004-0000-0000-0000-000000000004', 'district9', 'd9c00002-0000-0000-0000-000000000002', 'Defendant', 'Lead', 'Wei Chen', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '30 days'),
-- Case 3: Williams RICO (discovery) — 4 defendants
('d9ab0005-0000-0000-0000-000000000005', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '60 days'),
('d9ab0006-0000-0000-0000-000000000006', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'Defendant', 'Lead', 'James Williams', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '60 days'),
('d9ab0007-0000-0000-0000-000000000007', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'Defendant', 'Co-Defendant', 'Tyrone Brooks', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '60 days'),
('d9ab0008-0000-0000-0000-000000000008', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'Defendant', 'Co-Defendant', 'Keisha Watts', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '60 days'),
('d9ab0009-0000-0000-0000-000000000009', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'Defendant', 'Co-Defendant', 'Derek Simmons', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '60 days'),
-- Case 4: Petrov (pretrial_motions)
('d9ab000a-0000-0000-0000-000000000001', 'district9', 'd9c00004-0000-0000-0000-000000000004', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '90 days'),
('d9ab000b-0000-0000-0000-000000000002', 'district9', 'd9c00004-0000-0000-0000-000000000004', 'Defendant', 'Lead', 'Aleksandr Petrov', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '90 days'),
-- Case 5: Jackson (trial_ready)
('d9ab000c-0000-0000-0000-000000000003', 'district9', 'd9c00005-0000-0000-0000-000000000005', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '120 days'),
('d9ab000d-0000-0000-0000-000000000004', 'district9', 'd9c00005-0000-0000-0000-000000000005', 'Defendant', 'Lead', 'Marcus Jackson', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '120 days'),
-- Case 6: Morrison (in_trial)
('d9ab000e-0000-0000-0000-000000000005', 'district9', 'd9c00006-0000-0000-0000-000000000006', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '180 days'),
('d9ab000f-0000-0000-0000-000000000006', 'district9', 'd9c00006-0000-0000-0000-000000000006', 'Defendant', 'Lead', 'Linda Morrison', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '180 days'),
-- Case 7: Ahmed (sentenced)
('d9ab0010-0000-0000-0000-000000000007', 'district9', 'd9c00007-0000-0000-0000-000000000007', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '365 days'),
('d9ab0011-0000-0000-0000-000000000008', 'district9', 'd9c00007-0000-0000-0000-000000000007', 'Defendant', 'Lead', 'Farooq Ahmed', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '365 days'),
-- Case 8: Reeves (on_appeal)
('d9ab0012-0000-0000-0000-000000000009', 'district9', 'd9c00008-0000-0000-0000-000000000008', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '540 days'),
('d9ab0013-0000-0000-0000-000000000001', 'district9', 'd9c00008-0000-0000-0000-000000000008', 'Defendant', 'Lead', 'Darnell Reeves', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '540 days')
ON CONFLICT (id) DO NOTHING;

-- district12 parties
INSERT INTO parties (id, court_id, case_id, party_type, party_role, name, entity_type, represented, pro_se, service_method, status, joined_date)
VALUES
-- Case 9: Gonzalez (plea_negotiations)
('d12ab001-0000-0000-0000-000000000001', 'district12', 'd12c0001-0000-0000-0000-000000000001', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '75 days'),
('d12ab002-0000-0000-0000-000000000002', 'district12', 'd12c0001-0000-0000-0000-000000000001', 'Defendant', 'Lead', 'Rafael Gonzalez', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '75 days'),
-- Case 10: Park (awaiting_sentencing)
('d12ab003-0000-0000-0000-000000000003', 'district12', 'd12c0002-0000-0000-0000-000000000002', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '270 days'),
('d12ab004-0000-0000-0000-000000000004', 'district12', 'd12c0002-0000-0000-0000-000000000002', 'Defendant', 'Lead', 'Sung-Ho Park', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '270 days'),
-- Case 11: Thompson (dismissed)
('d12ab005-0000-0000-0000-000000000005', 'district12', 'd12c0003-0000-0000-0000-000000000003', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '150 days'),
('d12ab006-0000-0000-0000-000000000006', 'district12', 'd12c0003-0000-0000-0000-000000000003', 'Defendant', 'Lead', 'Brian Thompson', 'Individual', true, false, 'Electronic', 'Dismissed', NOW() - INTERVAL '150 days'),
-- Case 12: Volkov & Sokolov (pretrial_motions, sealed) — 2 defendants
('d12ab007-0000-0000-0000-000000000007', 'district12', 'd12c0004-0000-0000-0000-000000000004', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '45 days'),
('d12ab008-0000-0000-0000-000000000008', 'district12', 'd12c0004-0000-0000-0000-000000000004', 'Defendant', 'Lead', 'Viktor Volkov', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '45 days'),
('d12ab009-0000-0000-0000-000000000009', 'district12', 'd12c0004-0000-0000-0000-000000000004', 'Defendant', 'Co-Defendant', 'Dmitri Sokolov', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '45 days'),
-- Case 13: Davis (arraigned, pro se)
('d12ab00a-0000-0000-0000-000000000001', 'district12', 'd12c0005-0000-0000-0000-000000000005', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '25 days'),
('d12ab00b-0000-0000-0000-000000000002', 'district12', 'd12c0005-0000-0000-0000-000000000005', 'Defendant', 'Lead', 'Raymond Davis', 'Individual', false, true, 'Mail', 'Active', NOW() - INTERVAL '25 days'),
-- Case 14: Hernandez (discovery)
('d12ab00c-0000-0000-0000-000000000003', 'district12', 'd12c0006-0000-0000-0000-000000000006', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '50 days'),
('d12ab00d-0000-0000-0000-000000000004', 'district12', 'd12c0006-0000-0000-0000-000000000006', 'Defendant', 'Lead', 'Miguel Hernandez', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '50 days'),
-- Case 15: Carter (filed)
('d12ab00e-0000-0000-0000-000000000005', 'district12', 'd12c0007-0000-0000-0000-000000000007', 'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW() - INTERVAL '10 days'),
('d12ab00f-0000-0000-0000-000000000006', 'district12', 'd12c0007-0000-0000-0000-000000000007', 'Defendant', 'Lead', 'Terrence Carter', 'Individual', true, false, 'Electronic', 'Active', NOW() - INTERVAL '10 days')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- REPRESENTATIONS (~30 total)
-- ============================================================

-- district9 representations
INSERT INTO representations (id, court_id, attorney_id, party_id, case_id, representation_type, status, start_date, lead_counsel, court_appointed, withdrawal_reason, notes)
VALUES
-- Case 1: Rodriguez — AUSA Mitchell (gov), PD Rivera (def)
('d9ae0001-0000-0000-0000-000000000001', 'district9', 'd9a00001-0000-0000-0000-000000000001', 'd9ab0001-0000-0000-0000-000000000001', 'd9c00001-0000-0000-0000-000000000001', 'Government', 'Active', NOW() - INTERVAL '14 days', true, false, NULL, NULL),
('d9ae0002-0000-0000-0000-000000000002', 'district9', 'd9a00002-0000-0000-0000-000000000002', 'd9ab0002-0000-0000-0000-000000000002', 'd9c00001-0000-0000-0000-000000000001', 'Public Defender', 'Active', NOW() - INTERVAL '14 days', true, true, NULL, NULL),
-- Case 2: Chen — AUSA Mitchell (gov), PD Rivera (def)
('d9ae0003-0000-0000-0000-000000000003', 'district9', 'd9a00001-0000-0000-0000-000000000001', 'd9ab0003-0000-0000-0000-000000000003', 'd9c00002-0000-0000-0000-000000000002', 'Government', 'Active', NOW() - INTERVAL '30 days', true, false, NULL, NULL),
('d9ae0004-0000-0000-0000-000000000004', 'district9', 'd9a00002-0000-0000-0000-000000000002', 'd9ab0004-0000-0000-0000-000000000004', 'd9c00002-0000-0000-0000-000000000002', 'Public Defender', 'Active', NOW() - INTERVAL '30 days', true, true, NULL, NULL),
-- Case 3: Williams RICO — AUSA Mitchell (gov), Williams:Whitfield(Private), Brooks:Rivera(PD), Watts:Okonkwo(CJA), Simmons:Okonkwo(CJA)
('d9ae0005-0000-0000-0000-000000000005', 'district9', 'd9a00001-0000-0000-0000-000000000001', 'd9ab0005-0000-0000-0000-000000000005', 'd9c00003-0000-0000-0000-000000000003', 'Government', 'Active', NOW() - INTERVAL '60 days', true, false, NULL, NULL),
('d9ae0006-0000-0000-0000-000000000006', 'district9', 'd9a00003-0000-0000-0000-000000000003', 'd9ab0006-0000-0000-0000-000000000006', 'd9c00003-0000-0000-0000-000000000003', 'Private', 'Active', NOW() - INTERVAL '58 days', true, false, NULL, 'Retained privately by defendant Williams'),
('d9ae0007-0000-0000-0000-000000000007', 'district9', 'd9a00002-0000-0000-0000-000000000002', 'd9ab0007-0000-0000-0000-000000000007', 'd9c00003-0000-0000-0000-000000000003', 'Public Defender', 'Active', NOW() - INTERVAL '58 days', true, true, NULL, NULL),
('d9ae0008-0000-0000-0000-000000000008', 'district9', 'd9a00004-0000-0000-0000-000000000004', 'd9ab0008-0000-0000-0000-000000000008', 'd9c00003-0000-0000-0000-000000000003', 'CJA Panel', 'Active', NOW() - INTERVAL '57 days', true, true, NULL, 'CJA appointment for defendant Watts'),
('d9ae0009-0000-0000-0000-000000000009', 'district9', 'd9a00004-0000-0000-0000-000000000004', 'd9ab0009-0000-0000-0000-000000000009', 'd9c00003-0000-0000-0000-000000000003', 'CJA Panel', 'Active', NOW() - INTERVAL '57 days', true, true, NULL, 'CJA appointment for defendant Simmons'),
-- Case 4: Petrov — AUSA Mitchell (gov), Whitfield (Private)
('d9ae000a-0000-0000-0000-000000000001', 'district9', 'd9a00001-0000-0000-0000-000000000001', 'd9ab000a-0000-0000-0000-000000000001', 'd9c00004-0000-0000-0000-000000000004', 'Government', 'Active', NOW() - INTERVAL '90 days', true, false, NULL, NULL),
('d9ae000b-0000-0000-0000-000000000002', 'district9', 'd9a00003-0000-0000-0000-000000000003', 'd9ab000b-0000-0000-0000-000000000002', 'd9c00004-0000-0000-0000-000000000004', 'Private', 'Active', NOW() - INTERVAL '88 days', true, false, NULL, 'Retained privately by defendant Petrov'),
-- Case 5: Jackson — AUSA Mitchell (gov), Whitfield (Private)
('d9ae000c-0000-0000-0000-000000000003', 'district9', 'd9a00001-0000-0000-0000-000000000001', 'd9ab000c-0000-0000-0000-000000000003', 'd9c00005-0000-0000-0000-000000000005', 'Government', 'Active', NOW() - INTERVAL '120 days', true, false, NULL, NULL),
('d9ae000d-0000-0000-0000-000000000004', 'district9', 'd9a00003-0000-0000-0000-000000000003', 'd9ab000d-0000-0000-0000-000000000004', 'd9c00005-0000-0000-0000-000000000005', 'Private', 'Active', NOW() - INTERVAL '118 days', true, false, NULL, NULL),
-- Case 6: Morrison — AUSA Mitchell (gov), Whitfield (Private)
('d9ae000e-0000-0000-0000-000000000005', 'district9', 'd9a00001-0000-0000-0000-000000000001', 'd9ab000e-0000-0000-0000-000000000005', 'd9c00006-0000-0000-0000-000000000006', 'Government', 'Active', NOW() - INTERVAL '180 days', true, false, NULL, NULL),
('d9ae000f-0000-0000-0000-000000000006', 'district9', 'd9a00003-0000-0000-0000-000000000003', 'd9ab000f-0000-0000-0000-000000000006', 'd9c00006-0000-0000-0000-000000000006', 'Private', 'Active', NOW() - INTERVAL '178 days', true, false, NULL, NULL),
-- Case 7: Ahmed — AUSA Mitchell (gov), PD Rivera (def)
('d9ae0010-0000-0000-0000-000000000007', 'district9', 'd9a00001-0000-0000-0000-000000000001', 'd9ab0010-0000-0000-0000-000000000007', 'd9c00007-0000-0000-0000-000000000007', 'Government', 'Active', NOW() - INTERVAL '365 days', true, false, NULL, NULL),
('d9ae0011-0000-0000-0000-000000000008', 'district9', 'd9a00002-0000-0000-0000-000000000002', 'd9ab0011-0000-0000-0000-000000000008', 'd9c00007-0000-0000-0000-000000000007', 'Public Defender', 'Active', NOW() - INTERVAL '363 days', true, true, NULL, NULL),
-- Case 8: Reeves — AUSA Mitchell (gov), Rivera (PD, substituted), then Whitfield (appellate)
('d9ae0012-0000-0000-0000-000000000009', 'district9', 'd9a00001-0000-0000-0000-000000000001', 'd9ab0012-0000-0000-0000-000000000009', 'd9c00008-0000-0000-0000-000000000008', 'Government', 'Active', NOW() - INTERVAL '540 days', true, false, NULL, NULL),
('d9ae0013-0000-0000-0000-000000000001', 'district9', 'd9a00002-0000-0000-0000-000000000002', 'd9ab0013-0000-0000-0000-000000000001', 'd9c00008-0000-0000-0000-000000000008', 'Public Defender', 'Substituted', NOW() - INTERVAL '538 days', true, true, NULL, 'Trial counsel; substituted upon appeal'),
('d9ae0014-0000-0000-0000-000000000002', 'district9', 'd9a00003-0000-0000-0000-000000000003', 'd9ab0013-0000-0000-0000-000000000001', 'd9c00008-0000-0000-0000-000000000008', 'Private', 'Active', NOW() - INTERVAL '90 days', true, false, NULL, 'Appellate counsel retained for appeal')
ON CONFLICT (id) DO NOTHING;

-- district12 representations
INSERT INTO representations (id, court_id, attorney_id, party_id, case_id, representation_type, status, start_date, lead_counsel, court_appointed, withdrawal_reason, notes)
VALUES
-- Case 9: Gonzalez — AUSA Huang (gov), PD Blackwell (def)
('d12ae001-0000-0000-0000-000000000001', 'district12', 'd12a0001-0000-0000-0000-000000000001', 'd12ab001-0000-0000-0000-000000000001', 'd12c0001-0000-0000-0000-000000000001', 'Government', 'Active', NOW() - INTERVAL '75 days', true, false, NULL, NULL),
('d12ae002-0000-0000-0000-000000000002', 'district12', 'd12a0002-0000-0000-0000-000000000002', 'd12ab002-0000-0000-0000-000000000002', 'd12c0001-0000-0000-0000-000000000001', 'Public Defender', 'Active', NOW() - INTERVAL '73 days', true, true, NULL, NULL),
-- Case 10: Park — AUSA Huang (gov), Petrossian (Private)
('d12ae003-0000-0000-0000-000000000003', 'district12', 'd12a0001-0000-0000-0000-000000000001', 'd12ab003-0000-0000-0000-000000000003', 'd12c0002-0000-0000-0000-000000000002', 'Government', 'Active', NOW() - INTERVAL '270 days', true, false, NULL, NULL),
('d12ae004-0000-0000-0000-000000000004', 'district12', 'd12a0003-0000-0000-0000-000000000003', 'd12ab004-0000-0000-0000-000000000004', 'd12c0002-0000-0000-0000-000000000002', 'Private', 'Active', NOW() - INTERVAL '268 days', true, false, NULL, NULL),
-- Case 11: Thompson (dismissed) — AUSA Huang (gov), Petrossian (Private)
('d12ae005-0000-0000-0000-000000000005', 'district12', 'd12a0001-0000-0000-0000-000000000001', 'd12ab005-0000-0000-0000-000000000005', 'd12c0003-0000-0000-0000-000000000003', 'Government', 'Active', NOW() - INTERVAL '150 days', true, false, NULL, NULL),
('d12ae006-0000-0000-0000-000000000006', 'district12', 'd12a0003-0000-0000-0000-000000000003', 'd12ab006-0000-0000-0000-000000000006', 'd12c0003-0000-0000-0000-000000000003', 'Private', 'Active', NOW() - INTERVAL '148 days', true, false, NULL, NULL),
-- Case 12: Volkov & Sokolov — AUSA Huang (gov), Nakamura (Pro Hac Vice for Volkov), Blackwell (PD for Sokolov)
('d12ae007-0000-0000-0000-000000000007', 'district12', 'd12a0001-0000-0000-0000-000000000001', 'd12ab007-0000-0000-0000-000000000007', 'd12c0004-0000-0000-0000-000000000004', 'Government', 'Active', NOW() - INTERVAL '45 days', true, false, NULL, NULL),
('d12ae008-0000-0000-0000-000000000008', 'district12', 'd12a0004-0000-0000-0000-000000000004', 'd12ab008-0000-0000-0000-000000000008', 'd12c0004-0000-0000-0000-000000000004', 'Pro Hac Vice', 'Active', NOW() - INTERVAL '43 days', true, false, NULL, 'Admitted pro hac vice for defendant Volkov'),
('d12ae009-0000-0000-0000-000000000009', 'district12', 'd12a0002-0000-0000-0000-000000000002', 'd12ab009-0000-0000-0000-000000000009', 'd12c0004-0000-0000-0000-000000000004', 'Public Defender', 'Active', NOW() - INTERVAL '43 days', true, true, NULL, NULL),
-- Case 13: Davis (pro se) — AUSA Huang (gov), NO defense representation
('d12ae00a-0000-0000-0000-000000000001', 'district12', 'd12a0001-0000-0000-0000-000000000001', 'd12ab00a-0000-0000-0000-000000000001', 'd12c0005-0000-0000-0000-000000000005', 'Government', 'Active', NOW() - INTERVAL '25 days', true, false, NULL, NULL),
-- Case 14: Hernandez — AUSA Huang (gov), PD Blackwell (def)
('d12ae00b-0000-0000-0000-000000000002', 'district12', 'd12a0001-0000-0000-0000-000000000001', 'd12ab00c-0000-0000-0000-000000000003', 'd12c0006-0000-0000-0000-000000000006', 'Government', 'Active', NOW() - INTERVAL '50 days', true, false, NULL, NULL),
('d12ae00c-0000-0000-0000-000000000003', 'district12', 'd12a0002-0000-0000-0000-000000000002', 'd12ab00d-0000-0000-0000-000000000004', 'd12c0006-0000-0000-0000-000000000006', 'Public Defender', 'Active', NOW() - INTERVAL '48 days', true, true, NULL, NULL),
-- Case 15: Carter — AUSA Huang (gov), PD Blackwell (def)
('d12ae00d-0000-0000-0000-000000000004', 'district12', 'd12a0001-0000-0000-0000-000000000001', 'd12ab00e-0000-0000-0000-000000000005', 'd12c0007-0000-0000-0000-000000000007', 'Government', 'Active', NOW() - INTERVAL '10 days', true, false, NULL, NULL),
('d12ae00e-0000-0000-0000-000000000005', 'district12', 'd12a0002-0000-0000-0000-000000000002', 'd12ab00f-0000-0000-0000-000000000006', 'd12c0007-0000-0000-0000-000000000007', 'Public Defender', 'Active', NOW() - INTERVAL '8 days', true, true, NULL, NULL)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- DOCKET ENTRIES (~110 total)
-- ============================================================

-- Case 1: Rodriguez (filed) — 3 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9d00001-0000-0000-0000-000000000001', 'district9', 'd9c00001-0000-0000-0000-000000000001', 1, 'criminal_complaint', 'CRIMINAL COMPLAINT filed against Carlos Rodriguez. Arrest warrant issued.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '14 days'),
('d9d00002-0000-0000-0000-000000000002', 'district9', 'd9c00001-0000-0000-0000-000000000001', 2, 'summons', 'SUMMONS issued as to Carlos Rodriguez.', 'Clerk', false, false, NOW() - INTERVAL '14 days'),
('d9d00003-0000-0000-0000-000000000003', 'district9', 'd9c00001-0000-0000-0000-000000000001', 3, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Marcus J. Rivera on behalf of Carlos Rodriguez.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '12 days')
ON CONFLICT (id) DO NOTHING;

-- Case 2: Chen (arraigned) — 6 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9d00004-0000-0000-0000-000000000004', 'district9', 'd9c00002-0000-0000-0000-000000000002', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Wei Chen with 18 U.S.C. 1030(a)(2) and 18 U.S.C. 1028A. (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '30 days'),
('d9d00005-0000-0000-0000-000000000005', 'district9', 'd9c00002-0000-0000-0000-000000000002', 2, 'summons', 'SUMMONS issued as to Wei Chen. Initial appearance set.', 'Clerk', false, false, NOW() - INTERVAL '29 days'),
('d9d00006-0000-0000-0000-000000000006', 'district9', 'd9c00002-0000-0000-0000-000000000002', 3, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Marcus J. Rivera on behalf of Wei Chen.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '27 days'),
('d9d00007-0000-0000-0000-000000000007', 'district9', 'd9c00002-0000-0000-0000-000000000002', 4, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Ronnie Abrams. Arraignment held. Defendant entered plea of not guilty. Bail set at $100,000 cash bond.', 'Clerk', false, false, NOW() - INTERVAL '25 days'),
('d9d00008-0000-0000-0000-000000000008', 'district9', 'd9c00002-0000-0000-0000-000000000002', 5, 'notice', 'NOTICE of detention hearing scheduled for defendant Wei Chen.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '24 days'),
('d9d00009-0000-0000-0000-000000000009', 'district9', 'd9c00002-0000-0000-0000-000000000002', 6, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '22 days')
ON CONFLICT (id) DO NOTHING;

-- Case 3: Williams RICO (discovery) — 10 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9d0000a-0000-0000-0000-000000000001', 'district9', 'd9c00003-0000-0000-0000-000000000003', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging James Williams, Tyrone Brooks, Keisha Watts, and Derek Simmons with 18 U.S.C. 1962(c) and (d). (6 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '60 days'),
('d9d0000b-0000-0000-0000-000000000002', 'district9', 'd9c00003-0000-0000-0000-000000000003', 2, 'summons', 'SUMMONS issued as to all defendants in United States v. Williams et al.', 'Clerk', false, false, NOW() - INTERVAL '59 days'),
('d9d0000c-0000-0000-0000-000000000003', 'district9', 'd9c00003-0000-0000-0000-000000000003', 3, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Catherine L. Whitfield on behalf of James Williams.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '56 days'),
('d9d0000d-0000-0000-0000-000000000004', 'district9', 'd9c00003-0000-0000-0000-000000000003', 4, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Marcus J. Rivera on behalf of Tyrone Brooks, David R. Okonkwo on behalf of Keisha Watts and Derek Simmons.', 'Multiple Counsel', false, false, NOW() - INTERVAL '55 days'),
('d9d0000e-0000-0000-0000-000000000005', 'district9', 'd9c00003-0000-0000-0000-000000000003', 5, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Ronnie Abrams. Arraignment held for all defendants. All defendants entered pleas of not guilty. Bail determinations made individually.', 'Clerk', false, false, NOW() - INTERVAL '50 days'),
('d9d0000f-0000-0000-0000-000000000006', 'district9', 'd9c00003-0000-0000-0000-000000000003', 6, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline of 04/15/2026, motion deadline of 05/15/2026, and trial date of 07/06/2026. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '48 days'),
('d9d00010-0000-0000-0000-000000000007', 'district9', 'd9c00003-0000-0000-0000-000000000003', 7, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES pursuant to Fed. R. Crim. P. 16.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '42 days'),
('d9d00011-0000-0000-0000-000000000008', 'district9', 'd9c00003-0000-0000-0000-000000000003', 8, 'motion', 'MOTION for protective order regarding confidential business records filed by defendant Williams.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '35 days'),
('d9d00012-0000-0000-0000-000000000009', 'district9', 'd9c00003-0000-0000-0000-000000000003', 9, 'response', 'RESPONSE in Opposition to Motion for Protective Order filed by United States.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '28 days'),
('d9d00013-0000-0000-0000-000000000001', 'district9', 'd9c00003-0000-0000-0000-000000000003', 10, 'protective_order', 'PROTECTIVE ORDER regarding handling of confidential discovery materials. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '21 days')
ON CONFLICT (id) DO NOTHING;

-- Case 4: Petrov (pretrial_motions) — 10 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9d00014-0000-0000-0000-000000000002', 'district9', 'd9c00004-0000-0000-0000-000000000004', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Aleksandr Petrov with 18 U.S.C. 1956(a)(1) and 18 U.S.C. 1957. (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '90 days'),
('d9d00015-0000-0000-0000-000000000003', 'district9', 'd9c00004-0000-0000-0000-000000000004', 2, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Catherine L. Whitfield on behalf of Aleksandr Petrov.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '86 days'),
('d9d00016-0000-0000-0000-000000000004', 'district9', 'd9c00004-0000-0000-0000-000000000004', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Lance M. Africk. Arraignment held. Defendant entered plea of not guilty. Bail denied; defendant remanded to custody.', 'Clerk', false, false, NOW() - INTERVAL '82 days'),
('d9d00017-0000-0000-0000-000000000005', 'district9', 'd9c00004-0000-0000-0000-000000000004', 4, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Lance M. Africk.', 'Court', false, false, NOW() - INTERVAL '80 days'),
('d9d00018-0000-0000-0000-000000000006', 'district9', 'd9c00004-0000-0000-0000-000000000004', 5, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES including financial records and cryptocurrency transaction logs.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '70 days'),
('d9d00019-0000-0000-0000-000000000007', 'district9', 'd9c00004-0000-0000-0000-000000000004', 6, 'discovery_response', 'DEFENDANT''S DISCOVERY RESPONSE and reciprocal disclosures.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '55 days'),
('d9d0001a-0000-0000-0000-000000000008', 'district9', 'd9c00004-0000-0000-0000-000000000004', 7, 'motion', 'MOTION to suppress evidence obtained from warrantless search of defendant''s office filed by defendant Petrov.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '40 days'),
('d9d0001b-0000-0000-0000-000000000009', 'district9', 'd9c00004-0000-0000-0000-000000000004', 8, 'response', 'RESPONSE in Opposition to Motion to Suppress filed by United States.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '30 days'),
('d9d0001c-0000-0000-0000-000000000001', 'district9', 'd9c00004-0000-0000-0000-000000000004', 9, 'reply', 'REPLY in Support of Motion to Suppress filed by defendant Petrov.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '23 days'),
('d9d0001d-0000-0000-0000-000000000002', 'district9', 'd9c00004-0000-0000-0000-000000000004', 10, 'hearing_notice', 'NOTICE of hearing on Motion to Suppress set for 03/10/2026 before Hon. Lance M. Africk.', 'Court', false, false, NOW() - INTERVAL '14 days')
ON CONFLICT (id) DO NOTHING;

-- Case 5: Jackson (trial_ready) — 12 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9d0001e-0000-0000-0000-000000000003', 'district9', 'd9c00005-0000-0000-0000-000000000005', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Marcus Jackson with 18 U.S.C. 922(g)(1) and 26 U.S.C. 5861(d). (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '120 days'),
('d9d0001f-0000-0000-0000-000000000004', 'district9', 'd9c00005-0000-0000-0000-000000000005', 2, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Catherine L. Whitfield on behalf of Marcus Jackson.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '116 days'),
('d9d00020-0000-0000-0000-000000000005', 'district9', 'd9c00005-0000-0000-0000-000000000005', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Ronnie Abrams. Arraignment held. Defendant entered plea of not guilty. Bail set at $250,000 surety bond.', 'Clerk', false, false, NOW() - INTERVAL '110 days'),
('d9d00021-0000-0000-0000-000000000006', 'district9', 'd9c00005-0000-0000-0000-000000000005', 4, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '108 days'),
('d9d00022-0000-0000-0000-000000000007', 'district9', 'd9c00005-0000-0000-0000-000000000005', 5, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES including ATF trace reports and ballistics analysis.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '95 days'),
('d9d00023-0000-0000-0000-000000000008', 'district9', 'd9c00005-0000-0000-0000-000000000005', 6, 'discovery_response', 'DEFENDANT''S DISCOVERY RESPONSE and notice of expert witness.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '80 days'),
('d9d00024-0000-0000-0000-000000000009', 'district9', 'd9c00005-0000-0000-0000-000000000005', 7, 'motion', 'MOTION in limine to exclude prior conviction evidence filed by defendant Jackson.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '60 days'),
('d9d00025-0000-0000-0000-000000000001', 'district9', 'd9c00005-0000-0000-0000-000000000005', 8, 'response', 'RESPONSE in Opposition to Motion in Limine filed by United States.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '50 days'),
('d9d00026-0000-0000-0000-000000000002', 'district9', 'd9c00005-0000-0000-0000-000000000005', 9, 'order', 'ORDER granting in part and denying in part Motion in Limine. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '35 days'),
('d9d00027-0000-0000-0000-000000000003', 'district9', 'd9c00005-0000-0000-0000-000000000005', 10, 'witness_list', 'WITNESS LIST filed by United States. (12 witnesses)', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '21 days'),
('d9d00028-0000-0000-0000-000000000004', 'district9', 'd9c00005-0000-0000-0000-000000000005', 11, 'witness_list', 'WITNESS LIST filed by defendant Jackson. (4 witnesses)', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '21 days'),
('d9d00029-0000-0000-0000-000000000005', 'district9', 'd9c00005-0000-0000-0000-000000000005', 12, 'notice', 'NOTICE of trial readiness filed by United States. Trial set for 03/03/2026.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '7 days')
ON CONFLICT (id) DO NOTHING;

-- Case 6: Morrison (in_trial) — 14 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9d0002a-0000-0000-0000-000000000006', 'district9', 'd9c00006-0000-0000-0000-000000000006', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Linda Morrison with 18 U.S.C. 1347 and 18 U.S.C. 1341. (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '180 days'),
('d9d0002b-0000-0000-0000-000000000007', 'district9', 'd9c00006-0000-0000-0000-000000000006', 2, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Catherine L. Whitfield on behalf of Linda Morrison.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '175 days'),
('d9d0002c-0000-0000-0000-000000000008', 'district9', 'd9c00006-0000-0000-0000-000000000006', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Ronnie Abrams. Arraignment held. Defendant entered plea of not guilty. Bail set at $500,000 surety bond.', 'Clerk', false, false, NOW() - INTERVAL '170 days'),
('d9d0002d-0000-0000-0000-000000000009', 'district9', 'd9c00006-0000-0000-0000-000000000006', 4, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '168 days'),
('d9d0002e-0000-0000-0000-000000000001', 'district9', 'd9c00006-0000-0000-0000-000000000006', 5, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES including Medicare billing records and patient files.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '155 days'),
('d9d0002f-0000-0000-0000-000000000002', 'district9', 'd9c00006-0000-0000-0000-000000000006', 6, 'discovery_response', 'DEFENDANT''S DISCOVERY RESPONSE and reciprocal disclosures.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '135 days'),
('d9d00030-0000-0000-0000-000000000003', 'district9', 'd9c00006-0000-0000-0000-000000000006', 7, 'motion', 'MOTION to dismiss Count 2 for failure to state an offense filed by defendant Morrison.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '110 days'),
('d9d00031-0000-0000-0000-000000000004', 'district9', 'd9c00006-0000-0000-0000-000000000006', 8, 'response', 'RESPONSE in Opposition to Motion to Dismiss filed by United States.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '100 days'),
('d9d00032-0000-0000-0000-000000000005', 'district9', 'd9c00006-0000-0000-0000-000000000006', 9, 'order', 'ORDER denying Motion to Dismiss Count 2. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '85 days'),
('d9d00033-0000-0000-0000-000000000006', 'district9', 'd9c00006-0000-0000-0000-000000000006', 10, 'witness_list', 'WITNESS LIST filed by United States. (18 witnesses including cooperating witnesses)', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '42 days'),
('d9d00034-0000-0000-0000-000000000007', 'district9', 'd9c00006-0000-0000-0000-000000000006', 11, 'witness_list', 'WITNESS LIST filed by defendant Morrison. (6 witnesses)', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '42 days'),
('d9d00035-0000-0000-0000-000000000008', 'district9', 'd9c00006-0000-0000-0000-000000000006', 12, 'exhibit', 'EXHIBIT LIST filed by United States. (142 exhibits)', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '35 days'),
('d9d00036-0000-0000-0000-000000000009', 'district9', 'd9c00006-0000-0000-0000-000000000006', 13, 'hearing_minutes', 'MINUTE ENTRY for jury selection proceedings held before Hon. Ronnie Abrams. Jury of 12 plus 4 alternates selected and sworn.', 'Clerk', false, false, NOW() - INTERVAL '7 days'),
('d9d00037-0000-0000-0000-000000000001', 'district9', 'd9c00006-0000-0000-0000-000000000006', 14, 'hearing_minutes', 'MINUTE ENTRY for trial proceedings Day 1 held before Hon. Ronnie Abrams. Government''s opening statement delivered. First three witnesses called.', 'Clerk', false, false, NOW() - INTERVAL '5 days')
ON CONFLICT (id) DO NOTHING;

-- Case 7: Ahmed (sentenced) — 16 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9d00038-0000-0000-0000-000000000002', 'district9', 'd9c00007-0000-0000-0000-000000000007', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Farooq Ahmed with 26 U.S.C. 7201 and 26 U.S.C. 7206(1). (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '365 days'),
('d9d00039-0000-0000-0000-000000000003', 'district9', 'd9c00007-0000-0000-0000-000000000007', 2, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Marcus J. Rivera on behalf of Farooq Ahmed.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '360 days'),
('d9d0003a-0000-0000-0000-000000000004', 'district9', 'd9c00007-0000-0000-0000-000000000007', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Ronnie Abrams. Arraignment held. Defendant entered plea of not guilty. Released on personal recognizance.', 'Clerk', false, false, NOW() - INTERVAL '355 days'),
('d9d0003b-0000-0000-0000-000000000005', 'district9', 'd9c00007-0000-0000-0000-000000000007', 4, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '350 days'),
('d9d0003c-0000-0000-0000-000000000006', 'district9', 'd9c00007-0000-0000-0000-000000000007', 5, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES including IRS records and financial statements.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '330 days'),
('d9d0003d-0000-0000-0000-000000000007', 'district9', 'd9c00007-0000-0000-0000-000000000007', 6, 'discovery_response', 'DEFENDANT''S DISCOVERY RESPONSE.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '300 days'),
('d9d0003e-0000-0000-0000-000000000008', 'district9', 'd9c00007-0000-0000-0000-000000000007', 7, 'motion', 'MOTION to suppress statements made during IRS interview filed by defendant Ahmed.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '280 days'),
('d9d0003f-0000-0000-0000-000000000009', 'district9', 'd9c00007-0000-0000-0000-000000000007', 8, 'order', 'ORDER denying Motion to Suppress. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '260 days'),
('d9d00040-0000-0000-0000-000000000001', 'district9', 'd9c00007-0000-0000-0000-000000000007', 9, 'notice', 'NOTICE of change of plea hearing.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '250 days'),
('d9d00041-0000-0000-0000-000000000002', 'district9', 'd9c00007-0000-0000-0000-000000000007', 10, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Ronnie Abrams. Change of plea hearing. Defendant entered plea of guilty to both counts pursuant to written plea agreement.', 'Clerk', false, false, NOW() - INTERVAL '245 days'),
('d9d00042-0000-0000-0000-000000000003', 'district9', 'd9c00007-0000-0000-0000-000000000007', 11, 'order', 'ORDER accepting guilty plea and referring defendant for presentence investigation. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '245 days'),
('d9d00043-0000-0000-0000-000000000004', 'district9', 'd9c00007-0000-0000-0000-000000000007', 12, 'notice', 'PRESENTENCE INVESTIGATION REPORT filed under seal.', 'U.S. Probation Office', true, false, NOW() - INTERVAL '200 days'),
('d9d00044-0000-0000-0000-000000000005', 'district9', 'd9c00007-0000-0000-0000-000000000007', 13, 'motion', 'MOTION for downward departure based on extraordinary acceptance of responsibility filed by defendant Ahmed.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '180 days'),
('d9d00045-0000-0000-0000-000000000006', 'district9', 'd9c00007-0000-0000-0000-000000000007', 14, 'hearing_minutes', 'MINUTE ENTRY for sentencing hearing held before Hon. Ronnie Abrams. Government and defense arguments heard.', 'Clerk', false, false, NOW() - INTERVAL '150 days'),
('d9d00046-0000-0000-0000-000000000007', 'district9', 'd9c00007-0000-0000-0000-000000000007', 15, 'sentence', 'JUDGMENT AND COMMITMENT ORDER. Defendant sentenced to 30 months custody, 3 years supervised release, $2,300,000 restitution. Signed by Hon. Ronnie Abrams.', 'Court', false, false, NOW() - INTERVAL '150 days'),
('d9d00047-0000-0000-0000-000000000008', 'district9', 'd9c00007-0000-0000-0000-000000000007', 16, 'judgment', 'JUDGMENT in a criminal case as to Farooq Ahmed. Guilty on Counts 1 and 2.', 'Court', false, false, NOW() - INTERVAL '150 days')
ON CONFLICT (id) DO NOTHING;

-- Case 8: Reeves (on_appeal) — 18 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d9d00048-0000-0000-0000-000000000009', 'district9', 'd9c00008-0000-0000-0000-000000000008', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Darnell Reeves with 21 U.S.C. 841(a)(1) and 21 U.S.C. 846. (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '540 days'),
('d9d00049-0000-0000-0000-000000000001', 'district9', 'd9c00008-0000-0000-0000-000000000008', 2, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Marcus J. Rivera on behalf of Darnell Reeves.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '535 days'),
('d9d0004a-0000-0000-0000-000000000002', 'district9', 'd9c00008-0000-0000-0000-000000000008', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Nancy G. Abudu. Arraignment held. Defendant entered plea of not guilty. Bail denied; defendant remanded.', 'Clerk', false, false, NOW() - INTERVAL '530 days'),
('d9d0004b-0000-0000-0000-000000000003', 'district9', 'd9c00008-0000-0000-0000-000000000008', 4, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Nancy G. Abudu.', 'Court', false, false, NOW() - INTERVAL '525 days'),
('d9d0004c-0000-0000-0000-000000000004', 'district9', 'd9c00008-0000-0000-0000-000000000008', 5, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES including DEA surveillance recordings and lab analysis reports.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '500 days'),
('d9d0004d-0000-0000-0000-000000000005', 'district9', 'd9c00008-0000-0000-0000-000000000008', 6, 'discovery_response', 'DEFENDANT''S DISCOVERY RESPONSE.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '470 days'),
('d9d0004e-0000-0000-0000-000000000006', 'district9', 'd9c00008-0000-0000-0000-000000000008', 7, 'motion', 'MOTION to suppress physical evidence from vehicle search filed by defendant Reeves.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '440 days'),
('d9d0004f-0000-0000-0000-000000000007', 'district9', 'd9c00008-0000-0000-0000-000000000008', 8, 'order', 'ORDER denying Motion to Suppress. Signed by Hon. Nancy G. Abudu.', 'Court', false, false, NOW() - INTERVAL '400 days'),
('d9d00050-0000-0000-0000-000000000008', 'district9', 'd9c00008-0000-0000-0000-000000000008', 9, 'witness_list', 'WITNESS LIST filed by United States. (8 witnesses)', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '350 days'),
('d9d00051-0000-0000-0000-000000000009', 'district9', 'd9c00008-0000-0000-0000-000000000008', 10, 'hearing_minutes', 'MINUTE ENTRY for jury trial Day 1 through Day 5 held before Hon. Nancy G. Abudu.', 'Clerk', false, false, NOW() - INTERVAL '300 days'),
('d9d00052-0000-0000-0000-000000000001', 'district9', 'd9c00008-0000-0000-0000-000000000008', 11, 'verdict', 'VERDICT. Jury returns verdict of guilty on all counts as to Darnell Reeves.', 'Clerk', false, false, NOW() - INTERVAL '290 days'),
('d9d00053-0000-0000-0000-000000000002', 'district9', 'd9c00008-0000-0000-0000-000000000008', 12, 'notice', 'PRESENTENCE INVESTIGATION REPORT filed under seal.', 'U.S. Probation Office', true, false, NOW() - INTERVAL '240 days'),
('d9d00054-0000-0000-0000-000000000003', 'district9', 'd9c00008-0000-0000-0000-000000000008', 13, 'sentence', 'JUDGMENT AND COMMITMENT ORDER. Defendant sentenced to 180 months custody, 5 years supervised release. Signed by Hon. Nancy G. Abudu.', 'Court', false, false, NOW() - INTERVAL '200 days'),
('d9d00055-0000-0000-0000-000000000004', 'district9', 'd9c00008-0000-0000-0000-000000000008', 14, 'judgment', 'JUDGMENT in a criminal case as to Darnell Reeves. Guilty on Counts 1 and 2.', 'Court', false, false, NOW() - INTERVAL '200 days'),
('d9d00056-0000-0000-0000-000000000005', 'district9', 'd9c00008-0000-0000-0000-000000000008', 15, 'notice_of_appeal', 'NOTICE OF APPEAL filed by defendant Darnell Reeves.', 'PD Marcus J. Rivera', false, false, NOW() - INTERVAL '190 days'),
('d9d00057-0000-0000-0000-000000000006', 'district9', 'd9c00008-0000-0000-0000-000000000008', 16, 'substitution', 'ORDER granting substitution of counsel. Catherine L. Whitfield substituted for Marcus J. Rivera as appellate counsel.', 'Court', false, false, NOW() - INTERVAL '170 days'),
('d9d00058-0000-0000-0000-000000000007', 'district9', 'd9c00008-0000-0000-0000-000000000008', 17, 'appeal_brief', 'APPELLANT''S OPENING BRIEF challenging sufficiency of evidence and sentencing guidelines calculation.', 'Catherine L. Whitfield', false, false, NOW() - INTERVAL '120 days'),
('d9d00059-0000-0000-0000-000000000008', 'district9', 'd9c00008-0000-0000-0000-000000000008', 18, 'appeal_brief', 'GOVERNMENT''S ANSWERING BRIEF in opposition to appeal.', 'AUSA Sarah K. Mitchell', false, false, NOW() - INTERVAL '60 days')
ON CONFLICT (id) DO NOTHING;

-- Case 9: Gonzalez (plea_negotiations) — 8 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12d0001-0000-0000-0000-000000000001', 'district12', 'd12c0001-0000-0000-0000-000000000001', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Rafael Gonzalez with 8 U.S.C. 1324(a)(1)(A) and 18 U.S.C. 1546(a). (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '75 days'),
('d12d0002-0000-0000-0000-000000000002', 'district12', 'd12c0001-0000-0000-0000-000000000001', 2, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Robert A. Blackwell on behalf of Rafael Gonzalez.', 'PD Robert A. Blackwell', false, false, NOW() - INTERVAL '71 days'),
('d12d0003-0000-0000-0000-000000000003', 'district12', 'd12c0001-0000-0000-0000-000000000001', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Georgia N. Alexakis. Arraignment held. Defendant entered plea of not guilty. Released on personal recognizance.', 'Clerk', false, false, NOW() - INTERVAL '65 days'),
('d12d0004-0000-0000-0000-000000000004', 'district12', 'd12c0001-0000-0000-0000-000000000001', 4, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Georgia N. Alexakis.', 'Court', false, false, NOW() - INTERVAL '63 days'),
('d12d0005-0000-0000-0000-000000000005', 'district12', 'd12c0001-0000-0000-0000-000000000001', 5, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES including border patrol records and travel documents.', 'AUSA Jennifer M. Huang', false, false, NOW() - INTERVAL '55 days'),
('d12d0006-0000-0000-0000-000000000006', 'district12', 'd12c0001-0000-0000-0000-000000000001', 6, 'discovery_response', 'DEFENDANT''S DISCOVERY RESPONSE.', 'PD Robert A. Blackwell', false, false, NOW() - INTERVAL '40 days'),
('d12d0007-0000-0000-0000-000000000007', 'district12', 'd12c0001-0000-0000-0000-000000000001', 7, 'notice', 'NOTICE of plea agreement negotiations; parties request continuance of trial date.', 'PD Robert A. Blackwell', false, false, NOW() - INTERVAL '25 days'),
('d12d0008-0000-0000-0000-000000000008', 'district12', 'd12c0001-0000-0000-0000-000000000001', 8, 'order', 'ORDER granting continuance. Trial date reset. Signed by Hon. Georgia N. Alexakis.', 'Court', false, false, NOW() - INTERVAL '20 days')
ON CONFLICT (id) DO NOTHING;

-- Case 10: Park (awaiting_sentencing) — 8 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12d0009-0000-0000-0000-000000000009', 'district12', 'd12c0002-0000-0000-0000-000000000002', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Sung-Ho Park with 18 U.S.C. 1030(a)(1) and 18 U.S.C. 793(e). (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '270 days'),
('d12d000a-0000-0000-0000-000000000001', 'district12', 'd12c0002-0000-0000-0000-000000000002', 2, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Elena V. Petrossian on behalf of Sung-Ho Park.', 'Elena V. Petrossian', false, false, NOW() - INTERVAL '265 days'),
('d12d000b-0000-0000-0000-000000000002', 'district12', 'd12c0002-0000-0000-0000-000000000002', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Amir H. Ali. Arraignment held. Defendant entered plea of not guilty. Bail denied; defendant remanded to custody.', 'Clerk', false, false, NOW() - INTERVAL '260 days'),
('d12d000c-0000-0000-0000-000000000003', 'district12', 'd12c0002-0000-0000-0000-000000000002', 4, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Amir H. Ali.', 'Court', false, false, NOW() - INTERVAL '255 days'),
('d12d000d-0000-0000-0000-000000000004', 'district12', 'd12c0002-0000-0000-0000-000000000002', 5, 'hearing_minutes', 'MINUTE ENTRY for jury trial proceedings held before Hon. Amir H. Ali. Trial lasted 8 days.', 'Clerk', false, false, NOW() - INTERVAL '80 days'),
('d12d000e-0000-0000-0000-000000000005', 'district12', 'd12c0002-0000-0000-0000-000000000002', 6, 'verdict', 'VERDICT. Jury returns verdict of guilty on all counts as to Sung-Ho Park.', 'Clerk', false, false, NOW() - INTERVAL '70 days'),
('d12d000f-0000-0000-0000-000000000006', 'district12', 'd12c0002-0000-0000-0000-000000000002', 7, 'notice', 'PRESENTENCE INVESTIGATION REPORT filed under seal.', 'U.S. Probation Office', true, false, NOW() - INTERVAL '30 days'),
('d12d0010-0000-0000-0000-000000000007', 'district12', 'd12c0002-0000-0000-0000-000000000002', 8, 'hearing_notice', 'NOTICE of sentencing hearing set for 03/15/2026 before Hon. Amir H. Ali.', 'Court', false, false, NOW() - INTERVAL '14 days')
ON CONFLICT (id) DO NOTHING;

-- Case 11: Thompson (dismissed) — 8 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12d0011-0000-0000-0000-000000000008', 'district12', 'd12c0003-0000-0000-0000-000000000003', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Brian Thompson with 26 U.S.C. 5861(d). (1 count)', 'Grand Jury', false, false, NOW() - INTERVAL '150 days'),
('d12d0012-0000-0000-0000-000000000009', 'district12', 'd12c0003-0000-0000-0000-000000000003', 2, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Elena V. Petrossian on behalf of Brian Thompson.', 'Elena V. Petrossian', false, false, NOW() - INTERVAL '146 days'),
('d12d0013-0000-0000-0000-000000000001', 'district12', 'd12c0003-0000-0000-0000-000000000003', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Amir H. Ali. Arraignment held. Defendant entered plea of not guilty. Released on personal recognizance.', 'Clerk', false, false, NOW() - INTERVAL '142 days'),
('d12d0014-0000-0000-0000-000000000002', 'district12', 'd12c0003-0000-0000-0000-000000000003', 4, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline and motion deadline. Signed by Hon. Amir H. Ali.', 'Court', false, false, NOW() - INTERVAL '140 days'),
('d12d0015-0000-0000-0000-000000000003', 'district12', 'd12c0003-0000-0000-0000-000000000003', 5, 'motion', 'MOTION to suppress evidence obtained from warrantless search of defendant''s residence filed by defendant Thompson.', 'Elena V. Petrossian', false, false, NOW() - INTERVAL '120 days'),
('d12d0016-0000-0000-0000-000000000004', 'district12', 'd12c0003-0000-0000-0000-000000000003', 6, 'response', 'RESPONSE in Opposition to Motion to Suppress filed by United States.', 'AUSA Jennifer M. Huang', false, false, NOW() - INTERVAL '110 days'),
('d12d0017-0000-0000-0000-000000000005', 'district12', 'd12c0003-0000-0000-0000-000000000003', 7, 'order', 'ORDER granting Motion to Suppress. Evidence from warrantless search excluded. Signed by Hon. Amir H. Ali.', 'Court', false, false, NOW() - INTERVAL '95 days'),
('d12d0018-0000-0000-0000-000000000006', 'district12', 'd12c0003-0000-0000-0000-000000000003', 8, 'order', 'ORDER dismissing case with prejudice. Government unable to proceed without suppressed evidence. Signed by Hon. Amir H. Ali.', 'Court', false, false, NOW() - INTERVAL '90 days')
ON CONFLICT (id) DO NOTHING;

-- Case 12: Volkov & Sokolov (pretrial_motions, sealed) — 10 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12d0019-0000-0000-0000-000000000007', 'district12', 'd12c0004-0000-0000-0000-000000000004', 1, 'indictment', 'SEALED INDICTMENT returned by Grand Jury charging Viktor Volkov and Dmitri Sokolov with 18 U.S.C. 1956(a)(2) and 18 U.S.C. 1956(h). (4 counts)', 'Grand Jury', true, false, NOW() - INTERVAL '45 days'),
('d12d001a-0000-0000-0000-000000000008', 'district12', 'd12c0004-0000-0000-0000-000000000004', 2, 'sealing_order', 'SEALING ORDER. Case and all filings to remain under seal pending arrest of defendants. Signed by Hon. Amir H. Ali.', 'Court', true, false, NOW() - INTERVAL '45 days'),
('d12d001b-0000-0000-0000-000000000009', 'district12', 'd12c0004-0000-0000-0000-000000000004', 3, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Thomas W. Nakamura (pro hac vice) on behalf of Viktor Volkov, Robert A. Blackwell on behalf of Dmitri Sokolov.', 'Multiple Counsel', true, false, NOW() - INTERVAL '40 days'),
('d12d001c-0000-0000-0000-000000000001', 'district12', 'd12c0004-0000-0000-0000-000000000004', 4, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Amir H. Ali. Arraignment held. Both defendants entered pleas of not guilty. Bail denied for both defendants.', 'Clerk', true, false, NOW() - INTERVAL '35 days'),
('d12d001d-0000-0000-0000-000000000002', 'district12', 'd12c0004-0000-0000-0000-000000000004', 5, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Amir H. Ali.', 'Court', true, false, NOW() - INTERVAL '33 days'),
('d12d001e-0000-0000-0000-000000000003', 'district12', 'd12c0004-0000-0000-0000-000000000004', 6, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES including foreign bank records and real estate transaction records.', 'AUSA Jennifer M. Huang', true, false, NOW() - INTERVAL '28 days'),
('d12d001f-0000-0000-0000-000000000004', 'district12', 'd12c0004-0000-0000-0000-000000000004', 7, 'motion', 'MOTION to compel discovery of classified intelligence materials filed by defendant Volkov.', 'Thomas W. Nakamura', true, false, NOW() - INTERVAL '20 days'),
('d12d0020-0000-0000-0000-000000000005', 'district12', 'd12c0004-0000-0000-0000-000000000004', 8, 'response', 'RESPONSE in Opposition to Motion to Compel, invoking Classified Information Procedures Act (CIPA). Filed by United States.', 'AUSA Jennifer M. Huang', true, true, NOW() - INTERVAL '14 days'),
('d12d0021-0000-0000-0000-000000000006', 'district12', 'd12c0004-0000-0000-0000-000000000004', 9, 'motion', 'MOTION for severance of trials filed by defendant Sokolov.', 'PD Robert A. Blackwell', true, false, NOW() - INTERVAL '10 days'),
('d12d0022-0000-0000-0000-000000000007', 'district12', 'd12c0004-0000-0000-0000-000000000004', 10, 'hearing_notice', 'NOTICE of hearing on pending motions set for 03/05/2026 before Hon. Amir H. Ali.', 'Court', true, false, NOW() - INTERVAL '5 days')
ON CONFLICT (id) DO NOTHING;

-- Case 13: Davis (arraigned, pro se) — 6 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12d0023-0000-0000-0000-000000000008', 'district12', 'd12c0005-0000-0000-0000-000000000005', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Raymond Davis with 18 U.S.C. 1343 and 18 U.S.C. 1028(a)(7). (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '25 days'),
('d12d0024-0000-0000-0000-000000000009', 'district12', 'd12c0005-0000-0000-0000-000000000005', 2, 'summons', 'SUMMONS issued as to Raymond Davis.', 'Clerk', false, false, NOW() - INTERVAL '24 days'),
('d12d0025-0000-0000-0000-000000000001', 'district12', 'd12c0005-0000-0000-0000-000000000005', 3, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Georgia N. Alexakis. Arraignment held. Defendant entered plea of not guilty. Defendant asserts right to self-representation.', 'Clerk', false, false, NOW() - INTERVAL '18 days'),
('d12d0026-0000-0000-0000-000000000002', 'district12', 'd12c0005-0000-0000-0000-000000000005', 4, 'minute_order', 'MINUTE ENTRY for Faretta hearing held before Hon. Georgia N. Alexakis. Court conducted colloquy regarding dangers of self-representation. Defendant''s waiver of counsel found knowing and voluntary.', 'Clerk', false, false, NOW() - INTERVAL '16 days'),
('d12d0027-0000-0000-0000-000000000003', 'district12', 'd12c0005-0000-0000-0000-000000000005', 5, 'order', 'ORDER granting defendant''s motion to proceed pro se. Standby counsel not requested. Signed by Hon. Georgia N. Alexakis.', 'Court', false, false, NOW() - INTERVAL '16 days'),
('d12d0028-0000-0000-0000-000000000004', 'district12', 'd12c0005-0000-0000-0000-000000000005', 6, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Georgia N. Alexakis.', 'Court', false, false, NOW() - INTERVAL '14 days')
ON CONFLICT (id) DO NOTHING;

-- Case 14: Hernandez (discovery) — 10 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12d0029-0000-0000-0000-000000000005', 'district12', 'd12c0006-0000-0000-0000-000000000006', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Miguel Hernandez with 21 U.S.C. 846 and 21 U.S.C. 856(a)(1). (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '50 days'),
('d12d002a-0000-0000-0000-000000000006', 'district12', 'd12c0006-0000-0000-0000-000000000006', 2, 'summons', 'SUMMONS issued as to Miguel Hernandez.', 'Clerk', false, false, NOW() - INTERVAL '49 days'),
('d12d002b-0000-0000-0000-000000000007', 'district12', 'd12c0006-0000-0000-0000-000000000006', 3, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Robert A. Blackwell on behalf of Miguel Hernandez.', 'PD Robert A. Blackwell', false, false, NOW() - INTERVAL '46 days'),
('d12d002c-0000-0000-0000-000000000008', 'district12', 'd12c0006-0000-0000-0000-000000000006', 4, 'minute_order', 'MINUTE ENTRY for proceedings held before Hon. Georgia N. Alexakis. Arraignment held. Defendant entered plea of not guilty. Bail set at $150,000 surety bond.', 'Clerk', false, false, NOW() - INTERVAL '40 days'),
('d12d002d-0000-0000-0000-000000000009', 'district12', 'd12c0006-0000-0000-0000-000000000006', 5, 'scheduling_order', 'SCHEDULING ORDER setting discovery deadline, motion deadline, and trial date. Signed by Hon. Georgia N. Alexakis.', 'Court', false, false, NOW() - INTERVAL '38 days'),
('d12d002e-0000-0000-0000-000000000001', 'district12', 'd12c0006-0000-0000-0000-000000000006', 6, 'discovery_request', 'GOVERNMENT''S INITIAL DISCOVERY DISCLOSURES including DEA surveillance reports and wiretap transcripts.', 'AUSA Jennifer M. Huang', false, false, NOW() - INTERVAL '30 days'),
('d12d002f-0000-0000-0000-000000000002', 'district12', 'd12c0006-0000-0000-0000-000000000006', 7, 'discovery_response', 'DEFENDANT''S DISCOVERY RESPONSE and reciprocal disclosures.', 'PD Robert A. Blackwell', false, false, NOW() - INTERVAL '20 days'),
('d12d0030-0000-0000-0000-000000000003', 'district12', 'd12c0006-0000-0000-0000-000000000006', 8, 'motion', 'MOTION to suppress wiretap evidence for failure to comply with Title III requirements filed by defendant Hernandez.', 'PD Robert A. Blackwell', false, false, NOW() - INTERVAL '14 days'),
('d12d0031-0000-0000-0000-000000000004', 'district12', 'd12c0006-0000-0000-0000-000000000006', 9, 'response', 'RESPONSE in Opposition to Motion to Suppress Wiretap Evidence filed by United States.', 'AUSA Jennifer M. Huang', false, false, NOW() - INTERVAL '7 days'),
('d12d0032-0000-0000-0000-000000000005', 'district12', 'd12c0006-0000-0000-0000-000000000006', 10, 'hearing_notice', 'NOTICE of hearing on Motion to Suppress set for 03/12/2026 before Hon. Georgia N. Alexakis.', 'Court', false, false, NOW() - INTERVAL '3 days')
ON CONFLICT (id) DO NOTHING;

-- Case 15: Carter (filed) — 3 entries
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed, is_ex_parte, date_filed)
VALUES
('d12d0033-0000-0000-0000-000000000006', 'district12', 'd12c0007-0000-0000-0000-000000000007', 1, 'indictment', 'INDICTMENT returned by Grand Jury charging Terrence Carter with 18 U.S.C. 1962(d) and 18 U.S.C. 1956(h). (2 counts)', 'Grand Jury', false, false, NOW() - INTERVAL '10 days'),
('d12d0034-0000-0000-0000-000000000007', 'district12', 'd12c0007-0000-0000-0000-000000000007', 2, 'summons', 'SUMMONS issued as to Terrence Carter. Initial appearance scheduled.', 'Clerk', false, false, NOW() - INTERVAL '10 days'),
('d12d0035-0000-0000-0000-000000000008', 'district12', 'd12c0007-0000-0000-0000-000000000007', 3, 'appearance', 'NOTICE OF ATTORNEY APPEARANCE by Robert A. Blackwell on behalf of Terrence Carter.', 'PD Robert A. Blackwell', false, false, NOW() - INTERVAL '7 days')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- CALENDAR EVENTS (~28 total)
-- ============================================================

-- Case 2: Chen (arraigned) — 3 events
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d9ea0001-0000-0000-0000-000000000001', 'district9', 'd9c00002-0000-0000-0000-000000000002', 'd9b00003-0000-0000-0000-000000000003', 'initial_appearance', NOW() - INTERVAL '25 days', 30, 'Courtroom 5C', 'Initial appearance of defendant Wei Chen on cybercrime charges.', '{"AUSA Mitchell","PD Rivera"}', true, 'completed', 'Defendant appeared via video. Bail set at $100,000 cash.'),
('d9ea0002-0000-0000-0000-000000000002', 'district9', 'd9c00002-0000-0000-0000-000000000002', 'd9b00001-0000-0000-0000-000000000001', 'arraignment', NOW() - INTERVAL '20 days', 30, 'Courtroom 1A', 'Arraignment of Wei Chen. Plea entered.', '{"AUSA Mitchell","PD Rivera"}', true, 'completed', 'Defendant arraigned. Plea of not guilty entered.'),
('d9ea0003-0000-0000-0000-000000000003', 'district9', 'd9c00002-0000-0000-0000-000000000002', 'd9b00001-0000-0000-0000-000000000001', 'status_conference', NOW() + INTERVAL '14 days', 30, 'Courtroom 1A', 'Status conference to discuss discovery progress and scheduling.', '{"AUSA Mitchell","PD Rivera"}', true, 'scheduled', '')
ON CONFLICT (id) DO NOTHING;

-- Case 3: Williams RICO (discovery) — 4 events
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d9ea0004-0000-0000-0000-000000000004', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'd9b00001-0000-0000-0000-000000000001', 'status_conference', NOW() - INTERVAL '30 days', 60, 'Courtroom 1A', 'Status conference on RICO discovery issues. Multiple defense counsel present.', '{"AUSA Mitchell","Whitfield","PD Rivera","Okonkwo"}', true, 'completed', 'Discovery disputes discussed. Court ordered phased production schedule.'),
('d9ea0005-0000-0000-0000-000000000005', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'd9b00001-0000-0000-0000-000000000001', 'motion_hearing', NOW() - INTERVAL '15 days', 60, 'Courtroom 1A', 'Hearing on defense motion to compel additional discovery of wiretap materials.', '{"AUSA Mitchell","Whitfield","PD Rivera","Okonkwo"}', true, 'completed', 'Motion granted in part, denied in part. Government ordered to produce unredacted wiretap logs.'),
('d9ea0006-0000-0000-0000-000000000006', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'd9b00001-0000-0000-0000-000000000001', 'motion_hearing', NOW() + INTERVAL '7 days', 60, 'Courtroom 1A', 'Hearing on government motion for protective order regarding confidential informant identities.', '{"AUSA Mitchell","Whitfield","PD Rivera","Okonkwo"}', true, 'scheduled', ''),
('d9ea0007-0000-0000-0000-000000000007', 'district9', 'd9c00003-0000-0000-0000-000000000003', 'd9b00001-0000-0000-0000-000000000001', 'pretrial_conference', NOW() + INTERVAL '30 days', 60, 'Courtroom 1A', 'Pretrial conference to set trial date and address remaining discovery issues.', '{"AUSA Mitchell","Whitfield","PD Rivera","Okonkwo"}', true, 'scheduled', '')
ON CONFLICT (id) DO NOTHING;

-- Case 4: Petrov (pretrial_motions) — 2 events
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d9ea0008-0000-0000-0000-000000000008', 'district9', 'd9c00004-0000-0000-0000-000000000004', 'd9b00002-0000-0000-0000-000000000002', 'motion_hearing', NOW() - INTERVAL '10 days', 60, 'Courtroom 3B', 'Hearing on defense motion to suppress evidence obtained from warrantless search of cryptocurrency exchange records.', '{"AUSA Mitchell","Whitfield"}', true, 'completed', 'Arguments heard. Court took matter under advisement.'),
('d9ea0009-0000-0000-0000-000000000009', 'district9', 'd9c00004-0000-0000-0000-000000000004', 'd9b00002-0000-0000-0000-000000000002', 'evidentiary_hearing', NOW() + INTERVAL '14 days', 120, 'Courtroom 3B', 'Evidentiary hearing on suppression motion. Government to present testimony of lead investigator.', '{"AUSA Mitchell","Whitfield"}', true, 'scheduled', '')
ON CONFLICT (id) DO NOTHING;

-- Case 5: Jackson (trial_ready) — 2 events
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d9ea000a-0000-0000-0000-000000000001', 'district9', 'd9c00005-0000-0000-0000-000000000005', 'd9b00001-0000-0000-0000-000000000001', 'pretrial_conference', NOW() - INTERVAL '7 days', 60, 'Courtroom 1A', 'Final pretrial conference. Jury instructions, exhibit lists, and witness lists finalized.', '{"AUSA Mitchell","Whitfield"}', true, 'completed', 'Trial set to begin. Both sides ready. Estimated 5 trial days.'),
('d9ea000b-0000-0000-0000-000000000002', 'district9', 'd9c00005-0000-0000-0000-000000000005', 'd9b00001-0000-0000-0000-000000000001', 'jury_trial', NOW() + INTERVAL '5 days', 480, 'Courtroom 1A', 'JURY TRIAL — United States v. Jackson. Jury selection and opening statements.', '{"AUSA Mitchell","Whitfield"}', true, 'scheduled', 'URGENT: Speedy Trial Act deadline approaching. 5 days remaining.')
ON CONFLICT (id) DO NOTHING;

-- Case 6: Morrison (in_trial) — 5 events
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d9ea000c-0000-0000-0000-000000000003', 'district9', 'd9c00006-0000-0000-0000-000000000006', 'd9b00001-0000-0000-0000-000000000001', 'jury_trial', NOW() - INTERVAL '3 days', 480, 'Courtroom 1A', 'Trial Day 1 — Jury selection completed. Government opening statement delivered.', '{"AUSA Mitchell","Whitfield"}', true, 'completed', 'Jury of 12 plus 2 alternates seated. Government opening focused on fabricated billing records.'),
('d9ea000d-0000-0000-0000-000000000004', 'district9', 'd9c00006-0000-0000-0000-000000000006', 'd9b00001-0000-0000-0000-000000000001', 'jury_trial', NOW() - INTERVAL '2 days', 480, 'Courtroom 1A', 'Trial Day 2 — Government witnesses: FBI Special Agent Thompson, forensic accountant Dr. Patel.', '{"AUSA Mitchell","Whitfield"}', true, 'completed', 'Government presented documentary evidence of phantom billing. Cross-examination ongoing.'),
('d9ea000e-0000-0000-0000-000000000005', 'district9', 'd9c00006-0000-0000-0000-000000000006', 'd9b00001-0000-0000-0000-000000000001', 'jury_trial', NOW() - INTERVAL '1 day', 480, 'Courtroom 1A', 'Trial Day 3 — Government witnesses continued. Defense cross-examination of forensic accountant.', '{"AUSA Mitchell","Whitfield"}', true, 'completed', 'Heated cross-examination of Dr. Patel on methodology. Jury attentive.'),
('d9ea000f-0000-0000-0000-000000000006', 'district9', 'd9c00006-0000-0000-0000-000000000006', 'd9b00001-0000-0000-0000-000000000001', 'jury_trial', NOW(), 480, 'Courtroom 1A', 'Trial Day 4 — Government rests. Defense to begin case-in-chief.', '{"AUSA Mitchell","Whitfield"}', true, 'in_progress', 'Government rested after presenting 8 witnesses. Defense opening statement expected this afternoon.'),
('d9ea0010-0000-0000-0000-000000000007', 'district9', 'd9c00006-0000-0000-0000-000000000006', 'd9b00001-0000-0000-0000-000000000001', 'jury_trial', NOW() + INTERVAL '1 day', 480, 'Courtroom 1A', 'Trial Day 5 — Defense case-in-chief continues. Closing arguments if time permits.', '{"AUSA Mitchell","Whitfield"}', true, 'scheduled', '')
ON CONFLICT (id) DO NOTHING;

-- Case 7: Ahmed (sentenced) — 1 event
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d9ea0011-0000-0000-0000-000000000008', 'district9', 'd9c00007-0000-0000-0000-000000000007', 'd9b00001-0000-0000-0000-000000000001', 'sentencing', NOW() - INTERVAL '45 days', 120, 'Courtroom 1A', 'Sentencing hearing for Farooq Ahmed following guilty plea to tax evasion and filing false returns.', '{"AUSA Mitchell","PD Rivera"}', true, 'completed', 'Defendant sentenced to 36 months imprisonment, 3 years supervised release, restitution of $2.3M.')
ON CONFLICT (id) DO NOTHING;

-- Case 8: Reeves (on_appeal) — 1 event
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d9ea0012-0000-0000-0000-000000000009', 'district9', 'd9c00008-0000-0000-0000-000000000008', 'd9b00004-0000-0000-0000-000000000004', 'motion_hearing', NOW() + INTERVAL '60 days', 60, 'Courtroom 1A', 'Oral argument on appeal. Defense challenges sufficiency of evidence and sentencing calculation.', '{"AUSA Mitchell","Whitfield"}', true, 'scheduled', 'Appellate panel to hear argument on two issues: sufficiency of evidence and guidelines calculation.')
ON CONFLICT (id) DO NOTHING;

-- Case 9: Gonzalez (plea_negotiations) — 1 event
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d12ea001-0000-0000-0000-000000000001', 'district12', 'd12c0001-0000-0000-0000-000000000001', 'd12b0002-0000-0000-0000-000000000002', 'plea_hearing', NOW() + INTERVAL '10 days', 60, 'Courtroom 4B', 'Change of plea hearing. Defendant expected to enter guilty plea pursuant to plea agreement.', '{"AUSA Huang","PD Blackwell"}', true, 'scheduled', 'Plea agreement covers Count 1 only. Government to dismiss Count 2 at sentencing.')
ON CONFLICT (id) DO NOTHING;

-- Case 10: Park (awaiting_sentencing) — 1 event
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d12ea002-0000-0000-0000-000000000002', 'district12', 'd12c0002-0000-0000-0000-000000000002', 'd12b0001-0000-0000-0000-000000000001', 'sentencing', NOW() + INTERVAL '21 days', 120, 'Courtroom 2A', 'Sentencing hearing for Sung-Ho Park following jury conviction on both counts.', '{"AUSA Huang","Petrossian"}', true, 'scheduled', 'PSR filed. Guidelines range 87-108 months. Government seeking upward departure based on national security harm.')
ON CONFLICT (id) DO NOTHING;

-- Case 11: Thompson (dismissed) — 1 event
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d12ea003-0000-0000-0000-000000000003', 'district12', 'd12c0003-0000-0000-0000-000000000003', 'd12b0001-0000-0000-0000-000000000001', 'motion_hearing', NOW() - INTERVAL '60 days', 60, 'Courtroom 2A', 'Hearing on defense motion to suppress evidence. Case ultimately dismissed.', '{"AUSA Huang","Petrossian"}', true, 'completed', 'Court granted suppression motion. Government subsequently moved to dismiss all charges.')
ON CONFLICT (id) DO NOTHING;

-- Case 12: Volkov & Sokolov (sealed pretrial) — 1 event
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d12ea004-0000-0000-0000-000000000004', 'district12', 'd12c0004-0000-0000-0000-000000000004', 'd12b0001-0000-0000-0000-000000000001', 'motion_hearing', NOW() + INTERVAL '14 days', 60, 'Courtroom 2A', 'SEALED hearing on defense motions for discovery and bail reconsideration.', '{"AUSA Huang","Nakamura","PD Blackwell"}', false, 'scheduled', 'Sealed proceeding. All filings under protective order.')
ON CONFLICT (id) DO NOTHING;

-- Case 13: Davis (arraigned, pro se) — 1 event
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d12ea005-0000-0000-0000-000000000005', 'district12', 'd12c0005-0000-0000-0000-000000000005', 'd12b0002-0000-0000-0000-000000000002', 'status_conference', NOW() + INTERVAL '7 days', 30, 'Courtroom 4B', 'Status conference on pro se defendant discovery obligations and scheduling.', '{"AUSA Huang","Raymond Davis (pro se)"}', true, 'scheduled', 'Court to inquire into defendant ability to proceed pro se.')
ON CONFLICT (id) DO NOTHING;

-- Case 14: Hernandez (discovery) — 2 events
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d12ea006-0000-0000-0000-000000000006', 'district12', 'd12c0006-0000-0000-0000-000000000006', 'd12b0002-0000-0000-0000-000000000002', 'status_conference', NOW() - INTERVAL '20 days', 30, 'Courtroom 4B', 'Status conference on discovery progress in drug distribution case.', '{"AUSA Huang","PD Blackwell"}', true, 'completed', 'Parties reported progress. Additional time granted for wiretap transcription review.'),
('d12ea007-0000-0000-0000-000000000007', 'district12', 'd12c0006-0000-0000-0000-000000000006', 'd12b0002-0000-0000-0000-000000000002', 'scheduling_conference', NOW() + INTERVAL '10 days', 30, 'Courtroom 4B', 'Discovery conference to review wiretap evidence production and set motion deadline.', '{"AUSA Huang","PD Blackwell"}', true, 'scheduled', '')
ON CONFLICT (id) DO NOTHING;

-- Case 15: Carter (filed) — 1 event
INSERT INTO calendar_events (id, court_id, case_id, judge_id, event_type, scheduled_date, duration_minutes, courtroom, description, participants, is_public, status, notes)
VALUES
('d12ea008-0000-0000-0000-000000000008', 'district12', 'd12c0007-0000-0000-0000-000000000007', 'd12b0003-0000-0000-0000-000000000003', 'initial_appearance', NOW() + INTERVAL '3 days', 30, 'Courtroom 6A', 'Initial appearance and arraignment of Terrence Carter on RICO charges.', '{"AUSA Huang","PD Blackwell"}', true, 'scheduled', '')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- DEADLINES (~22 total)
-- ============================================================

-- Case 2: Chen (arraigned) — 2 deadlines
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district9', 'd9eb0001-0000-0000-0000-000000000001', 'd9c00002-0000-0000-0000-000000000002', 'Government initial discovery production', 'Fed. R. Crim. P. 16(a)', NOW() + INTERVAL '7 days', 'open', 'Government must produce initial discovery materials including Jencks material.'),
('district9', 'd9eb0002-0000-0000-0000-000000000002', 'd9c00002-0000-0000-0000-000000000002', 'Defense reciprocal discovery', 'Fed. R. Crim. P. 16(b)', NOW() + INTERVAL '21 days', 'open', 'Defense reciprocal discovery due 14 days after government production.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 3: Williams RICO (discovery) — 2 deadlines
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district9', 'd9eb0003-0000-0000-0000-000000000003', 'd9c00003-0000-0000-0000-000000000003', 'Government supplemental discovery production (RICO)', 'Fed. R. Crim. P. 16', NOW() - INTERVAL '5 days', 'expired', 'OVERDUE: Government was ordered to produce unredacted wiretap logs. Defendant Simmons counsel has filed motion to compel.'),
('district9', 'd9eb0004-0000-0000-0000-000000000004', 'd9c00003-0000-0000-0000-000000000003', 'Pretrial motions filing deadline', 'Local Rule 12.1', NOW() + INTERVAL '21 days', 'open', 'All pretrial motions due. Includes motions in limine, Daubert challenges, and severance motions.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 4: Petrov (pretrial_motions) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district9', 'd9eb0005-0000-0000-0000-000000000005', 'd9c00004-0000-0000-0000-000000000004', 'Government response to suppression motion', 'Fed. R. Crim. P. 12(d)', NOW() - INTERVAL '12 days', 'met', 'Government filed response opposing motion to suppress cryptocurrency exchange records.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 5: Jackson (trial_ready) — 2 deadlines
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district9', 'd9eb0006-0000-0000-0000-000000000006', 'd9c00005-0000-0000-0000-000000000005', 'SPEEDY TRIAL DEADLINE — trial must commence', '18 U.S.C. 3161 - Speedy Trial Act', NOW() + INTERVAL '5 days', 'open', 'CRITICAL: Speedy Trial Act 70-day clock expires. Trial MUST begin by this date or case subject to dismissal.'),
('district9', 'd9eb0007-0000-0000-0000-000000000007', 'd9c00005-0000-0000-0000-000000000005', 'Joint proposed jury instructions', 'Local Rule 51.1', NOW() + INTERVAL '3 days', 'open', 'Both parties to file jointly proposed jury instructions and verdict form.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 6: Morrison (in_trial) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district9', 'd9eb0008-0000-0000-0000-000000000008', 'd9c00006-0000-0000-0000-000000000006', 'Daily trial exhibit submissions', 'Standing Trial Order', NOW() - INTERVAL '1 day', 'met', 'Parties submitted exhibit lists and witness schedules for following trial day as required.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 7: Ahmed (sentenced) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district9', 'd9eb0009-0000-0000-0000-000000000009', 'd9c00007-0000-0000-0000-000000000007', 'Restitution payment schedule submission', 'Fed. R. Crim. P. 32', NOW() - INTERVAL '30 days', 'met', 'Defense filed proposed restitution payment schedule as ordered at sentencing.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 8: Reeves (on_appeal) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district9', 'd9eb000a-0000-0000-0000-000000000001', 'd9c00008-0000-0000-0000-000000000008', 'Appellant opening brief', 'Fed. R. App. P. 31', NOW() + INTERVAL '45 days', 'open', 'Defense appellate brief due. Challenging sufficiency of evidence and sentencing guidelines calculation.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 9: Gonzalez (plea_negotiations) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district12', 'd12eb001-0000-0000-0000-000000000001', 'd12c0001-0000-0000-0000-000000000001', 'Plea agreement submission', 'Fed. R. Crim. P. 11', NOW() + INTERVAL '7 days', 'open', 'Signed plea agreement to be filed with the court prior to change of plea hearing.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 10: Park (awaiting_sentencing) — 2 deadlines
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district12', 'd12eb002-0000-0000-0000-000000000002', 'd12c0002-0000-0000-0000-000000000002', 'PSR objection deadline', 'Fed. R. Crim. P. 32(f)', NOW() + INTERVAL '14 days', 'open', 'Parties to file objections to Presentence Investigation Report within 14 days of receipt.'),
('district12', 'd12eb003-0000-0000-0000-000000000003', 'd12c0002-0000-0000-0000-000000000002', 'Sentencing memoranda filing deadline', 'Fed. R. Crim. P. 32', NOW() + INTERVAL '18 days', 'open', 'Both parties to file sentencing memoranda including guidelines calculations and departure arguments.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 12: Volkov & Sokolov (sealed pretrial) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district12', 'd12eb004-0000-0000-0000-000000000004', 'd12c0004-0000-0000-0000-000000000004', 'Defense motions filing deadline', 'Fed. R. Crim. P. 12(b)', NOW() + INTERVAL '21 days', 'open', 'All pretrial motions due including motions to dismiss, suppress, and for bill of particulars.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 13: Davis (arraigned, pro se) — 3 deadlines
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district12', 'd12eb005-0000-0000-0000-000000000005', 'd12c0005-0000-0000-0000-000000000005', 'Pro se discovery request filing', 'Fed. R. Crim. P. 16', NOW() - INTERVAL '30 days', 'extended', 'Extended from original deadline. Pro se defendant granted additional time to formulate discovery requests.'),
('district12', 'd12eb006-0000-0000-0000-000000000006', 'd12c0005-0000-0000-0000-000000000005', 'Pro se discovery request filing (extended)', 'Fed. R. Crim. P. 16', NOW() - INTERVAL '15 days', 'extended', 'Second extension granted. Court expressed concern about repeated delays.'),
('district12', 'd12eb007-0000-0000-0000-000000000007', 'd12c0005-0000-0000-0000-000000000005', 'Pro se discovery request filing (final extension)', 'Fed. R. Crim. P. 16', NOW() + INTERVAL '5 days', 'open', 'FINAL extension. Court warned no further extensions will be granted. Defendant Davis must file by this date.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 14: Hernandez (discovery) — 2 deadlines
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district12', 'd12eb008-0000-0000-0000-000000000008', 'd12c0006-0000-0000-0000-000000000006', 'Government discovery production (wiretap evidence)', 'Fed. R. Crim. P. 16', NOW() - INTERVAL '10 days', 'extended', 'Extended twice due to volume of wiretap recordings requiring transcription. New deadline set by court.'),
('district12', 'd12eb009-0000-0000-0000-000000000009', 'd12c0006-0000-0000-0000-000000000006', 'Expert report deadline', 'Fed. R. Crim. P. 16(a)(1)(G)', NOW() + INTERVAL '28 days', 'open', 'Government expert reports on narcotics analysis and financial tracing due.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 15: Carter (filed) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district12', 'd12eb00a-0000-0000-0000-000000000001', 'd12c0007-0000-0000-0000-000000000007', 'Initial appearance deadline', '18 U.S.C. 3161(b)', NOW() + INTERVAL '3 days', 'open', 'Defendant must be brought before magistrate judge without unnecessary delay.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 1: Rodriguez (filed) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district9', 'd9eb000b-0000-0000-0000-000000000002', 'd9c00001-0000-0000-0000-000000000001', 'Arraignment scheduling deadline', 'Fed. R. Crim. P. 10', NOW() + INTERVAL '10 days', 'open', 'Defendant to be arraigned. Initial appearance completed; awaiting formal arraignment date.')
ON CONFLICT (court_id, id) DO NOTHING;

-- Case 11: Thompson (dismissed) — 1 deadline
INSERT INTO deadlines (court_id, id, case_id, title, rule_code, due_at, status, notes)
VALUES
('district12', 'd12eb00b-0000-0000-0000-000000000002', 'd12c0003-0000-0000-0000-000000000003', 'Government response to suppression motion', 'Fed. R. Crim. P. 12(d)', NOW() - INTERVAL '75 days', 'met', 'Government filed opposition to motion to suppress. Motion was subsequently granted and case dismissed.')
ON CONFLICT (court_id, id) DO NOTHING;

-- ============================================================
-- SPEEDY TRIAL CLOCKS (4 rows)
-- ============================================================

INSERT INTO speedy_trial (case_id, court_id, arrest_date, indictment_date, arraignment_date, trial_start_deadline, days_elapsed, days_remaining, is_tolled, waived)
VALUES
-- Case 2: Chen (arraigned) — early in timeline
('d9c00002-0000-0000-0000-000000000002', 'district9',
 NOW() - INTERVAL '45 days', NOW() - INTERVAL '40 days', NOW() - INTERVAL '30 days',
 (NOW() - INTERVAL '30 days') + INTERVAL '70 days',
 15, 55, false, false),
-- Case 4: Petrov (pretrial_motions) — tolled due to pending motion
('d9c00004-0000-0000-0000-000000000004', 'district9',
 NOW() - INTERVAL '90 days', NOW() - INTERVAL '85 days', NOW() - INTERVAL '70 days',
 (NOW() - INTERVAL '70 days') + INTERVAL '70 days',
 42, 28, true, false),
-- Case 5: Jackson (trial_ready) — CRITICAL: only 5 days remaining
('d9c00005-0000-0000-0000-000000000005', 'district9',
 NOW() - INTERVAL '95 days', NOW() - INTERVAL '90 days', NOW() - INTERVAL '80 days',
 (NOW() - INTERVAL '80 days') + INTERVAL '70 days',
 65, 5, false, false),
-- Case 14: Hernandez (discovery) — standard timeline
('d12c0006-0000-0000-0000-000000000006', 'district12',
 NOW() - INTERVAL '60 days', NOW() - INTERVAL '55 days', NOW() - INTERVAL '45 days',
 (NOW() - INTERVAL '45 days') + INTERVAL '70 days',
 30, 40, false, false)
ON CONFLICT (case_id) DO NOTHING;

-- ============================================================
-- EXCLUDABLE DELAYS (3 rows)
-- ============================================================

INSERT INTO excludable_delays (id, court_id, case_id, start_date, end_date, reason, statutory_reference, days_excluded, order_reference)
VALUES
-- Case 4: Petrov — pending suppression motion tolls clock
('d9ec0001-0000-0000-0000-000000000001', 'district9', 'd9c00004-0000-0000-0000-000000000004',
 NOW() - INTERVAL '70 days', NOW() - INTERVAL '42 days',
 'Pending motion to suppress evidence obtained from warrantless search of cryptocurrency exchange records',
 '18 U.S.C. 3161(h)(1)(D)', 28,
 'Order dated ' || to_char(NOW() - INTERVAL '70 days', 'MM/DD/YYYY') || ' granting stay pending resolution of suppression motion'),
-- Case 5: Jackson — defense continuance
('d9ec0002-0000-0000-0000-000000000002', 'district9', 'd9c00005-0000-0000-0000-000000000005',
 NOW() - INTERVAL '80 days', NOW() - INTERVAL '70 days',
 'Defense motion for continuance to prepare for trial and retain expert witness on firearms identification',
 '18 U.S.C. 3161(h)(7)(A)', 10,
 'Order dated ' || to_char(NOW() - INTERVAL '80 days', 'MM/DD/YYYY') || ' granting defense continuance'),
-- Case 5: Jackson — complex case designation
('d9ec0003-0000-0000-0000-000000000003', 'district9', 'd9c00005-0000-0000-0000-000000000005',
 NOW() - INTERVAL '70 days', NOW() - INTERVAL '55 days',
 'Complex case designation by court due to novel firearms identification issues and need for expert testimony',
 '18 U.S.C. 3161(h)(7)(B)(ii)', 15,
 'Order dated ' || to_char(NOW() - INTERVAL '70 days', 'MM/DD/YYYY') || ' designating case as complex under Speedy Trial Act')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- MOTIONS (~20 motions)
-- ============================================================

INSERT INTO motions (id, court_id, case_id, motion_type, filed_by, description, filed_date, status, ruling_date, ruling_text)
VALUES
-- Case 3 (RICO Williams): Severance — Granted
('d9af0001-0000-0000-0000-000000000001', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'Severance', 'Catherine L. Whitfield',
 'MOTION to Sever Defendant Derek Simmons from Joint Trial',
 NOW() - INTERVAL '20 days', 'Granted', NOW() - INTERVAL '20 days',
 'Motion granted. Defendant Simmons has demonstrated sufficient prejudice arising from joinder to warrant separate trial. The jury may be unable to compartmentalize evidence admissible only against co-defendants Williams and Brooks. Severance is ordered as to Count 4.'),
-- Case 3: Compel Discovery — Pending
('d9af0002-0000-0000-0000-000000000002', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'Compel', 'Catherine L. Whitfield',
 'MOTION to Compel Production of Financial Records from Meridian Financial Corp',
 NOW() - INTERVAL '5 days', 'Pending', NULL, NULL),
-- Case 3: Limine #1 — Pending (exclude character evidence)
('d9af0003-0000-0000-0000-000000000003', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'Limine', 'Catherine L. Whitfield',
 'MOTION in Limine to Exclude Character Evidence Regarding Prior Arrests Not Resulting in Conviction',
 NOW() - INTERVAL '3 days', 'Pending', NULL, NULL),
-- Case 3: Limine #2 — Denied (wiretap evidence)
('d9af0004-0000-0000-0000-000000000004', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'Limine', 'David R. Okonkwo',
 'MOTION in Limine to Exclude Wiretap Evidence Obtained Under Title III Authorization',
 NOW() - INTERVAL '15 days', 'Denied', NOW() - INTERVAL '15 days',
 'Motion denied. The Government has demonstrated compliance with all procedural requirements of Title III. The wiretap authorization was properly obtained, minimization protocols were followed, and the recordings are admissible.'),
-- Case 4 (Petrov): Suppress — Denied
('d9af0005-0000-0000-0000-000000000005', 'district9', 'd9c00004-0000-0000-0000-000000000004',
 'Suppress', 'Marcus J. Rivera',
 'MOTION to Suppress Evidence Obtained from Search of Cryptocurrency Exchange Records',
 NOW() - INTERVAL '10 days', 'Denied', NOW() - INTERVAL '10 days',
 'Motion denied. Evidence obtained pursuant to valid warrant issued by Magistrate Judge. The affidavit supporting the warrant established probable cause based on corroborated informant testimony and independent financial analysis.'),
-- Case 4: Dismiss — Pending
('d9af0006-0000-0000-0000-000000000006', 'district9', 'd9c00004-0000-0000-0000-000000000004',
 'Dismiss', 'Marcus J. Rivera',
 'MOTION to Dismiss Counts 3 and 4 for Insufficient Evidence',
 NOW() - INTERVAL '3 days', 'Pending', NULL, NULL),
-- Case 5 (Jackson): Continuance — Denied
('d9af0007-0000-0000-0000-000000000007', 'district9', 'd9c00005-0000-0000-0000-000000000005',
 'Continuance', 'David R. Okonkwo',
 'MOTION for Continuance of Trial Date to Retain Firearms Expert Witness',
 NOW() - INTERVAL '14 days', 'Denied', NOW() - INTERVAL '14 days',
 'Motion denied. The Speedy Trial Act deadline is imminent. Defendant has not shown that the ends of justice would be served by further continuance. Defense has had adequate time to secure expert testimony.'),
-- Case 5: Limine — Granted
('d9af0008-0000-0000-0000-000000000008', 'district9', 'd9c00005-0000-0000-0000-000000000005',
 'Limine', 'Sarah K. Mitchell',
 'GOVERNMENT''S MOTION in Limine to Admit Expert Testimony on Firearms Modification',
 NOW() - INTERVAL '7 days', 'Granted', NOW() - INTERVAL '7 days',
 'Motion granted. The Government''s firearms expert meets the qualifications standard under Daubert. Expert testimony regarding the modification of semi-automatic weapons to fully automatic capability is relevant and reliable.'),
-- Case 8 (Reeves): New Trial — Denied
('d9af0009-0000-0000-0000-000000000009', 'district9', 'd9c00008-0000-0000-0000-000000000008',
 'New Trial', 'Marcus J. Rivera',
 'MOTION for New Trial Pursuant to Federal Rule of Criminal Procedure 33',
 NOW() - INTERVAL '30 days', 'Denied', NOW() - INTERVAL '30 days',
 'Motion denied. Defendant fails to demonstrate that the verdict was against the great weight of evidence. The jury''s credibility determinations are entitled to deference, and the evidence presented at trial was sufficient to support the conviction on all counts.'),
-- Case 11 (Thompson): Dismiss — Granted
('d12af001-0000-0000-0000-000000000001', 'district12', 'd12c0003-0000-0000-0000-000000000003',
 'Dismiss', 'Robert A. Blackwell',
 'MOTION to Dismiss for Violation of Fourth Amendment Rights',
 NOW() - INTERVAL '60 days', 'Granted', NOW() - INTERVAL '60 days',
 'Motion granted. The Government has failed to establish sufficient evidence to proceed absent the suppressed evidence. The initial stop and subsequent search of defendant''s vehicle lacked reasonable suspicion. Case dismissed with prejudice.'),
-- Case 12 (Volkov): Seal — Granted
('d12af002-0000-0000-0000-000000000002', 'district12', 'd12c0004-0000-0000-0000-000000000004',
 'Other', 'Jennifer M. Huang',
 'MOTION to Seal Case and All Filings',
 NOW() - INTERVAL '20 days', 'Granted', NOW() - INTERVAL '20 days',
 'Motion granted. All filings in this matter shall be maintained under seal due to the ongoing nature of the international investigation and the risk of flight by subjects not yet apprehended. Public disclosure would jeopardize law enforcement objectives.'),
-- Case 13 (Davis): Extension #1 — Granted
('d12af003-0000-0000-0000-000000000003', 'district12', 'd12c0005-0000-0000-0000-000000000005',
 'Other', 'Elena V. Petrossian',
 'MOTION for Extension of Time to File Response to Government Discovery',
 NOW() - INTERVAL '45 days', 'Granted', NOW() - INTERVAL '45 days',
 'Motion granted. Defense counsel has shown good cause for the requested extension. Response deadline extended by 14 days.'),
-- Case 13: Extension #2 — Granted
('d12af004-0000-0000-0000-000000000004', 'district12', 'd12c0005-0000-0000-0000-000000000005',
 'Other', 'Elena V. Petrossian',
 'MOTION for Extension of Time to File Pretrial Motions',
 NOW() - INTERVAL '30 days', 'Granted', NOW() - INTERVAL '30 days',
 'Motion granted. Given the volume of discovery materials and complexity of the alleged scheme, additional time is warranted. Pretrial motion deadline extended by 21 days.'),
-- Case 13: Extension #3 — Granted
('d12af005-0000-0000-0000-000000000005', 'district12', 'd12c0005-0000-0000-0000-000000000005',
 'Other', 'Elena V. Petrossian',
 'MOTION for Extension of Time to File Expert Witness Disclosures',
 NOW() - INTERVAL '15 days', 'Granted', NOW() - INTERVAL '15 days',
 'Motion granted. Defense has retained a forensic accountant whose report is not yet complete. Expert disclosure deadline extended by 30 days.'),
-- Case 9 (Gonzalez): Accept Plea — Pending
('d12af006-0000-0000-0000-000000000006', 'district12', 'd12c0001-0000-0000-0000-000000000001',
 'Other', 'Robert A. Blackwell',
 'MOTION to Accept Plea Agreement',
 NOW() - INTERVAL '5 days', 'Pending', NULL, NULL),
-- Case 14 (Hernandez): Discovery — Pending
('d12af007-0000-0000-0000-000000000007', 'district12', 'd12c0006-0000-0000-0000-000000000006',
 'Discovery', 'Thomas W. Nakamura',
 'MOTION to Compel Additional Discovery Responses Regarding Confidential Informant Communications',
 NOW() - INTERVAL '5 days', 'Pending', NULL, NULL)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- EVIDENCE (~15 items)
-- ============================================================

INSERT INTO evidence (id, court_id, case_id, description, evidence_type, seized_date, seized_by, location, is_sealed)
VALUES
-- Case 3 (RICO): financial records
('d9ac0001-0000-0000-0000-000000000001', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'Meridian Financial Corp transaction records 2023-2025, approximately 14,000 pages of wire transfer documentation and account statements',
 'Documentary', NOW() - INTERVAL '90 days', 'FBI Financial Crimes Unit', 'FBI Evidence Vault, Rm 310', false),
-- Case 3: wiretap recordings
('d9ac0002-0000-0000-0000-000000000002', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'Court-authorized Title III wiretap recordings from three phone lines over 45-day surveillance period, totaling approximately 312 hours',
 'Digital', NOW() - INTERVAL '75 days', 'FBI', 'FBI Digital Evidence Lab, Rm 215', false),
-- Case 3: seized cash
('d9ac0003-0000-0000-0000-000000000003', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'U.S. currency totaling $2,347,500.00 seized from safe deposit boxes at First National Bank and Chase Manhattan',
 'Physical', NOW() - INTERVAL '60 days', 'FBI Special Agent James Torres', 'Court Evidence Locker, Room B-12', false),
-- Case 4 (Petrov): laptop
('d9ac0004-0000-0000-0000-000000000004', 'district9', 'd9c00004-0000-0000-0000-000000000004',
 'Dell Latitude laptop (Model 5520) seized from defendant residence containing cryptocurrency wallet software and financial records',
 'Digital', NOW() - INTERVAL '85 days', 'DEA Task Force', 'DEA Regional Evidence Facility', false),
-- Case 4: bank statements
('d9ac0005-0000-0000-0000-000000000005', 'district9', 'd9c00004-0000-0000-0000-000000000004',
 'Bank of America account statements for accounts ending in 4477 and 8812 showing suspicious wire transfers totaling $3.2 million over 18 months',
 'Documentary', NOW() - INTERVAL '80 days', 'IRS Criminal Investigation', 'U.S. Attorney Evidence Room', false),
-- Case 5 (Jackson): firearm
('d9ac0006-0000-0000-0000-000000000006', 'district9', 'd9c00005-0000-0000-0000-000000000005',
 'Glock 19 9mm handgun, serial #GKP4472, with modified selector switch enabling fully automatic fire',
 'Physical', NOW() - INTERVAL '100 days', 'ATF', 'ATF National Firearms Repository', false),
-- Case 5: surveillance video
('d9ac0007-0000-0000-0000-000000000007', 'district9', 'd9c00005-0000-0000-0000-000000000005',
 'Security camera footage from First National Bank parking lot showing defendant transferring weapons from vehicle on three separate occasions',
 'Digital', NOW() - INTERVAL '95 days', 'ATF', 'ATF Digital Evidence Storage', false),
-- Case 6 (Morrison, in_trial): 8 exhibits
('d9ac0008-0000-0000-0000-000000000008', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'Replica of medical billing terminal used to demonstrate phantom billing methodology to jury',
 'Demonstrative', NULL, NULL, 'Courtroom 1A Evidence Cart', false),
('d9ac0009-0000-0000-0000-000000000009', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'Medicare claim forms and Explanation of Benefits documents for 847 phantom patients, Exhibits 1-A through 1-H',
 'Documentary', NOW() - INTERVAL '180 days', 'HHS-OIG', 'U.S. Attorney Evidence Room', false),
('d9ac000a-0000-0000-0000-00000000000a', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'Corporate financial records from Morrison Medical Group Inc. showing discrepancies between reported revenue and actual patient services',
 'Documentary', NOW() - INTERVAL '175 days', 'FBI White Collar Crime Unit', 'U.S. Attorney Evidence Room', false),
('d9ac000b-0000-0000-0000-00000000000b', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'Email correspondence from defendant''s corporate email account discussing fabrication of patient records, 2,341 emails recovered',
 'Digital', NOW() - INTERVAL '170 days', 'FBI Cyber Division', 'FBI Digital Evidence Lab', false),
('d9ac000c-0000-0000-0000-00000000000c', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'Cell phone records from Verizon showing communications between defendant and co-conspirators during key dates of the scheme',
 'Digital', NOW() - INTERVAL '165 days', 'FBI', 'FBI Evidence Vault, Rm 310', false),
('d9ac000d-0000-0000-0000-00000000000d', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'DNA analysis report from latent samples recovered from falsified patient intake forms',
 'Forensic', NOW() - INTERVAL '140 days', 'FBI Laboratory Division', 'FBI Evidence Vault, Rm 310', false),
('d9ac000e-0000-0000-0000-00000000000e', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'Fingerprint comparison report matching defendant''s prints to fabricated medical records',
 'Forensic', NOW() - INTERVAL '135 days', 'FBI Laboratory Division', 'FBI Evidence Vault, Rm 310', false),
('d9ac000f-0000-0000-0000-00000000000f', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'Government Exhibit 50: Chronological timeline chart depicting the fraud scheme from inception through discovery, used during opening statement',
 'Demonstrative', NULL, NULL, 'Courtroom 1A Evidence Cart', false),
-- Case 12 (Volkov, sealed): financial documentation
('d12ac001-0000-0000-0000-000000000001', 'district12', 'd12c0004-0000-0000-0000-000000000004',
 'International wire transfer documentation from Volkov Holdings Ltd to offshore accounts in Cyprus and British Virgin Islands',
 'Documentary', NOW() - INTERVAL '70 days', 'FBI International Operations', 'Sealed Evidence Vault, Federal Courthouse', true)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- CUSTODY TRANSFERS (3 rows for Case 3 seized cash)
-- ============================================================

INSERT INTO custody_transfers (id, court_id, evidence_id, transferred_from, transferred_to, date, location, condition, notes)
VALUES
('d9ad0001-0000-0000-0000-000000000001', 'district9', 'd9ac0003-0000-0000-0000-000000000003',
 'FBI Special Agent James Torres', 'FBI Field Office Evidence Room',
 NOW() - INTERVAL '60 days', 'FBI Los Angeles Field Office',
 'Sealed evidence bag, counted and verified',
 'Currency counted in presence of two agents and photographed. Total: $2,347,500.00 in mixed denominations.'),
('d9ad0002-0000-0000-0000-000000000002', 'district9', 'd9ac0003-0000-0000-0000-000000000003',
 'FBI Evidence Custodian', 'U.S. Marshals Service',
 NOW() - INTERVAL '45 days', 'Federal Courthouse Annex',
 'Sealed, chain intact',
 'Transferred for secure courthouse storage pending trial. Seal verified by receiving officer.'),
('d9ad0003-0000-0000-0000-000000000003', 'district9', 'd9ac0003-0000-0000-0000-000000000003',
 'U.S. Marshal Deputy', 'Court Evidence Locker',
 NOW() - INTERVAL '30 days', 'Federal Courthouse, Room B-12',
 'Secured in court evidence locker',
 'Placed in high-security evidence locker for trial availability. Locker sealed with tamper-evident tape.')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- JUDICIAL ORDERS (~15 orders)
-- ============================================================

INSERT INTO judicial_orders (id, court_id, case_id, judge_id, order_type, title, content, status, is_sealed, signer_name, signed_at, issued_at, effective_date, related_motions)
VALUES
-- Case 2 (Chen): Scheduling Order
('d9ba0001-0000-0000-0000-000000000001', 'district9', 'd9c00002-0000-0000-0000-000000000002',
 'd9b00001-0000-0000-0000-000000000001', 'Scheduling', 'Scheduling Order',
 'IT IS HEREBY ORDERED that the following schedule shall govern pretrial proceedings in this matter: Discovery completion within 90 days; Pretrial motions due 30 days after close of discovery; Pretrial conference set for 14 days before trial. Counsel shall meet and confer regarding discovery within 10 business days of this Order.',
 'Filed', false, 'Hon. Ronnie Abrams',
 NOW() - INTERVAL '18 days', NOW() - INTERVAL '18 days', NOW() - INTERVAL '18 days',
 '{}'::UUID[]),
-- Case 3 (RICO Williams): Scheduling Order
('d9ba0002-0000-0000-0000-000000000002', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'd9b00001-0000-0000-0000-000000000001', 'Scheduling', 'Scheduling Order — Complex Case',
 'IT IS HEREBY ORDERED that given the complexity of this multi-defendant RICO prosecution, the following schedule shall apply: Government discovery production in rolling fashion over 120 days; Defense expert disclosures due 60 days before trial; Daubert motions due 45 days before trial. The Court designates this matter as complex under the Speedy Trial Act.',
 'Filed', false, 'Hon. Ronnie Abrams',
 NOW() - INTERVAL '40 days', NOW() - INTERVAL '40 days', NOW() - INTERVAL '40 days',
 '{}'::UUID[]),
-- Case 3: Protective Order
('d9ba0003-0000-0000-0000-000000000003', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'd9b00001-0000-0000-0000-000000000001', 'Protective', 'Protective Order Governing Discovery Materials',
 'IT IS HEREBY ORDERED that all discovery materials produced in this matter shall be subject to the following confidentiality restrictions: Materials designated CONFIDENTIAL may be disclosed only to counsel of record, retained experts, and court personnel. Financial records from Meridian Financial Corp shall be treated as HIGHLY CONFIDENTIAL and may not be copied or disseminated outside the defense team without prior court approval.',
 'Filed', false, 'Hon. Ronnie Abrams',
 NOW() - INTERVAL '25 days', NOW() - INTERVAL '25 days', NOW() - INTERVAL '25 days',
 '{}'::UUID[]),
-- Case 4 (Petrov): Detention Order
('d9ba0004-0000-0000-0000-000000000004', 'district9', 'd9c00004-0000-0000-0000-000000000004',
 'd9b00002-0000-0000-0000-000000000002', 'Detention', 'Order of Detention Pending Trial',
 'IT IS HEREBY ORDERED that defendant Aleksandr Petrov shall be detained pending trial. The Court finds by clear and convincing evidence that no condition or combination of conditions will reasonably assure the safety of the community and the appearance of the defendant. Defendant poses a significant flight risk given his foreign ties, access to substantial financial resources, and the severity of the charges carrying a maximum sentence of 20 years.',
 'Filed', false, 'Hon. Lance M. Africk',
 NOW() - INTERVAL '65 days', NOW() - INTERVAL '65 days', NOW() - INTERVAL '65 days',
 '{}'::UUID[]),
-- Case 5 (Jackson): Scheduling Order
('d9ba0005-0000-0000-0000-000000000005', 'district9', 'd9c00005-0000-0000-0000-000000000005',
 'd9b00001-0000-0000-0000-000000000001', 'Scheduling', 'Amended Scheduling Order',
 'IT IS HEREBY ORDERED that the trial in this matter shall proceed as scheduled. Discovery is closed. All pretrial motions have been resolved. The parties shall submit proposed jury instructions no later than 7 days before trial. Voir dire questionnaires shall be filed within 5 business days.',
 'Filed', false, 'Hon. Ronnie Abrams',
 NOW() - INTERVAL '60 days', NOW() - INTERVAL '60 days', NOW() - INTERVAL '60 days',
 '{}'::UUID[]),
-- Case 6 (Morrison): Scheduling Order
('d9ba0006-0000-0000-0000-000000000006', 'district9', 'd9c00006-0000-0000-0000-000000000006',
 'd9b00001-0000-0000-0000-000000000001', 'Scheduling', 'Trial Scheduling Order',
 'IT IS HEREBY ORDERED that trial in this matter shall commence on the date specified. The Government estimates a trial length of 3-4 weeks given the volume of financial evidence. Defense estimates 2 weeks for its case-in-chief. The Court shall sit Monday through Thursday, 9:00 AM to 4:30 PM, with a one-hour lunch recess.',
 'Filed', false, 'Hon. Ronnie Abrams',
 NOW() - INTERVAL '90 days', NOW() - INTERVAL '90 days', NOW() - INTERVAL '90 days',
 '{}'::UUID[]),
-- Case 7 (Ahmed): Sentencing Judgment
('d9ba0007-0000-0000-0000-000000000007', 'district9', 'd9c00007-0000-0000-0000-000000000007',
 'd9b00001-0000-0000-0000-000000000001', 'Sentencing', 'Judgment and Commitment Order',
 'IT IS THE JUDGMENT OF THE COURT that defendant Tariq Ahmed is hereby committed to the custody of the Bureau of Prisons for a term of 36 months, to be followed by 3 years of supervised release with standard and special conditions. Defendant shall pay restitution in the amount of $1,200,000.00 to the Internal Revenue Service. A fine of $50,000.00 is imposed. The Court has considered the factors set forth in 18 U.S.C. 3553(a) and the advisory Guidelines range.',
 'Filed', false, 'Hon. Ronnie Abrams',
 NOW() - INTERVAL '45 days', NOW() - INTERVAL '45 days', NOW() - INTERVAL '45 days',
 '{}'::UUID[]),
-- Case 8 (Reeves): Order Denying New Trial
('d9ba0008-0000-0000-0000-000000000008', 'district9', 'd9c00008-0000-0000-0000-000000000008',
 'd9b00004-0000-0000-0000-000000000004', 'Other', 'Order Denying Motion for New Trial',
 'IT IS HEREBY ORDERED that defendant''s Motion for New Trial pursuant to Fed. R. Crim. P. 33 is DENIED. The Court has carefully reviewed the trial record and finds the evidence sufficient to support the jury''s verdict. Defendant''s arguments regarding the weight of the evidence and alleged evidentiary errors do not meet the stringent standard for granting a new trial.',
 'Filed', false, 'Hon. Nancy G. Abudu',
 NOW() - INTERVAL '30 days', NOW() - INTERVAL '30 days', NOW() - INTERVAL '30 days',
 ARRAY['d9af0009-0000-0000-0000-000000000009']::UUID[]),
-- Case 9 (Gonzalez): Release Order
('d12ba001-0000-0000-0000-000000000001', 'district12', 'd12c0001-0000-0000-0000-000000000001',
 'd12b0002-0000-0000-0000-000000000002', 'Release', 'Order Setting Conditions of Release',
 'IT IS HEREBY ORDERED that defendant Maria Gonzalez shall be released on the following conditions: $25,000 unsecured bond; surrender of passport; electronic monitoring; residence restricted to the Northern District; report to Pretrial Services weekly; no contact with co-defendants or known associates involved in smuggling operations.',
 'Filed', false, 'Hon. Georgia N. Alexakis',
 NOW() - INTERVAL '35 days', NOW() - INTERVAL '35 days', NOW() - INTERVAL '35 days',
 '{}'::UUID[]),
-- Case 11 (Thompson): Dismissal Order
('d12ba002-0000-0000-0000-000000000002', 'district12', 'd12c0003-0000-0000-0000-000000000003',
 'd12b0001-0000-0000-0000-000000000001', 'Dismissal', 'Order of Dismissal With Prejudice',
 'IT IS HEREBY ORDERED that this case is DISMISSED WITH PREJUDICE. The Government''s remaining evidence is insufficient to sustain a conviction beyond a reasonable doubt. The Court having granted defendant''s motion to suppress the physical evidence obtained during the warrantless search, the Government concedes it cannot meet its burden. Defendant is discharged from all conditions of release.',
 'Filed', false, 'Hon. Amir H. Ali',
 NOW() - INTERVAL '60 days', NOW() - INTERVAL '60 days', NOW() - INTERVAL '60 days',
 ARRAY['d12af001-0000-0000-0000-000000000001']::UUID[]),
-- Case 12 (Volkov): Sealing Order
('d12ba003-0000-0000-0000-000000000003', 'district12', 'd12c0004-0000-0000-0000-000000000004',
 'd12b0001-0000-0000-0000-000000000001', 'Sealing', 'Order to Seal Case File',
 'IT IS HEREBY ORDERED that the entire case file, including all pleadings, motions, and exhibits, shall be maintained under seal. Public access to any filing is prohibited. The Government has demonstrated that unsealing would compromise an ongoing international investigation and endanger cooperating witnesses. This order shall remain in effect until further order of the Court.',
 'Filed', true, 'Hon. Amir H. Ali',
 NOW() - INTERVAL '50 days', NOW() - INTERVAL '50 days', NOW() - INTERVAL '50 days',
 ARRAY['d12af002-0000-0000-0000-000000000002']::UUID[]),
-- Case 14 (Hernandez): Discovery Order
('d12ba004-0000-0000-0000-000000000004', 'district12', 'd12c0006-0000-0000-0000-000000000006',
 'd12b0002-0000-0000-0000-000000000002', 'Discovery', 'Order Compelling Government Disclosure',
 'IT IS HEREBY ORDERED that the Government shall produce all communications with confidential informant CI-2025-1147, including but not limited to debriefing notes, payment records, and prior testimony in other proceedings, within 14 days of this Order. The Government shall also disclose any impeachment material related to the informant under Brady v. Maryland and Giglio v. United States.',
 'Filed', false, 'Hon. Georgia N. Alexakis',
 NOW() - INTERVAL '25 days', NOW() - INTERVAL '25 days', NOW() - INTERVAL '25 days',
 ARRAY['d12af007-0000-0000-0000-000000000007']::UUID[])
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- SENTENCING RECORDS (3 rows)
-- ============================================================

INSERT INTO sentencing (id, court_id, case_id, defendant_id, judge_id,
    base_offense_level, specific_offense_level, adjusted_offense_level, total_offense_level,
    criminal_history_category, criminal_history_points,
    guidelines_range_low_months, guidelines_range_high_months,
    custody_months, probation_months,
    fine_amount, restitution_amount, forfeiture_amount, special_assessment,
    departure_type, departure_reason, variance_type, variance_justification,
    supervised_release_months, appeal_waiver, sentencing_date, judgment_date)
VALUES
-- Case 7: Ahmed (tax_offense, sentenced)
('d9bb0001-0000-0000-0000-000000000001', 'district9',
 'd9c00007-0000-0000-0000-000000000007', 'd9de000a-0000-0000-0000-000000000001', 'd9b00001-0000-0000-0000-000000000001',
 18, 20, 22, 22,
 'I', 1,
 41, 51,
 36, 0,
 25000.00, 1200000.00, 0.00, 100.00,
 'None', NULL, 'Downward', 'Defendant provided substantial assistance to the government in related investigations pursuant to USSG 5K1.1',
 36, false, NOW() - INTERVAL '45 days', NOW() - INTERVAL '42 days'),
-- Case 8: Reeves (drug_offense, on_appeal)
('d9bb0002-0000-0000-0000-000000000002', 'district9',
 'd9c00008-0000-0000-0000-000000000008', 'd9de000b-0000-0000-0000-000000000002', 'd9b00001-0000-0000-0000-000000000001',
 28, 30, 32, 32,
 'III', 6,
 151, 188,
 168, 0,
 50000.00, 0.00, 500000.00, 100.00,
 'None', NULL, 'None', NULL,
 60, true, NOW() - INTERVAL '90 days', NOW() - INTERVAL '87 days'),
-- Case 10: Park (cybercrime, awaiting_sentencing — not yet sentenced)
('d12bb001-0000-0000-0000-000000000001', 'district12',
 'd12c0002-0000-0000-0000-000000000002', 'd12de002-0000-0000-0000-000000000002', 'd12b0001-0000-0000-0000-000000000001',
 24, 26, 28, 28,
 'II', 3,
 87, 108,
 NULL, NULL,
 NULL, NULL, NULL, NULL,
 NULL, NULL, NULL, NULL,
 NULL, false, NULL, NULL)
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- CLERK QUEUE ITEMS (12 rows)
-- ============================================================

INSERT INTO clerk_queue (id, court_id, queue_type, priority, status, title, description,
    source_type, source_id, case_id, case_number, assigned_to, submitted_by,
    current_step, metadata, completed_at)
VALUES
-- 1. Case 1 (Rodriguez) filing — criminal complaint docket entry
('d9be0001-0000-0000-0000-000000000001', 'district9', 'filing', 3, 'pending',
 'New Filing: USA v. Rodriguez - Criminal Complaint',
 'Criminal complaint filed against Carlos Rodriguez for distribution of fentanyl-laced counterfeit oxycodone pills. Requires docketing and NEF generation.',
 'filing', 'd9d00001-0000-0000-0000-000000000001',
 'd9c00001-0000-0000-0000-000000000001', '9:26-cr-00101',
 NULL, NULL, 'review', '{}', NULL),
-- 2. Case 15 (Carter) filing — indictment docket entry (district12)
('d9be0002-0000-0000-0000-000000000002', 'district12', 'filing', 3, 'pending',
 'New Filing: USA v. Carter - Indictment',
 'Grand jury indictment returned charging Terrence Carter with RICO conspiracy and money laundering conspiracy.',
 'filing', 'd12d0033-0000-0000-0000-000000000006',
 'd12c0007-0000-0000-0000-000000000007', '12:26-cr-00225',
 NULL, NULL, 'review', '{}', NULL),
-- 3. Case 9 (Gonzalez) plea — notice of plea negotiations docket entry (district12)
('d9be0003-0000-0000-0000-000000000003', 'district12', 'filing', 3, 'in_review',
 'Plea Agreement: USA v. Gonzalez',
 'Notice of plea agreement negotiations filed. Parties request continuance of trial date pending finalization of plea terms.',
 'filing', 'd12d0007-0000-0000-0000-000000000007',
 'd12c0001-0000-0000-0000-000000000001', '12:26-cr-00201',
 NULL, NULL, 'review', '{}', NULL),
-- 4. Case 3 (Williams RICO) discovery response — docket entry
('d9be0004-0000-0000-0000-000000000004', 'district9', 'filing', 3, 'processing',
 'Discovery Response: USA v. Williams et al.',
 'Government response in opposition to motion for protective order. Needs docketing and service notification.',
 'filing', 'd9d00012-0000-0000-0000-000000000009',
 'd9c00003-0000-0000-0000-000000000003', '9:26-cr-00103',
 NULL, NULL, 'docket', '{}', NULL),
-- 5. Case 5 (Jackson) trial readiness notice — docket entry
('d9be0005-0000-0000-0000-000000000005', 'district9', 'filing', 3, 'processing',
 'Trial Notice: USA v. Jackson',
 'Notice of trial readiness filed by the Government. Trial set for 03/03/2026. Requires NEF to all parties.',
 'filing', 'd9d00029-0000-0000-0000-000000000005',
 'd9c00005-0000-0000-0000-000000000005', '9:25-cr-00098',
 NULL, NULL, 'nef', '{}', NULL),
-- 6. Case 3 (Williams RICO) motion to compel
('d9be0006-0000-0000-0000-000000000006', 'district9', 'motion', 2, 'pending',
 'Motion to Compel: USA v. Williams et al.',
 'Motion to compel production of financial records from Meridian Financial Corp filed by defense. Requires routing to assigned judge.',
 'motion', 'd9af0002-0000-0000-0000-000000000002',
 'd9c00003-0000-0000-0000-000000000003', '9:26-cr-00103',
 NULL, NULL, 'review', '{}', NULL),
-- 7. Case 4 (Petrov) motion to suppress — completed
('d9be0007-0000-0000-0000-000000000007', 'district9', 'motion', 2, 'completed',
 'Motion to Suppress: USA v. Petrov',
 'Motion to suppress evidence obtained from search of cryptocurrency exchange records. Fully processed and ruled upon.',
 'motion', 'd9af0005-0000-0000-0000-000000000005',
 'd9c00004-0000-0000-0000-000000000004', '9:26-cr-00104',
 NULL, NULL, 'completed', '{}', NOW() - INTERVAL '8 days'),
-- 8. Case 12 (Volkov & Sokolov) sealed motion — processing, route to judge (district12)
('d9be0008-0000-0000-0000-000000000008', 'district12', 'motion', 2, 'processing',
 'Sealed Motion: USA v. Volkov & Sokolov',
 'Motion to seal case and all filings. Sealed proceeding requires special handling and routing to assigned judge under CIPA protocols.',
 'motion', 'd12af002-0000-0000-0000-000000000002',
 'd12c0004-0000-0000-0000-000000000004', '12:26-cr-00210',
 NULL, NULL, 'route_judge', '{}', NULL),
-- 9. Case 7 (Ahmed) judgment order — completed
('d9be0009-0000-0000-0000-000000000009', 'district9', 'order', 3, 'completed',
 'Judgment: USA v. Ahmed',
 'Judgment and commitment order following sentencing. Defendant sentenced to 36 months custody, 3 years supervised release, restitution of $1.2M.',
 'order', 'd9ba0007-0000-0000-0000-000000000007',
 'd9c00007-0000-0000-0000-000000000007', '9:24-cr-00042',
 NULL, NULL, 'completed', '{}', NOW() - INTERVAL '40 days'),
-- 10. Case 11 (Thompson) dismissal order — completed (district12)
('d9be000a-0000-0000-0000-000000000001', 'district12', 'order', 3, 'completed',
 'Order of Dismissal: USA v. Thompson',
 'Order of dismissal with prejudice. Case dismissed after successful suppression motion. Defendant discharged from all conditions of release.',
 'order', 'd12ba002-0000-0000-0000-000000000002',
 'd12c0003-0000-0000-0000-000000000003', '12:25-cr-00165',
 NULL, NULL, 'completed', '{}', NOW() - INTERVAL '58 days'),
-- 11. Case 5 (Jackson) speedy trial deadline alert — CRITICAL
('d9be000b-0000-0000-0000-000000000002', 'district9', 'deadline_alert', 1, 'pending',
 'URGENT: Speedy Trial Deadline in 5 Days - USA v. Jackson',
 'CRITICAL: Speedy Trial Act 70-day clock expires in 5 days. Trial MUST begin by this date or case is subject to dismissal. Immediate judicial attention required.',
 'deadline', 'd9eb0006-0000-0000-0000-000000000006',
 'd9c00005-0000-0000-0000-000000000005', '9:25-cr-00098',
 NULL, NULL, 'review', '{}', NULL),
-- 12. General admin — rejected
('d9be000c-0000-0000-0000-000000000003', 'district9', 'general', 4, 'rejected',
 'Administrative: Court Calendar Update',
 'Request to update court calendar with revised scheduling information. Rejected as duplicate of existing calendar entry.',
 'document', 'd9ac0001-0000-0000-0000-000000000001',
 NULL, NULL,
 NULL, NULL, 'review', '{"reject_reason":"Duplicate of existing calendar entry"}', NOW() - INTERVAL '2 days')
ON CONFLICT (id) DO NOTHING;

-- ============================================================
-- VICTIMS (4 rows)
-- ============================================================

INSERT INTO victims (id, court_id, case_id, name, victim_type, notification_email, notification_mail, notification_phone)
VALUES
-- Case 3 (Williams RICO): corporate victim
('d9bc0001-0000-0000-0000-000000000001', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'Meridian Financial Corp', 'Organization', 'legal@meridianfinancial.com', false, NULL),
-- Case 3 (Williams RICO): individual victim
('d9bc0002-0000-0000-0000-000000000002', 'district9', 'd9c00003-0000-0000-0000-000000000003',
 'James Whitmore', 'Individual', 'j.whitmore@email.com', false, '555-0150'),
-- Case 5 (Jackson firearms): minor victim
('d9bc0003-0000-0000-0000-000000000003', 'district9', 'd9c00005-0000-0000-0000-000000000005',
 'Tyler Bennett', 'Minor', 'guardian.bennett@email.com', true, '555-0155'),
-- Case 7 (Ahmed tax): government victim
('d9bc0004-0000-0000-0000-000000000004', 'district9', 'd9c00007-0000-0000-0000-000000000007',
 'Internal Revenue Service', 'Government', NULL, true, NULL)
ON CONFLICT (id) DO NOTHING;

END $$;
