# Organizations Feature Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add Organizations as a first-class entity in MyMe with a prospect pipeline for BD work.

**Architecture:** New `myme-organizations` crate with own SQLite database, following existing patterns (ProjectStore, project_service, ProjectModel). Two new QML pages (list + detail with tabs). Organization service channel wired through AppServices/bridge.

**Tech Stack:** Rust 2021, rusqlite, cxx-qt 0.8, QML/Qt6, Phosphor icons, serde

**Design doc:** `docs/plans/2026-03-02-organizations-design.md`

---

### Task 1: Create myme-organizations crate with types

**Files:**
- Create: `crates/myme-organizations/Cargo.toml`
- Create: `crates/myme-organizations/src/lib.rs`
- Create: `crates/myme-organizations/src/models.rs`
- Modify: `Cargo.toml` (workspace members)

**Step 1: Create Cargo.toml**

```toml
# crates/myme-organizations/Cargo.toml
[package]
name = "myme-organizations"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
anyhow.workspace = true
tracing.workspace = true
rusqlite = { version = "0.31", features = ["bundled"] }
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
tempfile = "3"

[lints]
workspace = true
```

**Step 2: Create models.rs with types**

```rust
// crates/myme-organizations/src/models.rs

use serde::{Deserialize, Serialize};

/// Prospect pipeline stage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProspectStage {
    Lead,
    Qualified,
    Contacted,
    Proposal,
    Negotiation,
    Won,
    Lost,
}

impl ProspectStage {
    /// Get all stage variants in pipeline order
    pub fn all() -> &'static [ProspectStage] {
        &[
            ProspectStage::Lead,
            ProspectStage::Qualified,
            ProspectStage::Contacted,
            ProspectStage::Proposal,
            ProspectStage::Negotiation,
            ProspectStage::Won,
            ProspectStage::Lost,
        ]
    }

    /// Display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            ProspectStage::Lead => "Lead",
            ProspectStage::Qualified => "Qualified",
            ProspectStage::Contacted => "Contacted",
            ProspectStage::Proposal => "Proposal",
            ProspectStage::Negotiation => "Negotiation",
            ProspectStage::Won => "Won",
            ProspectStage::Lost => "Lost",
        }
    }
}

/// An organization (client, company, partner)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_role: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// A prospect in the BD pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prospect {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub description: Option<String>,
    pub stage: ProspectStage,
    pub value: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_role: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

**Step 3: Create lib.rs**

```rust
// crates/myme-organizations/src/lib.rs
pub mod models;
pub mod store;

