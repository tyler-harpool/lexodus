-- Align criminal_cases CHECK constraints with OpenAPI spec enums.

-- Crime type: fraud, drug_offense, racketeering, cybercrime, tax_offense, money_laundering, immigration, firearms, other
ALTER TABLE criminal_cases DROP CONSTRAINT IF EXISTS criminal_cases_crime_type_check;
ALTER TABLE criminal_cases ADD CONSTRAINT criminal_cases_crime_type_check
    CHECK (crime_type IN ('fraud','drug_offense','racketeering','cybercrime','tax_offense','money_laundering','immigration','firearms','other'));

-- Case status: filed through on_appeal
ALTER TABLE criminal_cases DROP CONSTRAINT IF EXISTS criminal_cases_status_check;
ALTER TABLE criminal_cases ADD CONSTRAINT criminal_cases_status_check
    CHECK (status IN ('filed','arraigned','discovery','pretrial_motions','plea_negotiations','trial_ready','in_trial','awaiting_sentencing','sentenced','dismissed','on_appeal'));

-- Priority: low, medium, high, critical
ALTER TABLE criminal_cases DROP CONSTRAINT IF EXISTS criminal_cases_priority_check;
ALTER TABLE criminal_cases ADD CONSTRAINT criminal_cases_priority_check
    CHECK (priority IN ('low','medium','high','critical'));

-- Update defaults to match new enum values
ALTER TABLE criminal_cases ALTER COLUMN status SET DEFAULT 'filed';
ALTER TABLE criminal_cases ALTER COLUMN priority SET DEFAULT 'medium';

-- Make description NOT NULL with empty string default
ALTER TABLE criminal_cases ALTER COLUMN description SET NOT NULL;
ALTER TABLE criminal_cases ALTER COLUMN description SET DEFAULT '';

-- Make location NOT NULL with empty string default
ALTER TABLE criminal_cases ALTER COLUMN location SET NOT NULL;
ALTER TABLE criminal_cases ALTER COLUMN location SET DEFAULT '';
