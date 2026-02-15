-- Align party/representation CHECK constraint enums with OpenAPI spec (union of spec + DB).
-- Add missing columns for representations, service_records, parties, and attorneys.

-- =========================================================================
-- 1a. Widen Party Enums
-- =========================================================================

-- PartyType: add Appellant, Appellee, Counter-Claimant, Cross-Claimant
ALTER TABLE parties DROP CONSTRAINT IF EXISTS parties_party_type_check;
ALTER TABLE parties ADD CONSTRAINT parties_party_type_check
  CHECK (party_type IN (
    'Plaintiff','Defendant','Appellant','Appellee','Petitioner','Respondent',
    'Intervenor','Amicus Curiae','Third Party','Government','Witness',
    'Counter-Claimant','Cross-Claimant','Other'
  ));

-- PartyRole: merge DB (Lead, Co-Defendant, etc.) + spec (Principal, Guardian, etc.)
ALTER TABLE parties DROP CONSTRAINT IF EXISTS parties_party_role_check;
ALTER TABLE parties ADD CONSTRAINT parties_party_role_check
  CHECK (party_role IN (
    'Lead','Co-Defendant','Co-Plaintiff','Cross-Claimant','Counter-Claimant',
    'Garnishee','Real Party in Interest','Principal','Co-Party','Representative',
    'Guardian','Trustee','Executor','Administrator','Next Friend','Other'
  ));

-- PartyStatus: add In Contempt
ALTER TABLE parties DROP CONSTRAINT IF EXISTS parties_status_check;
ALTER TABLE parties ADD CONSTRAINT parties_status_check
  CHECK (status IN (
    'Active','Terminated','Defaulted','Dismissed','Settled','Deceased',
    'Unknown','In Contempt'
  ));

-- ServiceMethod on parties: add Certified Mail, Express Mail, ECF
ALTER TABLE parties DROP CONSTRAINT IF EXISTS parties_service_method_check;
ALTER TABLE parties ADD CONSTRAINT parties_service_method_check
  CHECK (service_method IS NULL OR service_method IN (
    'Electronic','Mail','Personal Service','Waiver','Publication',
    'Certified Mail','Express Mail','ECF','Other'
  ));

-- =========================================================================
-- 1b. Widen Representation Enums
-- =========================================================================

-- RepresentationType: merge DB + spec
ALTER TABLE representations DROP CONSTRAINT IF EXISTS representations_representation_type_check;
ALTER TABLE representations ADD CONSTRAINT representations_representation_type_check
  CHECK (representation_type IN (
    'Private','Court Appointed','Pro Bono','Public Defender','CJA Panel',
    'Government','General','Limited','Pro Hac Vice','Standby','Other'
  ));

-- RepresentationStatus: add Completed
ALTER TABLE representations DROP CONSTRAINT IF EXISTS representations_status_check;
ALTER TABLE representations ADD CONSTRAINT representations_status_check
  CHECK (status IN ('Active','Withdrawn','Terminated','Substituted','Suspended','Completed'));

-- WithdrawalReason: add CHECK (currently unconstrained TEXT)
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_constraint WHERE conname = 'representations_withdrawal_reason_check'
  ) THEN
    ALTER TABLE representations ADD CONSTRAINT representations_withdrawal_reason_check
      CHECK (withdrawal_reason IS NULL OR withdrawal_reason IN (
        'Client Request','Conflict of Interest','Non-Payment',
        'Completed Representation','Breakdown in Communication',
        'Health Reasons','Court Order','Other'
      ));
  END IF;
END $$;

-- =========================================================================
-- 1c. Add Missing Columns
-- =========================================================================

-- Representations: spec fields missing from DB
ALTER TABLE representations ADD COLUMN IF NOT EXISTS limited_appearance BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE representations ADD COLUMN IF NOT EXISTS cja_appointment_id UUID;
ALTER TABLE representations ADD COLUMN IF NOT EXISTS scope_of_representation TEXT;

-- Service records: spec field
ALTER TABLE service_records ADD COLUMN IF NOT EXISTS certificate_of_service TEXT;

-- Parties: PII fields from spec
ALTER TABLE parties ADD COLUMN IF NOT EXISTS ssn_last_four TEXT;
ALTER TABLE parties ADD COLUMN IF NOT EXISTS ein TEXT;

-- Parties & Attorneys: NEF SMS opt-in
ALTER TABLE parties ADD COLUMN IF NOT EXISTS nef_sms_opt_in BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE attorneys ADD COLUMN IF NOT EXISTS nef_sms_opt_in BOOLEAN NOT NULL DEFAULT FALSE;

-- =========================================================================
-- 1d. Update Seed Data (district9 — USA v. Garcia parties)
-- =========================================================================

-- Add contact info to USA v. Garcia parties
UPDATE parties SET email = 'ausa.garcia@usdoj.gov', phone = '+15551000001'
  WHERE id = 'c0000000-0000-0000-0000-000000000001' AND court_id = 'district9';
UPDATE parties SET email = 'mgarcia@defense.law', phone = '+15551000002', nef_sms_opt_in = true
  WHERE id = 'c0000000-0000-0000-0000-000000000002' AND court_id = 'district9';
