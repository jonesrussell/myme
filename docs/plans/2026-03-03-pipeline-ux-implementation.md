# Pipeline UX Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make the prospect pipeline actionable — richer Lead cards with urgency signals, a slide-in side panel with stage change, and org notes persistence.

**Architecture:** Four-layer change: (1) DB schema migration adds `source_url`/`closing_date` to `prospects` and `notes` to `organizations`; (2) integrations layer stops embedding these in description text; (3) Rust bridge gets new read invokables + org-notes methods; (4) QML Pipeline tab redesigned with wider Lead column, urgency badges, quick-action buttons, and a 380px side panel.

**Tech Stack:** Rust, rusqlite, cxx-qt, QML (Qt 6)

---

### Task 1: DB migration + Prospect struct fields

**Files:**
- Modify: `crates/myme-organizations/src/models.rs`
- Modify: `crates/myme-organizations/src/store.rs`

**Step 1: Write failing test for new Prospect fields round-trip**

Add to the `#[cfg(test)]` block in `store.rs`:

```rust
#[test]
fn test_prospect_source_url_and_closing_date() {
    let (store, _dir) = test_store();
    store.upsert_organization(&test_org()).unwrap();
    let prospect = Prospect {
        id: "p-rfp".to_string(),
        organization_id: "org-1".to_string(),
        name: "Air Quality RFP".to_string(),
        description: None,
        stage: ProspectStage::Lead,
        value: None,
        contact_name: None,
        contact_email: None,
        contact_role: None,
        source_url: Some("https://buyandsell.gc.ca/rfp/123".to_string()),
        closing_date: Some("2026-04-10".to_string()),
        created_at: "2026-03-03T00:00:00Z".to_string(),
        updated_at: "2026-03-03T00:00:00Z".to_string(),
    };
    store.upsert_prospect(&prospect).unwrap();
    let loaded = store.list_prospects("org-1").unwrap();
    assert_eq!(loaded[0].source_url.as_deref(), Some("https://buyandsell.gc.ca/rfp/123"));
    assert_eq!(loaded[0].closing_date.as_deref(), Some("2026-04-10"));
}

#[test]
fn test_org_notes_get_set() {
    let (store, _dir) = test_store();
    store.upsert_organization(&test_org()).unwrap();
    // defaults to empty
    assert_eq!(store.get_org_notes("org-1").unwrap(), "");
    // set and retrieve
    store.set_org_notes("org-1", "Follow up next week").unwrap();
    assert_eq!(store.get_org_notes("org-1").unwrap(), "Follow up next week");
    // overwrite
    store.set_org_notes("org-1", "Updated notes").unwrap();
    assert_eq!(store.get_org_notes("org-1").unwrap(), "Updated notes");
}

#[test]
fn test_schema_migration_v1_to_v2() {
    // Verifies that a fresh store lands on v2 and has the new columns
    let (store, _dir) = test_store();
    let ver: i32 = store.conn.query_row(
        "SELECT version FROM schema_version LIMIT 1",
        [],
        |r| r.get(0),
    ).unwrap();
    assert_eq!(ver, 2);
    // new columns exist — insert would fail if they didn't
    store.conn.execute(
        "INSERT INTO prospects (id, organization_id, name, stage, source_url, closing_date, created_at, updated_at)
         VALUES ('t', 'x', 'n', '\"lead\"', 'https://x.com', '2026-01-01', 'now', 'now')",
        [],
    ).unwrap();
}
```

**Step 2: Run tests to see them fail**

```bash
cd /home/fsd42/dev/myme
cargo test -p myme-organizations -- test_prospect_source_url test_org_notes test_schema_migration 2>&1 | tail -20
```

Expected: compile errors — `Prospect` has no `source_url` field yet.

**Step 3: Add fields to `Prospect` in `models.rs`**

In `crates/myme-organizations/src/models.rs`, after `contact_role`:

```rust
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
    pub source_url: Option<String>,      // NEW: direct link to the RFP source
    pub closing_date: Option<String>,    // NEW: ISO 8601 date e.g. "2026-04-10"
    pub created_at: String,
    pub updated_at: String,
}
```

**Step 4: Update `init_schema` in `store.rs`**

Replace `const SCHEMA_VERSION: i32 = 1;` with `const SCHEMA_VERSION: i32 = 2;`.

Replace the entire `init_schema` method:

