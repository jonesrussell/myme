# RFP Leads Pipeline Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Surface NorthCloud RFP leads into myme's Web Networks prospect pipeline so Russell can import government and nonprofit contract opportunities directly into his BD pipeline with one click.

**Architecture:** NorthCloud already crawls, classifies, and indexes RFPs from 6 sources into Elasticsearch. The search API exposes a `/api/v1/search` endpoint that accepts a `content_type` filter. We extend that API to support RFP-specific filters (province, sector, closing date, budget) and expose RFP metadata on hits. myme gains a `NorthCloudClient` in `myme-integrations` that calls the search API, maps results to `Prospect` records at `Lead` stage, and inserts them into `organizations.db`. A "Find Leads" button in OrganizationDetailPage Pipeline tab triggers the import.

**Tech Stack:** Go (north-cloud search service), Rust + cxx-qt (myme), reqwest (already in myme-integrations), QML (UI button)

---

## Part 1: NorthCloud — RFP search API

### Task 1: Add RFP types to search domain

**Files:**
- Modify: `search/internal/domain/content.go`
- Modify: `search/internal/domain/search.go`
- Create: `search/internal/domain/content_test.go`

**Context:** `ClassifiedContent` is deserialized from ES hits. `ToSearchHit()` converts it to `SearchHit` for the API response. Neither currently has an `rfp` field, so myme can't see RFP metadata like closing date, organization name, or budget.

---

**Step 1: Add `RFPData` struct to content.go**

In `search/internal/domain/content.go`, add after the `SearchCrimeInfo` struct:

```go
// RFPData contains structured metadata extracted from RFP documents
type RFPData struct {
	ExtractionMethod string   `json:"extraction_method,omitempty"`
	Title            string   `json:"title,omitempty"`
	ReferenceNumber  string   `json:"reference_number,omitempty"`
	OrganizationName string   `json:"organization_name,omitempty"`
	Description      string   `json:"description,omitempty"`
	PublishedDate    string   `json:"published_date,omitempty"`
	ClosingDate      string   `json:"closing_date,omitempty"`
	BudgetMin        *float64 `json:"budget_min,omitempty"`
	BudgetMax        *float64 `json:"budget_max,omitempty"`
	BudgetCurrency   string   `json:"budget_currency,omitempty"`
	ProcurementType  string   `json:"procurement_type,omitempty"`
	NAICSCodes       []string `json:"naics_codes,omitempty"`
	Categories       []string `json:"categories,omitempty"`
	Province         string   `json:"province,omitempty"`
	City             string   `json:"city,omitempty"`
	Country          string   `json:"country,omitempty"`
	Eligibility      string   `json:"eligibility,omitempty"`
	SourceURL        string   `json:"source_url,omitempty"`
	ContactName      string   `json:"contact_name,omitempty"`
	ContactEmail     string   `json:"contact_email,omitempty"`
}
```

Add `RFP *RFPData` field to `ClassifiedContent`:

```go
// In ClassifiedContent struct, after Crime field:
RFP *RFPData `json:"rfp,omitempty"`
```

Update `ToSearchHit()` to pass through RFP data:

```go
func (c *ClassifiedContent) ToSearchHit(score float64, highlight map[string][]string) *SearchHit {
	snippet := ""
	if len(highlight) == 0 && len(c.RawText) > 150 {
		snippet = c.RawText[:150] + "..."
	}

	return &SearchHit{
		ID:             c.ID,
		Title:          c.Title,
		URL:            c.URL,
		SourceName:     c.SourceName,
		PublishedDate:  c.PublishedDate,
		CrawledAt:      c.CrawledAt,
		QualityScore:   c.QualityScore,
		ContentType:    c.ContentType,
		Topics:         c.Topics,
		CrimeRelevance: c.GetCrimeRelevance(),
		OGImage:        c.OGImage,
		Score:          score,
		Highlight:      highlight,
		Snippet:        snippet,
		RFP:            c.RFP,  // NEW
	}
}
```

