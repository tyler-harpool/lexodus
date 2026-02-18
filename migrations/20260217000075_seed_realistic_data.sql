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

END $$;