```rust
fn init_schema(&self) -> Result<()> {
    self.conn
        .execute("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL)", [])?;

    let version: i32 = self
        .conn
        .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| row.get(0))
        .optional()?
        .unwrap_or(0);

    if version < 1 {
        // Fresh database: create all tables with current (v2) schema
        self.conn
            .execute_batch(
                "BEGIN TRANSACTION;

                CREATE TABLE IF NOT EXISTS organizations (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    website TEXT,
                    contact_name TEXT,
                    contact_email TEXT,
                    contact_phone TEXT,
                    contact_role TEXT,
                    notes TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS prospects (
                    id TEXT PRIMARY KEY,
                    organization_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    stage TEXT NOT NULL,
                    value TEXT,
                    contact_name TEXT,
                    contact_email TEXT,
                    contact_role TEXT,
                    source_url TEXT,
                    closing_date TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (organization_id) REFERENCES organizations(id)
                );

                CREATE TABLE IF NOT EXISTS organization_projects (
                    organization_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    PRIMARY KEY (organization_id, project_id),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id)
                );

                CREATE INDEX IF NOT EXISTS idx_prospects_org ON prospects(organization_id);
                CREATE INDEX IF NOT EXISTS idx_prospects_stage ON prospects(stage);
                CREATE INDEX IF NOT EXISTS idx_org_projects_org ON organization_projects(organization_id);
                CREATE INDEX IF NOT EXISTS idx_org_projects_proj ON organization_projects(project_id);

                DELETE FROM schema_version;
                INSERT INTO schema_version (version) VALUES (2);

                COMMIT;",
            )
            .context("Failed to initialize schema")?;
    } else if version < 2 {
        // Upgrade existing v1 database: add new columns
        self.conn
            .execute_batch(
                "BEGIN TRANSACTION;
                ALTER TABLE prospects ADD COLUMN source_url TEXT;
                ALTER TABLE prospects ADD COLUMN closing_date TEXT;
                ALTER TABLE organizations ADD COLUMN notes TEXT;
                DELETE FROM schema_version;
                INSERT INTO schema_version (version) VALUES (2);
                COMMIT;",
            )
            .context("Failed to migrate schema to v2")?;
    }

    Ok(())
}
```

**Step 5: Update `upsert_prospect` to include new fields**

Replace the `upsert_prospect` method:

```rust
pub fn upsert_prospect(&self, prospect: &Prospect) -> Result<()> {
    let stage_str = serde_json::to_string(&prospect.stage)?;

    self.conn.execute(
        "INSERT INTO prospects (id, organization_id, name, description, stage, value, contact_name, contact_email, contact_role, source_url, closing_date, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            stage = excluded.stage,
            value = excluded.value,
            contact_name = excluded.contact_name,
            contact_email = excluded.contact_email,
            contact_role = excluded.contact_role,
            source_url = excluded.source_url,
            closing_date = excluded.closing_date,
            updated_at = excluded.updated_at",
        params![
            prospect.id,
            prospect.organization_id,
            prospect.name,
            prospect.description,
            stage_str,
            prospect.value,
            prospect.contact_name,
            prospect.contact_email,
            prospect.contact_role,
            prospect.source_url,
            prospect.closing_date,
            prospect.created_at,
            prospect.updated_at,
        ],
    )?;
    Ok(())
}
```

**Step 6: Update `list_prospects` to read new fields**

Replace `list_prospects`:

```rust
pub fn list_prospects(&self, organization_id: &str) -> Result<Vec<Prospect>> {
    let mut stmt = self.conn.prepare(
        "SELECT id, organization_id, name, description, stage, value, contact_name, contact_email, contact_role, source_url, closing_date, created_at, updated_at
         FROM prospects WHERE organization_id = ?1 ORDER BY created_at",
    )?;

    let prospects = stmt
        .query_map([organization_id], |row| {
            let stage_str: String = row.get(4)?;
            let stage = match serde_json::from_str(&stage_str) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        "Invalid prospect stage '{}' in database, defaulting to Lead: {}",
                        stage_str,
                        e
                    );
                    ProspectStage::Lead
                }
            };
            Ok(Prospect {
                id: row.get(0)?,
                organization_id: row.get(1)?,
                name: row.get(2)?,
                description: row.get(3)?,
                stage,
                value: row.get(5)?,
                contact_name: row.get(6)?,
                contact_email: row.get(7)?,
                contact_role: row.get(8)?,
                source_url: row.get(9)?,
                closing_date: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(prospects)
}
```

**Step 7: Add org notes methods to `OrganizationStore`**

Add after `delete_organization`:

```rust
/// Get the notes for an organization. Returns empty string if none.
pub fn get_org_notes(&self, org_id: &str) -> Result<String> {
    let notes: Option<String> = self
        .conn
        .query_row(
            "SELECT notes FROM organizations WHERE id = ?1",
            [org_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();
    Ok(notes.unwrap_or_default())
}

/// Set the notes for an organization.
pub fn set_org_notes(&self, org_id: &str, notes: &str) -> Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    self.conn.execute(
        "UPDATE organizations SET notes = ?1, updated_at = ?2 WHERE id = ?3",
        params![notes, now, org_id],
    )?;
    Ok(())
}
```

**Step 8: Fix existing tests — add `source_url: None, closing_date: None` to Prospect constructions**

In `store.rs` tests, each `Prospect { ... }` struct literal needs two new fields:

```rust
source_url: None,
closing_date: None,
```

Add these to every `Prospect { ... }` construction in the test module (search for `stage: ProspectStage` to find them all).

**Step 9: Run all tests to verify they pass**

```bash
cargo test -p myme-organizations 2>&1 | tail -20
```

Expected: all tests pass including the three new ones.

**Step 10: Commit**

```bash
git add crates/myme-organizations/
git commit -m "feat(organizations): add source_url, closing_date to Prospect; org notes; schema v2"
```

---

### Task 2: Update northcloud `build_rfp_description` — remove embedded source/closing

**Files:**
- Modify: `crates/myme-integrations/src/northcloud.rs`

**Step 1: Update failing test for `build_rfp_description`**

The existing test `build_rfp_description_all_fields` asserts that "Source: ..." and "Closing: ..." are in the output. Change it to assert they are NOT:

```rust
#[test]
fn build_rfp_description_all_fields() {
    let rfp = RfpData {
        organization_name: Some("City of Ottawa".into()),
        description: Some("Web dev project".into()),
        closing_date: Some("2026-04-01".into()),
        city: Some("Ottawa".into()),
        ..Default::default()
    };
    let result = build_rfp_description(&rfp);
    assert!(result.contains("Issuer: City of Ottawa"));
    assert!(result.contains("Web dev project"));
    assert!(result.contains("Location: Ottawa"));
    // closing date and source URL now live in dedicated Prospect fields
    assert!(!result.contains("Closing:"));
    assert!(!result.contains("Source:"));
}

#[test]
fn build_rfp_description_minimal() {
    let rfp = RfpData::default();
    let result = build_rfp_description(&rfp);
    assert_eq!(result, "");
}

#[test]
fn build_rfp_description_empty_org_skipped() {
    let rfp = RfpData {
        organization_name: Some(String::new()),
        description: Some("A project".into()),
        ..Default::default()
    };
    let result = build_rfp_description(&rfp);
    assert!(!result.contains("Issuer:"));
    assert!(result.contains("A project"));
}
```

**Step 2: Run tests to verify they fail**

```bash
cargo test -p myme-integrations -- build_rfp_description 2>&1 | tail -10
```

Expected: FAIL — current function includes "Closing:" and "Source:".

**Step 3: Rewrite `build_rfp_description`**

Replace the public function (no longer takes a `url` parameter):

```rust
/// Build a multi-line prospect description from RFP metadata.
/// Does NOT include closing_date or source_url — those live in dedicated Prospect fields.
pub fn build_rfp_description(rfp: &RfpData) -> String {
    let mut parts = Vec::new();
    if let Some(org) = &rfp.organization_name {
        if !org.is_empty() {
            parts.push(format!("Issuer: {}", org));
        }
    }
    if let Some(desc) = &rfp.description {
        if !desc.is_empty() {
            parts.push(desc.clone());
        }
    }
    if let Some(city) = &rfp.city {
        if !city.is_empty() {
            parts.push(format!("Location: {}", city));
        }
    }
    parts.join("\n")
}
```

**Step 4: Run tests**

```bash
cargo test -p myme-integrations 2>&1 | tail -10
```

Expected: all pass.

**Step 5: Commit**

```bash
git add crates/myme-integrations/src/northcloud.rs
git commit -m "fix(integrations): remove source_url and closing_date from RFP description text"
```

---

### Task 3: Populate new Prospect fields in the import service

**Files:**
- Modify: `crates/myme-ui/src/services/organization_service.rs`

**Step 1: Update `do_import_rfp_leads` to pass new fields**

The `build_rfp_description` call no longer takes `url`. The `Prospect` construction needs `source_url` and `closing_date`.

Find the `Prospect { ... }` construction (around line 140) and replace it:

```rust
let description = if let Some(r) = rfp {
    build_rfp_description(r)
} else {
    hit.snippet.clone().unwrap_or_default()
};

let source_url = Some(hit.url.clone());
let closing_date = rfp.and_then(|r| r.closing_date.clone());

let prospect = Prospect {
    id: format!("nc-{}", hit.id),
    organization_id: org_id.to_string(),
    name,
    description: if description.is_empty() { None } else { Some(description) },
    stage: ProspectStage::Lead,
    value: if value.is_empty() { None } else { Some(value) },
    contact_name: if contact_name.is_empty() { None } else { Some(contact_name) },
    contact_email: if contact_email.is_empty() { None } else { Some(contact_email) },
    contact_role: None,
    source_url,
    closing_date,
    created_at: now.clone(),
    updated_at: now.clone(),
};
```

Also remove the old non-rfp description branch that embedded "Source: ..." — replace it with just the snippet.

**Step 2: Fix `create_prospect` in `prospect_model.rs`**

In `crates/myme-ui/src/models/prospect_model.rs`, find the `Prospect { ... }` construction in `create_prospect` and add:

```rust
source_url: None,
closing_date: None,
```

**Step 3: Fix `update_prospect` in `prospect_model.rs`**

In `update_prospect`, find the `Prospect { ... }` construction and preserve the existing values:

```rust
source_url: existing.source_url.clone(),
closing_date: existing.closing_date.clone(),
```

**Step 4: Build to catch any remaining compilation errors**

```bash
cargo build -p myme-ui 2>&1 | grep "^error" | head -20
```

Fix any remaining `Prospect { ... }` struct constructions missing the new fields.

**Step 5: Run full test suite**

```bash
cargo test -p myme-core -p myme-integrations -p myme-organizations 2>&1 | tail -20
```

Expected: all pass.

**Step 6: Commit**

```bash
git add crates/myme-ui/src/
git commit -m "feat(organizations): populate source_url and closing_date from RFP import"
```

---

### Task 4: New ProspectModel bridge invokables

**Files:**
- Modify: `crates/myme-ui/src/models/prospect_model.rs`

**Step 1: Add invokable declarations to the bridge**

In the `extern "RustQt"` block, after `get_prospect_created_at`:

