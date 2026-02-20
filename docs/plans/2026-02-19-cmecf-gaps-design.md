# CM/ECF Modernization Gaps — Design Document

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement the corresponding implementation plan task-by-task.

**Goal:** Close the 3 critical gaps preventing Lexodus from being a full CM/ECF replacement: MFA, filing fees, and free public access aligned with the Free Law Project.

**Architecture:** 3 phases sharing the existing Dioxus + Axum + Postgres stack. No new services. Feature-flagged for incremental rollout.

**Audience:** Full replacement — court staff, attorneys, and public.

---

## Phase 1: MFA (TOTP)

### Purpose

Add TOTP-based multi-factor authentication to meet the federal May 2025 mandate. Feature-flagged for per-court rollout.

### Database Changes

Add columns to `users`:
- `totp_secret_enc` (BYTEA) — AES-256-GCM encrypted TOTP secret
- `mfa_enabled` (BOOL DEFAULT false)
- `mfa_enforced_at` (TIMESTAMPTZ) — when user enabled MFA

New table `mfa_recovery_codes`:
- `id` (UUID PK)
- `user_id` (FK → users)
- `code_hash` (TEXT) — Argon2-hashed recovery code
- `used` (BOOL DEFAULT false)
- `created_at` (TIMESTAMPTZ)

### New Dependencies

- `totp-rs` — TOTP generation and validation (pure Rust)
- `qrcode` — QR code SVG generation for authenticator setup
- `aes-gcm` — AES-256-GCM encryption for TOTP secrets at rest

### New Environment Variable

- `MFA_ENCRYPTION_KEY` — 32-byte hex key for AES-256-GCM encryption of TOTP secrets

### Feature Flags

- `mfa: bool` — enables MFA setup and challenge routes (default false)
- `mfa_required: bool` — forces all users to enroll on next login (default false)

### Flows

**Setup (Settings > Security > Enable MFA):**
1. Server generates random TOTP secret via `totp-rs`
2. Encrypts secret with AES-256-GCM using `MFA_ENCRYPTION_KEY`, stores in `totp_secret_enc`
3. Returns QR code (SVG data URI) and manual entry key to frontend
4. User scans QR in authenticator app, enters first 6-digit code to confirm
5. Server validates code, sets `mfa_enabled = true`
6. Server generates 8 recovery codes, Argon2-hashes each, stores in `mfa_recovery_codes`
7. Returns plaintext codes once for user to save

**Login Challenge:**
1. User submits email + password. Server validates password.
2. If `mfa_enabled = true`, server returns `{ mfa_required: true, challenge_token: "..." }` instead of JWT. Challenge token is a short-lived (5 min) signed token containing `user_id` and `typ: "mfa_challenge"`.
3. Frontend shows TOTP input screen.
4. User enters 6-digit code (or recovery code).
5. Server decrypts `totp_secret_enc`, validates code with `totp-rs` (allowing 1 step of clock drift).
6. If valid, issues full JWT (access + refresh). If recovery code, also marks code as used in `mfa_recovery_codes`.

**OAuth Users:**
1. After OAuth callback, if `mfa_enabled = true`, redirect to `/mfa-challenge` with challenge token in cookie.
2. Same TOTP verification flow before issuing JWT.

**Disable MFA:**
1. Requires current TOTP code or recovery code to disable (prevent unauthorized disable).
2. Clears `totp_secret_enc`, sets `mfa_enabled = false`, deletes recovery codes.

### Security Considerations

- TOTP secrets encrypted at rest (AES-256-GCM) because server needs raw secret to verify codes — hashing is insufficient.
- Recovery codes are Argon2-hashed (one-way) — same pattern as passwords.
- Challenge tokens prevent session fixation — no JWT issued until MFA verified.
- Rate limit MFA attempts: 5 failures per challenge token, then token invalidated.

---

## Phase 2: Filing Fees (Stripe Per-Transaction)

