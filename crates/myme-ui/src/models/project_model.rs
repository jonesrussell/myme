// crates/myme-ui/src/models/project_model.rs

use core::pin::Pin;
use std::collections::HashMap;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::{GitHubClient, Project, ProjectStore, TaskStatus};

use crate::bridge;
use crate::services::{
    request_project_fetch_issues, request_project_fetch_repo, IssueInfo, ProjectServiceMessage,
    RepoInfo,
};

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
        fn get_github_repo(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_description(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn get_task_counts(self: &ProjectModel, index: i32) -> QString;

        #[qinvokable]
        fn add_project(self: Pin<&mut ProjectModel>, github_repo: &QString);

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
    AddingProject {
        repo_name: String,
    },
    SyncingProject {
        project_id: String,
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

    /// Get GitHub repo name at index
    pub fn get_github_repo(&self, index: i32) -> QString {
        self.rust()
            .get_project(index)
            .map(|p| QString::from(&p.github_repo))
            .unwrap_or_else(|| QString::from(""))
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

    /// Add a new project by GitHub repo name (e.g., "owner/repo") - non-blocking
    pub fn add_project(mut self: Pin<&mut Self>, github_repo: &QString) {
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("add_project: operation already in progress");
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

        let repo_name = github_repo.to_string();
        let parts: Vec<&str> = repo_name.split('/').collect();
        if parts.len() != 2 {
            self.as_mut()
                .set_error_message(QString::from("Invalid repo format. Use 'owner/repo'"));
            return;
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        // Initialize channel if needed
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
        self.as_mut().rust_mut().op_state = OpState::AddingProject {
            repo_name: repo_name.clone(),
        };

        // Spawn async operation (non-blocking)
        request_project_fetch_repo(&tx, github_client, owner, repo);
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

    /// Sync a project with GitHub (fetch issues and convert to tasks) - non-blocking
    pub fn sync_project(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("sync_project: operation already in progress");
            return;
        }

        let project = match self.as_ref().rust().get_project(index) {
            Some(p) => p.clone(),
            None => return,
        };

        let github_client = match &self.as_ref().rust().github_client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        // Parse owner/repo
        let parts: Vec<&str> = project.github_repo.split('/').collect();
        if parts.len() != 2 {
            self.as_mut()
                .set_error_message(QString::from("Invalid repo format in project"));
            return;
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();

        // Initialize channel if needed
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
        self.as_mut().rust_mut().op_state = OpState::SyncingProject {
            project_id: project.id.clone(),
        };

        // Spawn async operation (non-blocking)
        request_project_fetch_issues(&tx, github_client, project.id.clone(), owner, repo);
    }

    /// Poll for async operation results. Call this from a QML Timer (e.g., every 100ms).
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_project_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            ProjectServiceMessage::FetchRepoDone(result) => {
                // Handle add_project completion
                let repo_name = match &self.as_ref().rust().op_state {
                    OpState::AddingProject { repo_name } => repo_name.clone(),
                    _ => {
                        tracing::warn!("FetchRepoDone received but not in AddingProject state");
                        return;
                    }
                };

                self.as_mut().rust_mut().op_state = OpState::Idle;

                match result {
                    Ok(repo_info) => {
                        self.as_mut()
                            .handle_repo_fetched(repo_info, repo_name.clone());
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
            ProjectServiceMessage::FetchIssuesDone { project_id, result } => {
                // Handle sync_project completion
                self.as_mut().rust_mut().op_state = OpState::Idle;

                match result {
                    Ok(issues) => {
                        self.as_mut().handle_issues_fetched(project_id, issues);
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch issues: {}", e);
                        self.as_mut()
                            .rust_mut()
                            .set_error(&format!("Failed to sync: {}", e));
                        self.as_mut().set_loading(false);
                    }
                }
            }
        }
    }

    /// Handle successful repo fetch for add_project
    fn handle_repo_fetched(mut self: Pin<&mut Self>, repo_info: RepoInfo, _repo_name: String) {
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

        // Create project
        let project = Project {
            id: uuid::Uuid::new_v4().to_string(),
            github_repo: repo_info.full_name.clone(),
            description: repo_info.description.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            last_synced: None,
        };

        // Save to store
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

        match store_guard.upsert_project(&project) {
            Ok(_) => {
                tracing::info!("Added project: {}", project.github_repo);
                drop(store_guard);
                self.as_mut().rust_mut().projects.push(project.clone());
                self.as_mut()
                    .rust_mut()
                    .task_counts
                    .insert(project.id, TaskCounts::default());
                self.as_mut().set_loading(false);
                self.as_mut().projects_changed();
            }
            Err(e) => {
                tracing::error!("Failed to save project: {}", e);
                drop(store_guard);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Failed to save project: {}", e));
                self.as_mut().set_loading(false);
            }
        }
    }

    /// Handle successful issues fetch for sync_project
    fn handle_issues_fetched(mut self: Pin<&mut Self>, project_id: String, issues: Vec<IssueInfo>) {
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

        // Find the project
        let project = match self
            .as_ref()
            .rust()
            .projects
            .iter()
            .find(|p| p.id == project_id)
        {
            Some(p) => p.clone(),
            None => {
                self.as_mut().rust_mut().set_error("Project not found");
                self.as_mut().set_loading(false);
                return;
            }
        };

        tracing::info!(
            "Fetched {} issues for {}",
            issues.len(),
            project.github_repo
        );

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

        // Convert issues to tasks and save
        for issue in &issues {
            let status = TaskStatus::from_github(&issue.state, &issue.labels);

            let task = myme_services::Task {
                id: uuid::Uuid::new_v4().to_string(),
                project_id: project.id.clone(),
                github_issue_number: issue.number,
                title: issue.title.clone(),
                body: issue.body.clone(),
                status,
                labels: issue.labels.clone(),
                html_url: issue.html_url.clone(),
                created_at: issue.created_at.clone(),
                updated_at: issue.updated_at.clone(),
            };

            if let Err(e) = store_guard.upsert_task(&task) {
                tracing::warn!("Failed to save task for issue #{}: {}", issue.number, e);
            }
        }

        // Update project's last_synced timestamp
        let mut updated_project = project.clone();
        updated_project.last_synced = Some(chrono::Utc::now().to_rfc3339());
        if let Err(e) = store_guard.upsert_project(&updated_project) {
            tracing::warn!("Failed to update project sync time: {}", e);
        }

        // Get updated task counts
        let counts = store_guard
            .count_tasks_by_status(&project.id)
            .unwrap_or_default();

        drop(store_guard);

        // Update local state
        if let Some(p) = self
            .as_mut()
            .rust_mut()
            .projects
            .iter_mut()
            .find(|p| p.id == project_id)
        {
            p.last_synced = updated_project.last_synced.clone();
        }

        self.as_mut()
            .rust_mut()
            .task_counts
            .insert(project_id, TaskCounts::from_status_counts(&counts));

        self.as_mut().set_loading(false);
        self.as_mut().projects_changed();

        tracing::info!("Synced project: {}", project.github_repo);
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
