CREATE TABLE IF NOT EXISTS order_templates (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    court_id         TEXT NOT NULL REFERENCES courts(id),
    order_type       TEXT NOT NULL
        CHECK (order_type IN ('Scheduling','Protective','Restraining','Dismissal','Sentencing','Detention','Release','Discovery','Sealing','Contempt','Procedural','Standing','Other')),
    name             TEXT NOT NULL,
    description      TEXT,
    content_template TEXT NOT NULL,
    is_active        BOOLEAN NOT NULL DEFAULT TRUE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(court_id, name)
);
CREATE INDEX idx_order_templates_court ON order_templates(court_id);
CREATE INDEX idx_order_templates_court_type ON order_templates(court_id, order_type);
CREATE INDEX idx_order_templates_court_active ON order_templates(court_id, is_active);
