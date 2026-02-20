# Criminal Case Workflow Design

## Goal

Transform Lexodus from a data storage system into an event-driven case management system where every user action fires a trigger through the compliance engine, automating case progression, deadline generation, work routing, and notifications.

## Architecture

Every user action (filing, ruling, signing, scheduling) creates a DB record, fires a trigger event, and the compliance engine evaluates rules to produce downstream effects: status changes, deadlines, queue items, and notifications. The three core repeating workflows (Filing, Motion Ruling, Order Signing) compose into the full criminal case lifecycle from filing through appeal.

## Principles

- **Event-driven**: No hardcoded workflow logic. Rules engine drives all automation.
- **Role-centric UI**: Each role sees actions relevant to their work, not generic CRUD forms.
- **Inline actions**: Judges rule and sign without navigating away from their dashboard.
- **Queue-driven clerk work**: All clerk tasks arrive as auto-created queue items.
- **Compliance-first filing**: Every filing is evaluated by the engine before acceptance.

---

## 1. Event System: Trigger Events

### Existing Triggers (already seeded as lifecycle rules)
| Trigger | Fires When |
|---------|-----------|
| `complaint_filed` | Criminal complaint/indictment docketed |
| `answer_filed` | Answer/response to complaint filed |
| `judgment_entered` | Verdict or judgment entered on case |
| `sentencing_scheduled` | Sentencing hearing scheduled |
| `document_filed` | Generic document filed on docket |

### New Triggers to Add
| Trigger | Fires When | Engine Response |
|---------|-----------|-----------------|
| `motion_filed` | Attorney files any motion | Create clerk queue item, set briefing schedule (response: 14d, reply: 7d) |
| `motion_response_filed` | Opposition/response to motion filed | Start reply deadline (7 days), notify movant |
| `motion_ruled` | Judge rules on motion (grant/deny/etc.) | Auto-draft order, advance status if dispositive (e.g., motion to dismiss granted → "dismissed") |
| `order_drafted` | Clerk or system drafts an order | Create queue item for judge signature |
| `order_signed` | Judge signs an order | Route to clerk queue for filing + service |
| `order_filed` | Clerk files signed order on docket | Generate NEF, create service deadline (3 days) |
| `hearing_scheduled` | Any hearing created | Notify all parties, generate preparation deadlines |
| `hearing_completed` | Hearing marked completed | Generate minute entry draft, trigger post-hearing deadlines |
| `plea_entered` | Defendant enters plea at arraignment/change-of-plea | If guilty → advance to "awaiting_sentencing", generate PSR deadline (90d) |
| `verdict_returned` | Jury or bench verdict entered | Advance to "awaiting_sentencing", generate sentencing scheduling deadline |
| `case_assigned` | Judge assigned to case | Notify judge, generate initial scheduling conference deadline |
| `service_completed` | Proof of service recorded | Close service deadline, mark filing as served |
| `deadline_expired` | System: deadline passes without action | Alert all parties, create escalation queue item |
| `extension_requested` | Attorney requests deadline extension | Create queue item for judge review |
| `extension_ruled` | Judge grants/denies extension | Update deadline if granted, notify parties |

### Event Pipeline (every action)
```
User Action
  -> Create DB record (motion, order, filing, etc.)
    -> Fire trigger event with FilingContext
      -> Compliance engine evaluates matching rules
        -> ComplianceReport contains:
           - status_changes -> auto-applied to case
           - deadlines -> auto-created in deadlines table
           - queue_items -> auto-created in clerk_queue
           - notifications -> sent to affected parties (future: email/NEF)
           - fees -> assessed if applicable
           - blocks -> prevent action if prerequisites missing
```

---

## 2. Criminal Case Lifecycle

### Stage Flow
```
FILING -> ARRAIGNMENT -> DISCOVERY -> PRETRIAL -> TRIAL -> SENTENCING -> APPEAL
```

### Stage 1: Case Filing
| Role | Action | System Response |
|------|--------|-----------------|
| Attorney (AUSA) | Files complaint/indictment | Engine: `complaint_filed` -> queue item for clerk |
| Clerk | Reviews filing, validates format | Dockets entry, assigns CM/ECF case number |
| Clerk | Routes for judge assignment | Engine: `case_assigned` -> notify judge, generate scheduling deadline |
| Judge | Receives case on dashboard | Appears in "New Assignments" section |