---

**Step 2: Add `RFPFilters` and `RFP` to search.go**

In `search/internal/domain/search.go`, add after the job filter fields in `Filters`:

```go
// RFP filters
RfpProvince    string   `json:"rfp_province,omitempty"`    // e.g. "on", "bc", "ab"
RfpSector      []string `json:"rfp_sector,omitempty"`      // e.g. ["it", "web"]
RfpClosingAfter string  `json:"rfp_closing_after,omitempty"` // ISO 8601 date string
RfpBudgetMin   *float64 `json:"rfp_budget_min,omitempty"`
RfpBudgetMax   *float64 `json:"rfp_budget_max,omitempty"`
```

Add `RFP *RFPData` field to `SearchHit` (after `OGImage`):

```go
RFP *RFPData `json:"rfp,omitempty"`
```

---

**Step 3: Write the failing test**

Create `search/internal/domain/content_test.go`:

```go
package domain_test

import (
	"testing"

	"github.com/jonesrussell/north-cloud/search/internal/domain"
)

func TestToSearchHit_PropagatesRFPData(t *testing.T) {
	budgetMin := 10000.0
	content := &domain.ClassifiedContent{
		ID:          "rfp-001",
		Title:       "Web Redesign RFP",
		ContentType: "rfp",
		RFP: &domain.RFPData{
			OrganizationName: "City of Toronto",
			ClosingDate:      "2026-04-15",
			Province:         "on",
			BudgetMin:        &budgetMin,
		},
	}

	hit := content.ToSearchHit(1.0, nil)

	if hit.RFP == nil {
		t.Fatal("expected RFP data to be propagated, got nil")
	}
	if hit.RFP.OrganizationName != "City of Toronto" {
		t.Errorf("OrganizationName = %q, want %q", hit.RFP.OrganizationName, "City of Toronto")
	}
	if hit.RFP.Province != "on" {
		t.Errorf("Province = %q, want %q", hit.RFP.Province, "on")
	}
}

func TestToSearchHit_NilRFPPassesThrough(t *testing.T) {
	content := &domain.ClassifiedContent{
		ID:          "article-001",
		ContentType: "article",
	}
	hit := content.ToSearchHit(1.0, nil)
	if hit.RFP != nil {
		t.Errorf("expected nil RFP for non-RFP content, got %+v", hit.RFP)
	}
}
```

**Step 4: Run test to verify it fails**

```bash
cd /home/fsd42/dev/north-cloud
go test ./search/internal/domain/... -v -run TestToSearchHit
```

Expected: compile error — `RFP` field not yet on `SearchHit` or `ClassifiedContent`

**Step 5: Apply the code changes from Steps 1-2**

**Step 6: Run tests to verify they pass**

```bash
cd /home/fsd42/dev/north-cloud
go test ./search/internal/domain/... -v -run TestToSearchHit
```

Expected: PASS

**Step 7: Commit**

```bash
cd /home/fsd42/dev/north-cloud
git add search/internal/domain/content.go search/internal/domain/search.go search/internal/domain/content_test.go
git commit -m "feat(search): add RFP types to search domain"
```

---

### Task 2: Add RFP filters to ES query builder + update _source

**Files:**
- Modify: `search/internal/elasticsearch/query_builder.go`
- Modify: `search/internal/elasticsearch/query_builder_test.go`

**Context:** `buildFilters()` constructs the ES bool query filter clauses. It already handles topics, content_type, quality, job filters, etc. We add RFP-specific filters. We also need to add `rfp.*` to the default `_source` field list so the nested object is returned in hits.

---

**Step 1: Write failing tests**

Open `search/internal/elasticsearch/query_builder_test.go` and add to the existing test suite:

