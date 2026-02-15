CREATE TABLE IF NOT EXISTS service_records (
    id                     UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id               TEXT NOT NULL REFERENCES courts(id),
    document_id            UUID NOT NULL,
    party_id               UUID NOT NULL REFERENCES parties(id) ON DELETE CASCADE,
    service_date           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    service_method         TEXT NOT NULL
        CHECK (service_method IN ('Electronic','Mail','Personal Service','Waiver','Publication','Certified Mail','Express Mail','Other')),
    served_by              TEXT NOT NULL,
    proof_of_service_filed BOOLEAN NOT NULL DEFAULT FALSE,
    successful             BOOLEAN NOT NULL DEFAULT TRUE,
    attempts               INT NOT NULL DEFAULT 1,
    notes                  TEXT
);
CREATE INDEX idx_service_records_court ON service_records(court_id);
CREATE INDEX idx_service_records_court_document ON service_records(court_id, document_id);
CREATE INDEX idx_service_records_court_party ON service_records(court_id, party_id);
CREATE INDEX idx_service_records_court_date ON service_records(court_id, service_date);
CREATE INDEX idx_service_records_court_method ON service_records(court_id, service_method);
