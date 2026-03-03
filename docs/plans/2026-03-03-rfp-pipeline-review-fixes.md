# RFP Pipeline Review Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Address all critical and important issues from PR #25 review: fix `block_on()` on Qt thread, add deduplication, add HTTP timeouts, improve error reporting, and add test coverage.

**Architecture:** Convert the synchronous `import_rfp_leads` to the established async channel pattern (like `gmail_service.rs`). Add an `RfpImportDone` variant to `OrganizationServiceMessage`, spawn the HTTP+SQLite work on the tokio runtime via `organization_service.rs`, and poll results in `poll_channel`. Fix deduplication with deterministic IDs. Move testable helpers to `myme-integrations` for test coverage.

**Tech Stack:** Rust, cxx-qt, reqwest, wiremock (new dev-dep), QML

---

### Task 1: Add HTTP timeout and validate base_url in NorthCloudClient

**Files:**
- Modify: `crates/myme-integrations/src/northcloud.rs:65-80`

**Step 1: Write failing test for URL validation at construction**

Add to the existing test module in `northcloud.rs`:

```rust
#[test]
fn client_rejects_invalid_base_url() {
    let result = NorthCloudClient::new("not a url");
    assert!(result.is_err());
}

#[test]
fn client_normalizes_trailing_slash() {
    let client = NorthCloudClient::new("https://northcloud.one/").unwrap();
    assert_eq!(client.base_url.as_str(), "https://northcloud.one/");
    // The key is that search_rfps builds the URL correctly — tested in Task 3
}

#[test]
fn client_accepts_valid_url() {
    let client = NorthCloudClient::new("https://northcloud.one");
    assert!(client.is_ok());
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p myme-integrations -- client_rejects`
Expected: FAIL — currently `new()` accepts any string

**Step 3: Implement URL validation and timeout**

Replace `NorthCloudClient` struct and `new()`:

```rust
pub struct NorthCloudClient {
    client: Client,
    base_url: reqwest::Url,
}

impl NorthCloudClient {
    pub fn new(base_url: impl Into<String>) -> anyhow::Result<Self> {
        let raw = base_url.into();
        let parsed = reqwest::Url::parse(&raw)
            .context(format!("invalid NorthCloud base URL: {}", raw))?;
        let client = reqwest::Client::builder()
            .user_agent("MyMe/0.1.0")
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create NorthCloud HTTP client")?;
        Ok(Self {
            client,
            base_url: parsed,
        })
    }
```

Update `search_rfps` to use `self.base_url.join()` instead of `format!`:

```rust
    pub async fn search_rfps(&self, params: &RfpSearchParams) -> Result<RfpSearchResponse> {
        let mut url = self.base_url.join("/api/v1/search")
            .context("failed to build NorthCloud search URL")?;
        // ... rest unchanged
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p myme-integrations`
Expected: All pass

**Step 5: Commit**

```
fix(integrations): validate base_url at construction, add HTTP timeouts
```

---

### Task 2: Move helper functions to myme-integrations and add tests

**Files:**
- Modify: `crates/myme-integrations/src/northcloud.rs` (add functions + tests)
- Modify: `crates/myme-ui/src/models/prospect_model.rs:680-712` (remove functions, import from crate)

**Step 1: Write failing tests for `build_rfp_description`**

Add to `northcloud.rs` test module:

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
    let result = build_rfp_description(&rfp, "https://example.com/rfp/1");
    assert!(result.contains("Issuer: City of Ottawa"));
    assert!(result.contains("Web dev project"));
    assert!(result.contains("Closing: 2026-04-01"));
    assert!(result.contains("Location: Ottawa"));
    assert!(result.contains("Source: https://example.com/rfp/1"));
}

#[test]
fn build_rfp_description_minimal() {
    let rfp = RfpData::default();
    let result = build_rfp_description(&rfp, "");
    assert_eq!(result, "");
}

