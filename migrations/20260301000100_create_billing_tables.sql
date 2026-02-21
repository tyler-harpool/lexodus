-- Billing accounts track prepaid balances for per-search fees
CREATE TABLE IF NOT EXISTS billing_accounts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    balance_cents   BIGINT NOT NULL DEFAULT 0,
    account_type    TEXT NOT NULL DEFAULT 'standard'
                    CHECK (account_type IN ('standard', 'exempt', 'government')),
    stripe_customer_id TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id)
);

CREATE INDEX idx_billing_accounts_user ON billing_accounts(user_id);

-- Search transactions log every billable action
CREATE TABLE IF NOT EXISTS search_transactions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    query           TEXT NOT NULL,
    court_ids       TEXT[] NOT NULL DEFAULT '{}',
    result_count    INT NOT NULL DEFAULT 0,
    fee_cents       INT NOT NULL DEFAULT 0,
    action_type     TEXT NOT NULL DEFAULT 'search'
                    CHECK (action_type IN ('search', 'document_view', 'report', 'export')),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_search_transactions_user ON search_transactions(user_id);
CREATE INDEX idx_search_transactions_created ON search_transactions(created_at);

-- Fee schedule for search/billing fees (separate from court filing fee_schedule table)
CREATE TABLE IF NOT EXISTS search_fee_schedule (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action_type     TEXT NOT NULL UNIQUE
                    CHECK (action_type IN ('search', 'document_view', 'report', 'export')),
    fee_cents       INT NOT NULL DEFAULT 10,
    cap_cents       INT,
    description     TEXT NOT NULL DEFAULT '',
    effective_date  DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed default PACER-equivalent fee schedule
INSERT INTO search_fee_schedule (action_type, fee_cents, cap_cents, description) VALUES
    ('search', 10, NULL, 'Per-search fee'),
    ('document_view', 10, 300, 'Per-page fee for document access, $3.00 cap per document'),
    ('report', 10, NULL, 'Per-page fee for report generation'),
    ('export', 10, NULL, 'Per-page fee for CSV/PDF export')
ON CONFLICT (action_type) DO NOTHING;