pub use models::*;
pub use store::OrganizationStore;
```

**Step 4: Add to workspace Cargo.toml**

Add `"crates/myme-organizations"` to the `members` list in the root `Cargo.toml`.

**Step 5: Write tests for types**

Add to bottom of `models.rs`:

```rust
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_prospect_stage_serde_roundtrip() {
        let stage = ProspectStage::Negotiation;
        let json = serde_json::to_string(&stage).unwrap();
        assert_eq!(json, "\"negotiation\"");
        let parsed: ProspectStage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, stage);
    }

    #[test]
    fn test_prospect_stage_all() {
        assert_eq!(ProspectStage::all().len(), 7);
    }

    #[test]
    fn test_prospect_stage_display_names() {
        assert_eq!(ProspectStage::Lead.display_name(), "Lead");
        assert_eq!(ProspectStage::Won.display_name(), "Won");
    }
}
```

**Step 6: Run tests**

Run: `cd /home/fsd42/dev/myme && cargo test -p myme-organizations`
Expected: All 3 tests pass

**Step 7: Commit**

```bash
git add crates/myme-organizations/ Cargo.toml
git commit -m "feat(organizations): add myme-organizations crate with types"
```

---

### Task 2: Implement OrganizationStore (SQLite)

**Files:**
- Create: `crates/myme-organizations/src/store.rs`
- Modify: `crates/myme-organizations/src/lib.rs`

**Step 1: Write failing tests for store**

Add test module at bottom of `store.rs` (write the full file with tests first, implementation stubs that fail):

```rust
#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use tempfile::tempdir;

    fn test_store() -> OrganizationStore {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        OrganizationStore::open(&db_path).unwrap()
    }

    fn test_org() -> Organization {
        Organization {
            id: "org-1".to_string(),
            name: "Web Networks".to_string(),
            description: Some("Canadian nonprofit ISP".to_string()),
            website: Some("https://web.net".to_string()),
            contact_name: Some("Greg MacKenzie".to_string()),
            contact_email: Some("greg@web.net".to_string()),
            contact_phone: None,
            contact_role: Some("Director".to_string()),
            created_at: "2026-03-02T00:00:00Z".to_string(),
            updated_at: "2026-03-02T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_create_and_list_organizations() {
        let store = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let orgs = store.list_organizations().unwrap();
        assert_eq!(orgs.len(), 1);
        assert_eq!(orgs[0].name, "Web Networks");
    }

    #[test]
    fn test_get_organization() {
        let store = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let org = store.get_organization("org-1").unwrap();
        assert!(org.is_some());
        assert_eq!(org.unwrap().website.unwrap(), "https://web.net");
    }

    #[test]
    fn test_delete_organization_cascades() {
        let store = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let prospect = Prospect {
            id: "p-1".to_string(),
            organization_id: "org-1".to_string(),
            name: "Ontario Nature".to_string(),
            description: None,
            stage: ProspectStage::Lead,
            value: None,
            contact_name: None,
            contact_email: None,
            contact_role: None,
            created_at: "2026-03-02T00:00:00Z".to_string(),
            updated_at: "2026-03-02T00:00:00Z".to_string(),
        };
        store.upsert_prospect(&prospect).unwrap();
        store.delete_organization("org-1").unwrap();
        assert!(store.list_prospects("org-1").unwrap().is_empty());
        assert!(store.get_organization("org-1").unwrap().is_none());
    }

    #[test]
    fn test_prospect_crud() {
        let store = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let prospect = Prospect {
            id: "p-1".to_string(),
            organization_id: "org-1".to_string(),
            name: "CUPE Local 79".to_string(),
            description: Some("Website rebuild".to_string()),
            stage: ProspectStage::Lead,
            value: Some("$15,000".to_string()),
            contact_name: Some("Jane Doe".to_string()),
            contact_email: None,
            contact_role: None,
            created_at: "2026-03-02T00:00:00Z".to_string(),
            updated_at: "2026-03-02T00:00:00Z".to_string(),
        };
        store.upsert_prospect(&prospect).unwrap();
        let prospects = store.list_prospects("org-1").unwrap();
        assert_eq!(prospects.len(), 1);
        assert_eq!(prospects[0].name, "CUPE Local 79");
    }

    #[test]
    fn test_update_prospect_stage() {
        let store = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let prospect = Prospect {
            id: "p-1".to_string(),
            organization_id: "org-1".to_string(),
            name: "Test".to_string(),
            description: None,
            stage: ProspectStage::Lead,
            value: None,
            contact_name: None,
            contact_email: None,
            contact_role: None,
            created_at: "2026-03-02T00:00:00Z".to_string(),
            updated_at: "2026-03-02T00:00:00Z".to_string(),
        };
        store.upsert_prospect(&prospect).unwrap();
        store.update_prospect_stage("p-1", ProspectStage::Contacted).unwrap();
        let prospects = store.list_prospects("org-1").unwrap();
        assert_eq!(prospects[0].stage, ProspectStage::Contacted);
    }

    #[test]
    fn test_count_prospects_by_stage() {
        let store = test_store();
        store.upsert_organization(&test_org()).unwrap();
        for (i, stage) in [ProspectStage::Lead, ProspectStage::Lead, ProspectStage::Contacted].iter().enumerate() {
            let p = Prospect {
                id: format!("p-{}", i),
                organization_id: "org-1".to_string(),
                name: format!("Prospect {}", i),
                description: None,
                stage: *stage,
                value: None,
                contact_name: None,
                contact_email: None,
                contact_role: None,
                created_at: "2026-03-02T00:00:00Z".to_string(),
                updated_at: "2026-03-02T00:00:00Z".to_string(),
            };
            store.upsert_prospect(&p).unwrap();
        }
        let counts = store.count_prospects_by_stage("org-1").unwrap();
        let lead_count = counts.iter().find(|(s, _)| *s == ProspectStage::Lead).map(|(_, c)| *c).unwrap_or(0);
        assert_eq!(lead_count, 2);
    }

    #[test]
    fn test_project_linking() {
        let store = test_store();
        store.upsert_organization(&test_org()).unwrap();
        store.link_project("org-1", "proj-abc").unwrap();
        store.link_project("org-1", "proj-def").unwrap();
        let projects = store.list_linked_projects("org-1").unwrap();
        assert_eq!(projects.len(), 2);
        store.unlink_project("org-1", "proj-abc").unwrap();
        let projects = store.list_linked_projects("org-1").unwrap();
        assert_eq!(projects.len(), 1);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p myme-organizations`
Expected: Compilation errors (store functions don't exist yet)

**Step 3: Implement OrganizationStore**

Write the full `store.rs` implementation following `project_store.rs` patterns:
- `open(path)` with schema versioning
- `init_schema()` creating all 3 tables with indexes
- CRUD for organizations: `upsert_organization`, `list_organizations`, `get_organization`, `delete_organization`
- CRUD for prospects: `upsert_prospect`, `list_prospects(org_id)`, `delete_prospect`, `update_prospect_stage`
- Query: `count_prospects_by_stage(org_id)`
- Junction: `link_project`, `unlink_project`, `list_linked_projects`

Schema version starts at 1. Use `params![]` macro, `OptionalExtension` for gets. Serialize `ProspectStage` via `serde_json::to_string`/`from_str` same as `TaskStatus`.

**Step 4: Run tests to verify they pass**

Run: `cargo test -p myme-organizations`
Expected: All 7 tests pass

**Step 5: Commit**

```bash
git add crates/myme-organizations/src/store.rs crates/myme-organizations/src/lib.rs
git commit -m "feat(organizations): implement OrganizationStore with SQLite"
```

---

### Task 3: Wire into AppServices and bridge

**Files:**
- Modify: `crates/myme-ui/Cargo.toml` (add myme-organizations dependency)
- Create: `crates/myme-ui/src/services/organization_service.rs`
- Modify: `crates/myme-ui/src/services/mod.rs`
- Modify: `crates/myme-ui/src/app_services.rs`
- Modify: `crates/myme-ui/src/bridge.rs`

**Step 1: Add dependency to myme-ui**

In `crates/myme-ui/Cargo.toml`, add to `[dependencies]`:
```toml
myme-organizations = { path = "../myme-organizations" }
```

**Step 2: Create organization_service.rs**

Follow `project_service.rs` pattern. This service is synchronous (no GitHub API calls), so messages are simpler:

```rust
// crates/myme-ui/src/services/organization_service.rs

use myme_organizations::{Organization, Prospect, ProspectStage};

/// Error type for organization operations
#[derive(Debug, Clone)]
pub enum OrganizationError {
    Database(String),
    NotInitialized,
}

impl std::fmt::Display for OrganizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrganizationError::Database(s) => write!(f, "Organization error: {}", s),
            OrganizationError::NotInitialized => write!(f, "Organization service not initialized"),
        }
    }
}