#[test]
fn build_rfp_description_empty_org_skipped() {
    let rfp = RfpData {
        organization_name: Some(String::new()),
        description: Some("A project".into()),
        ..Default::default()
    };
    let result = build_rfp_description(&rfp, "");
    assert!(!result.contains("Issuer:"));
    assert!(result.contains("A project"));
}
```

**Step 2: Write failing tests for `rfp_budget_string`**

```rust
#[test]
fn rfp_budget_string_both_bounds() {
    let rfp = RfpData {
        budget_min: Some(10000.0),
        budget_max: Some(50000.0),
        budget_currency: Some("USD".into()),
        ..Default::default()
    };
    assert_eq!(rfp_budget_string(&rfp), "$10000\u{2013}$50000 USD");
}

#[test]
fn rfp_budget_string_min_only() {
    let rfp = RfpData {
        budget_min: Some(5000.0),
        ..Default::default()
    };
    assert_eq!(rfp_budget_string(&rfp), "$5000+ CAD");
}

#[test]
fn rfp_budget_string_max_only() {
    let rfp = RfpData {
        budget_max: Some(100000.0),
        ..Default::default()
    };
    assert_eq!(rfp_budget_string(&rfp), "Up to $100000 CAD");
}

#[test]
fn rfp_budget_string_no_budget() {
    let rfp = RfpData::default();
    assert_eq!(rfp_budget_string(&rfp), "");
}
```

**Step 3: Run tests to verify they fail**

Run: `cargo test -p myme-integrations -- build_rfp`
Expected: FAIL — functions don't exist in myme-integrations yet

**Step 4: Add `Default` derive to `RfpData` and move functions**

Add `Default` derive to `RfpData` (needed for tests):

```rust
#[derive(Debug, Clone, Deserialize, Default)]
pub struct RfpData {
```

Add the two public functions to `northcloud.rs` (below the `impl NorthCloudClient` block, above `#[cfg(test)]`):

```rust
/// Build a multi-line prospect description from RFP metadata.
pub fn build_rfp_description(rfp: &RfpData, url: &str) -> String {
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
    if let Some(closing) = &rfp.closing_date {
        if !closing.is_empty() {
            parts.push(format!("Closing: {}", closing));
        }
    }
    if let Some(city) = &rfp.city {
        if !city.is_empty() {
            parts.push(format!("Location: {}", city));
        }
    }
    if !url.is_empty() {
        parts.push(format!("Source: {}", url));
    }
    parts.join("\n")
}

/// Format budget range as a display string. Defaults to CAD.
pub fn rfp_budget_string(rfp: &RfpData) -> String {
    let currency = rfp.budget_currency.as_deref().unwrap_or("CAD");
    match (rfp.budget_min, rfp.budget_max) {
        (Some(min), Some(max)) => format!("${:.0}\u{2013}${:.0} {}", min, max, currency),
        (Some(min), None) => format!("${:.0}+ {}", min, currency),
        (None, Some(max)) => format!("Up to ${:.0} {}", max, currency),
        (None, None) => String::new(),
    }
}
```

Update `crates/myme-integrations/src/lib.rs` to export the new functions:

```rust
pub use northcloud::{
    build_rfp_description, rfp_budget_string,
    NorthCloudClient, RfpData, RfpHit, RfpSearchParams, RfpSearchResponse,
};
```

**Step 5: Update prospect_model.rs to use the moved functions**

Replace the local `build_rfp_description` and `rfp_budget_string` functions (lines 680-712) with imports. In `import_rfp_leads`, change:

```rust
let description = build_rfp_description(rfp, &hit.url);
let value = rfp_budget_string(rfp);
```

to:

```rust
let description = myme_integrations::build_rfp_description(rfp, &hit.url);
let value = myme_integrations::rfp_budget_string(rfp);
```

Remove the two local functions at the bottom of the file.

**Step 6: Run all tests**

Run: `cargo test -p myme-integrations`
Expected: All pass including new helper tests

**Step 7: Commit**

```
refactor(integrations): move RFP helpers to myme-integrations, add tests
```

---

### Task 3: Add wiremock integration tests for search_rfps

**Files:**
- Modify: `crates/myme-integrations/Cargo.toml` (add wiremock dev-dep)
- Modify: `crates/myme-integrations/src/northcloud.rs` (add async tests)

**Step 1: Add wiremock dev-dependency**

Add to `crates/myme-integrations/Cargo.toml` under `[dev-dependencies]`:

```toml
[dev-dependencies]
tempfile = "3"
wiremock = "0.6"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

**Step 2: Write wiremock integration tests**

Add to `northcloud.rs` test module:

```rust
#[tokio::test]
async fn search_rfps_success() {
    let mock_server = wiremock::MockServer::start().await;
    let body = serde_json::json!({
        "total_hits": 1,
        "hits": [{
            "id": "nc-123",
            "title": "Web RFP",
            "url": "https://example.com/rfp/1",
            "source_name": "Test",
            "rfp": {
                "organization_name": "City of Ottawa",
                "closing_date": "2026-04-01",
                "province": "on"
            }
        }]
    });
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::path("/api/v1/search"))
        .and(wiremock::matchers::query_param("content_type", "rfp"))
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_json(&body))
        .mount(&mock_server)
        .await;

    let client = NorthCloudClient::new(mock_server.uri()).unwrap();
    let response = client.search_rfps(&RfpSearchParams::default()).await.unwrap();
    assert_eq!(response.total_hits, 1);
    assert_eq!(response.hits[0].title, "Web RFP");
    assert_eq!(response.hits[0].rfp.as_ref().unwrap().province.as_deref(), Some("on"));
}

