-- Seed 4 additional courts for the 6-court PoC demo.
-- district9 and district12 already exist from the initial migration.
INSERT INTO courts (id, name, court_type) VALUES
    ('district1', 'Southern District of New York', 'district'),
    ('district2', 'Eastern District of New York', 'district'),
    ('district5', 'Northern District of California', 'district'),
    ('district7', 'Central District of California', 'district')
ON CONFLICT (id) DO NOTHING;

-- Judges for new courts (UUIDs are hex-only per project rules)
-- Title must be one of: Chief Judge, Judge, Senior Judge, Magistrate Judge, Visiting Judge
INSERT INTO judges (id, court_id, name, title, district, status) VALUES
    ('d1b00001-0000-0000-0000-000000000001', 'district1', 'Hon. Elena Marchetti', 'Judge', 'district1', 'Active'),
    ('d1b00002-0000-0000-0000-000000000002', 'district1', 'Hon. David Chen', 'Magistrate Judge', 'district1', 'Active'),
    ('d2b00001-0000-0000-0000-000000000001', 'district2', 'Hon. Michael Reyes', 'Judge', 'district2', 'Active'),
    ('d5b00001-0000-0000-0000-000000000001', 'district5', 'Hon. Robert Tanaka', 'Judge', 'district5', 'Active'),
    ('d5b00002-0000-0000-0000-000000000002', 'district5', 'Hon. Lisa Washington', 'Judge', 'district5', 'Active'),
    ('d7b00001-0000-0000-0000-000000000001', 'district7', 'Hon. Maria Santos', 'Judge', 'district7', 'Active'),
    ('d7b00002-0000-0000-0000-000000000002', 'district7', 'Hon. William Foster', 'Judge', 'district7', 'Active')
ON CONFLICT (id) DO NOTHING;

-- Attorneys for new courts (require email, phone, and address fields)
INSERT INTO attorneys (id, court_id, first_name, last_name, bar_number, status, firm_name, email, phone, address_street1, address_city, address_state, address_zip) VALUES
    ('d1a00001-0000-0000-0000-000000000001', 'district1', 'Thomas', 'Rivera', 'NY-10001', 'Active', 'Rivera & Associates', 'trivera@example.com', '212-555-0101', '100 Centre St', 'New York', 'NY', '10007'),
    ('d1a00002-0000-0000-0000-000000000002', 'district1', 'Jennifer', 'Walsh', 'NY-10002', 'Active', 'Walsh Law Group', 'jwalsh@example.com', '212-555-0102', '200 Broadway', 'New York', 'NY', '10007'),
    ('d2a00001-0000-0000-0000-000000000001', 'district2', 'Rachel', 'Kim', 'NY-20001', 'Active', 'Kim & Partners', 'rkim@example.com', '718-555-0201', '225 Cadman Plaza E', 'Brooklyn', 'NY', '11201'),
    ('d5a00001-0000-0000-0000-000000000001', 'district5', 'Kevin', 'Patel', 'CA-50001', 'Active', 'Patel Technology Law', 'kpatel@example.com', '415-555-0501', '450 Golden Gate Ave', 'San Francisco', 'CA', '94102'),
    ('d5a00002-0000-0000-0000-000000000002', 'district5', 'Amanda', 'Rodriguez', 'CA-50002', 'Active', 'US Attorney NDCA', 'arodriguez@example.com', '415-555-0502', '450 Golden Gate Ave', 'San Francisco', 'CA', '94102'),
    ('d7a00001-0000-0000-0000-000000000001', 'district7', 'Christina', 'Nguyen', 'CA-70001', 'Active', 'Nguyen White Collar Defense', 'cnguyen@example.com', '213-555-0701', '312 N Spring St', 'Los Angeles', 'CA', '90012'),
    ('d7a00002-0000-0000-0000-000000000002', 'district7', 'Robert', 'Garcia', 'CA-70002', 'Active', 'US Attorney CDCA', 'rgarcia@example.com', '213-555-0702', '312 N Spring St', 'Los Angeles', 'CA', '90012')
ON CONFLICT (id) DO NOTHING;

-- Criminal cases across courts
-- "Rivera" appears as a defendant name in both district1 and district5
-- to demonstrate cross-court search scenarios.
INSERT INTO criminal_cases (id, court_id, case_number, title, crime_type, status, priority, district_code) VALUES
    ('d1c00001-0000-0000-0000-000000000001', 'district1', '1:26-cr-00001', 'United States v. Rivera', 'money_laundering', 'pretrial_motions', 'high', 'district1'),
    ('d1c00002-0000-0000-0000-000000000002', 'district1', '1:26-cr-00002', 'United States v. Bennett', 'fraud', 'filed', 'medium', 'district1'),
    ('d2c00001-0000-0000-0000-000000000001', 'district2', '2:26-cr-00001', 'United States v. Rossi', 'drug_offense', 'in_trial', 'high', 'district2'),
    ('d2c00002-0000-0000-0000-000000000002', 'district2', '2:26-cr-00002', 'United States v. Thompson', 'firearms', 'plea_negotiations', 'medium', 'district2'),
    ('d5c00001-0000-0000-0000-000000000001', 'district5', '5:26-cr-00001', 'United States v. Rivera', 'fraud', 'discovery', 'high', 'district5'),
    ('d5c00002-0000-0000-0000-000000000002', 'district5', '5:26-cr-00002', 'United States v. Nakamura', 'cybercrime', 'pretrial_motions', 'high', 'district5'),
    ('d7c00001-0000-0000-0000-000000000001', 'district7', '7:26-cr-00001', 'United States v. Hernandez', 'drug_offense', 'in_trial', 'critical', 'district7'),
    ('d7c00002-0000-0000-0000-000000000002', 'district7', '7:26-cr-00002', 'United States v. Park', 'money_laundering', 'discovery', 'high', 'district7')
ON CONFLICT (id) DO NOTHING;

-- Case assignments (assignment_type uses 'Initial' per schema constraint)
INSERT INTO case_assignments (id, court_id, case_id, judge_id, assignment_type) VALUES
    ('d1e00001-0000-0000-0000-000000000001', 'district1', 'd1c00001-0000-0000-0000-000000000001', 'd1b00001-0000-0000-0000-000000000001', 'Initial'),
    ('d2e00001-0000-0000-0000-000000000001', 'district2', 'd2c00001-0000-0000-0000-000000000001', 'd2b00001-0000-0000-0000-000000000001', 'Initial'),
    ('d5e00001-0000-0000-0000-000000000001', 'district5', 'd5c00001-0000-0000-0000-000000000001', 'd5b00001-0000-0000-0000-000000000001', 'Initial'),
    ('d7e00001-0000-0000-0000-000000000001', 'district7', 'd7c00001-0000-0000-0000-000000000001', 'd7b00001-0000-0000-0000-000000000001', 'Initial')
ON CONFLICT (id) DO NOTHING;

-- Seed billing account for test user (user_id=1 seeded by migration 099)
INSERT INTO billing_accounts (user_id, balance_cents, account_type) VALUES
    (1, 5000, 'standard')
ON CONFLICT (user_id) DO NOTHING;