impl std::error::Error for OrganizationError {}

/// Messages for the organization service channel
#[derive(Debug)]
pub enum OrganizationServiceMessage {
    OrganizationsLoaded(Result<Vec<Organization>, OrganizationError>),
    ProspectsLoaded(Result<Vec<Prospect>, OrganizationError>),
    ProspectStageUpdated(Result<(), OrganizationError>),
    LinkedProjectsLoaded(Result<Vec<String>, OrganizationError>),
}
```

**Step 3: Register in services/mod.rs**

Add `pub mod organization_service;` and the corresponding `pub use` exports.

**Step 4: Add to AppServices**

Follow the exact pattern of project_store:
- Add `organization_store: RwLock<Option<Arc<parking_lot::Mutex<OrganizationStore>>>>` field
- Add `init_organization_store()` method (opens `organizations.db` in config_dir)
- Add `organization_store()` getter
- Add `organization` to the `service_channel_methods!` and `service_channel_shutdown!` macros
- Add the corresponding tx/rx fields

**Step 5: Add bridge functions**

Add to `bridge.rs`:
- `organization_store_or_init()` convenience function
- Add `organization` to the `service_channel_bridge!` macro

**Step 6: Run full build**

Run: `cargo build -p myme-ui 2>&1 | head -20`
Expected: Compiles successfully (no QML models using it yet, so no link errors)

**Step 7: Commit**

```bash
git add crates/myme-ui/Cargo.toml crates/myme-ui/src/services/ crates/myme-ui/src/app_services.rs crates/myme-ui/src/bridge.rs
git commit -m "feat(organizations): wire OrganizationStore into AppServices and bridge"
```

---

### Task 4: Create OrganizationModel (cxx-qt bridge)

**Files:**
- Create: `crates/myme-ui/src/models/organization_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`

**Step 1: Create organization_model.rs**

Follow `project_model.rs` pattern. Key differences:
- No GitHub client needed (all local SQLite)
- Simpler initialization (just organization store)
- Exposes org list data + prospect counts per org

```rust
// Struct fields:
// - loading: bool
// - error_message: QString
// - organizations: Vec<Organization>
// - prospect_counts: HashMap<String, ProspectCounts>  (stage counts per org)
// - organization_store: Option<Arc<Mutex<OrganizationStore>>>

