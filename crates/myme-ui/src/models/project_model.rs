// crates/myme-ui/src/models/project_model.rs

use core::pin::Pin;
use std::collections::HashMap;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::{GitHubClient, Project, ProjectStore, TaskStatus};

use crate::bridge;
use crate::services::{request_project_fetch_repo, ProjectServiceMessage};

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
        type ProjectModel = super::ProjectModelRust;

        #[qinvokable]
        fn fetch_projects(self: Pin<&mut ProjectModel>);

        #[qinvokable]
        fn row_count(self: &ProjectModel) -> i32;

        #[qinvokable]
        fn get_id(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_project_name(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_description(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_task_counts(self: &ProjectModel, index: i32) -> QString;

        /// Returns JSON array of repo_ids for the project, e.g. ["owner/repo1","owner/repo2"]
        #[qinvokable]
        fn get_repos_for_project(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn create_project(self: Pin<&mut ProjectModel>, name: &QString, description: &QString);

        #[qinvokable]
        fn add_repo_to_project(self: Pin<&mut ProjectModel>, project_index: i32, repo_id: &QString);

        /// Add repo to project by project ID (convenience when index not available)
        #[qinvokable]
        fn add_repo_to_project_by_id(self: Pin<&mut ProjectModel>, project_id: &QString, repo_id: &QString);

        #[qinvokable]
        fn remove_repo_from_project(self: Pin<&mut ProjectModel>, project_index: i32, repo_id: &QString);

        #[qinvokable]
        fn remove_project(self: Pin<&mut ProjectModel>, index: i32);

        #[qinvokable]
        fn sync_project(self: Pin<&mut ProjectModel>, index: i32);

        #[qinvokable]
        fn check_auth(self: Pin<&mut ProjectModel>);

        /// Poll for async operation results. Call this from a QML Timer.
        #[qinvokable]
        fn poll_channel(self: Pin<&mut ProjectModel>);

        #[qsignal]
        fn projects_changed(self: Pin<&mut ProjectModel>);

        #[qsignal]
        fn auth_changed(self: Pin<&mut ProjectModel>);
    }
}

/// Task counts per status for a project
#[derive(Default, Clone)]
struct TaskCounts {
    backlog: i32,
    todo: i32,
    in_progress: i32,
    blocked: i32,
    review: i32,
    done: i32,
}

impl TaskCounts {
    fn to_json(&self) -> String {
        format!(
            r#"{{"backlog":{},"todo":{},"in_progress":{},"blocked":{},"review":{},"done":{}}}"#,
            self.backlog, self.todo, self.in_progress, self.blocked, self.review, self.done
        )
    }

    fn from_status_counts(counts: &[(TaskStatus, i32)]) -> Self {
        let mut result = Self::default();
        for (status, count) in counts {
            match status {
                TaskStatus::Backlog => result.backlog = *count,
                TaskStatus::Todo => result.todo = *count,
                TaskStatus::InProgress => result.in_progress = *count,
                TaskStatus::Blocked => result.blocked = *count,
                TaskStatus::Review => result.review = *count,
                TaskStatus::Done => result.done = *count,
            }
        }
        result
    }
}

/// Operation state tracking
#[derive(Clone, PartialEq, Eq, Default)]
enum OpState {
    #[default]
    Idle,
    AddingRepoToProject {
        project_id: String,
        repo_id: String,
    },
}

#[derive(Default)]
pub struct ProjectModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    projects: Vec<Project>,
    task_counts: HashMap<String, TaskCounts>,
    github_client: Option<Arc<GitHubClient>>,
    project_store: Option<Arc<std::sync::Mutex<ProjectStore>>>,
    op_state: OpState,
}

impl ProjectModelRust {
    /// Auto-initialize from global services if not already initialized
    fn ensure_initialized(&mut self) {
        if self.project_store.is_some() {
            return;
        }

        // Get project store (initializes if needed)
        if let Some(store) = crate::bridge::get_project_store_or_init() {
            self.project_store = Some(store);
            tracing::info!("ProjectModel: project store initialized");
        } else {
            tracing::warn!("ProjectModel: project store not available");
        }

        // Get GitHub client (may not be available if not authenticated)
        if let Some((client, _runtime)) = crate::bridge::get_github_client_and_runtime() {
            self.github_client = Some(client);
            tracing::info!("ProjectModel: GitHub client initialized");
        } else {
            tracing::info!("ProjectModel: GitHub client not available (not authenticated)");
        }
    }

    /// Get project at index if valid
    fn get_project(&self, index: i32) -> Option<&Project> {
        if index < 0 {
            return None;
        }
        self.projects.get(index as usize)
    }

    /// Load task counts for all projects from the store
    fn load_task_counts(&mut self) {
        let store = match &self.project_store {
            Some(s) => s,
            None => return,
        };

        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(e) => {
                tracing::error!("Failed to lock project store: {}", e);
                return;
            }
        };

        self.task_counts.clear();
        for project in &self.projects {
            match store_guard.count_tasks_by_status(&project.id) {
                Ok(counts) => {
                    self.task_counts
                        .insert(project.id.clone(), TaskCounts::from_status_counts(&counts));
                }
                Err(e) => {
                    tracing::warn!("Failed to get task counts for {}: {}", project.id, e);
                }
            }
        }
    }

    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }
}

