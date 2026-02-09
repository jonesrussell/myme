// crates/myme-ui/src/models/workflow_model.rs

use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::ProjectStore;

use crate::bridge;
use crate::services::{request_fetch_workflows, RepoWorkflows, WorkflowServiceMessage};

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
        #[qproperty(bool, authenticated)]
        #[qproperty(QString, error_message)]
        type WorkflowModel = super::WorkflowModelRust;

        #[qinvokable]
        fn fetch_workflows(self: Pin<&mut WorkflowModel>);

        #[qinvokable]
        fn poll_channel(self: Pin<&mut WorkflowModel>);

        #[qinvokable]
        fn check_auth(self: Pin<&mut WorkflowModel>);

        #[qinvokable]
        fn row_count(self: &WorkflowModel) -> i32;

        #[qinvokable]
        fn get_repo_id(self: &WorkflowModel, index: i32) -> QString;

        #[qinvokable]
        fn get_workflow_count(self: &WorkflowModel, repo_index: i32) -> i32;

        #[qinvokable]
        fn get_workflow_name(self: &WorkflowModel, repo_index: i32, workflow_index: i32) -> QString;

        #[qinvokable]
        fn get_workflow_path(self: &WorkflowModel, repo_index: i32, workflow_index: i32) -> QString;

        #[qinvokable]
        fn get_workflow_state(self: &WorkflowModel, repo_index: i32, workflow_index: i32) -> QString;

        #[qinvokable]
        fn get_workflow_html_url(
            self: &WorkflowModel,
            repo_index: i32,
            workflow_index: i32,
        ) -> QString;

        #[qsignal]
        fn workflows_changed(self: Pin<&mut WorkflowModel>);
    }
}

#[derive(Default)]
pub struct WorkflowModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    repo_workflows: Vec<RepoWorkflows>,
    project_store: Option<Arc<parking_lot::Mutex<ProjectStore>>>,
}

impl WorkflowModelRust {
    fn ensure_initialized(&mut self) {
        if self.project_store.is_some() {
            return;
        }
        if let Some(store) = bridge::get_project_store_or_init() {
            self.project_store = Some(store);
            tracing::info!("WorkflowModel: project store initialized");
        }
        self.authenticated = bridge::get_github_client_and_runtime().is_some();
    }

    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }

    fn get_repo_workflows(&self, index: i32) -> Option<&RepoWorkflows> {
        if index < 0 {
            return None;
        }
        self.repo_workflows.get(index as usize)
    }

    fn get_workflow(&self, repo_index: i32, workflow_index: i32) -> Option<&myme_services::GitHubWorkflow> {
        let rw = self.get_repo_workflows(repo_index)?;
        if workflow_index < 0 {
            return None;
        }
        rw.workflows.get(workflow_index as usize)
    }
}

impl qobject::WorkflowModel {
    pub fn fetch_workflows(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();

        let store = match &self.as_ref().rust().project_store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Project store not initialized"));
                return;
            }
        };

        let repo_ids = match store.lock().list_all_linked_repo_ids() {
            Ok(ids) => ids,
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to list linked repos: {}", e));
                return;
            }
        };

        let (client, _runtime) = match bridge::get_github_client_and_runtime() {
            Some(pair) => pair,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        bridge::init_workflow_service_channel();
        let tx = match bridge::get_workflow_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Workflow service channel not ready"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        request_fetch_workflows(&tx, client, repo_ids);
    }

    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_workflow_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            WorkflowServiceMessage::FetchWorkflowsDone(result) => {
                self.as_mut().set_loading(false);
                match result {
                    Ok(mut data) => {
                        for rw in &mut data {
                            rw.workflows
                                .sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.path.cmp(&b.path)));
                        }
                        self.as_mut().rust_mut().repo_workflows = data;
                        self.as_mut().workflows_changed();
                    }
                    Err(e) => {
                        self.as_mut()
                            .rust_mut()
                            .set_error(&format!("{}", e));
                    }
                }
            }
        }
    }

    pub fn check_auth(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();
        let auth = bridge::get_github_client_and_runtime().is_some();
        if self.as_ref().rust().authenticated != auth {
            self.as_mut().rust_mut().authenticated = auth;
            // Note: no auth_changed signal on this model; page can watch authenticated
        }
    }

    pub fn row_count(&self) -> i32 {
        self.rust().repo_workflows.len() as i32
    }

    pub fn get_repo_id(&self, index: i32) -> QString {
        self.rust()
            .get_repo_workflows(index)
            .map(|rw| QString::from(&rw.repo_id))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_workflow_count(&self, repo_index: i32) -> i32 {
        self.rust()
            .get_repo_workflows(repo_index)
            .map(|rw| rw.workflows.len() as i32)
            .unwrap_or(0)
    }

    pub fn get_workflow_name(&self, repo_index: i32, workflow_index: i32) -> QString {
        self.rust()
            .get_workflow(repo_index, workflow_index)
            .map(|w| QString::from(&w.name))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_workflow_path(&self, repo_index: i32, workflow_index: i32) -> QString {
        self.rust()
            .get_workflow(repo_index, workflow_index)
            .map(|w| QString::from(&w.path))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_workflow_state(&self, repo_index: i32, workflow_index: i32) -> QString {
        self.rust()
            .get_workflow(repo_index, workflow_index)
            .map(|w| QString::from(&w.state))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_workflow_html_url(
        &self,
        repo_index: i32,
        workflow_index: i32,
    ) -> QString {
        self.rust()
            .get_workflow(repo_index, workflow_index)
            .and_then(|w| w.html_url.as_ref())
            .map(|s| QString::from(s.as_str()))
            .unwrap_or_else(|| QString::from(""))
    }
}