```go
func TestBuildFilters_RfpProvince(t *testing.T) {
	cfg := testConfig()
	qb := NewQueryBuilder(cfg)
	req := &domain.SearchRequest{
		Filters: &domain.Filters{
			ContentType: "rfp",
			RfpProvince: "on",
		},
		Pagination: &domain.Pagination{Page: 1, Size: 10},
		Sort:       &domain.Sort{Field: "relevance", Order: "desc"},
		Options:    &domain.Options{},
	}

	query := qb.Build(req)

	boolQuery := query["query"].(map[string]any)["bool"].(map[string]any)
	filters := boolQuery["filter"].([]any)

	found := false
	for _, f := range filters {
		fm := f.(map[string]any)
		if term, ok := fm["term"]; ok {
			if termMap, ok := term.(map[string]any); ok {
				if _, ok := termMap["rfp.province"]; ok {
					found = true
				}
			}
		}
	}
	if !found {
		t.Error("expected rfp.province filter clause, not found in query filters")
	}
}

func TestBuildFilters_RfpSector(t *testing.T) {
	cfg := testConfig()
	qb := NewQueryBuilder(cfg)
	req := &domain.SearchRequest{
		Filters: &domain.Filters{
			ContentType: "rfp",
			RfpSector:   []string{"it", "web"},
		},
		Pagination: &domain.Pagination{Page: 1, Size: 10},
		Sort:       &domain.Sort{Field: "relevance", Order: "desc"},
		Options:    &domain.Options{},
	}

	query := qb.Build(req)
	boolQuery := query["query"].(map[string]any)["bool"].(map[string]any)
	filters := boolQuery["filter"].([]any)

	found := false
	for _, f := range filters {
		fm := f.(map[string]any)
		if terms, ok := fm["terms"]; ok {
			if tm, ok := terms.(map[string]any); ok {
				if _, ok := tm["rfp.categories"]; ok {
					found = true
				}
			}
		}
	}
	if !found {
		t.Error("expected rfp.categories terms filter, not found")
	}
}

func TestBuildFilters_RfpClosingAfter(t *testing.T) {
	cfg := testConfig()
	qb := NewQueryBuilder(cfg)
	req := &domain.SearchRequest{
		Filters: &domain.Filters{
			ContentType:    "rfp",
			RfpClosingAfter: "2026-03-10",
		},
		Pagination: &domain.Pagination{Page: 1, Size: 10},
		Sort:       &domain.Sort{Field: "relevance", Order: "desc"},
		Options:    &domain.Options{},
	}

	query := qb.Build(req)
	boolQuery := query["query"].(map[string]any)["bool"].(map[string]any)
	filters := boolQuery["filter"].([]any)

	found := false
	for _, f := range filters {
		fm := f.(map[string]any)
		if rangeClause, ok := fm["range"]; ok {
			if rm, ok := rangeClause.(map[string]any); ok {
				if _, ok := rm["rfp.closing_date"]; ok {
					found = true
				}
			}
		}
	}
	if !found {
		t.Error("expected rfp.closing_date range filter, not found")
	}
}
```

**Step 2: Run tests to verify they fail**

```bash
cd /home/fsd42/dev/north-cloud
go test ./search/internal/elasticsearch/... -v -run TestBuildFilters_Rfp
```

Expected: FAIL — RFP filter fields not yet read in `buildFilters()`

**Step 3: Add RFP filter handling to `buildFilters()`**

In `search/internal/elasticsearch/query_builder.go`, add at the end of `buildFilters()`, before the `return result` statement:

```go
// RFP filters — only applied when content_type=rfp
if filters.RfpProvince != "" {
	result = append(result, map[string]any{
		"term": map[string]any{
			"rfp.province": filters.RfpProvince,
		},
	})
}

if len(filters.RfpSector) > 0 {
	result = append(result, map[string]any{
		"terms": map[string]any{
			"rfp.categories": filters.RfpSector,
		},
	})
}

if filters.RfpClosingAfter != "" {
	result = append(result, map[string]any{
		"range": map[string]any{
			"rfp.closing_date": map[string]any{
				"gte": filters.RfpClosingAfter,
			},
		},
	})
}

if filters.RfpBudgetMin != nil {
	result = append(result, map[string]any{
		"range": map[string]any{
			"rfp.budget_max": map[string]any{
				"gte": *filters.RfpBudgetMin,
			},
		},
	})
}
```