impl qobject::ProjectModel {
    /// Load projects from local store
    pub fn fetch_projects(mut self: Pin<&mut Self>) {
        // Auto-initialize if needed
        self.as_mut().rust_mut().ensure_initialized();

        let store = match &self.as_ref().rust().project_store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Project store not initialized"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();

        // Load projects from store (this is synchronous/local, so OK to do inline)
        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to access store: {}", e));
                self.as_mut().set_loading(false);
                return;
            }
        };

        match store_guard.list_projects() {
            Ok(projects) => {
                tracing::info!("Loaded {} projects from store", projects.len());
                drop(store_guard); // Release lock before modifying self
                self.as_mut().rust_mut().projects = projects;
                self.as_mut().rust_mut().load_task_counts();
                self.as_mut().set_loading(false);
                self.as_mut().projects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to load projects: {}", e);
                drop(store_guard);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to load projects: {}", e));
                self.as_mut().set_loading(false);
            }
        }
    }

    /// Return number of projects
    pub fn row_count(&self) -> i32 {
        self.rust().projects.len() as i32
    }

    /// Get project ID at index
    pub fn get_id(&self, index: i32) -> QString {
        self.rust()
            .get_project(index)
            .map(|p| QString::from(&p.id))
            .unwrap_or_else(|| QString::from(""))
    }

    /// Get project name at index
    pub fn get_project_name(&self, index: i32) -> QString {
        self.rust()
            .get_project(index)
            .map(|p| QString::from(&p.name))
            .unwrap_or_else(|| QString::from(""))
    }

    /// Get repos for project at index as JSON array, e.g. ["owner/repo1","owner/repo2"]
    pub fn get_repos_for_project(&self, index: i32) -> QString {
        let (store, project_id) = match self.rust().get_project(index) {
            Some(p) => (self.rust().project_store.clone(), p.id.clone()),
            None => return QString::from("[]"),
        };
        let store = match store {
            Some(s) => s,
            None => return QString::from("[]"),
        };
        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(_) => return QString::from("[]"),
        };
        match store_guard.list_repos_for_project(&project_id) {
            Ok(repos) => {
                let json = serde_json::to_string(&repos).unwrap_or_else(|_| "[]".to_string());
                QString::from(&json)
            }
            Err(_) => QString::from("[]"),
        }
    }

    /// Get project description at index
    pub fn get_description(&self, index: i32) -> QString {
        self.rust()
            .get_project(index)
            .and_then(|p| p.description.as_ref())
            .map(|d| QString::from(d))
            .unwrap_or_else(|| QString::from(""))
    }

    /// Get task counts as JSON string
    pub fn get_task_counts(&self, index: i32) -> QString {
        self.rust()
            .get_project(index)
            .and_then(|p| self.rust().task_counts.get(&p.id))
            .map(|c| QString::from(c.to_json()))
            .unwrap_or_else(|| QString::from(TaskCounts::default().to_json()))
    }

    /// Create a new project (synchronous)
    pub fn create_project(mut self: Pin<&mut Self>, name: &QString, description: &QString) {
        self.as_mut().rust_mut().ensure_initialized();

        let name = name.to_string().trim().to_string();
        if name.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Project name cannot be empty"));
            return;
        }

        let store = match &self.as_ref().rust().project_store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Project store not initialized"));
                return;
            }
        };

        let project = Project {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.clone(),
            description: {
                let d = description.to_string().trim().to_string();
                if d.is_empty() {
                    None
                } else {
                    Some(d)
                }
            },
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to access store: {}", e));
                return;
            }
        };

        match store_guard.upsert_project(&project) {
            Ok(_) => {
                drop(store_guard);
                self.as_mut().rust_mut().projects.push(project.clone());
                self.as_mut()
                    .rust_mut()
                    .task_counts
                    .insert(project.id.clone(), TaskCounts::default());
                self.as_mut().projects_changed();
                tracing::info!("Created project: {}", project.name);
            }
            Err(e) => {
                drop(store_guard);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to create project: {}", e));
            }
        }
    }

    /// Add a repo to a project (validates repo exists on GitHub first) - non-blocking
    pub fn add_repo_to_project(
        mut self: Pin<&mut Self>,
        project_index: i32,
        repo_id: &QString,
    ) {
        self.as_mut().rust_mut().ensure_initialized();

        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("add_repo_to_project: operation already in progress");
            return;
        }

        let project = match self.as_ref().rust().get_project(project_index) {
            Some(p) => p.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Invalid project index"));
                return;
            }
        };

        let repo_id_str = repo_id.to_string();
        let parts: Vec<&str> = repo_id_str.split('/').collect();
        if parts.len() != 2 {
            self.as_mut()
                .set_error_message(QString::from("Invalid repo format. Use 'owner/repo'"));
            return;
        }

        let github_client = match &self.as_ref().rust().github_client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        bridge::init_project_service_channel();
        let tx = match bridge::get_project_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::AddingRepoToProject {
            project_id: project.id.clone(),
            repo_id: repo_id_str.clone(),
        };

        request_project_fetch_repo(&tx, github_client, owner, repo);
    }

    /// Add repo to project by project ID
    pub fn add_repo_to_project_by_id(
        mut self: Pin<&mut Self>,
        project_id: &QString,
        repo_id: &QString,
    ) {
        self.as_mut().rust_mut().ensure_initialized();

        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("add_repo_to_project_by_id: operation already in progress");
            return;
        }

        let project_id_str = project_id.to_string();
        let repo_id_str = repo_id.to_string();
        let parts: Vec<&str> = repo_id_str.split('/').collect();
        if parts.len() != 2 {
            self.as_mut()
                .set_error_message(QString::from("Invalid repo format. Use 'owner/repo'"));
            return;
        }

        let github_client = match &self.as_ref().rust().github_client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        bridge::init_project_service_channel();
        let tx = match bridge::get_project_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::AddingRepoToProject {
            project_id: project_id_str.clone(),
            repo_id: repo_id_str.clone(),
        };

        request_project_fetch_repo(&tx, github_client, owner, repo);
    }

    /// Remove a repo from a project
    pub fn remove_repo_from_project(
        mut self: Pin<&mut Self>,
        project_index: i32,
        repo_id: &QString,
    ) {
        let project_id = match self.as_ref().rust().get_project(project_index) {
            Some(p) => p.id.clone(),
            None => return,
        };

        let repo_id_str = repo_id.to_string();
        let store = match &self.as_ref().rust().project_store {
            Some(s) => s.clone(),
            None => return,
        };

        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to access store: {}", e));
                return;
            }
        };

        match store_guard.remove_repo_from_project(&project_id, &repo_id_str) {
            Ok(_) => {
                drop(store_guard);
                self.as_mut().rust_mut().load_task_counts();
                self.as_mut().projects_changed();
                tracing::info!("Removed repo {} from project {}", repo_id_str, project_id);
            }
            Err(e) => {
                drop(store_guard);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to remove repo: {}", e));
            }
        }
    }

    /// Remove a project at index
    pub fn remove_project(mut self: Pin<&mut Self>, index: i32) {
        let project_id = match self.as_ref().rust().get_project(index) {
            Some(p) => p.id.clone(),
            None => return,
        };

        let store = match &self.as_ref().rust().project_store {
            Some(s) => s.clone(),
            None => return,
        };

        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to access store: {}", e));
                return;
            }
        };

        match store_guard.delete_project(&project_id) {
            Ok(_) => {
                tracing::info!("Removed project: {}", project_id);
                drop(store_guard);
                self.as_mut().rust_mut().projects.remove(index as usize);
                self.as_mut().rust_mut().task_counts.remove(&project_id);
                self.as_mut().projects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to delete project: {}", e);
                drop(store_guard);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to delete project: {}", e));
            }
        }
    }

    /// Refresh task counts for a project (local data only)
    pub fn sync_project(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().rust_mut().load_task_counts();
        self.as_mut().projects_changed();
    }

    /// Poll for async operation results. Call this from a QML Timer (e.g., every 100ms).
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_project_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            ProjectServiceMessage::FetchRepoDone(result) => {
                let (project_id, repo_id) = match &self.as_ref().rust().op_state {
                    OpState::AddingRepoToProject { project_id, repo_id } => {
                        (project_id.clone(), repo_id.clone())
                    }
                    _ => {
                        tracing::warn!("FetchRepoDone received but not in AddingRepoToProject state");
                        return;
                    }
                };

                self.as_mut().rust_mut().op_state = OpState::Idle;

                match result {
                    Ok(_repo_info) => {
                        self.as_mut().handle_repo_added(project_id, repo_id);
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch repo from GitHub: {}", e);
                        self.as_mut()
                            .rust_mut()
                            .set_error(&format!("Failed to fetch repo: {}", e));
                        self.as_mut().set_loading(false);
                    }
                }
            }
        }
    }

    /// Handle successful repo fetch for add_repo_to_project
    fn handle_repo_added(mut self: Pin<&mut Self>, project_id: String, repo_id: String) {
        let store = match &self.as_ref().rust().project_store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .rust_mut()
                    .set_error("Project store not initialized");
                self.as_mut().set_loading(false);
                return;
            }
        };

        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to access store: {}", e));
                self.as_mut().set_loading(false);
                return;
            }
        };

        match store_guard.add_repo_to_project(&project_id, &repo_id) {
            Ok(_) => {
                drop(store_guard);
                self.as_mut().rust_mut().load_task_counts();
                self.as_mut().set_loading(false);
                self.as_mut().projects_changed();
                tracing::info!("Added repo {} to project {}", repo_id, project_id);
            }
            Err(e) => {
                drop(store_guard);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to add repo: {}", e));
                self.as_mut().set_loading(false);
            }
        }
    }

    /// Check and update authentication status
    pub fn check_auth(mut self: Pin<&mut Self>) {
        let was_authenticated = self.as_ref().rust().authenticated;
        tracing::info!("check_auth: was_authenticated = {}", was_authenticated);

        // Re-initialize to check for updated auth state
        self.as_mut().rust_mut().github_client = None;
        self.as_mut().rust_mut().ensure_initialized();

        let is_authenticated = crate::bridge::is_github_authenticated();
        tracing::info!("check_auth: is_github_authenticated() = {}", is_authenticated);

        self.as_mut().set_authenticated(is_authenticated);
        tracing::info!("check_auth: set_authenticated({}) called", is_authenticated);

        if was_authenticated != is_authenticated {
            tracing::info!("check_auth: auth state changed, emitting auth_changed signal");
            self.as_mut().auth_changed();
        }
    }
}
