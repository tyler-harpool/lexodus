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

END $$;
