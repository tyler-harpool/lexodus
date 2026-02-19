# Realistic CM/ECF Seed Data Design

**Goal:** Seed the database with ~500+ rows of interconnected, realistic federal criminal court data that mimics real-world CM/ECF usage. Covers the full case lifecycle, multi-defendant conspiracies, sealed cases, speedy trial edge cases, and exercises every major table in the system.

**Approach:** Single monolithic seed migration (`000075_seed_realistic_data.sql`) with well-known UUIDs, idempotent guard, and `ON CONFLICT DO NOTHING` for safety. Matches the pattern of existing migrations 000060 and 000072.

**Target Courts:** district9 (8 cases) and district12 (7 cases).

**Future:** Civil cases will be a separate table and migration (not in scope here).

---

## Section 1: Judges & Attorneys

### Judges (8 total — real FJC data)

**district9 (4 judges):**
- Chief Judge — Article III district judge, high caseload, active
- Senior Judge — semi-retired, limited docket
- Magistrate Judge — handles initial appearances, discovery disputes
- Circuit Judge — sits by designation for appeals from this district

**district12 (4 judges):**
- District Judge — Article III, active
- District Judge — Article III, active, different specialization
- Magistrate Judge — handles pretrial matters
- Circuit Judge — appellate panel, handles cases on appeal

Judges sourced from the FJC dataset (`~/Downloads/judges.csv`) and CourtListener magistrate data (`~/Downloads/brmag-judges.json`). Pick currently-active judges with diverse backgrounds.

### Attorneys (8 total — fictional names)

| Role | District | Notes |
|------|----------|-------|
| AUSA | district9 | Government counsel |
| AUSA | district12 | Government counsel |
| Federal Public Defender | district9 | CJA appointed |
| Federal Public Defender | district12 | CJA appointed |
| Private Defense (big firm) | district9 | High-profile representation |
| Private Defense (solo) | district12 | Solo practitioner |
| CJA Panel Attorney | district9 | Court-appointed overflow |
| Pro Hac Vice Attorney | district12 | Admitted from another district |

---

## Section 2: Cases (15 cases)

### district9 (8 cases)

| # | Case Title | Crime Type | Status | Edge Case |
|---|-----------|-----------|--------|-----------|
| 1 | USA v. Rodriguez | drug_offense | filed | Fresh filing, queue item pending review |
| 2 | USA v. Chen | cybercrime | arraigned | Single defendant, bail set, speedy trial clock started |
| 3 | USA v. Williams et al. | racketeering | discovery | **Multi-defendant RICO** (4 co-defendants), severed charges, multiple attorneys |
| 4 | USA v. Petrov | money_laundering | pretrial_motions | Motion to suppress pending, motion to dismiss denied, speedy trial tolled |
| 5 | USA v. Jackson | firearms | trial_ready | **Near speedy trial deadline** (day 65 of 70), excludable delays logged |
| 6 | USA v. Morrison | fraud | in_trial | Active trial, daily calendar events, sealed witness list |
| 7 | USA v. Ahmed | tax_offense | sentenced | Full sentencing record with guidelines, departures, supervised release |
| 8 | USA v. Reeves | drug_offense | on_appeal | **Appeal case** — circuit judge assigned, notice of appeal filed |

### district12 (7 cases)

| # | Case Title | Crime Type | Status | Edge Case |
|---|-----------|-----------|--------|-----------|
| 9 | USA v. Gonzalez | immigration | plea_negotiations | Plea agreement drafted, hearing scheduled |
| 10 | USA v. Park | cybercrime | awaiting_sentencing | Guilty verdict, PSR ordered, sentencing date set |
| 11 | USA v. Thompson | firearms | dismissed | **Dismissed case** — motion to dismiss granted, case closed |
| 12 | USA v. Volkov & Sokolov | money_laundering | pretrial_motions | **Sealed case** — sealed indictment, ex-parte filings |
| 13 | USA v. Davis | fraud | arraigned | **Pro se defendant** — no attorney, multiple deadline extensions |
| 14 | USA v. Hernandez | drug_offense | discovery | CJA panel attorney, heavy discovery |
| 15 | USA v. Carter | racketeering | filed | Just-filed case, auto-created queue item at "review" step |