#[tokio::test]
async fn search_rfps_with_province_param() {
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("GET"))
        .and(wiremock::matchers::query_param("rfp_province", "on"))
        .respond_with(
            wiremock::ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"total_hits": 0, "hits": []})),
        )
        .mount(&mock_server)
        .await;

    let client = NorthCloudClient::new(mock_server.uri()).unwrap();
    let params = RfpSearchParams {
        rfp_province: Some("on".to_string()),
        ..Default::default()
    };
    let response = client.search_rfps(&params).await.unwrap();
    assert_eq!(response.total_hits, 0);
}

#[tokio::test]
async fn search_rfps_server_error() {
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::any())
        .respond_with(
            wiremock::ResponseTemplate::new(500).set_body_string("Internal Server Error"),
        )
        .mount(&mock_server)
        .await;

    let client = NorthCloudClient::new(mock_server.uri()).unwrap();
    let err = client.search_rfps(&RfpSearchParams::default()).await.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("500"), "Error should contain status code: {}", msg);
}

#[tokio::test]
async fn search_rfps_invalid_json() {
    let mock_server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::any())
        .respond_with(wiremock::ResponseTemplate::new(200).set_body_string("not json"))
        .mount(&mock_server)
        .await;

    let client = NorthCloudClient::new(mock_server.uri()).unwrap();
    let result = client.search_rfps(&RfpSearchParams::default()).await;
    assert!(result.is_err());
}
```

**Step 3: Run tests**

Run: `cargo test -p myme-integrations`
Expected: All pass

**Step 4: Commit**

```
test(integrations): add wiremock integration tests for NorthCloudClient
```

---

### Task 4: Add deduplication with deterministic IDs

**Files:**
- Modify: `crates/myme-ui/src/models/prospect_model.rs:649-650`

**Step 1: Replace random UUID with deterministic ID**

In `import_rfp_leads`, change the prospect construction (line 649-650):

```rust
// Before:
let prospect = Prospect {
    id: uuid::Uuid::new_v4().to_string(),

// After:
let prospect = Prospect {
    id: format!("nc-{}", hit.id),
```

This uses the NorthCloud hit ID as a stable key. Since `upsert_prospect` uses `ON CONFLICT(id) DO UPDATE`, repeated imports will update existing records instead of creating duplicates.

**Step 2: Verify existing tests still pass**

Run: `cargo test -p myme-core -p myme-integrations -p myme-organizations`
Expected: All pass

**Step 3: Commit**

```
fix(organizations): use deterministic IDs for RFP prospects to prevent duplicates
```

---

### Task 5: Separate failed vs skipped counts and use serde_json for error JSON

**Files:**
- Modify: `crates/myme-ui/src/models/prospect_model.rs:624-676`
- Modify: `crates/myme-ui/qml/pages/OrganizationDetailPage.qml:259-272`

**Step 1: Update import_rfp_leads error tracking**

Replace the counters and upsert error handling in `import_rfp_leads`:

```rust
        let now = chrono::Utc::now().to_rfc3339();
        let mut imported = 0i32;
        let mut skipped = 0i32;
        let mut failed = 0i32;

        let store_guard = store.lock();
        for hit in &response.hits {
            let rfp = match &hit.rfp {
                Some(r) => r,
                None => {
                    skipped += 1;
                    continue;
                }
            };

            // ... prospect construction unchanged ...

            match store_guard.upsert_prospect(&prospect) {
                Ok(_) => imported += 1,
                Err(e) => {
                    tracing::error!("Failed to upsert prospect '{}': {}", prospect.name, e);
                    failed += 1;
                }
            }
        }
        drop(store_guard);
```

Replace the JSON return at the end with `serde_json`:

```rust
        // Reload prospects to update UI
        self.as_mut().load_prospects(organization_id);

        let result = serde_json::json!({
            "imported": imported,
            "skipped": skipped,
            "failed": failed,
        });
        QString::from(result.to_string())
```

Also replace the error returns throughout the function to use `serde_json::json!`:

```rust
// Example for each error return:
let error = serde_json::json!({"error": e.to_string()});
return QString::from(error.to_string());
```

**Step 2: Update QML to show failures**

In `OrganizationDetailPage.qml`, update the result parsing (lines 259-272):

```javascript
                            onClicked: {
                                importingLeads = true
                                importHadError = false
                                importResult = ""
                                var resultJson = prospectModel.import_rfp_leads(detailPage.organizationId)
                                importingLeads = false
                                try {
                                    var result = JSON.parse(resultJson)
                                    if (result.error) {
                                        importHadError = true
                                        importResult = "Error: " + result.error
                                    } else if (result.failed > 0) {
                                        importHadError = true
                                        importResult = result.imported + " imported, " + result.failed + " failed, " + result.skipped + " skipped"
                                    } else {
                                        importHadError = false
                                        importResult = result.imported + " leads imported (" + result.skipped + " skipped)"
                                    }
                                } catch (e) {
                                    console.error("Find Leads JSON parse failed:", e, "raw:", resultJson)
                                    importHadError = true
                                    importResult = "Error processing results: " + resultJson.substring(0, 100)
                                }
                            }
```

**Step 3: Run tests**

Run: `cargo test -p myme-core -p myme-integrations -p myme-organizations`
Expected: All pass

**Step 4: Commit**

```
fix(organizations): separate failed vs skipped counts, use serde_json for error JSON
```

---

### Task 6: Convert import_rfp_leads to async channel pattern

This is the most complex task. It follows the established `gmail_service.rs` pattern:
1. Add message variant to `OrganizationServiceMessage`
2. Add `request_import_rfp_leads` function to `organization_service.rs`
3. Update `import_rfp_leads` to send request via channel
4. Update `poll_channel` to receive and process results
5. Update QML to use async loading pattern

**Files:**
- Modify: `crates/myme-ui/src/services/organization_service.rs`
- Modify: `crates/myme-ui/src/services/mod.rs`
- Modify: `crates/myme-ui/src/models/prospect_model.rs`
- Modify: `crates/myme-ui/qml/pages/OrganizationDetailPage.qml`

**Step 1: Add RfpImportDone message variant**

In `crates/myme-ui/src/services/organization_service.rs`, update imports and add the message variant:

```rust
use myme_organizations::{Organization, Prospect};

#[derive(Debug, Clone)]
pub enum OrganizationError {
    Database(String),
    Network(String),
    NotInitialized,
}

// ... update Display impl to include Network variant ...
impl std::fmt::Display for OrganizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrganizationError::Database(s) => write!(f, "Organization error: {}", s),
            OrganizationError::Network(s) => write!(f, "Network error: {}", s),
            OrganizationError::NotInitialized => {
                write!(f, "Organization service not initialized")
            }
        }
    }
}

