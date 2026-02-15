-- Align docket_entries entry_type CHECK constraint with OpenAPI DocketEntryType enum.
-- Also make filed_by nullable to match the shared-types DocketEntry struct.

ALTER TABLE docket_entries DROP CONSTRAINT IF EXISTS docket_entries_entry_type_check;
ALTER TABLE docket_entries ADD CONSTRAINT docket_entries_entry_type_check
    CHECK (entry_type IN (
        'complaint','indictment','information','criminal_complaint',
        'answer','motion','response','reply','notice',
        'order','minute_order','scheduling_order','protective_order','sealing_order',
        'discovery_request','discovery_response','deposition','interrogatories',
        'exhibit','witness_list','expert_report',
        'hearing_notice','hearing_minutes','transcript',
        'judgment','verdict','sentence',
        'summons','subpoena','service_return',
        'appearance','withdrawal','substitution',
        'notice_of_appeal','appeal_brief','appellate_order',
        'letter','status','other'
    ));

-- Make filed_by nullable (struct has Option<String>)
ALTER TABLE docket_entries ALTER COLUMN filed_by DROP NOT NULL;
ALTER TABLE docket_entries ALTER COLUMN filed_by SET DEFAULT '';