```rust
#[qinvokable]
fn get_prospect_source_url(self: &ProspectModel, index: i32) -> QString;

#[qinvokable]
fn get_prospect_closing_date(self: &ProspectModel, index: i32) -> QString;

/// Returns JSON array of indices for Lead prospects sorted by closing_date ascending (nulls last).
#[qinvokable]
fn lead_prospects_by_urgency(self: &ProspectModel) -> QString;

/// Get notes for the current organization.
#[qinvokable]
fn get_org_notes(self: &ProspectModel) -> QString;

/// Save notes for the current organization.
#[qinvokable]
fn set_org_notes(self: Pin<&mut ProspectModel>, notes: &QString);
```

**Step 2: Implement the new methods**

Add after `get_prospect_created_at`:

```rust
pub fn get_prospect_source_url(&self, index: i32) -> QString {
    self.rust()
        .get_prospect(index)
        .and_then(|p| p.source_url.as_ref())
        .map(|u| QString::from(u))
        .unwrap_or_else(|| QString::from(""))
}

pub fn get_prospect_closing_date(&self, index: i32) -> QString {
    self.rust()
        .get_prospect(index)
        .and_then(|p| p.closing_date.as_ref())
        .map(|d| QString::from(d))
        .unwrap_or_else(|| QString::from(""))
}

pub fn lead_prospects_by_urgency(&self) -> QString {
    let mut lead_indices: Vec<(usize, Option<String>)> = self
        .rust()
        .prospects
        .iter()
        .enumerate()
        .filter(|(_, p)| p.stage == ProspectStage::Lead)
        .map(|(i, p)| (i, p.closing_date.clone()))
        .collect();

    lead_indices.sort_by(|(_, a_date), (_, b_date)| match (a_date, b_date) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    let indices: Vec<i32> = lead_indices.into_iter().map(|(i, _)| i as i32).collect();
    let json = serde_json::to_string(&indices).unwrap_or_else(|_| "[]".to_string());
    QString::from(&json)
}

pub fn get_org_notes(&self) -> QString {
    let org_id = self.rust().organization_id.to_string();
    if org_id.is_empty() {
        return QString::from("");
    }
    let store = match &self.rust().organization_store {
        Some(s) => s,
        None => return QString::from(""),
    };
    match store.lock().get_org_notes(&org_id) {
        Ok(notes) => QString::from(notes),
        Err(e) => {
            tracing::error!("Failed to get org notes: {}", e);
            QString::from("")
        }
    }
}

pub fn set_org_notes(mut self: Pin<&mut Self>, notes: &QString) {
    let org_id = self.as_ref().rust().organization_id.to_string();
    if org_id.is_empty() {
        return;
    }
    let store = match &self.as_ref().rust().organization_store {
        Some(s) => s.clone(),
        None => return,
    };
    if let Err(e) = store.lock().set_org_notes(&org_id, &notes.to_string()) {
        tracing::error!("Failed to save org notes: {}", e);
        self.as_mut().set_error_message(QString::from(format!("Failed to save notes: {}", e)));
    }
}
```

**Step 3: Build**

```bash
cargo build -p myme-ui 2>&1 | grep "^error" | head -20
```

Expected: clean build.

**Step 4: Commit**

```bash
git add crates/myme-ui/src/models/prospect_model.rs
git commit -m "feat(ui-bridge): add source_url, closing_date, urgency sort, org notes invokables"
```

---

### Task 5: QML — Lead column triage redesign

**Files:**
- Modify: `crates/myme-ui/qml/pages/OrganizationDetailPage.qml`

**Step 1: Add page-level properties and helper functions**

Near the top of the `Page { ... }` body (after existing property declarations), add:

```qml
property int selectedProspectIndex: -1

// Urgency helpers
function daysUntil(dateStr) {
    if (!dateStr || dateStr === "") return -1
    var parts = dateStr.split("-")
    if (parts.length < 3) return -1
    var d = new Date(parts[0], parts[1] - 1, parts[2])
    var today = new Date()
    today.setHours(0, 0, 0, 0)
    return Math.ceil((d - today) / 86400000)
}

function urgencyColor(days) {
    if (days < 0) return Theme.textSecondary
    if (days <= 7) return "#E57373"
    if (days <= 21) return "#FF8A65"
    if (days <= 60) return "#F59E0B"
    return Theme.textSecondary
}

function urgencyLabel(days) {
    if (days < 0) return "Closed"
    if (days === 0) return "Closes today"
    if (days === 1) return "1 day left"
    return days + " days left"
}
```

**Step 2: Wrap kanban + side panel in a RowLayout**

Find `ScrollView {` (the kanban scroll view, around line 267). It currently sits inside a `ColumnLayout`. Replace that `ScrollView` with a `RowLayout` that holds the scroll view and the side panel sibling:

```qml
RowLayout {
    Layout.fillWidth: true
    Layout.fillHeight: true
    spacing: 0

    // Kanban scroll area
    ScrollView {
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.leftMargin: Theme.spacingMd
        Layout.rightMargin: selectedProspectIndex >= 0 ? 0 : Theme.spacingMd
        Layout.bottomMargin: Theme.spacingMd
        contentWidth: pipelineRow.implicitWidth
        clip: true

        RowLayout {
            id: pipelineRow
            spacing: Theme.spacingSm
            height: parent.height

            Repeater {
                model: detailPage.stages
                // ... existing column contents (see Step 3)
            }
        }
    }

    // Side panel
    Rectangle {
        id: sidePanel
        Layout.preferredWidth: 380
        Layout.fillHeight: true
        visible: selectedProspectIndex >= 0
        color: Theme.isDark ? "#0affffff" : "#08000000"
        border.color: Theme.borderLight
        border.width: 1

        // Contents added in Task 6
    }
}
```

