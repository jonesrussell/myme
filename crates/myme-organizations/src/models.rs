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
    /// Direct link to the source RFP posting. When present, must be a valid URL.
    pub source_url: Option<String>,
    /// Closing date in `YYYY-MM-DD` format. Used for urgency sorting;
    /// lexicographic ordering must equal chronological ordering.
    pub closing_date: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Sort Lead prospect indices by closing_date ascending, None last.
/// Returns indices into the input slice for prospects at Lead stage only.
pub fn lead_indices_by_urgency(prospects: &[Prospect]) -> Vec<usize> {
    let mut pairs: Vec<(usize, Option<&str>)> = prospects
        .iter()
        .enumerate()
        .filter(|(_, p)| p.stage == ProspectStage::Lead)
        .map(|(i, p)| (i, p.closing_date.as_deref()))
        .collect();

    pairs.sort_by(|(_, a), (_, b)| match (a, b) {
        (Some(a), Some(b)) => a.cmp(b),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });

    pairs.into_iter().map(|(i, _)| i).collect()
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

#[cfg(test)]
mod sort_tests {
    use super::*;

    fn make_prospect(id: &str, stage: ProspectStage, closing_date: Option<&str>) -> Prospect {
        Prospect {
            id: id.to_string(),
            organization_id: "org".to_string(),
            name: "Test".to_string(),
            description: None,
            stage,
            value: None,
            contact_name: None,
            contact_email: None,
            contact_role: None,
            source_url: None,
            closing_date: closing_date.map(String::from),
            created_at: "now".to_string(),
            updated_at: "now".to_string(),
        }
    }

    fn lead(id: &str, closing_date: Option<&str>) -> Prospect {
        make_prospect(id, ProspectStage::Lead, closing_date)
    }

    fn non_lead(id: &str) -> Prospect {
        make_prospect(id, ProspectStage::Qualified, Some("2026-01-01"))
    }

    #[test]
    fn test_lead_indices_urgency_sorted_ascending_nulls_last() {
        let prospects = vec![
            lead("p0", Some("2026-04-10")),   // index 0
            non_lead("p1"),                    // index 1 — excluded
            lead("p2", None),                  // index 2 — null, must be last
            lead("p3", Some("2026-03-05")),   // index 3 — soonest
        ];
        let order = lead_indices_by_urgency(&prospects);
        assert_eq!(order, vec![3, 0, 2]);
    }

    #[test]
    fn test_lead_indices_excludes_non_lead() {
        let prospects = vec![non_lead("p0"), non_lead("p1")];
        assert!(lead_indices_by_urgency(&prospects).is_empty());
    }

    #[test]
    fn test_lead_indices_all_null_dates() {
        let prospects = vec![lead("p0", None), lead("p1", None), lead("p2", None)];
        let order = lead_indices_by_urgency(&prospects);
        assert_eq!(order.len(), 3);
    }

    #[test]
    fn test_lead_indices_single_dated() {
        let prospects = vec![lead("p0", Some("2026-06-01"))];
        assert_eq!(lead_indices_by_urgency(&prospects), vec![0]);
    }
}
