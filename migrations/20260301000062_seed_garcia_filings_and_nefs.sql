-- Seed: extend USA v. Garcia (b0000000-...-001) with filings, NEFs, service records, and document events.
-- Idempotent via well-known UUIDs + ON CONFLICT DO NOTHING.

-- ──────────────────────────────────────────────────────────────────────
-- Filings (2) — tied to existing docket entries and documents
-- ──────────────────────────────────────────────────────────────────────

-- Filing for docket entry #3 (Motion to Suppress) + document e...-002
INSERT INTO filings (id, court_id, case_id, filing_type, filed_by, status, document_id, docket_entry_id)
VALUES (
    'f0000000-0000-0000-0000-000000000001',
    'district9',
    'b0000000-0000-0000-0000-000000000001',
    'Motion',
    'Defense Counsel',
    'Filed',
    'e0000000-0000-0000-0000-000000000002',
    'd0000000-0000-0000-0000-000000000003'
) ON CONFLICT (id) DO NOTHING;

-- Filing for docket entry #5 (Order Denying Motion) + document e...-003
INSERT INTO filings (id, court_id, case_id, filing_type, filed_by, status, document_id, docket_entry_id)
VALUES (
    'f0000000-0000-0000-0000-000000000002',
    'district9',
    'b0000000-0000-0000-0000-000000000001',
    'Other',
    'Hon. Patricia Chen',
    'Filed',
    'e0000000-0000-0000-0000-000000000003',
    'd0000000-0000-0000-0000-000000000005'
) ON CONFLICT (id) DO NOTHING;

-- ──────────────────────────────────────────────────────────────────────
-- NEFs (2) — one per filing, with recipients and HTML snapshot
-- ──────────────────────────────────────────────────────────────────────

INSERT INTO nefs (id, court_id, filing_id, document_id, case_id, docket_entry_id, recipients, html_snapshot)
VALUES (
    'aa000000-0000-0000-0000-000000000001',
    'district9',
    'f0000000-0000-0000-0000-000000000001',
    'e0000000-0000-0000-0000-000000000002',
    'b0000000-0000-0000-0000-000000000001',
    'd0000000-0000-0000-0000-000000000003',
    '[{"party_id":"c0000000-0000-0000-0000-000000000001","name":"United States of America","service_method":"Electronic","electronic":true},{"party_id":"c0000000-0000-0000-0000-000000000002","name":"Maria Garcia","service_method":"Electronic","electronic":true}]'::jsonb,
    '<div class="nef">
  <h2>NOTICE OF ELECTRONIC FILING</h2>
  <p><strong>Case:</strong> 26-CR-00042</p>
  <p><strong>Document:</strong> Motion to Suppress Evidence</p>
  <p><strong>Filed by:</strong> Defense Counsel</p>
  <p><strong>Date:</strong> February 14, 2026 at 10:30 AM UTC</p>
  <p><strong>Docket #:</strong> 3</p>
  <h3>Recipients</h3>
  <ul>
    <li>United States of America &mdash; Electronic</li>
    <li>Maria Garcia &mdash; Electronic</li>
  </ul>
</div>'
) ON CONFLICT (court_id, id) DO NOTHING;

INSERT INTO nefs (id, court_id, filing_id, document_id, case_id, docket_entry_id, recipients, html_snapshot)
VALUES (
    'aa000000-0000-0000-0000-000000000002',
    'district9',
    'f0000000-0000-0000-0000-000000000002',
    'e0000000-0000-0000-0000-000000000003',
    'b0000000-0000-0000-0000-000000000001',
    'd0000000-0000-0000-0000-000000000005',
    '[{"party_id":"c0000000-0000-0000-0000-000000000001","name":"United States of America","service_method":"Electronic","electronic":true},{"party_id":"c0000000-0000-0000-0000-000000000002","name":"Maria Garcia","service_method":"Electronic","electronic":true}]'::jsonb,
    '<div class="nef">
  <h2>NOTICE OF ELECTRONIC FILING</h2>
  <p><strong>Case:</strong> 26-CR-00042</p>
  <p><strong>Document:</strong> Order Denying Motion to Suppress</p>
  <p><strong>Filed by:</strong> Hon. Patricia Chen</p>
  <p><strong>Date:</strong> February 14, 2026 at 02:15 PM UTC</p>
  <p><strong>Docket #:</strong> 5</p>
  <h3>Recipients</h3>
  <ul>
    <li>United States of America &mdash; Electronic</li>
    <li>Maria Garcia &mdash; Electronic</li>
  </ul>
</div>'
) ON CONFLICT (court_id, id) DO NOTHING;

