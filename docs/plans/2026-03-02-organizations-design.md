# Organizations Feature Design

**Date:** 2026-03-02
**Status:** Approved
**Context:** Russell is doing BD for Web Networks (web.net). Needs a pipeline to manage prospects and a way to organize work by client organization.

## Overview

Add "Organizations" as a first-class entity in MyMe. An organization is a company/client that owns prospects (pipeline) and links to projects (delivery). The first organization will be Web Networks.

The funnel: **Organization -> Prospects -> Projects -> Tasks**

## Architecture Decision

**Approach A (chosen):** New `myme-organizations` crate with its own SQLite database. Follows the existing pattern where each domain (projects, notes, gmail, calendar) owns its storage and service layer independently.

## Data Model

### organizations table

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | UUID |
| name | TEXT NOT NULL | "Web Networks" |
| description | TEXT | What they do, general notes |
| website | TEXT | "https://web.net" |
| contact_name | TEXT | Primary contact |
| contact_email | TEXT | |
| contact_phone | TEXT | |
| contact_role | TEXT | |
| created_at | TEXT NOT NULL | ISO 8601 |
| updated_at | TEXT NOT NULL | ISO 8601 |

### prospects table

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | UUID |
| organization_id | TEXT NOT NULL | FK to organizations |
| name | TEXT NOT NULL | "Ontario Nature Website Rebuild" |
| description | TEXT | Scope, notes, context |
| stage | TEXT NOT NULL | Serde enum, see below |
| value | TEXT | Estimated dollar value as text |
| contact_name | TEXT | Person at prospect org |
| contact_email | TEXT | |
| contact_role | TEXT | |
| created_at | TEXT NOT NULL | ISO 8601 |
| updated_at | TEXT NOT NULL | ISO 8601 |

### organization_projects junction table

| Column | Type | Notes |
|--------|------|-------|
| organization_id | TEXT NOT NULL | FK to organizations |
| project_id | TEXT NOT NULL | FK to projects (in project store) |
| PRIMARY KEY | (organization_id, project_id) | |

Contact fields are plain text for now. Structured enough for a clean migration to a separate `contacts` table later.

## Rust Types

### ProspectStage enum

```
Lead -> Qualified -> Contacted -> Proposal -> Negotiation -> Won / Lost
```

Serialized as lowercase strings via serde, same pattern as TaskStatus.

### Structs

- `Organization` - all table fields
- `Prospect` - all table fields
- `OrganizationProject` - junction record

### OrganizationStore

- Same pattern as ProjectStore: `open(path)`, schema versioning, migrations
- Own SQLite file: `organizations.db`
- CRUD for organizations, prospects, junction
- `list_prospects_by_stage(org_id)` for pipeline view
- `count_prospects_by_stage(org_id)` for dashboard stats
- `link_project(org_id, project_id)` / `unlink_project`
- `convert_prospect_to_project(prospect_id)` returns data for project creation (UI handles actual creation via ProjectStore)

No cross-database queries. Junction stores project IDs as text. UI fetches project details from ProjectStore separately.

## Service Channel

```
OrganizationServiceMessage:
  LoadOrganizations
  CreateOrganization { ... }
  UpdateOrganization { ... }
  DeleteOrganization { id }
  LoadProspects { organization_id }
  CreateProspect { ... }
  UpdateProspect { ... }
  UpdateProspectStage { id, stage }  (drag-drop shortcut)
  DeleteProspect { id }
  LinkProject { org_id, project_id }
  UnlinkProject { org_id, project_id }
  ListLinkedProjects { org_id }
```

Uses mpsc channels with QML Timer polling, same as all other services.

## cxx-qt Bridge

Two models:

**OrganizationModel** - list view and CRUD. Exposes `row_count()`, `get_name(index)`, `get_id(index)`, etc.

**ProspectModel** - prospects for selected organization. Stage-based accessors: `prospects_for_stage(stage)`, `move_prospect(id, new_stage)`. Emits `prospects_changed` signal.

"Create Project from Prospect" flow: ProspectModel emits signal with prospect data, QML catches it and calls ProjectModel to create the project, then calls OrganizationModel to create junction link. No direct Rust coupling between stores.

## UI

### OrganizationsPage.qml (list view)

- Grid of organization cards (same style as ProjectsPage)
- Each card: name, website, prospect count, mini stage breakdown bar
- "Add Organization" button
- Click to navigate to detail page

### OrganizationDetailPage.qml (tabbed hub)

**Header:** Org name, website link, primary contact info, edit button

**Tab 1: Pipeline**
- Kanban board, 7 columns (Lead through Won/Lost)
- Drag-drop to change stage
- Cards show: prospect name, contact, value, age in days
- Same patterns as ProjectDetailPage Kanban

**Tab 2: Projects**
- List of linked projects
- "Link Existing Project" dropdown
- "Create Project" button on Won prospects
- Project cards link to ProjectDetailPage

**Tab 3: Notes**
- Editable description field for org-level context
- Meeting notes, strategy, general reference

### Navigation

"Organizations" entry in sidebar between Projects and Notes. Building/office icon.

### Styling

Warm Forge theme. No new design system work, existing patterns applied.

## Out of Scope (v1)

- Contacts as separate entity (text fields for now)
- Follow-up reminders / due dates on prospects
- Email integration (auto-linking Gmail threads)
- Reporting / analytics (win rates, pipeline totals)
- Import / bulk add prospects
- Search across organizations