/// Import result counts
#[derive(Debug, Clone)]
pub struct RfpImportResult {
    pub imported: i32,
    pub skipped: i32,
    pub failed: i32,
}

#[derive(Debug)]
pub enum OrganizationServiceMessage {
    OrganizationsLoaded(Result<Vec<Organization>, OrganizationError>),
    ProspectsLoaded(Result<Vec<Prospect>, OrganizationError>),
    ProspectStageUpdated(Result<(), OrganizationError>),
    LinkedProjectsLoaded(Result<Vec<String>, OrganizationError>),
    RfpImportDone(Result<(RfpImportResult, Vec<Prospect>), OrganizationError>),
}
```

**Step 2: Add request_import_rfp_leads function**

Add to `organization_service.rs`:

```rust
use crate::bridge;

/// Request to import RFP leads asynchronously.
pub fn request_import_rfp_leads(
    tx: &std::sync::mpsc::Sender<OrganizationServiceMessage>,
    organization_id: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(OrganizationServiceMessage::RfpImportDone(
                Err(OrganizationError::NotInitialized),
            ));
            return;
        }
    };

    runtime.spawn(async move {
        let result = do_import_rfp_leads(&organization_id).await;
        let _ = tx.send(OrganizationServiceMessage::RfpImportDone(result));
    });
}