// QML-invokable methods:
// - fetch_organizations()  (loads from store)
// - row_count() -> i32
// - get_id(index) -> QString
// - get_name(index) -> QString
// - get_description(index) -> QString
// - get_website(index) -> QString
// - get_contact_name(index) -> QString
// - get_contact_email(index) -> QString
// - get_prospect_counts(index) -> QString  (JSON like task_counts)
// - create_organization(name, description, website, contact_name, contact_email, contact_phone, contact_role)
// - delete_organization(index)
// - poll_channel()

// Signals:
// - organizations_changed()
```

Implement `ensure_initialized()` calling `bridge::get_organization_store_or_init()`. All operations are synchronous (local SQLite), so no async service needed for basic CRUD. The service channel is reserved for future async operations.

**Step 2: Create ProspectCounts helper**

```rust
#[derive(Default, Clone)]
struct ProspectCounts {
    lead: i32,
    qualified: i32,
    contacted: i32,
    proposal: i32,
    negotiation: i32,
    won: i32,
    lost: i32,
}

impl ProspectCounts {
    fn to_json(&self) -> String {
        format!(
            r#"{{"lead":{},"qualified":{},"contacted":{},"proposal":{},"negotiation":{},"won":{},"lost":{}}}"#,
            self.lead, self.qualified, self.contacted, self.proposal, self.negotiation, self.won, self.lost
        )
    }

    fn from_stage_counts(counts: &[(ProspectStage, i32)]) -> Self {
        let mut result = Self::default();
        for (stage, count) in counts {
            match stage {
                ProspectStage::Lead => result.lead = *count,
                ProspectStage::Qualified => result.qualified = *count,
                ProspectStage::Contacted => result.contacted = *count,
                ProspectStage::Proposal => result.proposal = *count,
                ProspectStage::Negotiation => result.negotiation = *count,
                ProspectStage::Won => result.won = *count,
                ProspectStage::Lost => result.lost = *count,
            }
        }
        result
    }

