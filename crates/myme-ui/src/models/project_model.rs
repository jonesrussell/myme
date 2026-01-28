// crates/myme-ui/src/models/project_model.rs

use core::pin::Pin;
use std::collections::HashMap;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::{GitHubClient, Project, ProjectStore, TaskStatus};

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

#[derive(Default)]
pub struct ProjectModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    projects: Vec<Project>,
    task_counts: HashMap<String, TaskCounts>,
    github_client: Option<Arc<GitHubClient>>,
    project_store: Option<Arc<std::sync::Mutex<ProjectStore>>>,
    runtime: Option<tokio::runtime::Handle>,
}

impl ProjectModelRust {
    /// Auto-initialize from global services if not already initialized
    fn ensure_initialized(&mut self) {
        if self.project_store.is_some() && self.runtime.is_some() {
            return;
        }

        // Get project store (initializes if needed)
        if let Some(store) = crate::bridge::get_project_store_or_init() {
            self.project_store = Some(store);
            tracing::info!("ProjectModel: project store initialized");
        } else {
            tracing::warn!("ProjectModel: project store not available");
        }

        // Get GitHub client and runtime (may not be available if not authenticated)
        if let Some((client, runtime)) = crate::bridge::get_github_client_and_runtime() {
            self.github_client = Some(client);
            self.runtime = Some(runtime);
            self.authenticated = true;
            tracing::info!("ProjectModel: GitHub client initialized");
        } else {
            // Get runtime from bridge (must use global runtime, not Handle::current())
            if let Some(runtime) = crate::bridge::get_runtime() {
                self.runtime = Some(runtime);
            }
            self.authenticated = false;
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
        self.as_mut().set_error_message(QString::from(""));

        // Load projects from store
        let store_guard = match store.lock() {
            Ok(g) => g,
            Err(e) => {
                self.as_mut()
                    .set_error_message(QString::from(format!("Failed to access store: {}", e)));
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
                    .set_error_message(QString::from(format!("Failed to load projects: {}", e)));
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

    /// Add a new project by GitHub repo name (e.g., "owner/repo")
    pub fn add_project(mut self: Pin<&mut Self>, github_repo: &QString) {
        self.as_mut().rust_mut().ensure_initialized();

        let store = match &self.as_ref().rust().project_store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Project store not initialized"));
                return;
            }
        };

        let github_client = match &self.as_ref().rust().github_client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("GitHub not authenticated"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Runtime not initialized"));
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

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        // Fetch repo info from GitHub
        let repo_result = runtime.block_on(async { github_client.get_repo(&owner, &repo).await });

        match repo_result {
            Ok(github_repo_info) => {
                // Create project
                let project = Project {
                    id: uuid::Uuid::new_v4().to_string(),
                    github_repo: github_repo_info.full_name.clone(),
                    description: github_repo_info.description.clone(),
                    created_at: chrono::Utc::now().to_rfc3339(),
                    last_synced: None,
                };

                // Save to store
                let store_guard = match store.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        self.as_mut().set_error_message(QString::from(format!(
                            "Failed to access store: {}",
                            e
                        )));
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
                        self.as_mut().set_error_message(QString::from(format!(
                            "Failed to save project: {}",
                            e
                        )));
                        self.as_mut().set_loading(false);
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to fetch repo from GitHub: {}", e);
                self.as_mut()
                    .set_error_message(QString::from(format!("Failed to fetch repo: {}", e)));
                self.as_mut().set_loading(false);
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
                    .set_error_message(QString::from(format!("Failed to access store: {}", e)));
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
                    .set_error_message(QString::from(format!("Failed to delete project: {}", e)));
            }
        }
    }

    /// Sync a project with GitHub (fetch issues and convert to tasks)
    pub fn sync_project(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().rust_mut().ensure_initialized();

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

        let store = match &self.as_ref().rust().project_store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Project store not initialized"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
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

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        // Fetch issues from GitHub
        let issues_result = runtime.block_on(async { github_client.list_issues(&owner, &repo).await });

        match issues_result {
            Ok(issues) => {
                tracing::info!(
                    "Fetched {} issues for {}",
                    issues.len(),
                    project.github_repo
                );

                let store_guard = match store.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        self.as_mut().set_error_message(QString::from(format!(
                            "Failed to access store: {}",
                            e
                        )));
                        self.as_mut().set_loading(false);
                        return;
                    }
                };

                // Convert issues to tasks and save
                for issue in &issues {
                    let label_names: Vec<String> =
                        issue.labels.iter().map(|l| l.name.clone()).collect();
                    let status = TaskStatus::from_github(&issue.state, &label_names);

                    let task = myme_services::Task {
                        id: uuid::Uuid::new_v4().to_string(),
                        project_id: project.id.clone(),
                        github_issue_number: issue.number,
                        title: issue.title.clone(),
                        body: issue.body.clone(),
                        status,
                        labels: label_names,
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
                    .find(|p| p.id == project.id)
                {
                    p.last_synced = updated_project.last_synced.clone();
                }

                self.as_mut()
                    .rust_mut()
                    .task_counts
                    .insert(project.id, TaskCounts::from_status_counts(&counts));

                self.as_mut().set_loading(false);
                self.as_mut().projects_changed();

                tracing::info!("Synced project: {}", project.github_repo);
            }
            Err(e) => {
                tracing::error!("Failed to fetch issues: {}", e);
                self.as_mut()
                    .set_error_message(QString::from(format!("Failed to sync: {}", e)));
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
        self.as_mut().rust_mut().runtime = None;
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