async fn do_import_rfp_leads(
    org_id: &str,
) -> Result<(RfpImportResult, Vec<Prospect>), OrganizationError> {
    use myme_integrations::{
        build_rfp_description, rfp_budget_string, NorthCloudClient, RfpSearchParams,
    };
    use myme_organizations::ProspectStage;

    let config = myme_core::Config::load_cached();
    let base_url = config.northcloud.base_url.clone();

    let client = NorthCloudClient::new(&base_url)
        .map_err(|e| OrganizationError::Network(e.to_string()))?;

    let params = RfpSearchParams {
        rfp_province: Some("on".to_string()),
        page: 1,
        size: 50,
        ..Default::default()
    };

    let response = client
        .search_rfps(&params)
        .await
        .map_err(|e| OrganizationError::Network(e.to_string()))?;

    let store = crate::app_services::organization_store_or_init()
        .ok_or(OrganizationError::NotInitialized)?;

    let now = chrono::Utc::now().to_rfc3339();
    let mut imported = 0i32;
    let mut skipped = 0i32;
    let mut failed = 0i32;

    let store_guard = store.lock();
    for hit in &response.hits {
        let rfp = match &hit.rfp {
            Some(r) => r,
            None => {
                skipped += 1;
                continue;
            }
        };

        let name = rfp
            .title
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&hit.title)
            .to_string();

        let description = build_rfp_description(rfp, &hit.url);
        let value = rfp_budget_string(rfp);
        let contact_name = rfp.organization_name.clone().unwrap_or_default();
        let contact_email = rfp.contact_email.clone().unwrap_or_default();

        let prospect = myme_organizations::Prospect {
            id: format!("nc-{}", hit.id),
            organization_id: org_id.to_string(),
            name,
            description: Some(description),
            stage: ProspectStage::Lead,
            value: if value.is_empty() { None } else { Some(value) },
            contact_name: if contact_name.is_empty() { None } else { Some(contact_name) },
            contact_email: if contact_email.is_empty() { None } else { Some(contact_email) },
            contact_role: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        match store_guard.upsert_prospect(&prospect) {
            Ok(_) => imported += 1,
            Err(e) => {
                tracing::error!("Failed to upsert prospect '{}': {}", prospect.name, e);
                failed += 1;
            }
        }
    }

    // Reload all prospects for this org to send back to the UI
    let prospects = store_guard
        .list_prospects(org_id)
        .map_err(|e| OrganizationError::Database(e.to_string()))?;

    drop(store_guard);

    Ok((RfpImportResult { imported, skipped, failed }, prospects))
}
```

**Step 3: Export the new function from services/mod.rs**

Update `crates/myme-ui/src/services/mod.rs`:

```rust
pub use organization_service::{
    request_import_rfp_leads, OrganizationError, OrganizationServiceMessage, RfpImportResult,
};
```

**Step 4: Rewrite import_rfp_leads in prospect_model.rs**

Replace the entire `import_rfp_leads` method to use the channel pattern:

```rust
    /// Import RFPs from NorthCloud as Lead-stage prospects.
    /// Non-blocking: spawns async work and returns immediately.
    /// Results arrive via poll_channel -> OrganizationServiceMessage::RfpImportDone.
    pub fn import_rfp_leads(
        mut self: Pin<&mut Self>,
        organization_id: &QString,
    ) -> QString {
        let org_id = organization_id.to_string();
        if org_id.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Organization ID is required"));
            return QString::from(r#"{"error":"organization_id is required"}"#);
        }

        bridge::init_organization_service_channel();
        let tx = match bridge::get_organization_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                return QString::from(r#"{"error":"service channel not ready"}"#);
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        crate::services::request_import_rfp_leads(&tx, org_id);

        // Return empty — actual result arrives via poll_channel
        QString::from(r#"{"pending":true}"#)
    }
```

**Step 5: Update poll_channel to handle RfpImportDone**

Replace `poll_channel` in prospect_model.rs:

```rust
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_organization_message() {
            Some(m) => m,
            None => return,
        };

        use crate::services::OrganizationServiceMessage;
        match msg {
            OrganizationServiceMessage::RfpImportDone(result) => {
                self.as_mut().set_loading(false);
                match result {
                    Ok((counts, prospects)) => {
                        self.as_mut().rust_mut().prospects = prospects;
                        self.as_mut().prospects_changed();

                        // Store result as JSON for QML to read
                        let result_json = serde_json::json!({
                            "imported": counts.imported,
                            "skipped": counts.skipped,
                            "failed": counts.failed,
                        });
                        self.as_mut().set_import_result(
                            QString::from(result_json.to_string())
                        );
                    }
                    Err(e) => {
                        tracing::error!("RFP import failed: {}", e);
                        self.as_mut().set_error_message(
                            QString::from(format!("Failed to import leads: {}", e))
                        );
                        let error_json = serde_json::json!({"error": e.to_string()});
                        self.as_mut().set_import_result(
                            QString::from(error_json.to_string())
                        );
                    }
                }
            }
            _ => {
                // Other message types reserved for future use
            }
        }
    }
```

**Step 6: Add import_result QProperty to ProspectModel**

Add a new QProperty for the import result. In the bridge declaration:

```rust
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error_message)]
        #[qproperty(QString, organization_id)]
        #[qproperty(QString, import_result)]
        type ProspectModel = super::ProspectModelRust;