**Step 3: Redesign the Lead column**

Inside the `Repeater { model: detailPage.stages }`, find the inner `ListView` delegate. The `model:` binding needs to use urgency-sorted indices for Lead:

```qml
model: {
    void(prospectModel.prospect_revision);
    if (modelData.key === "lead") {
        return JSON.parse(prospectModel.lead_prospects_by_urgency());
    }
    return JSON.parse(prospectModel.prospects_for_stage(modelData.key));
}
```

The stage column width should be wider for Lead:

```qml
Layout.preferredWidth: modelData.key === "lead" ? 320 : 220
```

**Step 4: Redesign the Lead card delegate**

Inside the `delegate: Rectangle { ... }` for prospect cards, replace the `ColumnLayout` content with a conditional that gives Lead cards a richer layout:

```qml
ColumnLayout {
    id: prospectCardLayout
    anchors.left: parent.left
    anchors.right: parent.right
    anchors.top: parent.top
    anchors.margins: Theme.spacingSm
    spacing: 4

    // Urgency + budget row (Lead only)
    RowLayout {
        visible: modelData_stage === "lead"
        Layout.fillWidth: true
        spacing: Theme.spacingXs

        property string modelData_stage: modelData  // bind before Repeater context changes
        // Note: inside the ListView delegate, modelData is the index (i32).
        // We read stage from the model via get_prospect_stage.
        // Use the outer stage key from the Repeater context captured by the parent Rectangle.
    }
    // ...
}
```

Wait — the delegate is inside a `Repeater` whose `modelData` is the prospect index (i32). The outer stage key is `modelData.key` from the outer `Repeater`. Inside a nested `Repeater`, the outer modelData is shadowed. We need to capture the outer stage key.

Replace the `// Stage column` `Rectangle` with this pattern to capture the stage key:

```qml
Rectangle {
    id: stageColumn
    Layout.preferredWidth: modelData.key === "lead" ? 320 : 220
    Layout.fillHeight: true
    color: Theme.isDark ? "#05ffffff" : "#03000000"
    radius: Theme.cardRadius
    border.color: Theme.cardBorderSubtle
    border.width: 1

    property string stageKey: modelData.key   // capture before inner Repeater shadows modelData

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingSm
        spacing: Theme.spacingSm

        // Stage header (unchanged)
        RowLayout { ... }

        // Prospect cards
        ListView {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            spacing: Theme.spacingXs

            model: {
                void(prospectModel.prospect_revision);
                if (stageColumn.stageKey === "lead") {
                    return JSON.parse(prospectModel.lead_prospects_by_urgency());
                }
                return JSON.parse(prospectModel.prospects_for_stage(stageColumn.stageKey));
            }

            delegate: Rectangle {
                id: prospectCard
                width: ListView.view.width
                height: cardContent.implicitHeight + Theme.spacingSm * 2
                radius: Theme.buttonRadius
                color: (detailPage.selectedProspectIndex === prospectIndex)
                    ? (Theme.isDark ? "#20ffffff" : "#10000000")
                    : (prospectMouse.containsMouse
                        ? (Theme.isDark ? Qt.lighter(Theme.cardBg, 1.05) : Qt.darker(Theme.cardBg, 1.02))
                        : Theme.cardBg)
                border.color: (detailPage.selectedProspectIndex === prospectIndex)
                    ? Theme.primary
                    : Theme.cardBorderSubtle
                border.width: 1

                property int prospectIndex: modelData
                property bool isLead: stageColumn.stageKey === "lead"

                opacity: 0
                Component.onCompleted: prospectFade.start()
                SequentialAnimation {
                    id: prospectFade
                    PauseAnimation { duration: index * 30 }
                    NumberAnimation { target: prospectCard; property: "opacity"; from: 0; to: 1; duration: 200; easing.type: Easing.OutCubic }
                }

                ColumnLayout {
                    id: cardContent
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.margins: Theme.spacingSm
                    spacing: 4

                    // --- Lead-specific urgency + budget row ---
                    RowLayout {
                        visible: prospectCard.isLead
                        Layout.fillWidth: true
                        spacing: Theme.spacingXs

                        property int days: detailPage.daysUntil(prospectModel.get_prospect_closing_date(prospectIndex))

                        Rectangle {
                            visible: parent.days >= 0
                            radius: 3
                            color: Qt.rgba(0, 0, 0, 0)
                            implicitHeight: urgencyText.implicitHeight + 4
                            implicitWidth: urgencyText.implicitWidth + 8

                            Label {
                                id: urgencyText
                                anchors.centerIn: parent
                                text: detailPage.urgencyLabel(parent.parent.days)
                                font.family: Theme.fontFamily
                                font.pixelSize: 10
                                font.weight: Font.Medium
                                color: detailPage.urgencyColor(parent.parent.days)
                            }
                        }

                        Item { Layout.fillWidth: true }

                        Label {
                            visible: text !== ""
                            text: prospectModel.get_prospect_value(prospectIndex)
                            font.family: Theme.fontFamily
                            font.pixelSize: 10
                            font.weight: Font.Medium
                            color: Theme.primary
                        }
                    }

                    // --- Name (2 lines for Lead, 1 for others) ---
                    Label {
                        text: prospectModel.get_prospect_name(prospectIndex)
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeSmall
                        font.weight: Font.Medium
                        color: Theme.text
                        elide: Text.ElideRight
                        maximumLineCount: prospectCard.isLead ? 2 : 1
                        wrapMode: prospectCard.isLead ? Text.WordWrap : Text.NoWrap
                        Layout.fillWidth: true
                    }

                    // --- Contact / org name ---
                    Label {
                        visible: text !== ""
                        text: prospectModel.get_prospect_contact_name(prospectIndex)
                        font.family: Theme.fontFamily
                        font.pixelSize: 11
                        color: Theme.textSecondary
                        elide: Text.ElideRight
                        Layout.fillWidth: true
                    }

                    // --- Lead quick-action buttons ---
                    RowLayout {
                        visible: prospectCard.isLead && prospectMouse.containsMouse
                        Layout.fillWidth: true
                        spacing: Theme.spacingXs

                        // Open RFP
                        Button {
                            text: "🔗 Open RFP"
                            visible: prospectModel.get_prospect_source_url(prospectIndex) !== ""
                            font.family: Theme.fontFamily
                            font.pixelSize: 10
                            contentItem: Label { text: parent.text; font: parent.font; color: Theme.primary; horizontalAlignment: Text.AlignHCenter }
                            background: Rectangle {
                                color: parent.hovered ? (Theme.isDark ? "#15ffffff" : "#08000000") : "transparent"
                                radius: Theme.buttonRadius
                                border.color: Theme.isDark ? "#15ffffff" : "#10000000"
                                border.width: 1
                                implicitHeight: 24
                            }
                            onClicked: Qt.openUrlExternally(prospectModel.get_prospect_source_url(prospectIndex))
                        }

                        Item { Layout.fillWidth: true }

                        // Qualify
                        Button {
                            text: "✓ Qualify"
                            font.family: Theme.fontFamily
                            font.pixelSize: 10
                            contentItem: Label { text: parent.text; font: parent.font; color: "#5BB98C"; horizontalAlignment: Text.AlignHCenter }
                            background: Rectangle {
                                color: parent.hovered ? "#155BB98C" : "transparent"
                                radius: Theme.buttonRadius
                                border.color: "#305BB98C"
                                border.width: 1
                                implicitHeight: 24
                                implicitWidth: 62
                            }
                            onClicked: {
                                if (detailPage.selectedProspectIndex === prospectIndex) {
                                    detailPage.selectedProspectIndex = -1;
                                }
                                prospectModel.move_prospect(prospectIndex, "qualified");
                            }
                        }

                        // Skip
                        Button {
                            text: "✗ Skip"
                            font.family: Theme.fontFamily
                            font.pixelSize: 10
                            contentItem: Label { text: parent.text; font: parent.font; color: "#E57373"; horizontalAlignment: Text.AlignHCenter }
                            background: Rectangle {
                                color: parent.hovered ? "#15E57373" : "transparent"
                                radius: Theme.buttonRadius
                                border.color: "#30E57373"
                                border.width: 1
                                implicitHeight: 24
                                implicitWidth: 50
                            }
                            onClicked: {
                                if (detailPage.selectedProspectIndex === prospectIndex) {
                                    detailPage.selectedProspectIndex = -1;
                                }
                                prospectModel.move_prospect(prospectIndex, "lost");
                            }
                        }
                    }
                }

                MouseArea {
                    id: prospectMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        if (detailPage.selectedProspectIndex === prospectIndex) {
                            detailPage.selectedProspectIndex = -1;
                        } else {
                            detailPage.selectedProspectIndex = prospectIndex;
                        }
                    }
                }
            }
        }

        // + Add button (unchanged)
        Button { ... }
    }
}
```

**Step 5: Test manually**

```bash
task run
```

1. Navigate to Web Networks → Pipeline
2. Lead column should be wider, cards show urgency badges
3. Hover a Lead card — "🔗 Open RFP", "✓ Qualify", "✗ Skip" buttons appear
4. Click "✓ Qualify" — card moves to Qualified column
5. Click "✗ Skip" — card disappears from Lead

**Step 6: Commit**

```bash
git add crates/myme-ui/qml/pages/OrganizationDetailPage.qml
git commit -m "feat(ui): lead triage redesign — urgency badges, wider column, quick-action buttons"
```

---

### Task 6: QML — Prospect side panel

**Files:**
- Modify: `crates/myme-ui/qml/pages/OrganizationDetailPage.qml`

**Step 1: Add Escape key handler to the page**

After the existing `Component.onCompleted`, add:

```qml
Keys.onEscapePressed: selectedProspectIndex = -1
focus: true
```

**Step 2: Implement the side panel Rectangle content**

Replace the `// Contents added in Task 6` placeholder in `sidePanel`:

