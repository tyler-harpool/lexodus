# Lexodus UI/UX Design — CMECF Modernization

## Goal

Design every screen and workflow for Lexodus, mapping all 396 API endpoints across 48 domains to a modern, role-adaptive, cross-platform UI. This is a full redesign with command palette, role-specific dashboards, workflow wizards, notification center, and real-time updates.

## Design Decisions

- **User personas:** Role-adaptive — UI changes based on Clerk, Judge, Attorney, Admin, or Public role
- **Navigation:** Grouped sidebar with 6 categories, role-based visibility
- **Case detail:** Tabbed sections (9 tabs) as the central hub
- **List pages:** DataTable + Sheet pattern (current proven approach)
- **Approach:** Full redesign — pattern-driven pages + workflow wizards + command palette + notifications

## Tech Stack

- Dioxus (Rust) — fullstack SSR + WASM client
- shared-ui crate — 47+ component primitives (DataTable, Sheet, Tabs, Card, etc.)
- 5 theme families (Federal, Chambers, Parchment, Cyberpunk, Solarized)
- CSS custom properties design token system

---

## 1. Navigation & Role System

### Sidebar Groups (6 categories)

**1. Core** (all roles)
- Dashboard
- Search (global)
- Notifications

**2. Case Management** (Clerk, Judge, Attorney)
- Cases
- Defendants
- Parties
- Victims
- Speedy Trial

**3. Court Operations** (Clerk, Judge)
- Calendar
- Deadlines
- Docket
- Filings
- Service Records
- NEFs (Notice of Electronic Filing)

**4. Legal Documents** (all roles, varies)
- Orders
- Opinions
- Evidence
- Documents
- PDF Generation

**5. People & Organizations** (Clerk, Admin)
- Attorneys
- Judges
- Users
- Conflict Checks
- Representations

**6. Administration** (Admin, Clerk)
- Configuration
- Compliance
- Rules
- Billing
- Extensions
- Feature Flags

### Role Visibility Matrix

| Group | Admin | Clerk | Judge | Attorney | Public |
|-------|-------|-------|-------|----------|--------|
| Core | All | All | All | All | Dashboard + Search |
| Case Management | All | All | Read + assigned | Own cases | Public dockets |
| Court Operations | All | All | Own calendar/deadlines | Own deadlines | None |
| Legal Documents | All | All | Author opinions/orders | View filed | Public opinions |
| People & Orgs | All | All | View | Own profile | Attorney lookup |
| Administration | All | Most | None | None | None |

### Command Palette (Cmd+K)

- Search across all entities (cases, attorneys, judges, docket entries)
- Quick actions: "Create new case", "File document", "Schedule hearing"
- Recent items (last 10 viewed entities)
- Navigation shortcuts to any page

---

## 2. Role-Adaptive Dashboards

### Clerk Dashboard
- **Active Cases** — count with trend indicator, click to filter case list
- **Pending Filings** — documents awaiting processing, urgency badges
- **Upcoming Deadlines** — next 7 days, color-coded by urgency (red/yellow/green)
- **Today's Calendar** — court events, expandable to full calendar
- **Compliance Stats** — deadline compliance rate, overdue items
- **Recent Docket Activity** — live feed of new entries, NEFs sent
- **Quick Actions:** "New Case", "File Document", "Process Filing", "Schedule Event"

### Judge Dashboard
- **My Caseload** — assigned cases by status (active/pending/trial)
- **Pending Motions** — motions awaiting ruling, sorted by age
- **Sentencing Calendar** — upcoming sentencing dates with defendant info
- **Opinion Drafts** — in-progress opinions with collaborator status
- **My Calendar** — today's hearings, conferences, trial days
- **Recent Orders** — orders issued this week
- **Quick Actions:** "Draft Opinion", "Issue Order", "Review Motion", "Recusal"

### Attorney Dashboard
- **My Cases** — active representations with status
- **Upcoming Deadlines** — filing deadlines with countdown timers
- **Court Calendar** — scheduled appearances
- **Recent Docket Entries** — new activity on my cases (NEF-style notifications)
- **CJA Status** — appointment panel status, voucher status (if CJA attorney)
- **Quick Actions:** "File Document", "Request Extension", "Check Deadlines"

### Admin Dashboard
- **System Health** — server status, database metrics
- **Tenant Stats** — courts managed, users active
- **User Management** — recent registrations, role changes
- **Compliance Overview** — cross-court compliance rates
- **Billing Summary** — tier distribution, revenue
- **Feature Flags** — active experiments

### Public Dashboard
- **Case Search** — prominent search bar for public docket lookup
- **Recent Opinions** — published opinions, searchable
- **Attorney Lookup** — find attorneys by name, bar number
- **Court Calendar** — public hearings schedule

---

## 3. Complete Screen Inventory