---

## Section 3: Docket Entries & Documents (~160 entries, ~60 documents)

Docket depth scales with case status:

| Status | Docket Entries | Pattern |
|--------|---------------|---------|
| Filed | 2-3 | Complaint/Indictment, Summons/Warrant |
| Arraigned | 5-6 | + Initial Appearance, Arraignment, Release Order, Scheduling Order |
| Discovery/Pretrial | 10-15 | + Discovery requests/responses, Motions, Orders on motions |
| Trial/Sentencing | 15-20 | + Witness/exhibit lists, Trial minutes, Verdict, PSR, Judgment |
| Appeal | 18-20 | + Notice of Appeal, Appellate briefing |
| Dismissed | 8-10 | Through motions + Dismissal order |

Special docket features:
- Case 3 (RICO): Separate filings per defendant, severance motion
- Case 6 (trial): Daily trial minute entries
- Case 12 (sealed): Sealed entries, ex-parte filings with `is_sealed = true`
- Each docket entry with a document gets a `documents` row (realistic file sizes, content types, storage keys)

---

## Section 4: Calendar Events, Deadlines & Speedy Trial

### Calendar Events (~28 events)

| Case | Events |
|------|--------|
| Case 2 | Initial appearance (completed), arraignment (completed), status conference (scheduled) |
| Case 3 | Status conference (completed), motion hearing x2 (completed + scheduled), pretrial conference (scheduled) |
| Case 4 | Motion hearing (completed — denied), evidentiary hearing (scheduled) |
| Case 5 | Pretrial conference (completed), **jury trial (scheduled in 5 days)** |
| Case 6 | Trial day 1-3 (completed), trial day 4 (in progress), trial day 5 (scheduled) |
| Case 7 | All hearings completed, sentencing hearing (completed) |
| Case 9 | Plea hearing (scheduled) |
| Case 10 | Trial (completed), sentencing (scheduled — 3 weeks out) |

### Deadlines (~22 deadlines)

| Type | Cases | Edge Cases |
|------|-------|------------|
| Discovery | Cases 3, 4, 14 | Case 14: extended twice |
| Motion Response | Cases 3, 4 | One overdue (case 3 co-defendant missed deadline) |
| Trial | Cases 5, 6 | Case 5: 5 days away |
| Sentencing | Case 10 | PSR due date, objection deadline |
| Appeal | Case 8 | Brief due date |
| Filing | Case 13 | Pro se: extended 3 times |

### Speedy Trial Clocks (4 cases)

| Case | Days Elapsed | Days Remaining | Edge Case |
|------|-------------|----------------|-----------|
| Case 2 | 15 | 55 | Normal, clock running |
| Case 4 | 42 | 28 | Tolled — excludable delay for suppression motion |
| Case 5 | 65 | **5** | **Critical** — near deadline, 2 excludable delays |
| Case 14 | 30 | 40 | Clock running, no delays |

---

## Section 5: Motions, Evidence, Parties & Representations

### Motions (~20 motions)

| Case | Motions | Statuses |
|------|---------|----------|
| Case 3 (RICO) | Sever, Compel Discovery, Limine x2 | Granted, Pending, Pending, Denied |
| Case 4 | Suppress, Dismiss | Denied (ruling text), Pending |
| Case 5 | Continuance, Limine | Denied (speedy trial), Granted |
| Case 8 | New Trial | Denied — led to appeal |
| Case 11 | Dismiss | **Granted** — case-ending |
| Case 12 | Seal, ex-parte motion | Granted (sealed) |
| Case 13 | Extension of Time x3 | All granted (pro se) |

### Evidence (~15 items with custody transfers)