```qml
Rectangle {
    id: sidePanel
    Layout.preferredWidth: 380
    Layout.fillHeight: true
    visible: selectedProspectIndex >= 0
    color: Theme.isDark ? "#0affffff" : "#06000000"
    border.color: Theme.borderLight
    border.width: 1

    // Slide-in animation
    opacity: selectedProspectIndex >= 0 ? 1 : 0
    Behavior on opacity { NumberAnimation { duration: 150; easing.type: Easing.OutCubic } }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingMd
        spacing: Theme.spacingMd
        visible: detailPage.selectedProspectIndex >= 0

        // Close button
        RowLayout {
            Layout.fillWidth: true

            Label {
                text: prospectModel.get_prospect_name(detailPage.selectedProspectIndex)
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                font.weight: Font.Bold
                color: Theme.text
                wrapMode: Text.WordWrap
                maximumLineCount: 2
                elide: Text.ElideRight
                Layout.fillWidth: true
            }

            Button {
                contentItem: Text {
                    text: Icons.x
                    font.family: Icons.family
                    font.pixelSize: 16
                    color: Theme.textSecondary
                    horizontalAlignment: Text.AlignHCenter
                }
                background: Rectangle {
                    color: parent.hovered ? (Theme.isDark ? "#10ffffff" : "#08000000") : "transparent"
                    radius: Theme.buttonRadius
                    implicitWidth: 28
                    implicitHeight: 28
                }
                onClicked: detailPage.selectedProspectIndex = -1
            }
        }

        // Stage selector
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm

            Label {
                text: "Stage"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textSecondary
                Layout.preferredWidth: 70
            }

            ComboBox {
                id: stageCombo
                Layout.fillWidth: true
                model: ["Lead", "Qualified", "Contacted", "Proposal", "Negotiation", "Won", "Lost"]
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall

                // Sync to selected prospect
                currentIndex: {
                    var s = prospectModel.get_prospect_stage(detailPage.selectedProspectIndex)
                    return Math.max(0, model.indexOf(s))
                }

                contentItem: Label {
                    leftPadding: 8
                    text: stageCombo.displayText
                    font: stageCombo.font
                    color: Theme.text
                    verticalAlignment: Text.AlignVCenter
                }

                background: Rectangle {
                    color: Theme.surfaceAlt
                    radius: Theme.buttonRadius
                    border.color: stageCombo.popup.visible ? Theme.primary : Theme.borderLight
                    border.width: 1
                    implicitHeight: 32
                }

                onActivated: {
                    var stageKey = currentText.toLowerCase()
                    prospectModel.move_prospect(detailPage.selectedProspectIndex, stageKey)
                }
            }
        }

        // Closing date with urgency
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm
            visible: prospectModel.get_prospect_closing_date(detailPage.selectedProspectIndex) !== ""

            property string closingDate: prospectModel.get_prospect_closing_date(detailPage.selectedProspectIndex)
            property int days: detailPage.daysUntil(closingDate)

            Label {
                text: "Closing"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textSecondary
                Layout.preferredWidth: 70
            }

            ColumnLayout {
                spacing: 2
                Label {
                    text: parent.parent.closingDate
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    color: Theme.text
                }
                Label {
                    visible: parent.parent.days >= 0
                    text: "● " + detailPage.urgencyLabel(parent.parent.days)
                    font.family: Theme.fontFamily
                    font.pixelSize: 11
                    color: detailPage.urgencyColor(parent.parent.days)
                }
            }
        }

        // Budget / value
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm
            visible: prospectModel.get_prospect_value(detailPage.selectedProspectIndex) !== ""

            Label {
                text: "Budget"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textSecondary
                Layout.preferredWidth: 70
            }

            Label {
                text: prospectModel.get_prospect_value(detailPage.selectedProspectIndex)
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                font.weight: Font.Medium
                color: Theme.primary
            }
        }

        // Contact
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm
            visible: prospectModel.get_prospect_contact_name(detailPage.selectedProspectIndex) !== ""

            Label {
                text: "Contact"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textSecondary
                Layout.preferredWidth: 70
            }

            Label {
                text: prospectModel.get_prospect_contact_name(detailPage.selectedProspectIndex)
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.text
                Layout.fillWidth: true
                elide: Text.ElideRight
            }
        }

        // Email
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm
            visible: prospectModel.get_prospect_contact_email(detailPage.selectedProspectIndex) !== ""

            Label {
                text: "Email"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.textSecondary
                Layout.preferredWidth: 70
            }

            Label {
                text: prospectModel.get_prospect_contact_email(detailPage.selectedProspectIndex)
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.primary
                Layout.fillWidth: true
                elide: Text.ElideRight
            }
        }

        // Open Source RFP button
        Button {
            visible: prospectModel.get_prospect_source_url(detailPage.selectedProspectIndex) !== ""
            Layout.fillWidth: true
            contentItem: Label {
                text: "🔗  Open Source RFP →"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.primaryText
                horizontalAlignment: Text.AlignHCenter
            }
            background: Rectangle {
                color: parent.hovered ? Theme.primaryHover : Theme.primary
                radius: Theme.buttonRadius
                implicitHeight: 36
            }
            onClicked: Qt.openUrlExternally(prospectModel.get_prospect_source_url(detailPage.selectedProspectIndex))
        }

        // Description
        Label {
            visible: prospectModel.get_prospect_description(detailPage.selectedProspectIndex) !== ""
            text: "Description"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
            font.weight: Font.Bold
            color: Theme.textSecondary
        }

        ScrollView {
            visible: prospectModel.get_prospect_description(detailPage.selectedProspectIndex) !== ""
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true

            Label {
                width: parent.width
                text: prospectModel.get_prospect_description(detailPage.selectedProspectIndex)
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                color: Theme.text
                wrapMode: Text.WordWrap
            }
        }

        // Edit + Delete row
        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.spacingSm

            Button {
                text: "Edit"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                contentItem: Label { text: parent.text; font: parent.font; color: Theme.primaryText; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? Theme.primaryHover : Theme.primary; radius: Theme.buttonRadius; implicitHeight: 32; implicitWidth: 70 }
                onClicked: {
                    editProspectIndex = detailPage.selectedProspectIndex;
                    editProspectDialog.open();
                }
            }

            Item { Layout.fillWidth: true }

            Button {
                text: "Delete"
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                contentItem: Label { text: parent.text; font: parent.font; color: "#E57373"; horizontalAlignment: Text.AlignHCenter }
                background: Rectangle { color: parent.hovered ? "#20E57373" : "transparent"; radius: Theme.buttonRadius; implicitHeight: 32 }
                onClicked: {
                    prospectModel.delete_prospect(detailPage.selectedProspectIndex);
                    detailPage.selectedProspectIndex = -1;
                }
            }
        }
    }
}
```