### Purpose

Charge attorneys court-mandated filing fees during the filing submission flow. Support IFP, government, and CJA exemptions. Uses existing Stripe integration (one-time payment mode).

### Database Changes

New table `filing_fee_schedule`:
- `id` (UUID PK)
- `court_id` (TEXT FK → courts)
- `entry_type` (TEXT) — maps to docket entry type or filing category
- `fee_cents` (INT) — fee amount in cents
- `description` (TEXT)
- `effective_date` (DATE)
- `active` (BOOL DEFAULT true)
- UNIQUE(court_id, entry_type, effective_date)

New table `filing_payments`:
- `id` (UUID PK)
- `court_id` (TEXT FK → courts)
- `filing_id` (UUID FK → filings)
- `user_id` (INT FK → users)
- `stripe_checkout_session_id` (TEXT)
- `stripe_payment_intent_id` (TEXT)
- `amount_cents` (INT)
- `status` (TEXT) — pending, paid, refunded, waived
- `fee_waiver_type` (TEXT NULL) — ifp, government, cja
- `created_at` (TIMESTAMPTZ)
- `paid_at` (TIMESTAMPTZ NULL)

Add column to `civil_cases`:
- `fee_status` (TEXT DEFAULT 'pending') — paid, ifp, government, waived

### Seed Data

Judicial Conference fee schedule (current as of 2026):
- Civil case filing: $402
- Habeas corpus: $5
- Motion to reopen: $402
- Appeal filing: $605
- Miscellaneous filings: varies

### Flow

1. **Attorney submits filing** via `POST /api/filings`. Server looks up fee in `filing_fee_schedule` by `entry_type + court_id`.

2. **Fee required:** If fee > 0 and no waiver applies, filing is created with `status = 'pending_payment'`. Server creates Stripe Checkout Session (one-time payment mode). Metadata includes `filing_id`, `court_id`, `user_id`. Returns checkout URL.

3. **Fee waived:** If case has `fee_status` IN ('ifp', 'government') or attorney is CJA-appointed on this case, filing skips payment. `filing_payments` row created with `status = 'waived'` and `fee_waiver_type` set. Filing proceeds normally.

4. **Attorney pays.** Stripe Checkout redirects back to Lexodus success URL.

5. **Webhook confirms.** Existing webhook handler processes `CheckoutSessionCompleted`. New branch checks metadata for `filing_id`. Updates `filing_payments.status = 'paid'`, sets `paid_at`. Advances filing to `status = 'submitted'`. Triggers NEF generation.

6. **Payment timeout.** Filings in `pending_payment` for >24 hours are flagged for clerk review.

7. **Refunds.** Admin endpoint `POST /api/filing-payments/:id/refund` creates Stripe refund via API, updates `filing_payments.status = 'refunded'`.

### Fee Waiver Administration

- Clerks/judges set `fee_status` on cases via existing `PATCH /api/cases/:id` or `PATCH /api/civil-cases/:id`
- IFP motions go through normal docket/order workflow — when judge grants IFP order, clerk updates `fee_status = 'ifp'`
- Government exemption: automatically detected if filing attorney's firm matches "U.S. Attorney" pattern, or manually set

### Not Building

- Prepaid deposit accounts
- Quarterly billing / invoicing
- Multi-item cart checkout
- Pay.gov integration (Stripe replaces this)

---

## Phase 3: Public Access Portal (Free Law Project Aligned)

### Purpose

Free, open public access to court records. CourtListener-compatible REST API. No fees for document access. Aligned with Free Law Project's mission of open court data.

### Architecture

Same Dioxus app with new `/public/*` route group. No authentication required for read-only access. Revenue model is court subscriptions (existing Stripe tiers), not public access fees.

### Public UI Routes

