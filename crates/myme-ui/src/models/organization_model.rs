use core::pin::Pin;
use std::collections::HashMap;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_organizations::{Organization, OrganizationStore, ProspectStage};

use crate::bridge;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error_message)]
        type OrganizationModel = super::OrganizationModelRust;

        #[qinvokable]
        fn fetch_organizations(self: Pin<&mut OrganizationModel>);

        #[qinvokable]
        fn row_count(self: &OrganizationModel) -> i32;

        #[qinvokable]
        fn get_id(self: &OrganizationModel, index: i32) -> QString;

        #[qinvokable]
        fn get_name(self: &OrganizationModel, index: i32) -> QString;

        #[qinvokable]
        fn get_description(self: &OrganizationModel, index: i32) -> QString;

        #[qinvokable]
        fn get_website(self: &OrganizationModel, index: i32) -> QString;

        #[qinvokable]
        fn get_contact_name(self: &OrganizationModel, index: i32) -> QString;

        #[qinvokable]
        fn get_contact_email(self: &OrganizationModel, index: i32) -> QString;

        #[qinvokable]
        fn get_prospect_counts(self: &OrganizationModel, index: i32) -> QString;

        #[qinvokable]
        fn create_organization(
            self: Pin<&mut OrganizationModel>,
            name: &QString,
            description: &QString,
            website: &QString,
            contact_name: &QString,
            contact_email: &QString,
            contact_phone: &QString,
            contact_role: &QString,
        );

        #[qinvokable]
        fn update_organization(
            self: Pin<&mut OrganizationModel>,
            index: i32,
            name: &QString,
            description: &QString,
            website: &QString,
            contact_name: &QString,
            contact_email: &QString,
            contact_phone: &QString,
            contact_role: &QString,
        );

        #[qinvokable]
        fn delete_organization(self: Pin<&mut OrganizationModel>, index: i32);

        #[qinvokable]
        fn poll_channel(self: Pin<&mut OrganizationModel>);

        #[qsignal]
        fn organizations_changed(self: Pin<&mut OrganizationModel>);
    }
}

/// Prospect counts per stage for an organization
#[derive(Default, Clone)]
struct ProspectCounts {
    lead: i32,
    qualified: i32,
    contacted: i32,
    proposal: i32,
    negotiation: i32,
    won: i32,
    lost: i32,
}

impl ProspectCounts {
    fn to_json(&self) -> String {
        format!(
            r#"{{"lead":{},"qualified":{},"contacted":{},"proposal":{},"negotiation":{},"won":{},"lost":{},"total":{}}}"#,
            self.lead,
            self.qualified,
            self.contacted,
            self.proposal,
            self.negotiation,
            self.won,
            self.lost,
            self.total()
        )
    }

    fn from_stage_counts(counts: &[(ProspectStage, i32)]) -> Self {
        let mut result = Self::default();
        for (stage, count) in counts {
            match stage {
                ProspectStage::Lead => result.lead = *count,
                ProspectStage::Qualified => result.qualified = *count,
                ProspectStage::Contacted => result.contacted = *count,
                ProspectStage::Proposal => result.proposal = *count,
                ProspectStage::Negotiation => result.negotiation = *count,
                ProspectStage::Won => result.won = *count,
                ProspectStage::Lost => result.lost = *count,
            }
        }
        result
    }

    fn total(&self) -> i32 {
        self.lead
            + self.qualified
            + self.contacted
            + self.proposal
            + self.negotiation
            + self.won
            + self.lost
    }
}

#[derive(Default)]
pub struct OrganizationModelRust {
    loading: bool,
    error_message: QString,
    organizations: Vec<Organization>,
    prospect_counts: HashMap<String, ProspectCounts>,
    organization_store: Option<Arc<parking_lot::Mutex<OrganizationStore>>>,
}

impl OrganizationModelRust {
    fn ensure_initialized(&mut self) {
        if self.organization_store.is_some() {
            return;
        }
        if let Some(store) = bridge::get_organization_store_or_init() {
            self.organization_store = Some(store);
            tracing::info!("OrganizationModel: store initialized");
        } else {
            tracing::warn!("OrganizationModel: store not available");
        }
    }

    fn get_org(&self, index: i32) -> Option<&Organization> {
        if index < 0 {
            return None;
        }
        self.organizations.get(index as usize)
    }

    fn load_prospect_counts(&mut self) {
        let store = match &self.organization_store {
            Some(s) => s,
            None => return,
        };
        let store_guard = store.lock();
        self.prospect_counts.clear();
        for org in &self.organizations {
            match store_guard.count_prospects_by_stage(&org.id) {
                Ok(counts) => {
                    self.prospect_counts
                        .insert(org.id.clone(), ProspectCounts::from_stage_counts(&counts));
                }
                Err(e) => {
                    tracing::warn!("Failed to get prospect counts for {}: {}", org.id, e);
                }
            }
        }
    }

}