### Stage 2: Arraignment
| Role | Action | System Response |
|------|--------|-----------------|
| Clerk | Schedules arraignment hearing | Engine: `hearing_scheduled` -> notify parties, generate deadlines |
| Judge | Conducts arraignment, records plea | Engine: `plea_entered` |
| Judge | Sets bail/detention conditions | Engine: auto-drafts detention/release order |
| Clerk | Issues detention/release order | Engine: `order_filed` -> NEF, service |
| System | If guilty plea entered | Engine: advance to "awaiting_sentencing", generate PSR deadline |

### Stage 3: Discovery
| Role | Action | System Response |
|------|--------|-----------------|
| Attorney (both) | Files discovery requests/responses | Engine: `document_filed` -> clerk queue, docket entry |
| Attorney | Files motions (suppress, compel, etc.) | Engine: `motion_filed` -> clerk queue, briefing schedule |
| Opposing counsel | Files response to motion | Engine: `motion_response_filed` -> reply deadline |
| Judge | Rules on motions | Engine: `motion_ruled` -> auto-draft order, advance status if dispositive |
| Clerk | Issues discovery orders | Engine: `order_filed` -> NEF, deadlines |
| System | Discovery deadline expires | Engine: `deadline_expired` -> alert, case eligible for pretrial |

### Stage 4: Pretrial Motions / Plea Negotiations
| Role | Action | System Response |
|------|--------|-----------------|
| Attorney | Files pretrial motions (limine, dismiss) | Same motion workflow |
| Judge | Rules on motions, holds pretrial conference | Engine evaluates, may advance status |
| Attorney | Plea negotiations -> plea agreement filed | Queue item for judge review |
| Judge | Accepts/rejects plea | Engine: `plea_entered` if accepted -> advance to sentencing |
| Clerk | Processes plea paperwork | Engine: auto-generates sentencing deadlines |

### Stage 5: Trial
| Role | Action | System Response |
|------|--------|-----------------|
| Clerk | Prepares trial calendar | Engine: `hearing_scheduled` for each trial day |
| Judge | Presides, rules on objections | Minute entries auto-docketed via `hearing_completed` |
| Attorney | Presents case, files trial motions | Real-time filings through standard flow |
| Judge/Jury | Returns verdict | Engine: `verdict_returned` -> advance to sentencing, generate deadlines |

### Stage 6: Sentencing
| Role | Action | System Response |
|------|--------|-----------------|
| Clerk | Orders PSR | Engine: generates 90-day deadline for probation office |
| Attorney (both) | Files sentencing memos | Standard filing workflow |
| Judge | Imposes sentence | Engine: `sentencing_scheduled` -> advance status, generate appeal deadline (14d) |
| Clerk | Issues Judgment & Commitment order | Engine: `order_filed` -> NEF, BOP notification deadline |

---

## 3. Three Core Workflows

### 3A. Filing Workflow (Attorney -> Clerk -> Docket)
```
Attorney submits filing
  -> Engine evaluates: compliance check, fee check, format validation
  -> If blocked: show reasons, prevent submission
  -> If clear: filing record created with status="pending"
  -> Queue item created for clerk (type: "filing", priority from engine)
  -> Clerk claims queue item
  -> Clerk reviews filing
  -> [Docket] -> docket entry created, fires trigger (motion_filed, document_filed, etc.)
     -> Engine evaluates lifecycle rules
     -> Downstream effects auto-applied (deadlines, status changes, queue items)
  -> [Return to Filer] -> filing status="returned", attorney notified
  -> [Reject] -> filing status="rejected", attorney notified with reason
```

### 3B. Motion Ruling Workflow (Attorney -> Clerk -> Judge -> Clerk)
```
Motion filed (via Filing Workflow, trigger: motion_filed)
  -> Engine: create briefing schedule deadlines
     - Response due: 14 days
     - Reply due: 7 days after response
  -> Engine: create queue item to route to judge after briefing complete

Response filed (trigger: motion_response_filed)
  -> Engine: start reply deadline (7 days)
  -> If no reply after 7 days: motion ripe for ruling

Judge reviews motion (from dashboard "Pending Motions"):
  -> Sees: motion text, response, reply, related orders
  -> Actions: [Grant] [Deny] [Grant in Part] [Set for Hearing] [Take Under Advisement]
  -> Ruling text field (pre-filled template based on disposition)
  -> Submit fires "motion_ruled" trigger
  -> Engine:
     - Creates order from ruling (status: "draft" or "pending_signature")
     - If dispositive (dismiss, summary judgment): may advance case status
     - Generates post-ruling deadlines if applicable
  -> Queue item for clerk: file the order

If "Set for Hearing":
  -> Creates hearing scheduling queue item for clerk
  -> After hearing: judge rules (same flow above)
```

