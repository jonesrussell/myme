use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

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
    base_url: String,
}

impl NorthCloudClient {
    pub fn new(base_url: impl Into<String>) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("MyMe/0.1.0")
            .build()
            .context("Failed to create NorthCloud HTTP client")?;
        Ok(Self {
            client,
            base_url: base_url.into(),
        })
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