fn opt_string(s: &QString) -> Option<String> {
    let trimmed = s.to_string().trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

impl qobject::OrganizationModel {
    pub fn fetch_organizations(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Organization store not initialized"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        let store_guard = store.lock();
        match store_guard.list_organizations() {
            Ok(orgs) => {
                tracing::info!("Loaded {} organizations from store", orgs.len());
                drop(store_guard);
                self.as_mut().rust_mut().organizations = orgs;
                self.as_mut().rust_mut().load_prospect_counts();
                self.as_mut().set_loading(false);
                self.as_mut().organizations_changed();
            }
            Err(e) => {
                tracing::error!("Failed to load organizations: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
                self.as_mut().set_loading(false);
            }
        }
    }

    pub fn row_count(&self) -> i32 {
        self.rust().organizations.len() as i32
    }

    pub fn get_id(&self, index: i32) -> QString {
        self.rust()
            .get_org(index)
            .map(|o| QString::from(&o.id))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_name(&self, index: i32) -> QString {
        self.rust()
            .get_org(index)
            .map(|o| QString::from(&o.name))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_description(&self, index: i32) -> QString {
        self.rust()
            .get_org(index)
            .and_then(|o| o.description.as_ref())
            .map(|d| QString::from(d))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_website(&self, index: i32) -> QString {
        self.rust()
            .get_org(index)
            .and_then(|o| o.website.as_ref())
            .map(|w| QString::from(w))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_contact_name(&self, index: i32) -> QString {
        self.rust()
            .get_org(index)
            .and_then(|o| o.contact_name.as_ref())
            .map(|c| QString::from(c))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_contact_email(&self, index: i32) -> QString {
        self.rust()
            .get_org(index)
            .and_then(|o| o.contact_email.as_ref())
            .map(|c| QString::from(c))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_prospect_counts(&self, index: i32) -> QString {
        self.rust()
            .get_org(index)
            .and_then(|o| self.rust().prospect_counts.get(&o.id))
            .map(|c| QString::from(c.to_json()))
            .unwrap_or_else(|| QString::from(ProspectCounts::default().to_json()))
    }

    pub fn create_organization(
        mut self: Pin<&mut Self>,
        name: &QString,
        description: &QString,
        website: &QString,
        contact_name: &QString,
        contact_email: &QString,
        contact_phone: &QString,
        contact_role: &QString,
    ) {
        self.as_mut().rust_mut().ensure_initialized();

        let name_str = name.to_string().trim().to_string();
        if name_str.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Organization name cannot be empty"));
            return;
        }

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Organization store not initialized"));
                return;
            }
        };

        let now = chrono::Utc::now().to_rfc3339();
        let org = Organization {
            id: uuid::Uuid::new_v4().to_string(),
            name: name_str,
            description: opt_string(description),
            website: opt_string(website),
            contact_name: opt_string(contact_name),
            contact_email: opt_string(contact_email),
            contact_phone: opt_string(contact_phone),
            contact_role: opt_string(contact_role),
            created_at: now.clone(),
            updated_at: now,
        };

        let store_guard = store.lock();
        match store_guard.upsert_organization(&org) {
            Ok(()) => {
                drop(store_guard);
                tracing::info!("Created organization: {}", org.name);
                self.as_mut().rust_mut().organizations.push(org.clone());
                self.as_mut()
                    .rust_mut()
                    .prospect_counts
                    .insert(org.id.clone(), ProspectCounts::default());
                self.as_mut().organizations_changed();
            }
            Err(e) => {
                tracing::error!("Failed to create organization: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn update_organization(
        mut self: Pin<&mut Self>,
        index: i32,
        name: &QString,
        description: &QString,
        website: &QString,
        contact_name: &QString,
        contact_email: &QString,
        contact_phone: &QString,
        contact_role: &QString,
    ) {
        let org_id = match self.as_ref().rust().get_org(index) {
            Some(o) => o.id.clone(),
            None => {
                tracing::warn!("update_organization: invalid index {}", index);
                self.as_mut()
                    .set_error_message(QString::from("Organization not found"));
                return;
            }
        };

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                tracing::warn!("update_organization: store not initialized");
                self.as_mut()
                    .set_error_message(QString::from("Organization store not initialized"));
                return;
            }
        };

        let name_str = name.to_string().trim().to_string();
        if name_str.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Organization name cannot be empty"));
            return;
        }

        let created_at = self.as_ref().rust().organizations[index as usize]
            .created_at
            .clone();
        let now = chrono::Utc::now().to_rfc3339();

        let org = Organization {
            id: org_id,
            name: name_str,
            description: opt_string(description),
            website: opt_string(website),
            contact_name: opt_string(contact_name),
            contact_email: opt_string(contact_email),
            contact_phone: opt_string(contact_phone),
            contact_role: opt_string(contact_role),
            created_at,
            updated_at: now,
        };

        let store_guard = store.lock();
        match store_guard.upsert_organization(&org) {
            Ok(()) => {
                drop(store_guard);
                tracing::info!("Updated organization: {}", org.name);
                self.as_mut().rust_mut().organizations[index as usize] = org;
                self.as_mut().organizations_changed();
            }
            Err(e) => {
                tracing::error!("Failed to update organization: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn delete_organization(mut self: Pin<&mut Self>, index: i32) {
        let org_id = match self.as_ref().rust().get_org(index) {
            Some(o) => o.id.clone(),
            None => {
                tracing::warn!("delete_organization: invalid index {}", index);
                return;
            }
        };

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                tracing::warn!("delete_organization: store not initialized");
                return;
            }
        };

        let store_guard = store.lock();
        match store_guard.delete_organization(&org_id) {
            Ok(()) => {
                tracing::info!("Deleted organization: {}", org_id);
                drop(store_guard);
                self.as_mut().rust_mut().organizations.remove(index as usize);
                self.as_mut().rust_mut().prospect_counts.remove(&org_id);
                self.as_mut().organizations_changed();
            }
            Err(e) => {
                tracing::error!("Failed to delete organization: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn poll_channel(self: Pin<&mut Self>) {
        // Organization operations are synchronous (local SQLite).
        // Channel reserved for future async operations.
        let _ = bridge::try_recv_organization_message();
    }
}
