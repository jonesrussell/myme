# Projects Feature Design

**Date**: 2026-01-21
**Status**: Approved

## Overview

Add a Projects feature to MyMe for tracking personal side projects with full two-way GitHub integration. Projects are backed by GitHub repositories, with tasks represented as GitHub issues displayed on a kanban board.

## Core Use Case

Personal side project tracking - a command center for your coding projects. Primary view shows status and progress at a glance. GitHub is the source of truth.

## Approach

**GitHub-First**: Projects are thin wrappers around GitHub repos. All tasks are GitHub issues. Kanban columns map to labels. MyMe acts as a specialized GitHub client.

Benefits:
- Single source of truth
- Works from any device (data lives on GitHub)
- No local sync conflicts
- Simple architecture

## Data Model

### Project (local cache)

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Local identifier |
| github_repo | String | Owner/name (e.g., "jonesrussell/myme") |
| description | String | Cached from repo or overridden locally |
| created_at | Timestamp | When added to MyMe |
| last_synced | Timestamp | Last GitHub sync |

### Task (mirrors GitHub Issue)

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Local identifier |
| github_issue_number | u32 | Issue number (e.g., #42) |
| project_id | UUID | FK to project |
| title | String | Issue title |
| body | String | Issue body (markdown) |
| status | Enum | Kanban column |
| labels | Vec<String> | Cached GitHub labels |
| last_synced | Timestamp | Last sync |

### Status ↔ GitHub Label Mapping

| MyMe Status | GitHub Label | Issue State |
|-------------|--------------|-------------|
| Backlog | `backlog` | open |
| Todo | `todo` | open |
| InProgress | `in-progress` | open |
| Blocked | `blocked` | open |
| Review | `review` | open |
| Done | (none needed) | closed |

## Project Creation & Note Promotion

### Three ways to create a project:

**1. Link Existing Repo**
- Click "Add Project" on dashboard
- Search your GitHub repos via API
- Select one → Project created, initial sync pulls open issues

**2. Create New Repo**
- Click "Add Project" → "New Repository"
- Enter name, description, public/private
- MyMe creates the repo via GitHub API
- Project created with empty task board

**3. Promote from Note**
- Note card gets "Promote to Project" action
- Dialog: Link existing repo or create new?
- If creating new: note title → repo name, note body → repo description + README.md
- If linking existing: note body → project description (local override)
- Original note is deleted after promotion

### Initial Sync Behavior

When linking an existing repo:
1. Fetch all open issues
2. Parse labels to determine kanban status (no label → "Todo" default)
3. Create missing status labels on repo if they don't exist (with permission)
4. Cache issue data locally
5. Show progress indicator on project card

## UI Design

### Dashboard (ProjectsPage.qml)

Grid of project cards showing:
- Repository name and owner
- Description (1-2 lines, truncated)
- Status summary bar (colored segments for task distribution)
- Last activity timestamp
- Sync status indicator

Cards sorted by last activity. Click to open kanban view.

Top actions:
- "Add Project" button
- Sync all button

### Kanban View (ProjectDetailPage.qml)

Six columns in horizontal scroll:
`Backlog → Todo → In Progress → Blocked → Review → Done`

Task cards show:
- Issue title
- Issue number (#42)
- Labels (colored chips)
- Truncated body preview (toggleable)

Interactions:
- Drag card between columns → Updates GitHub issue
- Click card → Opens detail panel
- "+" in column header → Create new issue
- Column headers show count

## GitHub Sync Mechanism

### Strategy: Polling + Immediate Push

No webhooks (would require server). Instead:
- **Pull**: Poll GitHub every 5 minutes (configurable)
- **Push**: Immediately push local changes when user acts

### Pull Sync (background)

1. Fetch issues updated since `last_synced`
2. For each issue:
   - Exists locally → update fields
   - New → create local task, parse labels for status
   - Closed remotely → move to Done
3. Update `last_synced`

### Push Sync (immediate)

| User Action | GitHub API Call |
|-------------|-----------------|
| Move card | PATCH labels, or close/reopen |
| Edit title/body | PATCH issue |
| Create task | POST new issue with labels |
| Delete task | Close issue |

### Conflict Handling

GitHub wins. Next pull overwrites local cache. Acceptable because:
- Personal projects (single user)
- GitHub is source of truth
- Edits push immediately (conflicts rare)

### Offline Behavior

Queue changes locally, push when online. Show "pending sync" indicator.

## Architecture

### myme-services (new: github.rs)

`GitHubClient` - Async client for GitHub REST API:
- `list_repos()`, `get_repo()`, `create_repo()`
- `list_issues()`, `create_issue()`, `update_issue()`, `close_issue()`
- `add_labels()`, `remove_labels()`

Uses `reqwest` with bearer token from `myme-auth`. Handles rate limits.

### myme-core (extend config.rs)

```toml
[projects]
sync_interval_minutes = 5
auto_create_labels = true
```

### myme-ui (new models)

- `ProjectModel` - QObject for dashboard (projects list)
- `KanbanModel` - QObject for kanban board (single project)

Methods: `fetch_projects()`, `add_project()`, `sync_project()`, `move_task()`, `create_task()`, etc.

### myme-ui/qml (new pages)

- `ProjectsPage.qml` - Dashboard
- `ProjectDetailPage.qml` - Kanban board
- `ProjectCard.qml` - Dashboard card component
- `TaskCard.qml` - Draggable kanban card

### Local Storage

SQLite database in config directory for caching. Keeps app fast without constant GitHub calls.

## Error Handling

| Error | Handling |
|-------|----------|
| Auth expired | Prompt re-auth, preserve pending changes |
| Rate limited | Back off, user message, retry later |
| Network offline | Queue changes, "offline" indicator |
| Repo inaccessible | Mark "disconnected", offer removal |
| Issue 404 | Remove from cache on next sync |

## Scope

### In Scope (v1)

- Link/create repos as projects
- Kanban board with 6 fixed columns
- Two-way issue sync with label-based status
- Note promotion to project
- Dashboard with summary cards
- Create/edit/move/delete tasks

### Out of Scope (YAGNI)

- Pull request tracking
- Multiple assignees / team features
- Milestones / GitHub project boards
- Time tracking
- Custom fields
- Issue comments (view only, no create/edit)
- Notifications / watching
- Cross-project views or reports
