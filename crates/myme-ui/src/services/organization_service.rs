use myme_organizations::{Organization, Prospect};

use crate::bridge;

/// Error type for organization operations.
#[derive(Debug, Clone)]
pub enum OrganizationError {
    Database(String),
    Network(String),
    NotInitialized,
}

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

impl std::error::Error for OrganizationError {}

/// Import result counts.
#[derive(Debug, Clone)]
pub struct RfpImportResult {
    pub imported: i32,
    pub skipped: i32,
    pub failed: i32,
}

/// Messages sent from async operations back to the UI thread.
#[derive(Debug)]
pub enum OrganizationServiceMessage {
    #[allow(dead_code)]
    OrganizationsLoaded(Result<Vec<Organization>, OrganizationError>),
    #[allow(dead_code)]
    ProspectsLoaded(Result<Vec<Prospect>, OrganizationError>),
    #[allow(dead_code)]
    ProspectStageUpdated(Result<(), OrganizationError>),
    #[allow(dead_code)]
    LinkedProjectsLoaded(Result<Vec<String>, OrganizationError>),
    RfpImportDone(Result<(RfpImportResult, Vec<Prospect>), OrganizationError>),
}

/// Request to import RFP leads asynchronously.
pub fn request_import_rfp_leads(
    tx: &std::sync::mpsc::Sender<OrganizationServiceMessage>,
    organization_id: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(OrganizationServiceMessage::RfpImportDone(Err(
                OrganizationError::NotInitialized,
            )));
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

        let prospect = Prospect {
            id: format!("nc-{}", hit.id),
            organization_id: org_id.to_string(),
            name,
            description: Some(description),
            stage: ProspectStage::Lead,
            value: if value.is_empty() { None } else { Some(value) },
            contact_name: if contact_name.is_empty() {
                None
            } else {
                Some(contact_name)
            },
            contact_email: if contact_email.is_empty() {
                None
            } else {
                Some(contact_email)
            },
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

    Ok((
        RfpImportResult {
            imported,
            skipped,
            failed,
        },
        prospects,
    ))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn organization_error_display() {
        assert!(
            format!("{}", OrganizationError::Database("timeout".into())).contains("Organization")
        );
        assert!(format!("{}", OrganizationError::Network("timeout".into())).contains("Network"));
        assert!(format!("{}", OrganizationError::NotInitialized).contains("not initialized"));
    }

    #[test]
    fn organization_service_message_variants() {
        let _msg: OrganizationServiceMessage =
            OrganizationServiceMessage::OrganizationsLoaded(Err(OrganizationError::NotInitialized));
    }
}