| Case | Items | Edge Case |
|------|-------|-----------|
| Case 3 (RICO) | Financial records, wiretap recordings, seized cash | 3 custody transfers on cash |
| Case 4 | Laptop (seized), bank statements | Suppression motion targets laptop |
| Case 5 | Firearm, surveillance video | Sealed forensic report |
| Case 6 (trial) | 8 trial exhibits | Mix of admitted/pending |

### Parties (~35) & Representations (~30)

- Every case: Government party (USA) + defendant party(ies)
- Case 3: 4 defendant parties, each with separate counsel
- Case 12: 2 defendants, sealed
- Case 13: Pro se (`represented: false`, no representation row)
- Case 8: Representation terminated, appellate counsel substituted

### Service Records (~40)

- Electronic service for ECF-registered parties
- Mail service for pro se (case 13)
- Personal service for sealed filings (case 12)

---

## Section 6: Sentencing, Orders, Queue Items & Victims

### Sentencing Records (3 cases)

| Case | Guidelines | Sentence | Edge Case |
|------|-----------|----------|-----------|
| Case 7 (tax) | Level 22, Cat I, 41-51 months | 36 months — **downward variance** | Restitution $1.2M, supervised release 3yr, special conditions |
| Case 10 (cyber) | Level 28, Cat II, 87-108 months | Awaiting sentencing | Guidelines calculated, objections filed |
| Case 8 (drug) | Level 32, Cat III, 151-188 months | 168 months — within guidelines | Prior sentences, BOP designation FCI medium, appeal waiver |

### Judicial Orders (~15)

Scheduling orders, protective orders, detention/release orders, sealing order, dismissal order, sentencing judgment. Mix of Draft, Signed, and Filed statuses.

### Clerk Queue Items (12 items)

| Source | Type | Status | Step | Priority |
|--------|------|--------|------|----------|
| Case 1 filing | filing | pending | review | 3 (normal) |
| Case 15 filing | filing | pending | review | 3 |
| Case 9 plea agreement | filing | in_review | review | 3 |
| Case 3 discovery resp | filing | processing | docket | 3 |
| Case 5 trial notice | filing | processing | nef | 3 |
| Case 3 motion to compel | motion | pending | review | 2 (high) |
| Case 4 suppression motion | motion | completed | completed | 2 |
| Case 12 sealed motion | motion | processing | route_judge | 2 |
| Case 7 judgment | order | completed | completed | 3 |
| Case 11 dismissal | order | completed | completed | 3 |
| Case 5 speedy trial alert | deadline_alert | pending | review | **1 (critical)** |
| General admin item | general | rejected | review | 4 (low) |

### Victims (4 across 3 cases)

| Case | Victims | Type |
|------|---------|------|
| Case 3 (RICO) | Corporation + Individual | Organization, Individual |
| Case 7 (tax) | IRS | Government |
| Case 5 (firearms) | Minor victim | Minor, notifications enabled |

---

## Summary

| Entity | Count |
|--------|-------|
| Judges | 8 |
| Attorneys | 8 |
| Cases | 15 |
| Defendants | ~22 |
| Charges | ~30 |
| Docket Entries | ~160 |
| Documents | ~60 |
| Calendar Events | ~28 |
| Deadlines | ~22 |
| Speedy Trial Clocks | 4 |
| Motions | ~20 |
| Evidence | ~15 |
| Parties | ~35 |
| Representations | ~30 |
| Service Records | ~40 |
| Orders | ~15 |
| Sentencing Records | 3 |
| Queue Items | 12 |
| Victims | 4 |
| **Total** | **~500+ rows** |

## Technical Notes

- All entities use well-known UUIDs (e.g., `d9case001-...`, `d12case001-...`) for idempotency
- Migration uses `DO $$ BEGIN ... END $$` with guard: skip if data already exists
- All inserts use `ON CONFLICT DO NOTHING` for re-runnability
- Dates use relative offsets from `NOW()` so data stays fresh
- FK ordering: courts → judges → attorneys → cases → defendants → charges → parties → representations → docket entries → documents → calendar events → deadlines → motions → evidence → orders → sentencing → queue items → victims → service records
