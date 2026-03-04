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
    pub source_url: Option<String>,
    pub closing_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

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

    #[test]
    fn test_prospect_stage_serde_all_variants_roundtrip() {
        for stage in ProspectStage::all() {
            let json = serde_json::to_string(stage).unwrap();
            let parsed: ProspectStage = serde_json::from_str(&json).unwrap();
            assert_eq!(*stage, parsed, "Stage {:?} failed roundtrip", stage);
        }
    }

    #[test]
    fn test_prospect_stage_serde_format_is_lowercase() {
        assert_eq!(serde_json::to_string(&ProspectStage::Lead).unwrap(), "\"lead\"");
        assert_eq!(serde_json::to_string(&ProspectStage::Qualified).unwrap(), "\"qualified\"");
        assert_eq!(serde_json::to_string(&ProspectStage::Contacted).unwrap(), "\"contacted\"");
        assert_eq!(serde_json::to_string(&ProspectStage::Proposal).unwrap(), "\"proposal\"");
        assert_eq!(serde_json::to_string(&ProspectStage::Negotiation).unwrap(), "\"negotiation\"");
        assert_eq!(serde_json::to_string(&ProspectStage::Won).unwrap(), "\"won\"");
        assert_eq!(serde_json::to_string(&ProspectStage::Lost).unwrap(), "\"lost\"");
    }
}