**Step 4: Add `rfp` to the default `_source` list**

In `Build()`, find the `_source` slice and add `"rfp"`:

```go
query["_source"] = []string{
	"id", "title", "url", "source_name",
	"published_date", "crawled_at",
	"quality_score", "content_type", "topics",
	"crime", "body", "raw_text", "og_image",
	"rfp",  // ADD THIS
}
```

**Step 5: Run tests to verify they pass**

```bash
cd /home/fsd42/dev/north-cloud
go test ./search/internal/elasticsearch/... -v -run TestBuildFilters_Rfp
```

Expected: PASS

**Step 6: Run full test suite**

```bash
cd /home/fsd42/dev/north-cloud
go test ./search/... -v
```

Expected: all existing tests still pass

**Step 7: Commit**

```bash
cd /home/fsd42/dev/north-cloud
git add search/internal/elasticsearch/query_builder.go search/internal/elasticsearch/query_builder_test.go
git commit -m "feat(search): add RFP filters to query builder"
```

---

### Task 3: Deploy NorthCloud search service

**Context:** The search service runs in Docker on northcloud.one. After code changes, rebuild and restart.

**Step 1: Build and deploy**

```bash
cd /home/fsd42/dev/north-cloud
ssh jones@northcloud.one 'cd ~/north-cloud && git pull && docker compose -f docker-compose.prod.yml up -d --build search'
```

**Step 2: Verify deployment**

```bash
ssh jones@northcloud.one 'docker logs north-cloud-search-1 2>&1 | tail -20'
```

Expected: service starts cleanly, no errors

**Step 3: Smoke test RFP filter**

```bash
curl -s "https://northcloud.one/api/v1/search?content_type=rfp&rfp_province=on&q=" | jq '.total_hits, .hits[0].rfp'
```

Expected: `total_hits` > 0, `rfp` object present on first hit with fields like `closing_date`, `organization_name`

**Step 4: Commit deploy notes**

```bash
cd /home/fsd42/dev/north-cloud
git tag v0.rfp-search-api
```

---

## Part 2: myme — NorthCloud client and UI

### Task 4: Add NorthCloud HTTP client to myme-integrations

**Files:**
- Create: `crates/myme-integrations/src/northcloud.rs`
- Modify: `crates/myme-integrations/src/lib.rs`
- Modify: `crates/myme-core/src/config.rs` (add `NorthCloudConfig`)

**Context:** `myme-integrations` already has `reqwest`, `serde`, `serde_json`, and `anyhow`. The pattern to follow is the existing GitHub client in `crates/myme-integrations/src/github.rs`. We need a `NorthCloudClient` that calls `GET /api/v1/search` with `content_type=rfp` and returns typed results.

---

**Step 1: Add `NorthCloudConfig` to myme-core config.rs**

In `crates/myme-core/src/config.rs`, add the struct:

```rust
/// NorthCloud API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NorthCloudConfig {
    /// Base URL for the NorthCloud API
    #[serde(default = "default_northcloud_base_url")]
    pub base_url: String,
}

fn default_northcloud_base_url() -> String {
    "https://northcloud.one".to_string()
}

impl Default for NorthCloudConfig {
    fn default() -> Self {
        Self { base_url: default_northcloud_base_url() }
    }
}
```

Add the field to `Config`:

```rust
/// NorthCloud API settings
#[serde(default)]
pub northcloud: NorthCloudConfig,
```

Add to `Config::default()`:

```rust
northcloud: NorthCloudConfig::default(),
```

Export from `lib.rs`:

