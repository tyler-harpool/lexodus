# Realistic CM/ECF Seed Data Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create a single idempotent seed migration that populates ~500+ rows of realistic federal criminal court data across both test districts, covering the full case lifecycle and edge cases.

**Architecture:** One migration file (`000075_seed_realistic_data.sql`) using well-known UUIDs, `ON CONFLICT DO NOTHING`, and a guard clause. Insert order follows FK dependencies: courts → judges → attorneys → cases → defendants → charges → parties → representations → docket entries → documents → calendar events → deadlines → speedy trial → motions → evidence → orders → sentencing → queue items → victims.

**Tech Stack:** PostgreSQL, sqlx migrations

**Design Doc:** `docs/plans/2026-02-17-realistic-seed-data-design.md`

---

## Task 1: Create migration file with guard, judges, and attorneys

**Files:**
- Create: `migrations/20260217000075_seed_realistic_data.sql`
- Create: `migrations/20260217000075_seed_realistic_data.down.sql`

**Step 1: Write the migration file with guard and judges**

Create `migrations/20260217000075_seed_realistic_data.sql`. Start with a `DO $$ BEGIN ... END $$` block with a guard that skips if realistic data already exists (check for a well-known case UUID). Insert 8 judges from real FJC/CourtListener data:

**district9 judges (UUIDs: `d9j00001-...` through `d9j00004-...`):**
- Chief Judge Ronnie Abrams (SDNY → repurposed as district9, Female, title='Chief Judge')
- Senior Judge Lance M. Africk (EDLA → repurposed, Male, senior_status_date=2024-10-01, title='Senior Judge')
- Magistrate Judge Gray M. Borden (NDAL → repurposed, Male, title='Magistrate Judge')
- Circuit Judge Nancy G. Abudu (11th Circuit, Female, title='Visiting Judge', for appeals)

**district12 judges (UUIDs: `d12j0001-...` through `d12j0004-...`):**
- Judge Amir H. Ali (DDC → repurposed, Male, title='Judge')
- Judge Georgia N. Alexakis (NDIL → repurposed, Female, title='Judge')
- Magistrate Judge Sonja F. Bivins (SDAL → repurposed, Female, title='Magistrate Judge')
- Circuit Judge Seth R. Aframe (1st Circuit, Male, title='Visiting Judge', for appeals)

**8 attorneys (UUIDs: `d9a00001-...` through `d12a0004-...`):**

| UUID prefix | District | Role | Name | Bar# | Status |
|-------------|----------|------|------|------|--------|
| d9a00001 | district9 | AUSA | Sarah K. Mitchell | DC-2019-04521 | Active |
| d9a00002 | district9 | Federal PD | Marcus J. Rivera | TX-2015-08832 | Active, cja_panel_member=true |
| d9a00003 | district9 | Private (BigLaw) | Catherine L. Whitfield | NY-2012-15567 | Active |
| d9a00004 | district9 | CJA Panel | David R. Okonkwo | CA-2018-22104 | Active, cja_panel_member=true |
| d12a0001 | district12 | AUSA | Jennifer M. Huang | IL-2017-11893 | Active |
| d12a0002 | district12 | Federal PD | Robert A. Blackwell | GA-2014-07261 | Active, cja_panel_member=true |
| d12a0003 | district12 | Private (solo) | Elena V. Petrossian | FL-2016-19440 | Active |
| d12a0004 | district12 | Pro Hac Vice | Thomas W. Nakamura | WA-2013-06178 | Active |

**Step 2: Write the down migration**

```sql
-- Remove seed data by deleting well-known cases (cascades handle children)
DELETE FROM criminal_cases WHERE id IN (
    'd9c00001-0000-0000-0000-000000000001',
    'd9c00002-0000-0000-0000-000000000002',
    -- ... all 15 case UUIDs
);
DELETE FROM attorneys WHERE id IN ('d9a00001-...' /* all 8 */);
DELETE FROM judges WHERE id IN ('d9j00001-...' /* all 8 */);
```

**Step 3: Verify migration syntax**