### 3C. Order Workflow (Draft -> Sign -> File -> Serve)
```
Order created (from motion ruling, or clerk-drafted):
  -> Status: "draft" or "pending_signature"
  -> If pending_signature: queue item for judge

Judge reviews order:
  -> [Sign] -> fires "order_signed", status: "signed"
     -> Queue item for clerk to file
  -> [Return with Notes] -> status: "draft", clerk notified
  -> [Reject] -> status: "rejected", clerk notified with reason

Clerk files signed order:
  -> Fires "order_filed" trigger
  -> Engine: create docket entry, generate NEF, create service deadline
  -> Status: "filed"

Service:
  -> Clerk records service on each party
  -> Fires "service_completed" per party
  -> Engine: close service deadline when all parties served
```

---

## 4. Role-Specific UI Design

### 4A. Judge UI

**Dashboard ("Judicial Dashboard"):**
Three sections, ordered by urgency:

1. **Orders Pending Signature** (existing, enhanced)
   - Each order: expandable inline preview
   - Actions: [Sign] [Return to Clerk] — no navigation needed
   - Signing fires `order_signed` trigger

2. **Pending Motions** (existing, enhanced)
   - Each motion: expandable to show full briefing chain (motion + response + reply)
   - Status indicator: "Fully Briefed" / "Awaiting Response" / "Awaiting Reply"
   - Actions (only when fully briefed): [Grant] [Deny] [Grant in Part] [Set for Hearing] [Under Advisement]
   - Ruling fires `motion_ruled` trigger

3. **Today's Hearings** (existing, enhanced)
   - Each hearing: case title, type, courtroom, parties
   - Link to case detail with hearing prep context
   - Post-hearing: [Mark Complete] with minute entry notes

