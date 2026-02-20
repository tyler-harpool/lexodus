-- Seed fee schedule for district9 and district12
-- Standard federal court fees per 28 USC 1914 and the Judicial Conference fee schedule

-- District 9 fee entries
INSERT INTO fee_schedule (court_id, fee_id, category, description, amount_cents, statute, waivable, waiver_form, cap_cents, cap_description)
VALUES
    ('district9', 'civil_filing',           'Filing',         'Civil case filing fee',              40500,  '28 USC 1914(a)',  true,  'AO 239', NULL, NULL),
    ('district9', 'appeal_filing',          'Appeal',         'Appeal filing fee',                  60500,  '28 USC 1913',     true,  'AO 239', NULL, NULL),
    ('district9', 'habeas_filing',          'Filing',         'Habeas corpus filing fee',             500,  '28 USC 1914(a)',  true,  'AO 240', NULL, NULL),
    ('district9', 'pro_hac_vice',           'Filing',         'Pro hac vice admission fee',         20000,  'Local Rule 83.1', false, NULL,      NULL, NULL),
    ('district9', 'search',                 'Search',         'Search fee per name or item',         3200,  '28 USC 1914(b)',  false, NULL,      NULL, NULL),
    ('district9', 'certification',          'Certification',  'Certification fee per document',      1100,  '28 USC 1914(b)',  false, NULL,      NULL, NULL),
    ('district9', 'copy_pacer',             'Copy',           'PACER copy fee per page',               10,  '28 USC 1914(b)',  false, NULL,      300,  '$3.00 per document cap'),
    ('district9', 'reproduction_recording', 'Copy',           'Reproduction of recording fee',       3200,  '28 USC 1914(b)',  false, NULL,      NULL, NULL),
    ('district9', 'exemplification',        'Certification',  'Exemplification fee',                 2200,  '28 USC 1914(b)',  false, NULL,      NULL, NULL),
    ('district9', 'bar_admission',          'Filing',         'Admission to bar fee',               18800,  'Local Rule 83.1', false, NULL,      NULL, NULL),
    ('district9', 'withdrawal_motion',      'Filing',         'Motion to withdraw (no fee)',            0,  'N/A',             false, NULL,      NULL, NULL),
    ('district9', 'returned_check',         'Miscellaneous',  'Returned check fee',                  5300,  '31 USC 3528',     false, NULL,      NULL, NULL),
    ('district9', 'ifp_application',        'Filing',         'In forma pauperis application',          0,  '28 USC 1915',     false, NULL,      NULL, NULL),
    ('district9', 'garnishment',            'Service',        'Writ of garnishment fee',              500,  '28 USC 1921',     false, NULL,      NULL, NULL),
    ('district9', 'misc_case_opening',      'Filing',         'Miscellaneous case opening fee',      5200,  '28 USC 1914',     false, NULL,      NULL, NULL);

-- District 12 fee entries (identical fee schedule)
INSERT INTO fee_schedule (court_id, fee_id, category, description, amount_cents, statute, waivable, waiver_form, cap_cents, cap_description)
VALUES
    ('district12', 'civil_filing',           'Filing',         'Civil case filing fee',              40500,  '28 USC 1914(a)',  true,  'AO 239', NULL, NULL),
    ('district12', 'appeal_filing',          'Appeal',         'Appeal filing fee',                  60500,  '28 USC 1913',     true,  'AO 239', NULL, NULL),
    ('district12', 'habeas_filing',          'Filing',         'Habeas corpus filing fee',             500,  '28 USC 1914(a)',  true,  'AO 240', NULL, NULL),
    ('district12', 'pro_hac_vice',           'Filing',         'Pro hac vice admission fee',         20000,  'Local Rule 83.1', false, NULL,      NULL, NULL),
    ('district12', 'search',                 'Search',         'Search fee per name or item',         3200,  '28 USC 1914(b)',  false, NULL,      NULL, NULL),
    ('district12', 'certification',          'Certification',  'Certification fee per document',      1100,  '28 USC 1914(b)',  false, NULL,      NULL, NULL),
    ('district12', 'copy_pacer',             'Copy',           'PACER copy fee per page',               10,  '28 USC 1914(b)',  false, NULL,      300,  '$3.00 per document cap'),
    ('district12', 'reproduction_recording', 'Copy',           'Reproduction of recording fee',       3200,  '28 USC 1914(b)',  false, NULL,      NULL, NULL),
    ('district12', 'exemplification',        'Certification',  'Exemplification fee',                 2200,  '28 USC 1914(b)',  false, NULL,      NULL, NULL),
    ('district12', 'bar_admission',          'Filing',         'Admission to bar fee',               18800,  'Local Rule 83.1', false, NULL,      NULL, NULL),
    ('district12', 'withdrawal_motion',      'Filing',         'Motion to withdraw (no fee)',            0,  'N/A',             false, NULL,      NULL, NULL),
    ('district12', 'returned_check',         'Miscellaneous',  'Returned check fee',                  5300,  '31 USC 3528',     false, NULL,      NULL, NULL),
    ('district12', 'ifp_application',        'Filing',         'In forma pauperis application',          0,  '28 USC 1915',     false, NULL,      NULL, NULL),
    ('district12', 'garnishment',            'Service',        'Writ of garnishment fee',              500,  '28 USC 1921',     false, NULL,      NULL, NULL),
    ('district12', 'misc_case_opening',      'Filing',         'Miscellaneous case opening fee',      5200,  '28 USC 1914',     false, NULL,      NULL, NULL);