    fn total(&self) -> i32 {
        self.lead + self.qualified + self.contacted + self.proposal + self.negotiation + self.won + self.lost
    }
}
```

**Step 3: Register in models/mod.rs**

Add `pub mod organization_model;`

**Step 4: Build and verify**

Run: `cargo build -p myme-ui 2>&1 | head -20`
Expected: Compiles (cxx-qt will generate the C++ bridge code)

**Step 5: Commit**

```bash
git add crates/myme-ui/src/models/organization_model.rs crates/myme-ui/src/models/mod.rs
git commit -m "feat(organizations): add OrganizationModel cxx-qt bridge"
```

---

### Task 5: Create ProspectModel (cxx-qt bridge)

**Files:**
- Create: `crates/myme-ui/src/models/prospect_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`

**Step 1: Create prospect_model.rs**

This model handles prospects for a selected organization. It provides the data for the pipeline Kanban view.

```rust
// Struct fields:
// - loading: bool
// - error_message: QString
// - organization_id: QString  (currently selected org)
// - prospects: Vec<Prospect>
// - organization_store: Option<Arc<Mutex<OrganizationStore>>>

// QML-invokable methods:
// - load_prospects(organization_id: &QString)
// - prospect_count() -> i32
// - get_prospect_id(index) -> QString
// - get_prospect_name(index) -> QString
// - get_prospect_description(index) -> QString
// - get_prospect_stage(index) -> QString
// - get_prospect_value(index) -> QString
// - get_prospect_contact_name(index) -> QString
// - get_prospect_contact_email(index) -> QString
// - get_prospect_created_at(index) -> QString
// - prospects_for_stage(stage: &QString) -> QString  (JSON array of indices)
// - count_for_stage(stage: &QString) -> i32
// - move_prospect(index: i32, new_stage: &QString)  (drag-drop)
// - create_prospect(org_id, name, description, stage, value, contact_name, contact_email, contact_role)
// - update_prospect(index, name, description, value, contact_name, contact_email, contact_role)
// - delete_prospect(index)
// - linked_projects() -> QString  (JSON array of project IDs for current org)
// - link_project(project_id: &QString)
// - unlink_project(project_id: &QString)
// - poll_channel()

