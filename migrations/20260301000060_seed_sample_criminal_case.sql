-- Seed: realistic criminal case "USA v. Garcia" in district9.
-- Uses well-known UUIDs for idempotency (ON CONFLICT DO NOTHING).

-- Judge
INSERT INTO judges (id, court_id, name, title, district)
VALUES (
    'a0000000-0000-0000-0000-000000000001',
    'district9',
    'Hon. Patricia Chen',
    'Judge',
    'district9'
) ON CONFLICT (id) DO NOTHING;

-- Case
INSERT INTO criminal_cases (id, court_id, case_number, title, crime_type, status, priority, district_code, assigned_judge_id)
VALUES (
    'b0000000-0000-0000-0000-000000000001',
    'district9',
    '26-CR-00042',
    'USA v. Garcia',
    'fraud',
    'pretrial_motions',
    'high',
    'district9',
    'a0000000-0000-0000-0000-000000000001'
) ON CONFLICT (id) DO NOTHING;

-- Parties
INSERT INTO parties (id, court_id, case_id, party_type, party_role, name, entity_type, represented, pro_se, service_method, status, joined_date)
VALUES
    ('c0000000-0000-0000-0000-000000000001', 'district9', 'b0000000-0000-0000-0000-000000000001',
     'Government', 'Lead', 'United States of America', 'Government', false, false, 'Electronic', 'Active', NOW()),
    ('c0000000-0000-0000-0000-000000000002', 'district9', 'b0000000-0000-0000-0000-000000000001',
     'Defendant', 'Lead', 'Maria Garcia', 'Individual', true, false, 'Electronic', 'Active', NOW())
ON CONFLICT (id) DO NOTHING;

-- Docket entries (6 entries with sequential entry_numbers)
INSERT INTO docket_entries (id, court_id, case_id, entry_number, entry_type, description, filed_by, is_sealed)
VALUES
    ('d0000000-0000-0000-0000-000000000001', 'district9', 'b0000000-0000-0000-0000-000000000001',
     1, 'indictment', 'Indictment returned by Grand Jury charging wire fraud (18 USC 1343), bank fraud (18 USC 1344), and conspiracy (18 USC 371).', 'Grand Jury', true),
    ('d0000000-0000-0000-0000-000000000002', 'district9', 'b0000000-0000-0000-0000-000000000001',
     2, 'minute_order', 'Arraignment held. Defendant appeared with counsel. Not guilty plea entered. Trial set for August 2026.', 'Courtroom Deputy', false),
    ('d0000000-0000-0000-0000-000000000003', 'district9', 'b0000000-0000-0000-0000-000000000001',
     3, 'motion', 'MOTION to Suppress Evidence obtained from warrantless search of defendant residence on January 15, 2026, filed by Defense Counsel.', 'Defense Counsel', false),
    ('d0000000-0000-0000-0000-000000000004', 'district9', 'b0000000-0000-0000-0000-000000000001',
     4, 'response', 'Government RESPONSE in Opposition to Motion to Suppress. Evidence obtained under exigent circumstances exception.', 'AUSA Williams', false),
    ('d0000000-0000-0000-0000-000000000005', 'district9', 'b0000000-0000-0000-0000-000000000001',
     5, 'order', 'ORDER denying Motion to Suppress (Dkt. 3). Evidence admissible under exigent circumstances. Signed by Judge Chen.', 'Hon. Patricia Chen', false),
    ('d0000000-0000-0000-0000-000000000006', 'district9', 'b0000000-0000-0000-0000-000000000001',
     6, 'hearing_notice', 'NOTICE of Trial Setting. Jury trial scheduled for August 18, 2026 at 9:00 AM in Courtroom 4A.', 'Courtroom Deputy', false)
ON CONFLICT (id) DO NOTHING;

-- Documents linked to entries #1, #3, #5
INSERT INTO documents (id, court_id, case_id, title, document_type, storage_key, checksum, file_size, content_type, is_sealed, uploaded_by, sealing_level, seal_reason_code)
VALUES
    ('e0000000-0000-0000-0000-000000000001', 'district9', 'b0000000-0000-0000-0000-000000000001',
     'Indictment - USA v. Garcia', 'Indictment', 'district9/documents/seed/indictment.pdf', 'seed-checksum-001', 245760, 'application/pdf',
     true, 'system', 'SealedCourtOnly', 'SealedIndictment'),
    ('e0000000-0000-0000-0000-000000000002', 'district9', 'b0000000-0000-0000-0000-000000000001',
     'Motion to Suppress Evidence', 'Motion', 'district9/documents/seed/motion-suppress.pdf', 'seed-checksum-002', 184320, 'application/pdf',
     false, 'system', 'Public', NULL),
    ('e0000000-0000-0000-0000-000000000003', 'district9', 'b0000000-0000-0000-0000-000000000001',
     'Order Denying Motion to Suppress', 'Order', 'district9/documents/seed/order-deny-suppress.pdf', 'seed-checksum-003', 122880, 'application/pdf',
     false, 'system', 'Public', NULL)
ON CONFLICT (id) DO NOTHING;

-- Link documents to docket entries
UPDATE docket_entries SET document_id = 'e0000000-0000-0000-0000-000000000001'
WHERE id = 'd0000000-0000-0000-0000-000000000001' AND document_id IS NULL;

UPDATE docket_entries SET document_id = 'e0000000-0000-0000-0000-000000000002'
WHERE id = 'd0000000-0000-0000-0000-000000000003' AND document_id IS NULL;

UPDATE docket_entries SET document_id = 'e0000000-0000-0000-0000-000000000003'
WHERE id = 'd0000000-0000-0000-0000-000000000005' AND document_id IS NULL;
