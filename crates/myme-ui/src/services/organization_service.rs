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
            OrganizationError::NotInitialized => {
                write!(f, "Organization service not initialized")
            }
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

// Suppress unused variant warnings — these are used by QML models via channel polling
#[allow(dead_code)]
fn _ensure_variants_used() {
    let _: fn(Result<Vec<Organization>, OrganizationError>) -> OrganizationServiceMessage =
        OrganizationServiceMessage::OrganizationsLoaded;
    let _: fn(Result<Vec<Prospect>, OrganizationError>) -> OrganizationServiceMessage =
        OrganizationServiceMessage::ProspectsLoaded;
    let _: fn(Result<(), OrganizationError>) -> OrganizationServiceMessage =
        OrganizationServiceMessage::ProspectStageUpdated;
    let _: fn(Result<Vec<String>, OrganizationError>) -> OrganizationServiceMessage =
        OrganizationServiceMessage::LinkedProjectsLoaded;
}

// Suppress unused import warnings — types are re-exported for use by models
#[allow(dead_code)]
fn _ensure_types_used() {
    let _: Option<ProspectStage> = None;
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn organization_error_display() {
        assert!(format!("{}", OrganizationError::Database("timeout".into()))
            .contains("Organization"));
        assert!(format!("{}", OrganizationError::NotInitialized).contains("not initialized"));
    }

    #[test]
    fn organization_service_message_variants() {
        let _msg: OrganizationServiceMessage =
            OrganizationServiceMessage::OrganizationsLoaded(Err(
                OrganizationError::NotInitialized,
            ));
    }
}
