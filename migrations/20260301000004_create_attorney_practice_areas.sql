CREATE TABLE attorney_practice_areas (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id    TEXT NOT NULL REFERENCES courts(id),
    attorney_id UUID NOT NULL REFERENCES attorneys(id) ON DELETE CASCADE,
    area        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, attorney_id, area)
);
CREATE INDEX idx_practice_areas_attorney ON attorney_practice_areas(attorney_id);
