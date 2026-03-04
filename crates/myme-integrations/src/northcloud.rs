use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

/// RFP metadata extracted by the NorthCloud classifier
#[derive(Debug, Clone, Deserialize, Default)]
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

/// Response from GET /api/search
#[derive(Debug, Clone, Deserialize)]
pub struct RfpSearchResponse {
    pub total_hits: i64,
    pub hits: Vec<RfpHit>,
}

/// Parameters for an RFP search
#[derive(Debug, Clone)]
pub struct RfpSearchParams {
    pub rfp_province: Option<String>,
    pub rfp_sector: Vec<String>,
    pub rfp_closing_after: Option<String>,
    pub page: u32,
    pub size: u32,
}

impl Default for RfpSearchParams {
    fn default() -> Self {
        Self {
            rfp_province: None,
            rfp_sector: Vec::new(),
            rfp_closing_after: None,
            page: 1,
            size: 20,
        }
    }
}

/// HTTP client for the NorthCloud search API
pub struct NorthCloudClient {
    client: Client,
    base_url: reqwest::Url,
}

impl NorthCloudClient {
    pub fn new(base_url: impl Into<String>) -> anyhow::Result<Self> {
        let raw = base_url.into();
        let parsed =
            reqwest::Url::parse(&raw).context(format!("invalid NorthCloud base URL: {}", raw))?;
        let client = reqwest::Client::builder()
            .user_agent("MyMe/0.1.0")
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .context("Failed to create NorthCloud HTTP client")?;
        Ok(Self { client, base_url: parsed })
    }

    /// Fetch RFP leads from the NorthCloud search API.
    /// Always filters by content_type=rfp.
    pub async fn search_rfps(&self, params: &RfpSearchParams) -> Result<RfpSearchResponse> {
        let mut url =
            self.base_url.join("/api/search").context("failed to build NorthCloud search URL")?;

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

        let response =
            self.client.get(url).send().await.context("NorthCloud API request failed")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("NorthCloud API returned {}: {}", status, body);
        }

        let content_type_header = response.headers().get(reqwest::header::CONTENT_TYPE);
        let content_type = match content_type_header {
            None => "",
            Some(v) => match v.to_str() {
                Ok(s) => s,
                Err(_) => {
                    tracing::warn!(
                        "NorthCloud API returned non-UTF-8 Content-Type header: {:?}",
                        v.as_bytes()
                    );
                    ""
                }
            },
        };
        if !content_type.contains("application/json") {
            anyhow::bail!(
                "NorthCloud API returned non-JSON response (Content-Type: {}). Is the API URL correct?",
                if content_type.is_empty() { "missing" } else { content_type }
            );
        }

        response
            .json::<RfpSearchResponse>()
            .await
            .context("failed to parse NorthCloud API response")
    }
}

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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn rfp_search_params_default() {
        let params = RfpSearchParams::default();
        assert!(params.rfp_province.is_none());
        assert!(params.rfp_sector.is_empty());
        assert_eq!(params.page, 1);
        assert_eq!(params.size, 20);
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
    fn client_rejects_invalid_base_url() {
        let result = NorthCloudClient::new("not a url");
        assert!(result.is_err());
    }

    #[test]
    fn client_accepts_valid_url() {
        let client = NorthCloudClient::new("https://northcloud.one");
        assert!(client.is_ok());
    }

    #[test]
    fn client_normalizes_trailing_slash() {
        let client = NorthCloudClient::new("https://northcloud.one/").unwrap();
        assert_eq!(client.base_url.as_str(), "https://northcloud.one/");
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
        let rfp = RfpData { budget_min: Some(5000.0), ..Default::default() };
        assert_eq!(rfp_budget_string(&rfp), "$5000+ CAD");
    }

    #[test]
    fn rfp_budget_string_max_only() {
        let rfp = RfpData { budget_max: Some(100000.0), ..Default::default() };
        assert_eq!(rfp_budget_string(&rfp), "Up to $100000 CAD");
    }

    #[test]
    fn rfp_budget_string_no_budget() {
        let rfp = RfpData::default();
        assert_eq!(rfp_budget_string(&rfp), "");
    }

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
            .and(wiremock::matchers::path("/api/search"))
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
        let params = RfpSearchParams { rfp_province: Some("on".to_string()), ..Default::default() };
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
    async fn search_rfps_non_json_content_type() {
        // Verifies the content-type guard: a 200 with text/plain triggers a clear error
        let mock_server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&mock_server)
            .await;

        let client = NorthCloudClient::new(mock_server.uri()).unwrap();
        let err = client.search_rfps(&RfpSearchParams::default()).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("non-JSON"), "Should report non-JSON content type: {}", msg);
    }

    #[tokio::test]
    async fn search_rfps_invalid_json_with_json_content_type() {
        // Verifies the JSON parse error path: 200 with application/json but wrong shape
        // set_body_json ensures content-type: application/json, bypassing the guard
        let mock_server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!("this is a string not an object")),
            )
            .mount(&mock_server)
            .await;

        let client = NorthCloudClient::new(mock_server.uri()).unwrap();
        let err = client.search_rfps(&RfpSearchParams::default()).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("parse"), "Should report JSON parse failure: {}", msg);
    }

    #[tokio::test]
    async fn search_rfps_html_response_gives_clear_error() {
        // The main use case for the content-type guard: login redirect or captive portal
        // insert_header must come after set_body_string to override text/plain
        let mock_server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::any())
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_raw(
                b"<html><body>Please log in</body></html>".to_vec(),
                "text/html; charset=utf-8",
            ))
            .mount(&mock_server)
            .await;

        let client = NorthCloudClient::new(mock_server.uri()).unwrap();
        let err = client.search_rfps(&RfpSearchParams::default()).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("non-JSON"), "Should report non-JSON content type: {}", msg);
        assert!(
            msg.contains("text/html"),
            "Should include the actual content type in error: {}",
            msg
        );
    }

    #[tokio::test]
    async fn search_rfps_default_params_omits_province() {
        let mock_server = wiremock::MockServer::start().await;
        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/api/search"))
            .respond_with(
                wiremock::ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"total_hits": 0, "hits": []})),
            )
            .mount(&mock_server)
            .await;

        let client = NorthCloudClient::new(mock_server.uri()).unwrap();
        client.search_rfps(&RfpSearchParams::default()).await.unwrap();

        let requests = mock_server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        let query = requests[0].url.query().unwrap_or("");
        assert!(
            !query.contains("rfp_province"),
            "Default params should not include rfp_province, got: {}",
            query
        );
    }
}
