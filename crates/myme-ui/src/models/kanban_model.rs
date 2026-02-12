// crates/myme-ui/src/models/kanban_model.rs

use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::{ProjectStore, Task, TaskStatus};

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
        #[qproperty(QString, project_id)]
        #[qproperty(QString, repo_ids)]
        type KanbanModel = super::KanbanModelRust;

        #[qinvokable]
        fn load_project(self: Pin<&mut KanbanModel>, project_id: QString);

        #[qinvokable]
        fn get_repo_id(self: &KanbanModel, task_index: i32) -> QString;

        #[qinvokable]
        fn row_count(self: &KanbanModel) -> i32;

        #[qinvokable]
        fn get_task_number(self: &KanbanModel, index: i32) -> i32;

        #[qinvokable]
        fn get_title(self: &KanbanModel, index: i32) -> QString;

        #[qinvokable]
        fn get_body(self: &KanbanModel, index: i32) -> QString;

        #[qinvokable]
        fn get_status(self: &KanbanModel, index: i32) -> QString;

        #[qinvokable]
        fn count_by_status(self: &KanbanModel, status: QString) -> i32;

        #[qinvokable]
        fn tasks_for_status(self: &KanbanModel, status: QString) -> QString;

        #[qinvokable]
        fn move_task(self: Pin<&mut KanbanModel>, index: i32, new_status: QString);

        #[qinvokable]
        fn create_task(self: Pin<&mut KanbanModel>, title: QString, body: QString, status: QString);

        #[qinvokable]
        fn update_task(self: Pin<&mut KanbanModel>, index: i32, title: QString, body: QString);

        #[qsignal]
        fn tasks_changed(self: Pin<&mut KanbanModel>);
    }
}

#[derive(Default)]
pub struct KanbanModelRust {
    loading: bool,
    error_message: QString,
    project_id: QString,
    repo_ids: QString,
    tasks: Vec<Task>,
    store: Option<Arc<parking_lot::Mutex<ProjectStore>>>,
}

impl KanbanModelRust {
    fn ensure_initialized(&mut self) {
        if self.store.is_some() {
            return;
        }
        if let Some(store) = crate::bridge::get_project_store_or_init() {
            self.store = Some(store);
            tracing::info!("KanbanModel: project store initialized");
        } else {
            tracing::warn!("KanbanModel: project store not available");
        }
    }

    fn get_task(&self, index: i32) -> Option<&Task> {
        if index < 0 {
            return None;
        }
        self.tasks.get(index as usize)
    }

    fn status_from_string(s: &str) -> TaskStatus {
        match s.to_lowercase().as_str() {
            "backlog" => TaskStatus::Backlog,
            "todo" => TaskStatus::Todo,
            "in_progress" | "in-progress" | "inprogress" => TaskStatus::InProgress,
            "blocked" => TaskStatus::Blocked,
            "review" => TaskStatus::Review,
            "done" => TaskStatus::Done,
            _ => TaskStatus::Todo,
        }
    }

    fn status_to_string(status: TaskStatus) -> &'static str {
        match status {
            TaskStatus::Backlog => "backlog",
            TaskStatus::Todo => "todo",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Blocked => "blocked",
            TaskStatus::Review => "review",
            TaskStatus::Done => "done",
        }
    }

    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }
}

impl qobject::KanbanModel {
    pub fn load_project(mut self: Pin<&mut Self>, project_id: QString) {
        self.as_mut().rust_mut().ensure_initialized();

        let store = match &self.as_ref().rust().store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Project store not initialized"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().set_project_id(project_id.clone());
        self.as_mut().set_repo_ids(QString::from("[]"));

        let project_id_str = project_id.to_string();

        let store_guard = store.lock();

        match store_guard.get_project(&project_id_str) {
            Ok(Some(_)) => {}
            Ok(None) => {
                self.as_mut()
                    .set_error_message(QString::from("Project not found"));
                self.as_mut().set_loading(false);
                return;
            }
            Err(e) => {
                self.as_mut()
                    .rust_mut()
                    .set_error(myme_core::AppError::from(e).user_message());
                self.as_mut().set_loading(false);
                return;
            }
        }

        let repo_ids = store_guard
            .list_repos_for_project(&project_id_str)
            .unwrap_or_default();
        let repo_ids_json = serde_json::to_string(&repo_ids).unwrap_or_else(|_| "[]".to_string());
        self.as_mut().set_repo_ids(QString::from(&repo_ids_json));

        match store_guard.list_tasks_for_project(&project_id_str) {
            Ok(tasks) => {
                tracing::info!(
                    "Loaded {} tasks for project {}",
                    tasks.len(),
                    project_id_str
                );
                drop(store_guard);
                self.as_mut().rust_mut().tasks = tasks;
                self.as_mut().set_loading(false);
                self.as_mut().tasks_changed();
            }
            Err(e) => {
                tracing::error!("Failed to load tasks: {}", e);
                drop(store_guard);
                self.as_mut()
                    .rust_mut()
                    .set_error(myme_core::AppError::from(e).user_message());
                self.as_mut().set_loading(false);
            }
        }
    }

