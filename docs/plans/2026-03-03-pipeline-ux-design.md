# Pipeline UX — Lead Triage & Prospect Detail Design

**Date:** 2026-03-03
**Status:** Approved

## Problem

The Pipeline kanban has leads but is not actionable:
- No way to change a prospect's stage (kanban is read-only)
- No closing date or source URL visible on cards — the two most important RFP signals
- Closing date and source URL are buried as freeform text in description instead of proper fields
- Edit dialog is a modal — too slow for triage
- Notes tab doesn't persist

## Goal

Make the pipeline useable for converting RFP leads to money. Optimised for someone new to BD: show the key signals at a glance, make the next action (open the RFP) one click, and make stage changes fast.

## Design

### Section 1: Data Model

**`myme-organizations` — Prospect struct:**

Add two new optional fields:
```rust
pub source_url: Option<String>,
pub closing_date: Option<String>,  // ISO 8601 date string e.g. "2026-04-01"
```

SQLite migration (additive, safe):
```sql
ALTER TABLE prospects ADD COLUMN source_url TEXT;
ALTER TABLE prospects ADD COLUMN closing_date TEXT;
```

**`myme-organizations` — Organization notes:**

Add `notes` TEXT column to `organizations` table:
```sql
ALTER TABLE organizations ADD COLUMN notes TEXT;
```

Add `get_notes(org_id)` and `set_notes(org_id, notes)` to `OrganizationStore`.

**`myme-integrations` — RFP import:**

`build_rfp_description` stops embedding "Source: ..." and "Closing: ..." lines.
Import populates `prospect.source_url` and `prospect.closing_date` from `RfpData` directly.

**`myme-ui` Rust bridge — new invokables on ProspectModel:**
- `get_prospect_source_url(index: i32) -> QString`
- `get_prospect_closing_date(index: i32) -> QString`
- `sort_prospects_by_closing_date()` — sorts in-memory Vec ascending, nulls last
- `update_prospect_stage(index: i32, stage: &QString)` — saves stage change immediately

**`myme-ui` Rust bridge — new invokables on OrganizationModel:**
- `get_org_notes(org_id: &QString) -> QString`
- `set_org_notes(org_id: &QString, notes: &QString)`

---

### Section 2: Lead Triage List

The Lead column is wider (320px vs 220px for other columns) and sorted by closing date ascending (soonest first, nulls at bottom).

**Card layout (Lead column only):**
```
┌────────────────────────────────────────┐
│ ● 3 days left          $45,000–$80,000 │  ← urgency badge + budget
│ Canadian Air and Precipitation Monit.. │  ← name (2 lines max)
│ Dept. of Environment (ECCC)            │  ← contact/org name
│                                        │
│ [🔗 Open RFP]  [✓ Qualify]  [✗ Skip]  │  ← action row (always visible)
└────────────────────────────────────────┘
```

**Urgency color coding on days-left badge:**
- Red (`#E57373`): ≤7 days
- Orange (`#FF8A65`): 8–21 days
- Yellow (`#F59E0B`): 22–60 days
- Grey (textSecondary): no closing date or >60 days

**Quick actions:**
- **Open RFP** — `Qt.openUrlExternally(sourceUrl)`, opens in system browser
- **Qualify →** — calls `update_prospect_stage(index, "qualified")`, card leaves Lead
- **Skip** — calls `update_prospect_stage(index, "lost")`, card leaves Lead

Other stage columns keep the existing 220px card layout (name + value + contact, click to open side panel).

---

### Section 3: Prospect Side Panel

Clicking a card (except action buttons) opens a side panel sliding in from the right. Works for all stages.

**Layout:** The Pipeline tab splits horizontally — kanban scrolls left, panel is fixed 380px on the right. Slides in with 200ms ease animation. Closes on Escape or clicking outside.

```
┌─────────────────────────────┬──────────────────────────────┐
│  Kanban (scrollable)        │  Prospect name (2 lines)     │
│                             │  ──────────────────────────  │
│                             │  Stage: [Lead ▾]             │
│                             │                              │
│                             │  Closing   Mar 6, 2026       │
│                             │            ● 3 days left     │
│                             │  Budget    $45,000–$80,000   │
│                             │  Contact   Dept. of Env.     │
│                             │  Email     rfp@eccc.gc.ca    │
│                             │                              │
│                             │  [🔗 Open Source RFP →]      │
│                             │                              │
│                             │  Description                 │
│                             │  ┌──────────────────────┐    │
│                             │  │ Scrollable read-only  │    │
│                             │  │ RFP text              │    │
│                             │  └──────────────────────┘    │
│                             │                              │
│                             │  [Edit]        [Delete]      │
└─────────────────────────────┴──────────────────────────────┘
```

**Stage ComboBox:** Shows current stage, all 7 options. Selecting a new value calls `update_prospect_stage` immediately — no Save button.

**Open Source RFP button:** Primary CTA. Calls `Qt.openUrlExternally(sourceUrl)`. Hidden if sourceUrl is empty.

**Edit button:** Opens the existing edit dialog (name, description, value, contact fields).

---

### Section 4: Stage Change & Notes Persistence

**Two mechanisms for stage change:**
1. Card quick actions on Lead column ("Qualify →", "Skip") — one-click, no panel
2. Side panel ComboBox for all stages — full 7-stage selector, saves immediately

**"Skip" behavior:** Sets stage to `lost`. Does not delete the prospect — recoverable.

**Notes tab persistence:**
- Org notes stored in `organizations.notes` TEXT column
- `TextArea` loads notes via `get_org_notes()` on tab open
- Saves via `set_org_notes()` debounced 500ms on text change

---

## Out of Scope

- Drag-and-drop between kanban columns
- Lead scoring / ranking algorithm
- Email compose from contact email
- Filter chips / search within leads
- Multi-organization bulk actions

## Files Affected

| Layer | Files |
|---|---|
| `myme-organizations` | `store.rs` (migration + notes methods), `lib.rs` (Prospect fields) |
| `myme-integrations` | `northcloud.rs` (populate source_url, closing_date; remove from description) |
| `myme-ui` Rust | `prospect_model.rs` (new invokables), `organization_model.rs` (org notes) |
| QML | `OrganizationDetailPage.qml` (Pipeline redesign + notes persistence) |