```rust
pub use config::{Config, GitHubConfig, NorthCloudConfig, NotesConfig, TemperatureUnit, WeatherConfig};
```

**Step 2: Write failing test for NorthCloudClient**

Create `crates/myme-integrations/src/northcloud.rs`:

```rust
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// RFP metadata extracted by the NorthCloud classifier
#[derive(Debug, Clone, Deserialize)]
pub struct RfpData {
    pub organization_name: Option<String>,
    pub title: Option<String>,
    pub reference_number: Option<String>,
    pub description: Option<String>,
    pub closing_date: Option<String>,
    pub budget_min: Option<f64>,
    pub budget_max: Option<f64>,
    pub budget_currency: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub categories: Option<Vec<String>>,
    pub source_url: Option<String>,
    pub contact_name: Option<String>,
    pub contact_email: Option<String>,
}

/// A single search hit from the NorthCloud API
#[derive(Debug, Clone, Deserialize)]
pub struct RfpHit {
    pub id: String,
    pub title: String,
    pub url: String,
    pub source_name: String,
    pub snippet: Option<String>,
    pub rfp: Option<RfpData>,
}

/// Response from GET /api/v1/search
#[derive(Debug, Clone, Deserialize)]
pub struct RfpSearchResponse {
    pub total_hits: i64,
    pub hits: Vec<RfpHit>,
}

/// Parameters for an RFP search
#[derive(Debug, Clone, Default, Serialize)]
pub struct RfpSearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfp_province: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rfp_sector: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rfp_closing_after: Option<String>,
    pub page: u32,
    pub size: u32,
}

/// HTTP client for the NorthCloud search API
pub struct NorthCloudClient {
    client: Client,
    base_url: String,
}

impl NorthCloudClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
        }
    }

    /// Fetch RFP leads from the NorthCloud search API.
    /// Always filters by content_type=rfp.
    pub async fn search_rfps(&self, params: &RfpSearchParams) -> Result<RfpSearchResponse> {
        let mut url = reqwest::Url::parse(&format!("{}/api/v1/search", self.base_url))
            .context("invalid NorthCloud base URL")?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("content_type", "rfp");
            query.append_pair("q", "");
            query.append_pair("page", &params.page.to_string());
            query.append_pair("size", &params.size.to_string());

            if let Some(province) = &params.rfp_province {
                query.append_pair("rfp_province", province);
            }
            if !params.rfp_sector.is_empty() {
                query.append_pair("rfp_sector", &params.rfp_sector.join(","));
            }
            if let Some(closing_after) = &params.rfp_closing_after {
                query.append_pair("rfp_closing_after", closing_after);
            }
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("NorthCloud API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("NorthCloud API returned {}: {}", status, body);
        }

        response
            .json::<RfpSearchResponse>()
            .await
            .context("failed to parse NorthCloud API response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfp_search_params_default() {
        let params = RfpSearchParams { page: 1, size: 20, ..Default::default() };
        assert!(params.rfp_province.is_none());
        assert!(params.rfp_sector.is_empty());
    }

    #[test]
    fn rfp_hit_deserialize() {
        let json = r#"{
            "id": "abc",
            "title": "Web RFP",
            "url": "https://example.com/rfp/1",
            "source_name": "Buy and Sell Canada",
            "snippet": "A web development RFP",
            "rfp": {
                "organization_name": "City of Ottawa",
                "closing_date": "2026-04-01",
                "province": "on"
            }
        }"#;
        let hit: RfpHit = serde_json::from_str(json).unwrap();
        assert_eq!(hit.title, "Web RFP");
        assert_eq!(hit.rfp.as_ref().unwrap().province.as_deref(), Some("on"));
    }

    #[test]
    fn rfp_hit_deserialize_no_rfp_field() {
        let json = r#"{
            "id": "abc",
            "title": "Some Article",
            "url": "https://example.com/1",
            "source_name": "Test Source"
        }"#;
        let hit: RfpHit = serde_json::from_str(json).unwrap();
        assert!(hit.rfp.is_none());
    }
}
```

