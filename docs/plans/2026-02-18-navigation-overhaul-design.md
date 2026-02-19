# Navigation Overhaul + Global Search Design

**Goal:** Replace the database-admin-style sidebar (20+ items organized by table) with a workflow-first sidebar (7 items) matching real CM/ECF patterns. Add Tantivy-powered command palette for global search. Delete redundant standalone pages.

**Core principle:** Everything flows through cases. Queue → Case → Tabs.

---

## Sidebar

**New structure (3 groups, 7 items):**

```
WORK
  Queue           → /dashboard (clerk inbox)
  Cases           → /cases (browse/filter)
  Calendar        → /calendar (court schedule)

PEOPLE
  Attorneys       → /attorneys
  Judges          → /judges

ADMIN
  Reports         → /reports (new placeholder)
  Settings        → /settings
```

## Command Palette (Cmd+K)

Tantivy-powered full-text search. Activates via:
- Cmd+K keyboard shortcut
- Click the existing search icon in the header

**Indexed entities (phase 1):**
- Cases: case_number, title, crime_type, status, defendant names
- Attorneys: name, bar_number, firm
- Judges: name, title, courtroom

**UX:** Spotlight-style overlay. Results grouped by type. Recent items shown when empty. Click result → navigate to detail page.

**Tech:** `tantivy = "0.22"` crate. In-memory index built at server startup from DB. Searched via server function. No external service dependency.

## Pages to Delete

Remove these standalone pages, their route variants, and sidebar entries:

| Page | Reason |
|------|--------|
| /defendants, /defendants/:id | Case detail tab |
| /parties, /parties/:id | Case detail tab |
| /victims | Case detail tab |
| /deadlines, /deadlines/:id | Case detail tab |
| /docket | Case detail tab |
| /filings | Case detail tab |
| /service-records | Case detail tab |
| /orders, /orders/:id | Case detail tab |
| /opinions, /opinions/:id | Case detail tab |
| /evidence | Case detail tab |
| /documents | Case detail tab |
| /sentencing | Case detail tab |
| /compliance | Merged into /reports |
| /rules | Merged into /reports |
| /users | Merged into /settings or admin |
| /products | Template scaffolding, not court-related |

## Pages to Keep

| Page | Purpose |
|------|---------|
| /dashboard | Clerk work queue |
| /cases | Case list with search/filter |
| /cases/new | Create case form |
| /cases/:id | Case detail (all tabs — the hub) |
| /calendar | Global court calendar |
| /attorneys | Attorney list |
| /attorneys/:id | Attorney detail |
| /judges | Judge list |
| /judges/:id | Judge detail |
| /settings | User settings |
| /reports | Reports/compliance (new) |

## Workflow

```
Clerk opens app
  → Lands on Queue (dashboard)
  → Sees pending items sorted by priority
  → Claims an item
  → Navigates to Case Detail
  → Uses tabs: Docket, Parties, Deadlines, Orders, etc.
  → Advances queue item through pipeline steps
  → Returns to Queue

At any point: Cmd+K to search for any case or person
```