Run: `psql postgres://dioxus:dioxus@localhost:5432/lexodus -f migrations/20260217000075_seed_realistic_data.sql`
Expected: No errors.

**Step 4: Revert and re-run via sqlx**

Run: `psql postgres://dioxus:dioxus@localhost:5432/lexodus -f migrations/20260217000075_seed_realistic_data.down.sql`
Then: `sqlx migrate run`
Expected: Migration applied successfully.

**Step 5: Commit**

```
git add migrations/20260217000075_seed_realistic_data.sql migrations/20260217000075_seed_realistic_data.down.sql
git commit -m "feat(seed): add migration skeleton with judges and attorneys"
```

---

## Task 2: Add 15 criminal cases with defendants and charges

**Files:**
- Modify: `migrations/20260217000075_seed_realistic_data.sql`

**Step 1: Add 15 cases**

Insert 15 cases using the design doc's case table. Use well-known UUIDs `d9c00001-0000-0000-0000-00000000000N` for district9 and `d12c0001-0000-0000-0000-00000000000N` for district12. Each case gets:
- case_number in format `N:26-cr-NNNNN` (realistic federal format)
- assigned_judge_id pointing to appropriate judge
- status matching design (filed, arraigned, discovery, etc.)
- Case 12 (Volkov): `is_sealed = true`

**Step 2: Add ~22 defendants**

Most cases get 1 defendant. Special cases:
- Case 3 (Williams RICO): 4 defendants — James Williams, Tyrone Brooks, Keisha Watts, Derek Simmons
- Case 12 (Volkov): 2 defendants — Viktor Volkov, Dmitri Sokolov

Each defendant gets realistic custody_status, bail_type, bail_amount:
- Case 1 (Rodriguez): Released, Personal Recognizance
- Case 4 (Petrov): In Custody, Bail Denied
- Case 5 (Jackson): Bond, Surety, $250,000
- Case 6 (Morrison): Bond, Surety, $500,000
- Case 13 (Davis): Released, Personal Recognizance (pro se)

**Step 3: Add ~30 charges**

Each defendant gets 1-3 charges with real federal statutes:
- Drug offenses: 21 USC 841, 21 USC 846 (conspiracy)
- Cybercrime: 18 USC 1030 (CFAA), 18 USC 1343 (wire fraud)
- RICO: 18 USC 1962 (RICO), 18 USC 1956 (money laundering)
- Firearms: 18 USC 922(g) (felon in possession), 18 USC 924(c) (use in crime)
- Fraud: 18 USC 1341 (mail fraud), 18 USC 1343 (wire fraud)
- Tax: 26 USC 7201 (tax evasion), 26 USC 7206 (false return)
- Immigration: 8 USC 1326 (illegal reentry)

Plea and verdict columns set based on case status:
- Filed/arraigned cases: plea='Not Yet Entered' or 'Not Guilty'
- Case 9 (plea negotiations): plea='Not Yet Entered' (about to change)
- Case 10 (awaiting sentencing): verdict='Guilty'
- Case 7 (sentenced): verdict='Guilty', plea='Guilty'
- Case 11 (dismissed): verdict='Dismissed'

**Step 4: Verify**

Run: `sqlx migrate run` (after reverting first if needed)
Expected: Migration applies, verify with `SELECT count(*) FROM criminal_cases` → 16 (15 new + 1 existing Garcia case).

**Step 5: Commit**

```
git add migrations/20260217000075_seed_realistic_data.sql
git commit -m "feat(seed): add 15 cases with defendants and charges"
```

---

## Task 3: Add parties, representations, and docket entries

**Files:**
- Modify: `migrations/20260217000075_seed_realistic_data.sql`

**Step 1: Add ~35 parties**

Each case gets:
- Government party: `party_type='Government', party_role='Lead', name='United States of America', entity_type='Government', service_method='Electronic'`
- Defendant party per defendant: `party_type='Defendant', party_role='Lead' (first) or 'Co-Defendant'`
- Case 13 (Davis): defendant party with `pro_se=true, represented=false`

**Step 2: Add ~30 representations**