**Step 3: Add module to lib.rs**

In `crates/myme-integrations/src/lib.rs`, add:

```rust
pub mod northcloud;
pub use northcloud::{NorthCloudClient, RfpHit, RfpSearchParams, RfpSearchResponse};
```

**Step 4: Run tests**

```bash
cd /home/fsd42/dev/myme
cargo test -p myme-integrations -- northcloud
```

Expected: PASS

**Step 5: Commit**

```bash
cd /home/fsd42/dev/myme
git add crates/myme-integrations/src/northcloud.rs crates/myme-integrations/src/lib.rs crates/myme-core/src/config.rs crates/myme-core/src/lib.rs
git commit -m "feat(integrations): add NorthCloud RFP client"
```

---

### Task 5: Add import_rfp_leads invokable to ProspectModel

**Files:**
- Modify: `crates/myme-ui/src/models/prospect_model.rs`
- Modify: `crates/myme-ui/Cargo.toml` (add myme-integrations if not present)

**Context:** `ProspectModel` already has `create_prospect()`. We add `import_rfp_leads(organization_id)` which: loads NorthCloud config, calls `NorthCloudClient::search_rfps()`, maps each hit to a `Prospect` at `Lead` stage, upserts into the org store, then emits `prospects_changed`. The invokable runs synchronously via the Tokio runtime already available in `AppServices`.

Note: myme uses `parking_lot::Mutex` guards and `AppServices::runtime()` for async work. Follow the pattern from `fetch_organizations` in `organization_model.rs`.

---

**Step 1: Check if myme-integrations is already in myme-ui Cargo.toml**

```bash
grep "myme-integrations" /home/fsd42/dev/myme/crates/myme-ui/Cargo.toml
```

If not present, add to `[dependencies]`:

```toml
myme-integrations = { path = "../myme-integrations" }
```

**Step 2: Add the bridge declaration to prospect_model.rs**

In the `extern "RustQt"` block in `crates/myme-ui/src/models/prospect_model.rs`, add:

```rust
/// Import RFPs from NorthCloud as Lead-stage prospects.
/// Returns JSON: {"imported": N, "skipped": N, "error": "..." }
#[qinvokable]
fn import_rfp_leads(
    self: Pin<&mut ProspectModel>,
    organization_id: &QString,
) -> QString;
```

**Step 3: Add the implementation to ProspectModelRust**

In the `impl ProspectModel` block, add:

```rust
fn import_rfp_leads(
    self: Pin<&mut ProspectModel>,
    organization_id: &QString,
) -> QString {
    use myme_integrations::{NorthCloudClient, RfpSearchParams};
    use myme_organizations::Prospect;
    use myme_core::Config;

    let org_id = organization_id.to_string();
    if org_id.is_empty() {
        return QString::from(r#"{"error":"organization_id is required"}"#);
    }

    let config = Config::load_cached();
    let base_url = config.northcloud.base_url.clone();

    let params = RfpSearchParams {
        rfp_province: Some("on".to_string()), // TODO: make configurable per org
        page: 1,
        size: 50,
        ..Default::default()
    };

    let rt = match crate::app_services::services().runtime_handle() {
        Some(h) => h,
        None => return QString::from(r#"{"error":"runtime not available"}"#),
    };

    let result = rt.block_on(async move {
        let client = NorthCloudClient::new(base_url);
        client.search_rfps(&params).await
    });

    let response = match result {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("RFP import failed: {}", e);
            return QString::from(format!(r#"{{"error":"{}"}}"#, e));
        }
    };

    let store = match crate::app_services::organization_store_or_init() {
        Some(s) => s,
        None => return QString::from(r#"{"error":"organization store not available"}"#),
    };

    let now = chrono::Utc::now().to_rfc3339();
    let mut imported = 0i32;
    let mut skipped = 0i32;

    for hit in &response.hits {
        let rfp = match &hit.rfp {
            Some(r) => r,
            None => { skipped += 1; continue; }
        };

        let name = rfp.title.as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&hit.title)
            .to_string();

        let description = build_rfp_description(rfp, &hit.url);
        let value = rfp_budget_string(rfp);
        let contact_name = rfp.contact_name.clone().unwrap_or_default();
        let contact_email = rfp.contact_email.clone().unwrap_or_default();
        let org_name = rfp.organization_name.clone().unwrap_or_default();

        let prospect = Prospect {
            id: uuid::Uuid::new_v4().to_string(),
            organization_id: org_id.clone(),
            name,
            description: Some(description),
            stage: myme_organizations::ProspectStage::Lead,
            value: Some(value),
            contact_name: Some(org_name), // issuing org as contact name
            contact_email: Some(contact_email),
            contact_role: None,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        let guard = store.lock();
        match guard.upsert_prospect(&prospect) {
            Ok(_) => imported += 1,
            Err(e) => {
                tracing::warn!("Failed to upsert prospect {}: {}", prospect.name, e);
                skipped += 1;
            }
        }
    }

    // Reload prospects to update UI
    drop(store); // release lock before calling load
    self.load_prospects(organization_id);

    QString::from(format!(r#"{{"imported":{},"skipped":{}}}"#, imported, skipped))
}
```

Add two private helpers after the impl block:

```rust
fn build_rfp_description(rfp: &myme_integrations::northcloud::RfpData, url: &str) -> String {
    let mut parts = Vec::new();
    if let Some(desc) = &rfp.description {
        if !desc.is_empty() {
            parts.push(desc.clone());
        }
    }
    if let Some(closing) = &rfp.closing_date {
        parts.push(format!("Closing: {}", closing));
    }
    if let Some(city) = &rfp.city {
        parts.push(format!("Location: {}", city));
    }
    if !url.is_empty() {
        parts.push(format!("Source: {}", url));
    }
    parts.join("\n")
}

fn rfp_budget_string(rfp: &myme_integrations::northcloud::RfpData) -> String {
    match (rfp.budget_min, rfp.budget_max) {
        (Some(min), Some(max)) => format!("${:.0}–${:.0} {}", min, max, rfp.budget_currency.as_deref().unwrap_or("CAD")),
        (Some(min), None) => format!("${:.0}+ {}", min, rfp.budget_currency.as_deref().unwrap_or("CAD")),
        (None, Some(max)) => format!("Up to ${:.0} {}", max, rfp.budget_currency.as_deref().unwrap_or("CAD")),
        (None, None) => String::new(),
    }
}
```

**Note on `runtime_handle`:** Check if `AppServices` already exposes a handle. If not, add to `app_services.rs`:

```rust
pub fn runtime_handle(&self) -> Option<tokio::runtime::Handle> {
    Some(self.runtime.handle().clone())
}
```

And expose as a top-level fn:

```rust
pub fn runtime_handle() -> Option<tokio::runtime::Handle> {
    services().runtime_handle()
}
```

**Step 4: Add uuid to myme-ui Cargo.toml if not present**

```bash
grep "uuid" /home/fsd42/dev/myme/crates/myme-ui/Cargo.toml
```

If missing, add:

```toml
uuid = { version = "1", features = ["v4"] }
```

And check workspace Cargo.toml:

```bash
grep "uuid" /home/fsd42/dev/myme/Cargo.toml
```

If workspace defines it, use `uuid.workspace = true` instead.

**Step 5: Build to check for errors**

```bash
cd /home/fsd42/dev/myme
cargo build -p myme-ui 2>&1 | head -50
```

Fix any compilation errors before proceeding.

**Step 6: Commit**

