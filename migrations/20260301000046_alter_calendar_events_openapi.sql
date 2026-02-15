-- Align calendar_events table with OpenAPI CalendarEntry schema.
-- Adds missing columns, updates CHECK constraints to use snake_case enum values.

-- 1. Add missing columns from OpenAPI spec
ALTER TABLE calendar_events
    ADD COLUMN IF NOT EXISTS actual_start TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS actual_end   TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS call_time    TIMESTAMPTZ;

-- 2. Ensure courtroom/description/notes are NOT NULL (OpenAPI requires them)
UPDATE calendar_events SET courtroom = '' WHERE courtroom IS NULL;
UPDATE calendar_events SET description = '' WHERE description IS NULL;
UPDATE calendar_events SET notes = '' WHERE notes IS NULL;
ALTER TABLE calendar_events ALTER COLUMN courtroom SET NOT NULL;
ALTER TABLE calendar_events ALTER COLUMN courtroom SET DEFAULT '';
ALTER TABLE calendar_events ALTER COLUMN description SET NOT NULL;
ALTER TABLE calendar_events ALTER COLUMN description SET DEFAULT '';
ALTER TABLE calendar_events ALTER COLUMN notes SET NOT NULL;
ALTER TABLE calendar_events ALTER COLUMN notes SET DEFAULT '';

-- 3. Drop old CHECK constraints
ALTER TABLE calendar_events DROP CONSTRAINT IF EXISTS calendar_events_event_type_check;
ALTER TABLE calendar_events DROP CONSTRAINT IF EXISTS calendar_events_status_check;

-- 4. Add new CHECK constraints matching OpenAPI CalendarEventType enum (snake_case)
ALTER TABLE calendar_events ADD CONSTRAINT calendar_events_event_type_check
    CHECK (event_type IN (
        'initial_appearance','arraignment','bail_hearing','plea_hearing',
        'trial_date','sentencing','violation_hearing','status_conference',
        'scheduling_conference','settlement_conference','pretrial_conference',
        'motion_hearing','evidentiary_hearing','jury_selection','jury_trial',
        'bench_trial','show_cause_hearing','contempt_hearing','emergency_hearing',
        'telephonic','video_conference'
    ));

-- 5. Add new CHECK constraints matching OpenAPI EventStatus enum (snake_case)
ALTER TABLE calendar_events ADD CONSTRAINT calendar_events_status_check
    CHECK (status IN (
        'scheduled','confirmed','in_progress','completed',
        'cancelled','postponed','recessed','continued'
    ));

-- 6. Update default status to match new snake_case convention
ALTER TABLE calendar_events ALTER COLUMN status SET DEFAULT 'scheduled';