Link attorneys to defendant parties:
- Government parties → AUSA attorney (representation_type='Government')
- Defendant parties → defense attorney (Private, Public Defender, CJA Panel as appropriate)
- Case 3 (RICO): 4 different attorneys for 4 defendants
- Case 8 (appeal): original representation terminated + new appellate counsel
- Case 13 (Davis): NO representation row (pro se)

**Step 3: Add ~160 docket entries with ~60 documents**

Insert docket entries following the depth pattern from the design doc. Each entry gets sequential entry_numbers per case. For entries that represent documents (motions, orders, briefs), also insert a documents row and link via `document_id`.

Key docket sequences per case status (showing entry_type values):

**Filed (cases 1, 15): 3 entries each**
1. criminal_complaint / indictment
2. summons / warrant (arrest warrant)
3. minute_order (initial appearance)

**Arraigned (cases 2, 13): 6 entries each**
1-3 (same as filed) +
4. minute_order (arraignment — not guilty plea)
5. scheduling_order (with document)
6. notice (conditions of release)

**Discovery (cases 3, 14): 12-14 entries each**
1-6 (same as arraigned) +
7. discovery_request (gov't Rule 16 disclosure)
8. discovery_response (defense reciprocal)
9-10. motions + responses
11. protective_order (case 3: RICO discovery)
12. status (status conference minute entry)

**Pretrial motions (cases 4, 12): 10-12 entries each**
1-8 (through discovery) +
9. motion (suppress / seal)
10. response
11. order (ruling)
12. hearing_minutes

**Trial ready (case 5): 14 entries**
Through pretrial + witness_list + exhibit + hearing_notice (trial date)

**In trial (case 6): 18 entries**
Through trial ready + trial daily minutes (3 days) + sealed witness_list

**Sentenced (case 7): 20 entries**
Through trial + verdict + sentence + judgment

**Appeal (case 8): 22 entries**
Through sentenced + notice_of_appeal + appeal_brief

**Dismissed (case 11): 10 entries**
Through pretrial + motion (dismiss) + order (dismissal)

**Plea negotiations (case 9): 8 entries**
Through arraigned + motion (plea agreement filing) + hearing_notice

**Awaiting sentencing (case 10): 16 entries**
Through trial + verdict + notice (PSR scheduling)

Sealed entries: Case 12 docket entries 1-3 get `is_sealed = true`. Case 6 witness list gets `is_sealed = true`.

**Step 4: Verify**

Run migration, check: `SELECT count(*) FROM docket_entries` and `SELECT count(*) FROM documents`.

**Step 5: Commit**

```
git add migrations/20260217000075_seed_realistic_data.sql
git commit -m "feat(seed): add parties, representations, docket entries and documents"
```

---

## Task 4: Add calendar events, deadlines, speedy trial, and excludable delays

**Files:**
- Modify: `migrations/20260217000075_seed_realistic_data.sql`

**Step 1: Add ~28 calendar events**

Use relative dates from `NOW()` so data stays fresh:
- Past events: `NOW() - INTERVAL 'N days'` with status='Completed'
- Future events: `NOW() + INTERVAL 'N days'` with status='Scheduled'
- Case 6 trial: daily events, day 4 = 'In Progress' (today-ish)

Each event links to case_id and judge_id. Courtroom values like 'Courtroom 4A', 'Courtroom 12B'. Duration 30-180 minutes depending on type.

**Step 2: Add ~22 deadlines**

Mix of Pending, Completed, Overdue, Extended statuses:
- Case 3: 1 discovery deadline (overdue — co-defendant missed it)
- Case 5: trial deadline 5 days from now (Pending, is_jurisdictional=true)
- Case 13: 3 filing deadlines (all Extended — pro se extensions)
- Case 14: 2 discovery deadlines (Extended twice)

Include `triggering_event`, `applicable_rule` (e.g., 'Fed. R. Crim. P. 16', 'Speedy Trial Act'), `responsible_party`.

**Step 3: Add 4 speedy trial clocks**

| Case | arrest_date | indictment_date | arraignment_date | days_elapsed | days_remaining | is_tolled |
|------|------------|-----------------|------------------|-------------|----------------|-----------|
| Case 2 | NOW()-45d | NOW()-40d | NOW()-30d | 15 | 55 | false |
| Case 4 | NOW()-90d | NOW()-85d | NOW()-70d | 42 | 28 | true |
| Case 5 | NOW()-95d | NOW()-90d | NOW()-80d | 65 | 5 | false |
| Case 14 | NOW()-60d | NOW()-55d | NOW()-45d | 30 | 40 | false |

**Step 4: Add excludable delays**

- Case 4: 1 delay (motion to suppress pending, 28 days excluded, statutory_reference='18 USC 3161(h)(1)(D)')
- Case 5: 2 delays (continuance motion 10 days + complex case designation 15 days)

**Step 5: Verify**

Run migration, check counts for calendar_events, deadlines, speedy_trial, excludable_delays.

**Step 6: Commit**

```
git add migrations/20260217000075_seed_realistic_data.sql
git commit -m "feat(seed): add calendar events, deadlines, speedy trial clocks"
```

---

## Task 5: Add motions, evidence, custody transfers, and orders

**Files:**
- Modify: `migrations/20260217000075_seed_realistic_data.sql`

**Step 1: Add ~20 motions**

Each motion links to case_id via FK. Include realistic `filed_by`, `description`, `ruling_text` (for decided motions). Statuses per design:
- Case 3: Sever (Granted), Compel (Pending), Limine x2 (Pending, Denied)
- Case 4: Suppress (Denied, ruling_text='Motion denied. Evidence obtained pursuant to valid search warrant...'), Dismiss (Pending)
- Case 5: Continuance (Denied, ruling_text='Motion denied. Speedy Trial Act deadline approaching...'), Limine (Granted)
- Case 8: New Trial (Denied)
- Case 11: Dismiss (Granted, ruling_text='Motion granted. Government failed to establish...')
- Case 12: Seal (Granted)
- Case 13: Extension x3 (all Granted)

**Step 2: Add ~15 evidence items**

- Case 3: financial_records (Documentary), wiretap_recordings (Digital), seized_cash (Physical, location='FBI Evidence Vault')
- Case 4: laptop (Digital, seized_by='DEA Task Force'), bank_statements (Documentary)
- Case 5: firearm (Physical), surveillance_video (Digital)
- Case 6: 8 trial exhibits (mix of Physical, Documentary, Digital, Forensic)

**Step 3: Add custody transfers for case 3 seized cash**

3 transfers showing chain of custody:
1. Seized by FBI Agent → FBI Field Office Evidence Room
2. FBI Evidence Room → US Marshals Service
3. US Marshals → Court Evidence Locker

**Step 4: Add ~15 judicial orders**

Each order links to case_id, judge_id. Include realistic `content` text (1-2 paragraphs). `related_motions` array links to motion UUIDs where applicable.

Order types per design: Scheduling, Protective, Detention, Release, Dismissal, Sealing, Sentencing. Mix of Draft, Signed, Filed statuses.

**Step 5: Verify**

Run migration, check counts.

**Step 6: Commit**

```
git add migrations/20260217000075_seed_realistic_data.sql
git commit -m "feat(seed): add motions, evidence, orders"
```

---

## Task 6: Add sentencing records, queue items, and victims

**Files:**
- Modify: `migrations/20260217000075_seed_realistic_data.sql`

**Step 1: Add 3 sentencing records**

| Case | Defendant | Judge | Offense Level | History Cat | Range | Sentence | Departure |
|------|----------|-------|--------------|------------|-------|----------|-----------|
| Case 7 (Ahmed) | Ahmed | Abrams | 22 | I | 41-51mo | 36mo custody | Downward variance, cooperation |
| Case 8 (Reeves) | Reeves | Abrams | 32 | III | 151-188mo | 168mo | None, within guidelines |
| Case 10 (Park) | Park | Ali | 28 | II | 87-108mo | NULL (pending) | NULL |

Case 7: restitution_amount=1200000, supervised_release_months=36, special_assessment=100, fine_amount=25000
Case 8: appeal_waiver=true, supervised_release_months=60, forfeiture_amount=500000

**Step 2: Add 12 clerk queue items**

Use UUIDs `d9q00001-...` through `d9q00012-...`. Each links to a source entity (filing, motion, or order UUID) and optionally a case. Mix of all 5 statuses and all pipeline steps per the design doc.

The `source_id` should point to actual docket_entry or motion UUIDs created earlier. For completed items, set `completed_at = NOW() - INTERVAL '...'`.

**Step 3: Add 4 victims across 3 cases**

- Case 3: 'Meridian Financial Corp' (Organization), 'James Whitmore' (Individual)
- Case 5: 'Tyler Bennett' (Minor, notification_email set, notification_phone set)
- Case 7: 'Internal Revenue Service' (Government, notification_mail=true)

**Step 4: Verify**

Run migration, check counts for sentencing, clerk_queue, victims.

**Step 5: Commit**

```
git add migrations/20260217000075_seed_realistic_data.sql
git commit -m "feat(seed): add sentencing, queue items, and victims"
```

---

## Task 7: Run full verification

**Files:** None (verification only)

**Step 1: Run the migration fresh**

```
sqlx migrate run
```

Expected: Migration 000075 applied successfully.

**Step 2: Verify row counts**

Run:
```sql
SELECT 'judges' as t, count(*) FROM judges
UNION ALL SELECT 'attorneys', count(*) FROM attorneys
UNION ALL SELECT 'cases', count(*) FROM criminal_cases
UNION ALL SELECT 'defendants', count(*) FROM defendants
UNION ALL SELECT 'charges', count(*) FROM charges
UNION ALL SELECT 'parties', count(*) FROM parties
UNION ALL SELECT 'representations', count(*) FROM representations
UNION ALL SELECT 'docket_entries', count(*) FROM docket_entries
UNION ALL SELECT 'documents', count(*) FROM documents
UNION ALL SELECT 'calendar_events', count(*) FROM calendar_events
UNION ALL SELECT 'deadlines', count(*) FROM deadlines
UNION ALL SELECT 'speedy_trial', count(*) FROM speedy_trial
UNION ALL SELECT 'motions', count(*) FROM motions
UNION ALL SELECT 'evidence', count(*) FROM evidence
UNION ALL SELECT 'orders', count(*) FROM judicial_orders
UNION ALL SELECT 'sentencing', count(*) FROM sentencing
UNION ALL SELECT 'queue', count(*) FROM clerk_queue
UNION ALL SELECT 'victims', count(*) FROM victims
ORDER BY 1;
```

**Step 3: Verify FK integrity**

```sql
-- All cases have valid judges
SELECT c.id, c.title FROM criminal_cases c WHERE c.assigned_judge_id IS NOT NULL AND c.assigned_judge_id NOT IN (SELECT id FROM judges);
-- Should return 0 rows

-- All docket entries have valid cases
SELECT de.id FROM docket_entries de WHERE de.case_id NOT IN (SELECT id FROM criminal_cases);
-- Should return 0 rows
```

**Step 4: Update sqlx offline cache**

Run: `cargo sqlx prepare --workspace`

**Step 5: Run full test suite**

Run: `cargo test -p tests -- --test-threads=1`
Expected: All tests pass (seed data should not affect tests since tests truncate tables).

**Step 6: Commit sqlx cache if changed**

```
git add .sqlx/ migrations/
git commit -m "feat(seed): finalize realistic CM/ECF seed data with verification"
```

---

## Summary

| Task | What it delivers |
|------|-----------------|
| Task 1 | Migration skeleton + 8 real judges + 8 attorneys |
| Task 2 | 15 cases + ~22 defendants + ~30 charges |
| Task 3 | ~35 parties + ~30 representations + ~160 docket entries + ~60 documents |
| Task 4 | ~28 calendar events + ~22 deadlines + 4 speedy trial clocks + excludable delays |
| Task 5 | ~20 motions + ~15 evidence + custody transfers + ~15 orders |
| Task 6 | 3 sentencing records + 12 queue items + 4 victims |
| Task 7 | Full verification + test suite + sqlx cache |