```

Add the field to `ProspectModelRust`:

```rust
#[derive(Default)]
pub struct ProspectModelRust {
    loading: bool,
    error_message: QString,
    organization_id: QString,
    import_result: QString,
    prospects: Vec<Prospect>,
    organization_store: Option<Arc<parking_lot::Mutex<OrganizationStore>>>,
}
```

**Step 7: Update QML to use async pattern**

Update `OrganizationDetailPage.qml`. Remove old properties (lines 23-25):

```qml
    // Remove these:
    // property bool importingLeads: false
    // property bool importHadError: false
    // property string importResult: ""
```

Remove the `onCurrentTabChanged` handler for import state (lines 16-19).

Update the `Timer` to always poll when loading:

```qml
    Timer {
        id: prospectPollTimer
        interval: 100
        running: prospectModel.loading
        repeat: true
        onTriggered: prospectModel.poll_channel()
    }
```

Update the toolbar and button (lines 218-274):

```qml
                    // Find Leads toolbar
                    RowLayout {
                        Layout.fillWidth: true
                        Layout.topMargin: Theme.spacingMd
                        Layout.leftMargin: Theme.spacingMd
                        Layout.rightMargin: Theme.spacingMd
                        spacing: Theme.spacingMd

                        Item { Layout.fillWidth: true }

                        Label {
                            visible: importStatusText !== ""
                            text: importStatusText
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            color: importStatusIsError ? (Theme.isDark ? "#f5a5a5" : "#c53030") : Theme.textSecondary

                            property string importStatusText: {
                                var raw = prospectModel.import_result
                                if (raw === "" || raw === '{"pending":true}') return ""
                                try {
                                    var r = JSON.parse(raw)
                                    if (r.error) return "Error: " + r.error
                                    if (r.failed > 0) return r.imported + " imported, " + r.failed + " failed"
                                    return r.imported + " leads imported (" + r.skipped + " skipped)"
                                } catch (e) { return "Error processing results" }
                            }
                            property bool importStatusIsError: {
                                var raw = prospectModel.import_result
                                if (raw === "") return false
                                try {
                                    var r = JSON.parse(raw)
                                    return !!r.error || (r.failed > 0)
                                } catch (e) { return true }
                            }
                        }

                        Button {
                            text: prospectModel.loading ? "Importing..." : "Find Leads"
                            enabled: !prospectModel.loading
                            font.family: Theme.fontFamily
                            font.pixelSize: Theme.fontSizeSmall
                            contentItem: Label {
                                text: parent.text
                                font: parent.font
                                color: Theme.primaryText
                                horizontalAlignment: Text.AlignHCenter
                            }
                            background: Rectangle {
                                color: parent.enabled ? (parent.hovered ? Qt.darker(Theme.primary, 1.1) : Theme.primary) : (Theme.isDark ? "#ffffff30" : "#00000020")
                                radius: Theme.buttonRadius
                                implicitHeight: 32
                                implicitWidth: 110
                            }
                            onClicked: {
                                prospectModel.import_rfp_leads(detailPage.organizationId)
                            }
                        }
                    }