```bash
cd /home/fsd42/dev/myme
git add crates/myme-ui/src/models/prospect_model.rs crates/myme-ui/Cargo.toml crates/myme-ui/src/app_services.rs
git commit -m "feat(organizations): add import_rfp_leads invokable to ProspectModel"
```

---

### Task 6: Add "Find Leads" button to OrganizationDetailPage

**Files:**
- Modify: `crates/myme-ui/qml/pages/OrganizationDetailPage.qml`

**Context:** The Pipeline tab (Tab 1) shows the Kanban board. We add a toolbar row above the Kanban with a "Find Leads" button. On click it calls `prospectModel.import_rfp_leads(orgId)`, shows a loading indicator, and displays the result count.

---

**Step 1: Find the Pipeline tab in OrganizationDetailPage.qml**

```bash
grep -n "Pipeline\|tabBar\|TabBar\|TabButton" /home/fsd42/dev/myme/crates/myme-ui/qml/pages/OrganizationDetailPage.qml | head -20
```

Locate the Pipeline tab content area (the first `StackLayout` item or equivalent).

**Step 2: Add state properties near the top of the Page/Item**

After existing property declarations, add:

```qml
property bool importingLeads: false
property string importResult: ""
```

**Step 3: Add the Find Leads toolbar row**

Inside the Pipeline tab content, before the Kanban `Row`/`ListView`/`Repeater`, add:

```qml
// Find Leads toolbar
RowLayout {
    width: parent.width
    spacing: Theme.spacing

    Item { Layout.fillWidth: true }  // spacer

    Text {
        visible: importResult !== ""
        text: importResult
        color: Theme.textSecondary
        font.pixelSize: Theme.fontSizeSmall
    }

    Button {
        text: importingLeads ? "Importing..." : "Find Leads"
        enabled: !importingLeads
        onClicked: {
            importingLeads = true
            importResult = ""
            var resultJson = prospectModel.import_rfp_leads(organizationId)
            importingLeads = false
            try {
                var result = JSON.parse(resultJson)
                if (result.error) {
                    importResult = "Error: " + result.error
                } else {
                    importResult = result.imported + " leads imported"
                }
            } catch (e) {
                importResult = "Unexpected error"
            }
        }
    }
}
```

**Note:** `organizationId` should already be a property on the page set when navigating to it. Check what property name holds the current org's ID — look for `property string organizationId` or similar near the top of the file. Adjust the `import_rfp_leads(organizationId)` call to match.

**Step 4: Build**

```bash
cd /home/fsd42/dev/myme
cargo build -p myme-ui 2>&1 | grep -i "error\|warning" | head -30
```

Expected: clean build

**Step 5: Run and test manually**

```bash
cd /home/fsd42/dev/myme
task run
```

1. Navigate to Organizations → Web Networks
2. Click Pipeline tab
3. Click "Find Leads"
4. Verify leads appear in the Lead column

**Step 6: Commit**

```bash
cd /home/fsd42/dev/myme
git add crates/myme-ui/qml/pages/OrganizationDetailPage.qml
git commit -m "feat(ui): add Find Leads button to OrganizationDetailPage pipeline tab"
```

---

## After All Tasks: Smoke Test End-to-End

1. NorthCloud search API returns RFP hits with `rfp` field populated:
   ```bash
   curl -s "https://northcloud.one/api/v1/search?content_type=rfp&rfp_province=on&q=&size=3" | jq '.hits[0].rfp'
   ```

2. myme app shows Web Networks with populated Lead column after clicking "Find Leads"

3. Each imported Lead has: name (RFP title), description (desc + closing date + source URL), value (budget range if available), contact (issuing org name)

---

## Known Gaps / Future Work

- Province filter is hardcoded to `on` in the invokable — make it a per-org setting
- No deduplication: running "Find Leads" twice will create duplicate prospects (fix: check for existing prospect with same name + org before inserting)
- No "since last import" filter — all open RFPs are fetched each time
- Sector filter not yet exposed in UI (future: filter chips above Find Leads button)