// Signals:
// - prospects_changed()
// - prospect_converted(name: QString, description: QString)  (emitted when Won prospect triggers project creation)
```

**Step 2: Register in models/mod.rs**

Add `pub mod prospect_model;`

**Step 3: Build and verify**

Run: `cargo build -p myme-ui 2>&1 | head -20`
Expected: Compiles

**Step 4: Commit**

```bash
git add crates/myme-ui/src/models/prospect_model.rs crates/myme-ui/src/models/mod.rs
git commit -m "feat(organizations): add ProspectModel cxx-qt bridge"
```

---

### Task 6: Add Phosphor building icon

**Files:**
- Modify: `crates/myme-ui/qml/Icons.qml`

**Step 1: Add building icon**

Add to the Navigation section of `Icons.qml`:
```qml
readonly property string buildings: "\ue9c1"
```

(Phosphor icon `buildings` = `\ue9c1`. Verify against Phosphor icon set if this codepoint is incorrect.)

**Step 2: Commit**

```bash
git add crates/myme-ui/qml/Icons.qml
git commit -m "feat(organizations): add buildings icon to Phosphor icon set"
```

---

### Task 7: Create OrganizationsPage.qml (list view)

**Files:**
- Create: `crates/myme-ui/qml/pages/OrganizationsPage.qml`
- Modify: `qml.qrc`

**Step 1: Create OrganizationsPage.qml**

Follow `ProjectsPage.qml` pattern. Grid of organization cards with:
- Organization name (bold)
- Website (secondary text, clickable)
- Contact name
- Mini prospect stage bar (same concept as task status bars on project cards)
- Prospect count badge
- "Add Organization" button at top

Include `OrganizationModel` instance with Timer polling pattern.

**Step 2: Add to qml.qrc**

Add: `<file>crates/myme-ui/qml/pages/OrganizationsPage.qml</file>`

**Step 3: Build and verify**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles

**Step 4: Commit**

```bash
git add crates/myme-ui/qml/pages/OrganizationsPage.qml qml.qrc
git commit -m "feat(organizations): add OrganizationsPage list view"
```

---

### Task 8: Create OrganizationDetailPage.qml (tabbed hub)

**Files:**
- Create: `crates/myme-ui/qml/pages/OrganizationDetailPage.qml`
- Modify: `qml.qrc`

**Step 1: Create OrganizationDetailPage.qml**

Tabbed page with 3 tabs:

**Header:** Org name, website, contact info, edit button (inline editing)

**Tab 1: Pipeline**
- Kanban board with 7 columns (Lead, Qualified, Contacted, Proposal, Negotiation, Won, Lost)
- Prospect cards showing: name, contact, value, days since created
- Drag-drop between columns (uses `move_prospect`)
- "Add Prospect" button
- Won prospects get a "Create Project" button
- Follow `ProjectDetailPage.qml` Kanban patterns for drag-drop

**Tab 2: Projects**
- List of linked projects (fetched via `linked_projects()`)
- "Link Project" button with dropdown of existing projects
- Each project card links to ProjectDetailPage
- Won prospect "Create Project" creates project and links it

**Tab 3: Notes**
- Editable TextArea bound to organization description
- Auto-saves on edit (with debounce)

Include `ProspectModel` instance with Timer polling pattern.

**Step 2: Add to qml.qrc**

Add: `<file>crates/myme-ui/qml/pages/OrganizationDetailPage.qml</file>`

**Step 3: Build and verify**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles

**Step 4: Commit**

```bash
git add crates/myme-ui/qml/pages/OrganizationDetailPage.qml qml.qrc
git commit -m "feat(organizations): add OrganizationDetailPage with Pipeline, Projects, Notes tabs"
```

---

### Task 9: Wire into navigation (sidebar, shortcuts, Main.qml)

**Files:**
- Modify: `crates/myme-ui/qml/components/Sidebar.qml`
- Modify: `crates/myme-ui/qml/Main.qml`

**Step 1: Add to sidebar ListModel**

In `Sidebar.qml`, add between Projects and Repos in the `navModel`:
```qml
ListElement { title: "Organizations"; page: "OrganizationsPage"; icon: "" }
```

**Step 2: Add icon mapping**

In the `getNavIcon` function, add:
```javascript
"OrganizationsPage": Icons.buildings,
```

**Step 3: Add keyboard shortcut in Main.qml**

Renumber existing shortcuts to make room. Add Organizations as Ctrl+5 (after Projects at Ctrl+4... actually, insert between Projects and Repos). Adjust numbering:
- Ctrl+5 stays as ProjectsPage
- Add `Shortcut { sequence: "Ctrl+9"; onActivated: root.navigateToPage("OrganizationsPage") }` (or renumber as appropriate)

Actually, simpler: just add Ctrl+9 for Organizations to avoid renumbering.

**Step 4: Add navigation handler for detail page**

In `Main.qml` or `AppContext`, ensure the StackView can push `OrganizationDetailPage.qml` when an org card is clicked. The OrganizationsPage will call:
```javascript
AppContext.pageStack.push(Qt.resolvedUrl("OrganizationDetailPage.qml"), { organizationId: orgId, organizationName: orgName })
```

**Step 5: Build and verify**

Run: `cargo build 2>&1 | head -20`
Expected: Compiles

**Step 6: Commit**

```bash
git add crates/myme-ui/qml/components/Sidebar.qml crates/myme-ui/qml/Main.qml
git commit -m "feat(organizations): add sidebar navigation and keyboard shortcut"
```

---

### Task 10: Integration test and manual verification

**Step 1: Run full test suite**

Run: `cargo test --workspace`
Expected: All tests pass

**Step 2: Build full application**

Run: `cargo build`
Expected: Clean build

**Step 3: Manual testing checklist**

If the app can be launched, verify:
- [ ] Organizations appears in sidebar
- [ ] Can create an organization (Web Networks)
- [ ] Organization card shows in grid
- [ ] Can click into organization detail page
- [ ] Pipeline tab shows 7 columns
- [ ] Can create a prospect
- [ ] Can drag prospect between stages
- [ ] Projects tab shows linked projects
- [ ] Notes tab shows editable description

**Step 4: Final commit if any fixes needed**

```bash
git add -A
git commit -m "feat(organizations): integration fixes"
```