**Case Detail (Judge view):**
- Action buttons: only [Recuse] (no Edit, no Delete, no Queue)
- Tabs show case data + judge-specific actions on relevant tabs
- Orders tab: can sign orders inline
- Docket tab: read-only (judge doesn't docket)

**Permission Changes for Judge:**
- Remove: QUEUE, EDIT, DELETE buttons from case detail header
- Add: RECUSE button (if assigned judge)
- Orders tab: [Sign] button visible
- Motions (in docket): [Rule] button visible on pending motions

### 4B. Clerk UI

**Dashboard (Queue-centric):**
Enhanced existing clerk queue with:

1. **Stats Cards** (existing): Pending, My Items, Today, Urgent
2. **Queue Items** (existing, enhanced):
   - Each item shows: case number, filing type, priority, current pipeline step
   - Pipeline visualization: review -> docket -> nef -> route_judge -> serve -> complete
   - Claim -> opens inline action panel (not full page navigation)
3. **Recently Completed** (new): last 10 items processed today

**Queue Action Panel (inline, not navigate away):**
Depending on queue type:
- **Filing**: [Docket] [Return to Filer] [Reject] + compliance warnings from engine
- **Order to File**: [File on Docket] [Return to Judge] + order preview
- **Service**: [Record Service] per party + proof of service fields
- **Deadline Alert**: [Extend] [Dismiss] + escalation options

**Case Detail (Clerk view):**
- Action buttons: [EDIT] [CASES] (no DELETE unless admin)
- Full tab access with create/edit capabilities
- Orders tab: [Draft Order] [Issue Signed Order] buttons
- Docket tab: [Add Entry] button, full filing processing

**Permission Changes for Clerk:**
- Remove: DELETE button (admin only)
- Keep: EDIT, CASES
- Add: Queue-specific actions based on pipeline step

### 4C. Attorney UI

**Dashboard:**
Three sections:

1. **Action Needed** (new — replaces "Filing Deadlines")
   - Deadlines approaching with [File Response] / [File Motion] / [Request Extension] actions
   - Color-coded: red (<3 days), yellow (<7 days), green (>7 days)
   - Filing action opens inline form, not separate page

2. **Recent Orders on My Cases** (new)
   - Orders filed in last 7 days on attorney's cases
   - Shows: order type, judge, case, date
   - Helps attorney track what judge decided

3. **Upcoming Appearances** (existing)
   - Hearings with case context and preparation links

**Case Detail (Attorney view):**
- Action buttons: [CASES] [FILE MOTION] [FILE DOCUMENT] (no Edit, no Delete)
- Docket tab: read-only + [File] button for responses to existing entries
- Deadlines tab: [Request Extension] button on approaching deadlines
- No access to: Edit case metadata, manage judges, sentencing (unless assigned)

**Permission Changes for Attorney:**
- Remove: EDIT, DELETE, QUEUE buttons
- Add: FILE MOTION, FILE DOCUMENT buttons
- Deadlines: [Request Extension] visible
- Docket: [File Response] on entries that expect response

### 4D. Public UI

- **No dashboard** (no MY WORK section)
- **Can search cases** via command palette / search page (public case info only)
- **Can view public information**: case status, parties, non-sealed docket entries, public orders
- **Case detail**: read-only, no action buttons
- **Docket**: visible entries only (sealed entries hidden, restricted documents redacted)
- **No filing, no actions, no queue**

---

## 5. New Rule Actions to Implement

Extend the existing 9 RuleAction types:

| Action | Status | Implementation Needed |
|--------|--------|----------------------|
| `GenerateDeadline` | Enforced | Already works |
| `AdvanceStatus` | Enforced | Already works |
| `StartSpeedyTrial` | Enforced | Already works |
| `BlockFiling` | Evaluated | **Wire into filing submission** (reject if blocked) |
| `RequireFee` | Evaluated | **Wire into filing form** (show fee, collect payment) |
| `SendNotification` | Not implemented | **New: in-app notification + future email** |
| `FlagForReview` | Not implemented | **New: create queue item with review flag** |
| `RequireRedaction` | Not implemented | **Defer** (future document management) |
| `LogCompliance` | Not implemented | **New: write to audit log table** |
| `CreateQueueItem` | **New action type** | Auto-create clerk queue item with type/priority |
| `GenerateOrder` | **New action type** | Auto-draft order from template based on trigger context |
| `NotifyParties` | **New action type** | Notify all parties on the case (future: NEF email) |

---

## 6. Database Changes Needed

### New Tables
- `case_events` — audit log of all trigger events fired on a case (event_type, actor_id, timestamp, context JSON)
- `notifications` — in-app notifications per user (message, read/unread, link_to)

### Modified Tables
- `motions` — add: `ruling_disposition` (enum), `ruling_text`, `ruling_date`, `ruling_judge_id`, `briefing_status` (enum: awaiting_response, awaiting_reply, fully_briefed, ruling_issued)
- `clerk_queue` — add: `trigger_event` (what created this item), `source_entity_id` (motion/order/filing that triggered it)
- `rules` — new seed rules for all new triggers

### New Indexes
- `case_events(case_id, event_type, created_at)` — for case timeline queries
- `notifications(user_id, read, created_at)` — for unread notification count

---

## 7. Implementation Priority

### Phase 1: Motion Ruling Workflow (Judge's #1 need)
- Add ruling UI to judge dashboard (inline grant/deny/etc.)
- Fire `motion_ruled` trigger
- Auto-draft order from ruling
- Seed rules for `motion_filed`, `motion_ruled` triggers
- Fix permission gating (remove Edit/Delete/Queue from judge view)

### Phase 2: Order Signing Pipeline (Judge + Clerk handoff)
- Add [Sign] button on judge dashboard orders
- Fire `order_signed` trigger -> auto-create clerk queue item
- Clerk [File on Docket] action fires `order_filed`
- Seed rules for `order_signed`, `order_filed` triggers

### Phase 3: Filing Workflow (Attorney -> Clerk)
- Attorney filing form with compliance engine pre-check
- Filing creates queue item (auto via engine)
- Clerk processes filing from queue (inline panel)
- Seed rules for `motion_filed`, `document_filed` triggers

### Phase 4: Case Event Audit Trail
- `case_events` table + migration
- Log every trigger event with actor and context
- Case detail timeline shows events (not just docket entries)

### Phase 5: Notifications + Deadline Escalation
- In-app notification system
- `SendNotification` rule action wired
- `deadline_expired` system trigger (cron or background task)
- Notification bell in header shows unread count

### Phase 6: Attorney Dashboard Actions
- [File Response] / [File Motion] / [Request Extension] inline actions
- Recent Orders section
- Filing compliance feedback (engine blocks/warnings shown in form)
