use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_organizations::{OrganizationStore, Prospect, ProspectStage};

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
        #[qproperty(QString, organization_id)]
        type ProspectModel = super::ProspectModelRust;

        #[qinvokable]
        fn load_prospects(self: Pin<&mut ProspectModel>, organization_id: &QString);

        #[qinvokable]
        fn prospect_count(self: &ProspectModel) -> i32;

        #[qinvokable]
        fn get_prospect_id(self: &ProspectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_prospect_name(self: &ProspectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_prospect_description(self: &ProspectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_prospect_stage(self: &ProspectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_prospect_value(self: &ProspectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_prospect_contact_name(self: &ProspectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_prospect_contact_email(self: &ProspectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_prospect_created_at(self: &ProspectModel, index: i32) -> QString;

        /// Returns JSON array of indices for prospects in the given stage
        #[qinvokable]
        fn prospects_for_stage(self: &ProspectModel, stage: &QString) -> QString;

        #[qinvokable]
        fn count_for_stage(self: &ProspectModel, stage: &QString) -> i32;

        #[qinvokable]
        fn move_prospect(self: Pin<&mut ProspectModel>, index: i32, new_stage: &QString);

        #[qinvokable]
        fn create_prospect(
            self: Pin<&mut ProspectModel>,
            org_id: &QString,
            name: &QString,
            description: &QString,
            stage: &QString,
            value: &QString,
            contact_name: &QString,
            contact_email: &QString,
            contact_role: &QString,
        );

        #[qinvokable]
        fn update_prospect(
            self: Pin<&mut ProspectModel>,
            index: i32,
            name: &QString,
            description: &QString,
            value: &QString,
            contact_name: &QString,
            contact_email: &QString,
            contact_role: &QString,
        );

        #[qinvokable]
        fn delete_prospect(self: Pin<&mut ProspectModel>, index: i32);

        /// Returns JSON array of project IDs linked to the current organization
        #[qinvokable]
        fn linked_projects(self: &ProspectModel) -> QString;

        #[qinvokable]
        fn link_project(self: Pin<&mut ProspectModel>, project_id: &QString);

        #[qinvokable]
        fn unlink_project(self: Pin<&mut ProspectModel>, project_id: &QString);

        #[qinvokable]
        fn poll_channel(self: Pin<&mut ProspectModel>);

        #[qsignal]
        fn prospects_changed(self: Pin<&mut ProspectModel>);
    }
}

#[derive(Default)]
pub struct ProspectModelRust {
    loading: bool,
    error_message: QString,
    organization_id: QString,
    prospects: Vec<Prospect>,
    organization_store: Option<Arc<parking_lot::Mutex<OrganizationStore>>>,
}

impl ProspectModelRust {
    fn ensure_initialized(&mut self) {
        if self.organization_store.is_some() {
            return;
        }
        if let Some(store) = bridge::get_organization_store_or_init() {
            self.organization_store = Some(store);
            tracing::info!("ProspectModel: store initialized");
        } else {
            tracing::warn!("ProspectModel: store not available");
        }
    }

    fn get_prospect(&self, index: i32) -> Option<&Prospect> {
        if index < 0 {
            return None;
        }
        self.prospects.get(index as usize)
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

fn parse_stage(s: &str) -> ProspectStage {
    let json_str = format!("\"{}\"", s);
    match serde_json::from_str(&json_str) {
        Ok(stage) => stage,
        Err(e) => {
            tracing::warn!("Invalid stage string '{}', defaulting to Lead: {}", s, e);
            ProspectStage::Lead
        }
    }
}

impl qobject::ProspectModel {
    pub fn load_prospects(mut self: Pin<&mut Self>, organization_id: &QString) {
        self.as_mut().rust_mut().ensure_initialized();
        self.as_mut()
            .set_organization_id(organization_id.clone());

        let org_id = organization_id.to_string();
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
        match store_guard.list_prospects(&org_id) {
            Ok(prospects) => {
                tracing::info!("Loaded {} prospects for org {}", prospects.len(), org_id);
                drop(store_guard);
                self.as_mut().rust_mut().prospects = prospects;
                self.as_mut().set_loading(false);
                self.as_mut().prospects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to load prospects: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
                self.as_mut().set_loading(false);
            }
        }
    }

    pub fn prospect_count(&self) -> i32 {
        self.rust().prospects.len() as i32
    }

    pub fn get_prospect_id(&self, index: i32) -> QString {
        self.rust()
            .get_prospect(index)
            .map(|p| QString::from(&p.id))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_prospect_name(&self, index: i32) -> QString {
        self.rust()
            .get_prospect(index)
            .map(|p| QString::from(&p.name))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_prospect_description(&self, index: i32) -> QString {
        self.rust()
            .get_prospect(index)
            .and_then(|p| p.description.as_ref())
            .map(|d| QString::from(d))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_prospect_stage(&self, index: i32) -> QString {
        self.rust()
            .get_prospect(index)
            .map(|p| QString::from(p.stage.display_name()))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_prospect_value(&self, index: i32) -> QString {
        self.rust()
            .get_prospect(index)
            .and_then(|p| p.value.as_ref())
            .map(|v| QString::from(v))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_prospect_contact_name(&self, index: i32) -> QString {
        self.rust()
            .get_prospect(index)
            .and_then(|p| p.contact_name.as_ref())
            .map(|c| QString::from(c))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_prospect_contact_email(&self, index: i32) -> QString {
        self.rust()
            .get_prospect(index)
            .and_then(|p| p.contact_email.as_ref())
            .map(|c| QString::from(c))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_prospect_created_at(&self, index: i32) -> QString {
        self.rust()
            .get_prospect(index)
            .map(|p| QString::from(&p.created_at))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn prospects_for_stage(&self, stage: &QString) -> QString {
        let target_stage = parse_stage(&stage.to_string());
        let indices: Vec<i32> = self
            .rust()
            .prospects
            .iter()
            .enumerate()
            .filter(|(_, p)| p.stage == target_stage)
            .map(|(i, _)| i as i32)
            .collect();
        let json = serde_json::to_string(&indices).unwrap_or_else(|_| "[]".to_string());
        QString::from(&json)
    }

    pub fn count_for_stage(&self, stage: &QString) -> i32 {
        let target_stage = parse_stage(&stage.to_string());
        self.rust()
            .prospects
            .iter()
            .filter(|p| p.stage == target_stage)
            .count() as i32
    }

    pub fn move_prospect(mut self: Pin<&mut Self>, index: i32, new_stage: &QString) {
        let prospect_id = match self.as_ref().rust().get_prospect(index) {
            Some(p) => p.id.clone(),
            None => {
                tracing::warn!("move_prospect: invalid index {}", index);
                return;
            }
        };

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                tracing::warn!("move_prospect: store not initialized");
                return;
            }
        };

        let stage = parse_stage(&new_stage.to_string());
        let store_guard = store.lock();
        match store_guard.update_prospect_stage(&prospect_id, stage) {
            Ok(()) => {
                drop(store_guard);
                self.as_mut().rust_mut().prospects[index as usize].stage = stage;
                self.as_mut().prospects_changed();
                tracing::info!(
                    "Moved prospect {} to stage {:?}",
                    prospect_id,
                    stage
                );
            }
            Err(e) => {
                tracing::error!("Failed to move prospect: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn create_prospect(
        mut self: Pin<&mut Self>,
        org_id: &QString,
        name: &QString,
        description: &QString,
        stage: &QString,
        value: &QString,
        contact_name: &QString,
        contact_email: &QString,
        contact_role: &QString,
    ) {
        self.as_mut().rust_mut().ensure_initialized();

        let name_str = name.to_string().trim().to_string();
        if name_str.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Prospect name cannot be empty"));
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
        let prospect = Prospect {
            id: uuid::Uuid::new_v4().to_string(),
            organization_id: org_id.to_string(),
            name: name_str,
            description: opt_string(description),
            stage: parse_stage(&stage.to_string()),
            value: opt_string(value),
            contact_name: opt_string(contact_name),
            contact_email: opt_string(contact_email),
            contact_role: opt_string(contact_role),
            created_at: now.clone(),
            updated_at: now,
        };

        let store_guard = store.lock();
        match store_guard.upsert_prospect(&prospect) {
            Ok(()) => {
                drop(store_guard);
                tracing::info!("Created prospect: {}", prospect.name);
                self.as_mut().rust_mut().prospects.push(prospect);
                self.as_mut().prospects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to create prospect: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn update_prospect(
        mut self: Pin<&mut Self>,
        index: i32,
        name: &QString,
        description: &QString,
        value: &QString,
        contact_name: &QString,
        contact_email: &QString,
        contact_role: &QString,
    ) {
        let existing = match self.as_ref().rust().get_prospect(index) {
            Some(p) => p.clone(),
            None => {
                tracing::warn!("update_prospect: invalid index {}", index);
                self.as_mut()
                    .set_error_message(QString::from("Prospect not found"));
                return;
            }
        };

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                tracing::warn!("update_prospect: store not initialized");
                self.as_mut()
                    .set_error_message(QString::from("Organization store not initialized"));
                return;
            }
        };

        let name_str = name.to_string().trim().to_string();
        if name_str.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Prospect name cannot be empty"));
            return;
        }

        let now = chrono::Utc::now().to_rfc3339();
        let updated = Prospect {
            id: existing.id,
            organization_id: existing.organization_id,
            name: name_str,
            description: opt_string(description),
            stage: existing.stage,
            value: opt_string(value),
            contact_name: opt_string(contact_name),
            contact_email: opt_string(contact_email),
            contact_role: opt_string(contact_role),
            created_at: existing.created_at,
            updated_at: now,
        };

        let store_guard = store.lock();
        match store_guard.upsert_prospect(&updated) {
            Ok(()) => {
                drop(store_guard);
                tracing::info!("Updated prospect: {}", updated.name);
                self.as_mut().rust_mut().prospects[index as usize] = updated;
                self.as_mut().prospects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to update prospect: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn delete_prospect(mut self: Pin<&mut Self>, index: i32) {
        let prospect_id = match self.as_ref().rust().get_prospect(index) {
            Some(p) => p.id.clone(),
            None => {
                tracing::warn!("delete_prospect: invalid index {}", index);
                return;
            }
        };

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                tracing::warn!("delete_prospect: store not initialized");
                return;
            }
        };

        let store_guard = store.lock();
        match store_guard.delete_prospect(&prospect_id) {
            Ok(()) => {
                tracing::info!("Deleted prospect: {}", prospect_id);
                drop(store_guard);
                self.as_mut().rust_mut().prospects.remove(index as usize);
                self.as_mut().prospects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to delete prospect: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn linked_projects(&self) -> QString {
        let org_id = self.rust().organization_id.to_string();
        if org_id.is_empty() {
            return QString::from("[]");
        }
        let store = match &self.rust().organization_store {
            Some(s) => s,
            None => return QString::from("[]"),
        };
        let store_guard = store.lock();
        match store_guard.list_linked_projects(&org_id) {
            Ok(projects) => {
                let json =
                    serde_json::to_string(&projects).unwrap_or_else(|_| "[]".to_string());
                QString::from(&json)
            }
            Err(e) => {
                tracing::error!("Failed to list linked projects for org {}: {}", org_id, e);
                QString::from("[]")
            }
        }
    }

    pub fn link_project(mut self: Pin<&mut Self>, project_id: &QString) {
        let org_id = self.as_ref().rust().organization_id.to_string();
        if org_id.is_empty() {
            tracing::warn!("link_project: no organization_id set");
            return;
        }

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                tracing::warn!("link_project: store not initialized");
                return;
            }
        };

        let project_id_str = project_id.to_string();
        let store_guard = store.lock();
        match store_guard.link_project(&org_id, &project_id_str) {
            Ok(()) => {
                drop(store_guard);
                tracing::info!(
                    "Linked project {} to org {}",
                    project_id_str,
                    org_id
                );
                self.as_mut().prospects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to link project: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn unlink_project(mut self: Pin<&mut Self>, project_id: &QString) {
        let org_id = self.as_ref().rust().organization_id.to_string();
        if org_id.is_empty() {
            tracing::warn!("unlink_project: no organization_id set");
            return;
        }

        let store = match &self.as_ref().rust().organization_store {
            Some(s) => s.clone(),
            None => {
                tracing::warn!("unlink_project: store not initialized");
                return;
            }
        };

        let project_id_str = project_id.to_string();
        let store_guard = store.lock();
        match store_guard.unlink_project(&org_id, &project_id_str) {
            Ok(()) => {
                drop(store_guard);
                tracing::info!(
                    "Unlinked project {} from org {}",
                    project_id_str,
                    org_id
                );
                self.as_mut().prospects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to unlink project: {}", e);
                drop(store_guard);
                self.as_mut()
                    .set_error_message(QString::from(myme_core::AppError::from(e).user_message()));
            }
        }
    }

    pub fn poll_channel(self: Pin<&mut Self>) {
        // Prospect operations are synchronous (local SQLite).
        // Channel reserved for future async operations.
        let _ = bridge::try_recv_organization_message();
    }
}