**Step 3: Test manually**

```bash
task run
```

1. Click any prospect card (not an action button) → side panel slides in from right
2. Kanban shrinks, panel shows at right
3. Stage ComboBox shows current stage — change it → card moves in kanban
4. "Open Source RFP →" button opens browser for RFP leads
5. Urgency badge and budget visible for RFP leads
6. Press Escape → panel closes
7. Click same card again → panel closes

**Step 4: Commit**

```bash
git add crates/myme-ui/qml/pages/OrganizationDetailPage.qml
git commit -m "feat(ui): prospect side panel with stage selector, source URL, urgency detail"
```

---

### Task 7: QML — Notes tab persistence

**Files:**
- Modify: `crates/myme-ui/qml/pages/OrganizationDetailPage.qml`

**Step 1: Update the Notes tab**

Replace the entire Notes tab (Tab 2) `Item { id: notesTab ... }`:

```qml
Item {
    id: notesTab

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.spacingLg
        spacing: Theme.spacingSm

        Label {
            text: "Organization Notes"
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeNormal
            font.weight: Font.Bold
            color: Theme.text
        }

        ScrollView {
            Layout.fillWidth: true
            Layout.fillHeight: true

            TextArea {
                id: notesArea
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeNormal
                color: Theme.text
                wrapMode: TextArea.Wrap
                placeholderText: "Write notes about this organization..."

                background: Rectangle {
                    color: Theme.isDark ? "#05ffffff" : "#03000000"
                    radius: Theme.cardRadius
                    border.color: notesArea.activeFocus ? Theme.primary : (Theme.isDark ? "#10ffffff" : "#10000000")
                    border.width: 1
                }

                onTextChanged: notesDebounce.restart()
            }
        }
    }

    Timer {
        id: notesDebounce
        interval: 500
        repeat: false
        onTriggered: prospectModel.set_org_notes(notesArea.text)
    }
}
```

**Step 2: Load notes when tab becomes active**

Update the existing `onCurrentTabChanged` handler:

```qml
onCurrentTabChanged: {
    prospectModel.import_result = ""
    if (currentTab === 2) {
        notesArea.text = prospectModel.get_org_notes()
    }
}
```

Also load notes on page open. Update `Component.onCompleted`:

```qml
Component.onCompleted: {
    prospectModel.load_prospects(detailPage.organizationId);
    notesArea.text = prospectModel.get_org_notes();
}
```

**Step 3: Remove the "not saved" warning label**

Delete the `Label { text: "Notes persistence coming soon..." }` — no longer accurate.

**Step 4: Test manually**

```bash
task run
```

1. Go to Notes tab → type some text
2. Wait 500ms → text auto-saves (no UI feedback needed)
3. Navigate away to another org, come back → notes reload
4. Restart the app → notes persist

**Step 5: Run full test suite**

```bash
cargo test -p myme-core -p myme-integrations -p myme-organizations 2>&1 | tail -10
```

Expected: all pass.

**Step 6: Commit**

```bash
git add crates/myme-ui/qml/pages/OrganizationDetailPage.qml
git commit -m "feat(ui): org notes tab persistence via prospectModel invokables"
```

---

## Summary of all changes

| File | Change |
|---|---|
| `myme-organizations/src/models.rs` | `source_url`, `closing_date` on `Prospect` |
| `myme-organizations/src/store.rs` | Schema v2 migration, updated SQL, `get/set_org_notes` |
| `myme-integrations/src/northcloud.rs` | `build_rfp_description` removes embedded source/closing |
| `myme-ui/src/services/organization_service.rs` | Populate `source_url`, `closing_date` in import |
| `myme-ui/src/models/prospect_model.rs` | 5 new invokables, `source_url`/`closing_date` in CRUD |
| `myme-ui/qml/pages/OrganizationDetailPage.qml` | Lead triage, side panel, notes persistence |
