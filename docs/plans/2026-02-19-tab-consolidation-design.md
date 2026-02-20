# Case Detail Tab Consolidation Design

## Problem

The case detail page has 10 horizontal tabs (Overview, Case Info, Docket, Parties, Deadlines, Orders, Sentencing, Evidence, Calendar, Speedy Trial). This creates an unruly UX for clerks managing hundreds of cases. Additionally, the "Case Info" tab is a 2,200-line component that embeds the entire docket/filing/service system inline — a structural issue that makes maintenance difficult.

## Design Decision

**Docket-first, 5-tab layout with subtabs.** This mirrors how CM/ECF clerks think: the docket IS the case record, and everything else is either people, scheduling, or sentencing.

## New Tab Structure

```
┌──────────┬─────────┬──────────┬────────────┬─────────────┐
│ Overview │ Docket  │ Parties  │ Scheduling │ Sentencing  │
└──────────┴────┬────┴──────────┴─────┬──────┴─────────────┘
                │                     │
          ┌─────┴──────┐        ┌─────┴──────────┐
          │ Entries    │        │ Calendar       │
          │ Orders     │        │ Deadlines      │
          │ Evidence   │        │ Speedy Trial   │
          └────────────┘        └────────────────┘
```

### Tab 1: Overview (merged)

Absorbs current Overview + Case Info into one read-only landing page:
- Case metadata grid (number, status, crime type, district, priority, opened date)
- Assigned judge card
- Recent activity timeline
- Description section
- "Edit Case" button opens existing CaseFormSheet

### Tab 2: Docket (with subtabs)

Three subtabs via nested `Tabs` component:
- **Entries** (default): The full docket system — event composer, filing form, docket table, expandable detail panels, document actions (seal/strike/replace), NEF viewer, service records, attachment management. Extracted from the 2,200-line CaseInfoDisplay into `tabs/docket.rs`.
- **Orders**: Existing OrdersTab (draft/list orders).
- **Evidence**: Existing EvidenceTab (evidence list/management).

### Tab 3: Parties (standalone)

Existing PartiesTab, unchanged.

### Tab 4: Scheduling (with subtabs)

Three subtabs:
- **Calendar** (default): Existing CalendarTab.
- **Deadlines**: Existing DeadlinesTab.
- **Speedy Trial**: Existing SpeedyTrialTab.

### Tab 5: Sentencing (standalone)

Existing SentencingTab, unchanged. Complex enough and criminal-case-specific to warrant its own tab.

## Implementation Strategy

### Phase 1: Extract DocketTab from detail.rs

The CaseInfoDisplay component (lines 209-2413 of detail.rs) contains:
- ~100 lines of case metadata display (moves to merged Overview)
- ~2,100 lines of docket system (DocketTab, EventComposer, DocketEntryForm, FilingForm, DocketTable, DocketRow, EntryDetailPanel, FilesSection, FilingNoticeSection, ServiceSection, ServiceRecordTableRow, ServiceRecordForm, ServiceRecordRow, AttachmentRow, DocumentActionsSection, NefModal)

Extract all docket components into `tabs/docket.rs`. This is the biggest refactor — moves 2,100 lines out of detail.rs into a proper tab module.

### Phase 2: Merge Overview + Case Info

Create a new merged OverviewTab that combines:
- Case metadata grid (from old CaseInfoDisplay)
- Assigned judge card (from old OverviewTab)
- Recent activity timeline (from old OverviewTab)
- Read-only with "Edit Case" button

### Phase 3: Create container tabs

Create two new container components:
- `tabs/docket_container.rs` — nested Tabs with Entries/Orders/Evidence subtabs
- `tabs/scheduling.rs` — nested Tabs with Calendar/Deadlines/Speedy Trial subtabs

### Phase 4: Rewire detail.rs

Update CaseDetailView to use 5 main tabs pointing to the new container components. Delete CaseInfoDisplay. detail.rs drops from ~2,400 lines to ~200.

## Edit Scope

- `detail.rs` — gut CaseInfoDisplay, rewire 5 tabs
- `tabs/docket.rs` — new file, receives 2,100 lines from detail.rs
- `tabs/overview.rs` — merge case metadata into existing overview
- `tabs/docket_container.rs` — new file, subtab wrapper
- `tabs/scheduling.rs` — new file, subtab wrapper
- `tabs/mod.rs` — update module declarations
- No changes to: `orders.rs`, `evidence.rs`, `calendar_tab.rs`, `deadlines.rs`, `speedy_trial.rs`, `sentencing.rs`, `parties.rs`
