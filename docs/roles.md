# Lexodus User Roles

> **Status:** Based on code analysis. Needs verification against running app.

## Role Resolution

Roles are resolved at two levels:

1. **Platform level** -- `AuthUser.role` (e.g. "admin")
2. **Court level** -- `AuthUser.court_roles` maps `court_id -> role_string`

The `use_user_role()` hook combines both: platform admins get `Admin` everywhere, everyone else gets their per-court role or `Public` if they have no membership in the selected court.

## Roles

### Public (default)

No court membership. Can view public information only.

### Attorney

Filing and viewing role. Represents counsel of record.

**Expected access (needs verification):**

- View cases, docket entries, calendar events, deadlines, attorney records
- File documents via the Filing form
- Attach files to docket entries
- Download attachments
- Cannot add text docket entries
- Cannot promote attachments to official documents
- Cannot seal/unseal, replace, or strike documents
- Cannot manage users or court memberships

### Clerk

Administrative workhorse. Manages the docket and court operations.

**Expected access (needs verification):**

- Full CRUD on cases (create, edit, delete, status changes)
- Add text docket entries
- File documents
- Promote attachments to official documents ("Register as Filed Document")
- Seal/unseal documents (with sealing level and reason code)
- Replace document files (grace-period aware)
- Strike documents from the record
- Create and delete attorney records
- Manage court memberships (assign/remove roles for users in their court)
- Change user tiers

### Judge

Limited administrative role focused on judicial actions.

**Expected access (needs verification):**

- View and edit cases
- Add text docket entries
- File documents
- Seal/unseal documents

**Potential gaps vs Clerk (unverified):**

- May not be able to promote attachments to documents (code shows `Clerk | Admin` only)
- May not be able to replace or strike documents
- May not be able to manage users or court memberships

These gaps need to be tested in the running app to determine if they are intentional or bugs.

### Admin (Platform)

Full access everywhere. Platform-level role that bypasses all court-level checks.

- All Clerk powers in every court
- All Judge powers in every court
- Access to admin panel (Enterprise tier)
- Access to analytics (Pro+ tier)
- User management across all courts

## Permission Matrix

| Action | Public | Attorney | Clerk | Judge | Admin |
|--------|--------|----------|-------|-------|-------|
| View cases | ? | ? | ? | ? | ? |
| Create cases | ? | ? | ? | ? | ? |
| Edit/delete cases | ? | ? | ? | ? | ? |
| Add docket entries | ? | ? | ? | ? | ? |
| File documents | ? | ? | ? | ? | ? |
| Promote attachments | ? | ? | ? | ? | ? |
| Seal/unseal docs | ? | ? | ? | ? | ? |
| Replace doc files | ? | ? | ? | ? | ? |
| Strike documents | ? | ? | ? | ? | ? |
| View document events | ? | ? | ? | ? | ? |
| Create attorneys | ? | ? | ? | ? | ? |
| Manage users/roles | ? | ? | ? | ? | ? |
| Schedule calendar events | ? | ? | ? | ? | ? |
| Create deadlines | ? | ? | ? | ? | ? |

**Legend:** `?` = needs manual verification in the running app

## Code References

- Auth state & hooks: `crates/app/src/auth.rs`
- Role enum: `crates/shared-types/src/models.rs` (UserRole)
- Court context: `crates/app/src/main.rs` (CourtContext)
- Docket entry gating: `crates/app/src/routes/cases/detail.rs` line 473 (`can_add_entry`)
- Attachment promotion gating: `crates/app/src/routes/cases/detail.rs` line 1643 (`can_promote`)
- Document actions gating: `crates/app/src/routes/cases/detail.rs` line 1127 (`can_manage_docs`)
- User management gating: `crates/app/src/routes/users.rs` (`use_can_manage_memberships`)
- REST role checks: `crates/server/src/rest/document.rs` (`require_clerk_or_judge`)
- Court role resolution: `crates/server/src/auth/court_role.rs`

## Next Steps

1. Run the app (`dx serve`)
2. Log in as each role (attorney, clerk, judge) in district9
3. Walk through every page and action
4. Fill in the permission matrix with confirmed results
5. File bugs for any unintended permission gaps
