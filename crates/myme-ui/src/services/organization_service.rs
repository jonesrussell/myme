use myme_organizations::{Organization, Prospect};

/// Error type for future async organization operations.
/// Currently unused — organization operations are synchronous against local SQLite.
#[derive(Debug, Clone)]
#[allow(dead_code)]
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

/// Messages for future async organization operations.
/// Currently unused — organization models perform synchronous SQLite operations directly.
/// The channel infrastructure is scaffolded for future async needs.
#[derive(Debug)]
#[allow(dead_code)]
pub enum OrganizationServiceMessage {
    OrganizationsLoaded(Result<Vec<Organization>, OrganizationError>),
    ProspectsLoaded(Result<Vec<Prospect>, OrganizationError>),
    ProspectStageUpdated(Result<(), OrganizationError>),
    LinkedProjectsLoaded(Result<Vec<String>, OrganizationError>),
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
        assert!(format!("{}", OrganizationError::NotInitialized).contains("not initialized"));
    }

    #[test]
    fn organization_service_message_variants() {
        let _msg: OrganizationServiceMessage =
            OrganizationServiceMessage::OrganizationsLoaded(Err(OrganizationError::NotInitialized));
    }
}