| Route | Content |
|---|---|
| `/public/search` | Cross-court case search by name, number, party, attorney |
| `/public/case/:id` | Case summary — parties, status, judge, key dates |
| `/public/case/:id/docket` | Full docket sheet with document download links |
| `/public/document/:id` | Document download (PDF, unsealed only) |
| `/public/opinions` | Published opinions, searchable and filterable |

### CourtListener-Compatible REST API

New route group `/api/v1/public/` — no auth required, rate-limited.

```
GET /api/v1/public/search/            → case search
GET /api/v1/public/dockets/:id/       → docket with metadata
GET /api/v1/public/docket-entries/    → entries filtered by docket, date range
GET /api/v1/public/opinions/          → published opinions
GET /api/v1/public/people/            → parties and attorneys
GET /api/v1/public/courts/            → court metadata
GET /api/v1/public/bulk/              → daily database dumps (gzipped JSONL)
```

**Field mapping to CourtListener format:**

| Lexodus Field | CourtListener Field |
|---|---|
| `case_number` | `docket_number` |
| `court_id` | `court` |
| `date_filed` (on docket entry) | `date_filed` |
| `title` (on case) | `case_name` |
| `crime_type` / NOS code | `nature_of_suit` |
| self-referencing URL | `absolute_url` |
| `assigned_judge_id` → judge name | `assigned_to_str` |

Internal fields with no CourtListener equivalent pass through as-is (additive, not breaking).

### Access Control

- All public queries filter `WHERE is_sealed = false` automatically
- Sealed cases return 404 (not 403) — do not reveal existence
- Restricted documents excluded from search results and download
- Staff endpoints unchanged — full access through existing auth
- Public endpoints are strictly read-only (GET only)

### Rate Limiting

| Tier | Limit | How |
|---|---|---|
| Anonymous | 60 req/min per IP | Default for no-auth requests |
| Registered (free account) | 300 req/min | Optional account for API consumers (RECAP, researchers) |
| Bulk endpoint | 1 req/hour | Daily dump, cached, heavy query |

Uses existing `RateLimitState` middleware with per-IP keying instead of per-court-district.

### Bulk Data Export

`GET /api/v1/public/bulk/` returns gzipped JSONL files:
- `cases.jsonl.gz` — all public cases
- `docket-entries.jsonl.gz` — all public docket entries
- `opinions.jsonl.gz` — all published opinions

Generated nightly via background task. Stored in S3. Enables Free Law Project bulk ingest without API hammering.

### Feature Flag

- `public_portal: bool` in `FeatureFlags` (default false)
- When false, `/public/*` and `/api/v1/public/*` return 404

### Not Building

- PACER billing/metering (free access)
- User accounts for public read access (optional for rate limit upgrade only)
- Audio/video streaming
- RECAP browser extension (Free Law Project maintains that)
- Cross-court federated search (single-instance multi-tenant handles this natively)

---

## Implementation Order

| Phase | Scope | Depends On |
|---|---|---|
| 1. MFA | Auth layer only | Nothing |
| 2. Filing Fees | Stripe + filing flow | Nothing (parallel-safe with Phase 1) |
| 3. Public Portal | New routes + API | Phase 2 (fee_status field used in public queries) |

Phases 1 and 2 can be developed in parallel. Phase 3 depends on Phase 2 only for the `fee_status` column on cases (to correctly filter public data).

---

## Decisions Log

| Decision | Choice | Rationale |
|---|---|---|
| MFA method | TOTP only | Meets federal mandate, simple, widely adopted |
| MFA secret storage | AES-256-GCM encrypted | Must be reversible (server verifies codes) |
| Filing payment | Stripe Checkout (redirect) | Already implemented for subscriptions, minimal new code |
| Public access fees | Free (no fees) | Aligned with Free Law Project mission |
| Public API format | CourtListener-compatible | Enables RECAP integration, open data ecosystem |
| Portal architecture | Same app, role-gated | Single codebase, simpler deployment |
| Bulk data | Nightly JSONL dumps on S3 | Prevents API abuse, enables bulk research |