```

**Step 8: Remove the NOTE comment about synchronous behavior (line 21-22)**

It's no longer accurate since the call is now async.

**Step 9: Build and verify**

Run: `cargo test -p myme-core -p myme-integrations -p myme-organizations`
Expected: All pass

Run: `task os:build:rust`
Expected: Compiles successfully

**Step 10: Commit**

```
feat(organizations): convert import_rfp_leads to async channel pattern

Fixes Critical Gotcha #1 violation: no longer calls block_on() on the
Qt main thread. Uses the established OrganizationServiceMessage channel
pattern. The "Find Leads" button now shows a real loading state and
the UI remains responsive during the network request.
```

---

### Task 7: Run full test suite and final verification

**Files:** None (verification only)

**Step 1: Run the full test suite**

Run: `cargo test -p myme-core -p myme-integrations -p myme-organizations`
Expected: All tests pass

**Step 2: Run Rust build**

Run: `task os:build:rust`
Expected: Clean build

**Step 3: Verify no clippy warnings**

Run: `cargo clippy -p myme-integrations -p myme-core -- -D warnings`
Expected: No warnings

**Step 4: Commit any remaining fixes**

Only if clippy or build reveals issues.

---

## Summary of changes by file

| File | Changes |
|------|---------|
| `northcloud.rs` | URL validation, timeout, `Default` on `RfpData`, helpers, wiremock tests |
| `Cargo.toml` (integrations) | Add `wiremock` dev-dep |
| `lib.rs` (integrations) | Export helpers |
| `organization_service.rs` | Add `RfpImportDone`, `request_import_rfp_leads`, async impl |
| `mod.rs` (services) | Export new functions |
| `prospect_model.rs` | Async channel pattern, `import_result` QProperty, remove local helpers |
| `OrganizationDetailPage.qml` | Reactive status from QProperty, real loading state |