    pub fn get_repo_id(&self, _task_index: i32) -> QString {
        QString::from("")
    }

    pub fn row_count(&self) -> i32 {
        self.rust().tasks.len() as i32
    }

    pub fn get_task_number(&self, index: i32) -> i32 {
        (index + 1) as i32
    }

    pub fn get_title(&self, index: i32) -> QString {
        self.rust()
            .get_task(index)
            .map(|t| QString::from(&t.title))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_body(&self, index: i32) -> QString {
        self.rust()
            .get_task(index)
            .and_then(|t| t.body.as_ref())
            .map(|b| QString::from(b))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_status(&self, index: i32) -> QString {
        self.rust()
            .get_task(index)
            .map(|t| QString::from(KanbanModelRust::status_to_string(t.status)))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn count_by_status(&self, status: QString) -> i32 {
        let target_status = KanbanModelRust::status_from_string(&status.to_string());
        self.rust()
            .tasks
            .iter()
            .filter(|t| t.status == target_status)
            .count() as i32
    }

    pub fn tasks_for_status(&self, status: QString) -> QString {
        let target_status = KanbanModelRust::status_from_string(&status.to_string());
        let indices: Vec<i32> = self
            .rust()
            .tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| t.status == target_status)
            .map(|(i, _)| i as i32)
            .collect();
        let json = serde_json::to_string(&indices).unwrap_or_else(|_| "[]".to_string());
        QString::from(json)
    }

    pub fn move_task(mut self: Pin<&mut Self>, index: i32, new_status: QString) {
        self.as_mut().rust_mut().ensure_initialized();

        let mut task = match self.as_ref().rust().get_task(index) {
            Some(t) => t.clone(),
            None => return,
        };

        let new_status_enum = KanbanModelRust::status_from_string(&new_status.to_string());
        if task.status == new_status_enum {
            return;
        }

        task.status = new_status_enum;
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let store = match &self.as_ref().rust().store {
            Some(s) => s.clone(),
            None => return,
        };

        let store_guard = store.lock();
        if let Err(e) = store_guard.upsert_task(&task) {
            self.as_mut()
                .rust_mut()
                .set_error(myme_core::AppError::from(e).user_message());
            return;
        }

        if let Some(t) = self.as_mut().rust_mut().tasks.get_mut(index as usize) {
            *t = task;
        }

        self.as_mut().tasks_changed();
    }

    pub fn create_task(mut self: Pin<&mut Self>, title: QString, body: QString, status: QString) {
        self.as_mut().rust_mut().ensure_initialized();

        let title_str = title.to_string().trim().to_string();
        if title_str.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Task title cannot be empty"));
            return;
        }

        let project_id_str = self.as_ref().rust().project_id.to_string();
        let status_enum = KanbanModelRust::status_from_string(&status.to_string());
        let now = chrono::Utc::now().to_rfc3339();

        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id_str.clone(),
            title: title_str.clone(),
            body: {
                let b = body.to_string().trim().to_string();
                if b.is_empty() {
                    None
                } else {
                    Some(b)
                }
            },
            status: status_enum,
            created_at: now.clone(),
            updated_at: now,
        };

        let store = match &self.as_ref().rust().store {
            Some(s) => s.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Project store not initialized"));
                return;
            }
        };

        let store_guard = store.lock();
        if let Err(e) = store_guard.upsert_task(&task) {
            self.as_mut()
                .rust_mut()
                .set_error(myme_core::AppError::from(e).user_message());
            return;
        }

        self.as_mut().rust_mut().tasks.push(task);
        self.as_mut().tasks_changed();
        tracing::info!("Created task: {}", title_str);
    }

    pub fn update_task(mut self: Pin<&mut Self>, index: i32, title: QString, body: QString) {
        self.as_mut().rust_mut().ensure_initialized();

        let mut task = match self.as_ref().rust().get_task(index) {
            Some(t) => t.clone(),
            None => return,
        };

        let title_str = title.to_string().trim().to_string();
        if title_str.is_empty() {
            self.as_mut()
                .set_error_message(QString::from("Task title cannot be empty"));
            return;
        }

        task.title = title_str;
        task.body = {
            let b = body.to_string().trim().to_string();
            if b.is_empty() {
                None
            } else {
                Some(b)
            }
        };
        task.updated_at = chrono::Utc::now().to_rfc3339();

        let store = match &self.as_ref().rust().store {
            Some(s) => s.clone(),
            None => return,
        };

        let store_guard = store.lock();
        if let Err(e) = store_guard.upsert_task(&task) {
            self.as_mut()
                .rust_mut()
                .set_error(myme_core::AppError::from(e).user_message());
            return;
        }

        if let Some(t) = self.as_mut().rust_mut().tasks.get_mut(index as usize) {
            *t = task;
        }

        self.as_mut().tasks_changed();
    }
}