-- ──────────────────────────────────────────────────────────────────────
-- Service Records (4) — 2 per filing, one per party, all electronic
-- ──────────────────────────────────────────────────────────────────────

-- Filing 1 (Motion to Suppress) — service to Government
INSERT INTO service_records (id, court_id, document_id, party_id, service_method, served_by, successful, proof_of_service_filed, notes)
VALUES (
    'bb000000-0000-0000-0000-000000000001',
    'district9',
    'e0000000-0000-0000-0000-000000000002',
    'c0000000-0000-0000-0000-000000000001',
    'Electronic',
    'CM/ECF System',
    true,
    true,
    'Auto-served via Notice of Electronic Filing'
) ON CONFLICT (id) DO NOTHING;

-- Filing 1 — service to Defendant
INSERT INTO service_records (id, court_id, document_id, party_id, service_method, served_by, successful, proof_of_service_filed, notes)
VALUES (
    'bb000000-0000-0000-0000-000000000002',
    'district9',
    'e0000000-0000-0000-0000-000000000002',
    'c0000000-0000-0000-0000-000000000002',
    'Electronic',
    'CM/ECF System',
    true,
    true,
    'Auto-served via Notice of Electronic Filing'
) ON CONFLICT (id) DO NOTHING;

-- Filing 2 (Order Denying Motion) — service to Government
INSERT INTO service_records (id, court_id, document_id, party_id, service_method, served_by, successful, proof_of_service_filed, notes)
VALUES (
    'bb000000-0000-0000-0000-000000000003',
    'district9',
    'e0000000-0000-0000-0000-000000000003',
    'c0000000-0000-0000-0000-000000000001',
    'Electronic',
    'CM/ECF System',
    true,
    true,
    'Auto-served via Notice of Electronic Filing'
) ON CONFLICT (id) DO NOTHING;

-- Filing 2 — service to Defendant
INSERT INTO service_records (id, court_id, document_id, party_id, service_method, served_by, successful, proof_of_service_filed, notes)
VALUES (
    'bb000000-0000-0000-0000-000000000004',
    'district9',
    'e0000000-0000-0000-0000-000000000003',
    'c0000000-0000-0000-0000-000000000002',
    'Electronic',
    'CM/ECF System',
    true,
    true,
    'Auto-served via Notice of Electronic Filing'
) ON CONFLICT (id) DO NOTHING;

-- ──────────────────────────────────────────────────────────────────────
-- Document Events (3) — seal/unseal/replace lifecycle
-- ──────────────────────────────────────────────────────────────────────

-- Sealed event on indictment document (e...-001)
INSERT INTO document_events (id, court_id, document_id, event_type, actor, detail)
VALUES (
    'cc000000-0000-0000-0000-000000000001',
    'district9',
    'e0000000-0000-0000-0000-000000000001',
    'sealed',
    'Hon. Patricia Chen',
    '{"sealing_level":"SealedCourtOnly","reason_code":"SealedIndictment"}'::jsonb
) ON CONFLICT (id) DO NOTHING;

-- Unsealed event on indictment (after arraignment)
INSERT INTO document_events (id, court_id, document_id, event_type, actor, detail, created_at)
VALUES (
    'cc000000-0000-0000-0000-000000000002',
    'district9',
    'e0000000-0000-0000-0000-000000000001',
    'unsealed',
    'Hon. Patricia Chen',
    '{}'::jsonb,
    NOW() + INTERVAL '1 hour'
) ON CONFLICT (id) DO NOTHING;

-- Replaced event on motion document (e...-002) — showing lifecycle
INSERT INTO document_events (id, court_id, document_id, event_type, actor, detail, created_at)
VALUES (
    'cc000000-0000-0000-0000-000000000003',
    'district9',
    'e0000000-0000-0000-0000-000000000002',
    'replaced',
    'Defense Counsel',
    '{"replacement_document_id":"e0000000-0000-0000-0000-000000000099","reason":"Corrected exhibit references"}'::jsonb,
    NOW() + INTERVAL '2 hours'
) ON CONFLICT (id) DO NOTHING;