Pattern key: **L** = List (DataTable + pagination + search + filters), **C** = Create (Sheet), **D** = Detail, **S** = Specialized

### Core (4 screens)

| Screen | Type | Endpoints | Notes |
|--------|------|-----------|-------|
| Dashboard | S | Multiple | Role-adaptive |
| Global Search | S | All search | Federated search |
| Notification Center | S | NEFs, reminders | Real-time bell + page |
| Settings | S | auth/*, config/* | Already built |

### Case Management (17 screens)

| Screen | Type | Endpoints | Notes |
|--------|------|-----------|-------|
| Case List | L | GET /cases | Filters: status, crime type, judge |
| Case Create | C | POST /cases | Wizard: basic → parties → assignment |
| Case Detail | S | GET /cases/{id} | 9-tab hub (see Section 4) |
| Defendant List | L | GET /defendants | Tab within case detail |
| Defendant Create | C | POST /defendants | Sheet from within case |
| Defendant Detail | D | GET /defendants/{id} | Charges, plea history |
| Party List | L | GET /parties | Tab within case detail |
| Party Create | C | POST /parties | Sheet with type selector |
| Party Detail | D | GET /parties/{id} | Representations linked |
| Victim List | L | GET /victims | Tab within case detail |
| Victim Create | C | POST /victims | Sheet for victim info |
| Victim Detail | D | GET /victims/{id} | Notification preferences |
| Speedy Trial Panel | S | GET /speedy-trial/* | Timeline visualization |
| Speedy Trial Exclusions | C | POST /speedy-trial/exclude | Sheet for exclusion periods |
| Case Seal | C | POST /cases/{id}/seal | Confirmation dialog |
| Case Unseal | C | POST /cases/{id}/unseal | Confirmation dialog |
| Case Statistics | S | GET /cases/statistics | Charts and metrics |

### Court Operations (20 screens)

| Screen | Type | Endpoints | Notes |
|--------|------|-----------|-------|
| Calendar List | L | GET /calendar | Month/week/day view |
| Calendar Create | C | POST /calendar | Event type, courtroom, participants |
| Calendar Detail | D | GET /calendar/{id} | Attendees, conflicts |
| Calendar Courtroom View | S | courtroom-utilization | Gantt-like schedule |
| Deadline List | L | GET /deadlines | Filters: urgency, type, case |
| Deadline Create | C | POST /deadlines | Auto-calculate from rules |
| Deadline Detail | D | GET /deadlines/{id} | Extension history, reminders |
| Deadline Compliance | S | compliance-report | Charts, stats, drill-down |
| Extension Request | C | POST /extensions | Sheet from deadline detail |
| Reminder Create | C | POST /reminders | Sheet from deadline detail |
| Docket List | L | GET /docket | Full docket sheet |
| Docket Entry Create | C | POST /docket | Filing type, document attach |
| Docket Entry Detail | D | GET /docket/{id} | Attachments, service status |
| Docket Search | S | GET /docket/search | Full-text search |
| Filing List | L | GET /filings | Pending/processed filings |
| Filing Create | C | POST /filings | Document upload + metadata |
| Filing Detail | D | GET /filings/{id} | Processing status |
| Service Record List | L | GET /service-records | Per-document tracking |
| Service Record Create | C | POST /service-records | Bulk create option |
| NEF List | L | GET /nefs | Electronic filing notices |

### Legal Documents (18 screens)

| Screen | Type | Endpoints | Notes |
|--------|------|-----------|-------|
| Order List | L | GET /orders | Filters: type, judge, case |
| Order Create | C | POST /orders | Template selector + editor |
| Order Detail | D | GET /orders/{id} | PDF preview, signing |
| Order Template List | L | GET /order-templates | Admin: manage templates |
| Order Template Editor | S | POST/PUT /order-templates | Template authoring |
| Opinion List | L | GET /opinions | Published + draft views |
| Opinion Create | C | POST /opinions | Case, type, panel |
| Opinion Detail | D | GET /opinions/{id} | Full text, citations, votes |
| Opinion Draft Editor | S | POST /opinions/{id}/drafts | Rich text workspace |
| Opinion Citations | L | GET citations | Citation management |
| Opinion Votes | S | GET/POST votes | Panel voting interface |
| Evidence List | L | GET /evidence | Chain of custody tracking |
| Evidence Create | C | POST /evidence | Upload + metadata + custody |
| Evidence Detail | D | GET /evidence/{id} | Custody chain timeline |
| Document List | L | GET /documents | All case documents |
| Document Upload | C | POST /documents | File upload + classification |
| Document Detail | D | GET /documents/{id} | Preview, seal/strike |
| PDF Batch Generator | S | POST /pdf/batch | Batch generation queue |

### People & Organizations (16 screens)

| Screen | Type | Endpoints | Notes |
|--------|------|-----------|-------|
| Attorney List | L | GET /attorneys | Partially built |
| Attorney Create | C | POST /attorneys | Bio → bar → admissions |
| Attorney Detail | D | GET /attorneys/{id} | Tabbed: Profile, Admissions, CJA, Cases, Metrics |
| Attorney Bar Admissions | L/C | bar-admissions/* | Tab within attorney detail |
| Attorney CJA Panel | S | cja-panel/* | Appointment management |
| Attorney Pro Hac Vice | L/C | pro-hac-vice/* | PHV request management |
| Attorney Metrics | S | metrics, case-load | Performance dashboard |
| Judge List | L | GET /judges | Filters: status, district |
| Judge Create | C | POST /judges | Profile + assignment prefs |
| Judge Detail | D | GET /judges/{id} | Tabbed: Profile, Caseload, Opinions, Conflicts |
| Judge Conflict Mgmt | S | conflicts/*, recusals/* | Conflict + recusal workflow |
| Judge Workload | S | GET /judges/workload | Distribution visualization |
| User List | L | GET /users | Admin: all users |
| User Detail | D | GET /users/{id} | Role management, activity |
| Conflict Check | S | POST /conflict-checks | Cross-entity conflict search |
| Representation History | L | GET /representations | Attorney-case linkage |

### Sentencing Module (8 screens)

| Screen | Type | Endpoints | Notes |
|--------|------|-----------|-------|
| Sentencing List | L | GET /sentencing | Pending, upcoming, by defendant |
| Sentencing Detail | D | GET /sentencing/{id} | Tabbed: Overview, Guidelines, Departures, Conditions |
| Sentencing Create | C | POST /sentencing | Linked to case + defendant |
| Guidelines Calculator | S | calculate-guidelines | Interactive calculator |
| Offense Level Builder | S | calculate-offense-level | Step-by-step builder |
| Departure Manager | S | departures/* | Tracking departures |
| Sentencing Statistics | S | statistics/* | Multi-dimensional analytics |
| Supervised Release | S | supervised-release/* | Post-sentencing tracking |

### Administration (10 screens)

| Screen | Type | Endpoints | Notes |
|--------|------|-----------|-------|
| Configuration | S | GET/PUT /config | Court-specific settings |
| Configuration Preview | S | GET /config/preview | Preview before applying |
| Compliance Dashboard | S | GET /compliance/* | Cross-domain compliance |
| Rules Management | L/C/D | GET/POST/PUT /rules | Court rules CRUD |
| Billing Overview | S | GET /billing/* | Tier management, invoices |
| Extension Management | L | GET /extensions | System-wide extensions |
| Feature Flags | S | GET /features | Toggle features per court |
| Tenant Management | S | POST /admin/tenants/* | Multi-tenant admin |
| Tenant Stats | S | GET /admin/tenants/stats | Usage analytics |
| Health Monitor | S | GET /api/health | System health dashboard |

**Total: ~93 screens** covering all 396 endpoints.

---

## 4. Case Detail Hub (Central Screen)

### Layout

```
┌─────────────────────────────────────────────────────────┐
│ PageHeader: "United States v. Rodriguez"                │
│ Badge: Active | Badge: Felony | Badge: District 9       │
│ Actions: [Edit] [Seal] [PDF] [...]                      │
├─────────────────────────────────────────────────────────┤
│ Overview | Docket | Parties | Deadlines | Orders |      │
│ Sentencing | Evidence | Calendar | Speedy Trial          │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  [Active Tab Content Area]                              │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Tab Contents

**Overview** — Case summary card, assigned judge hover card, key dates timeline, recent activity feed (10 latest docket entries/orders/filings), quick stats (parties, pending deadlines, docket count)

**Docket** — Full docket sheet DataTable (entry number, date, description, filed by). Filters: entry type, date range, filed by. Search within entries. "New Entry" → Sheet. Click entry → inline expand with attachments + service records.

**Parties** — Defendant section: cards with name, charges, plea status, counsel. Prosecution section: AUSA assigned. Other parties: witnesses, intervenors. Each party links to representations. "Add Party" → Sheet with type selector.

**Deadlines** — DataTable: type, due date, status (countdown badge), assigned to. Urgency colors: overdue (red), <3 days (yellow), >3 days (green). "Calculate Deadlines" auto-generates statutory deadlines. "Request Extension" → Sheet. "Set Reminder" → Sheet.

**Orders** — DataTable: type, date issued, judge, status. "Draft Order" → template selector → editor. Click → detail with PDF preview.

**Sentencing** (visible after conviction) — Sentencing summary, guidelines calculator, departure history, supervised release terms, "Prepare Sentencing" workflow launcher.

**Evidence** — DataTable: exhibit number, description, type, custody status. Chain of custody timeline. "Add Evidence" → Sheet. Seal/strike actions.

**Calendar** — Upcoming events for this case. Mini calendar view. "Schedule Event" → Sheet.

**Speedy Trial** — Visual timeline of clock with running day count. Exclusion periods highlighted. "Add Exclusion" → Sheet. Clock status: running / tolled / expired.

---

## 5. Secondary Hub Screens

### Attorney Detail (7 tabs)
- **Profile** — Contact info, firm, status, photo
- **Admissions** — State bar + federal court admissions table
- **CJA** — Panel membership, appointments, vouchers
- **Cases** — Active representations with case links
- **Metrics** — Win rate, case load, performance charts
- **Discipline** — Disciplinary actions history
- **Pro Hac Vice** — PHV applications and status

### Judge Detail (7 tabs)
- **Profile** — Bio, chambers, courtroom, status
- **Caseload** — Assigned cases DataTable
- **Calendar** — Personal judicial calendar
- **Opinions** — Authored opinions with publication status
- **Conflicts** — Conflict records and recusal history
- **Workload** — Metrics and case distribution
- **Vacation** — Vacation schedule management

---

## 6. Workflow Wizards

### Wizard 1: New Case Filing
**Steps:** Case Info → Defendant(s) → Parties → Judge Assignment (with conflict check) → Initial Docket → Deadline Calculation → Speedy Trial Start → Review & File

**API chain:** POST /cases → POST /defendants → POST /parties → POST /representations → POST /judges/assignments → POST /docket → POST /deadlines/calculate → POST /speedy-trial/start

### Wizard 2: Sentencing Preparation
**Steps:** Select Defendant → Offense Level Calculation → Criminal History → Guidelines Range Lookup → Departures → Special Conditions → BOP Designation → Review

### Wizard 3: Attorney Onboarding
**Steps:** Attorney Profile → Bar Admissions → Federal Admissions → Practice Areas → ECF Registration → CJA Panel (optional) → Review & Submit

### Wizard 4: Opinion Drafting
**Steps:** Select Case → Opinion Type → Draft Editor → Citations → Headnotes → Panel Review/Votes → Publish

### Wizard 5: Bulk Document Filing
**Steps:** Select Case → Upload Documents → Classify → Service Records → NEF → Confirm

### Wizard 6: Conflict Check
**Steps:** Select Entity → Select Case → Auto-Scan → Review Results → Action (Clear/Flag/Recuse)

---

## 7. Cross-Cutting Features

### Global Search (Cmd+K)
Federated search across all entities. Quick actions. Recent items. Scoped search within case context.

### Notification Center
Bell icon with unread badge. Types: NEF received, deadline approaching, case assignment changes, opinion comments, extension responses, calendar reminders. Full page view with filters.

### Real-Time Activity Feed
Dashboard and Case Detail overview. Chronological actions on user's cases. Clickable entries navigate to relevant entity.

### PDF Generation
Single document, batch PDF, in-browser preview, federal court formatting.

### Bulk Operations
Attorney bulk status update, service record bulk create, deadline bulk extend.

### Keyboard Shortcuts
- `Cmd+K` — Command palette
- `Cmd+N` — New (context-aware)
- `Cmd+S` — Save current form
- `Escape` — Close sheet/dialog
- `J/K` — Navigate up/down in lists
- `Enter` — Open selected item
- `?` — Show keyboard shortcuts help

---

## 8. UI Components Used

All screens built from the existing shared-ui primitives:

| Component | Usage |
|-----------|-------|
| DataTable | Every list page (attorneys, cases, docket, etc.) |
| Sheet | Every create/edit form (slide-over panel) |
| Tabs | Case detail, attorney detail, judge detail, sentencing detail |
| Card | Dashboard widgets, summary cards, entity cards |
| Badge | Status indicators, urgency levels, role badges |
| PageHeader | Every page top section with title + actions |
| SearchBar | Every list page filter bar |
| Pagination | Every DataTable footer |
| AlertDialog | Confirmation for delete, seal, unseal, destructive actions |
| HoverCard | Quick preview on entity references (judge name, case link) |
| Accordion | Collapsible sections in detail pages |
| Skeleton | Loading states for all async data |
| Progress | Compliance rates, wizard step progress |
| Calendar | Calendar pages, date pickers in forms |
| Tooltip | Action button hints, abbreviation explanations |
| Button | Primary/Secondary/Destructive actions everywhere |
| Form/Input/Label | All create/edit forms in sheets |
| Separator | Visual section dividers |

---

## 9. Implementation Scope

- **~93 screens** to build
- **6 workflow wizards** spanning multiple domains
- **5 role-adaptive dashboards**
- **3 tabbed hub pages** (Case, Attorney, Judge)
- **1 command palette** with federated search
- **1 notification center** with real-time updates
- **~44 new domain modules** (4 already built: attorneys, cases, calendar, deadlines)
