use std::pin::Pin;
use std::sync::Arc;

use myme_services::{Todo, TodoClient, TodoCreateRequest, TodoUpdateRequest};

#[cxx_qt::bridge(cxx_file_stem = "todo_model")]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        include!("cxx-qt-lib/qlist.h");
        type QList_QString = cxx_qt_lib::QList<QString>;
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error_message)]
        type TodoModel = super::TodoModelRust;

        /// Fetch todos from the API
        #[qinvokable]
        fn fetch_todos(self: Pin<&mut TodoModel>);

        /// Add a new note/todo
        #[qinvokable]
        fn add_todo(self: Pin<&mut TodoModel>, content: QString);

        /// Mark a note as done/not done
        #[qinvokable]
        fn toggle_done(self: Pin<&mut TodoModel>, index: i32);

        /// Delete a note
        #[qinvokable]
        fn delete_todo(self: Pin<&mut TodoModel>, index: i32);

        /// Get the number of todos
        #[qinvokable]
        fn row_count(self: &TodoModel) -> i32;

        /// Get note content at index
        #[qinvokable]
        fn get_content(self: &TodoModel, index: i32) -> QString;

        /// Get done status at index
        #[qinvokable]
        fn get_done(self: &TodoModel, index: i32) -> bool;

        /// Get note ID at index (UUID string)
        #[qinvokable]
        fn get_id(self: &TodoModel, index: i32) -> QString;

        /// Get created_at timestamp
        #[qinvokable]
        fn get_created_at(self: &TodoModel, index: i32) -> QString;

        /// Signal emitted when the todo list changes
        #[qsignal]
        fn todos_changed(self: Pin<&mut TodoModel>);
    }
}

/// Rust implementation of the TodoModel
#[derive(Default)]
pub struct TodoModelRust {
    loading: bool,
    error_message: QString,
    todos: Vec<Todo>,
    client: Option<Arc<TodoClient>>,
    runtime: Option<tokio::runtime::Handle>,
}

impl TodoModelRust {
    /// Set the todo client and runtime
    pub fn initialize(&mut self, client: Arc<TodoClient>, runtime: tokio::runtime::Handle) {
        self.client = Some(client);
        self.runtime = Some(runtime);
    }
}

impl ffi::TodoModel {
    /// Fetch todos from the API
    pub fn fetch_todos(mut self: Pin<&mut Self>) {
        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                tracing::error!("TodoModel not initialized with client");
                self.as_mut().set_error_message("Not initialized".into());
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => {
                tracing::error!("TodoModel not initialized with runtime");
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message("".into());

        // Spawn async task to fetch todos
        runtime.spawn({
            async move {
                match client.list_todos().await {
                    Ok(todos) => {
                        tracing::info!("Successfully fetched {} notes", todos.len());
                        // Note: In a real implementation, we'd need to send these
                        // back to the Qt thread using a channel or signal
                        // For now, this demonstrates the pattern
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch notes: {}", e);
                    }
                }
            }
        });
    }

    /// Add a new note
    pub fn add_todo(mut self: Pin<&mut Self>, content: QString) {
        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut().set_error_message("Not initialized".into());
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        let content_str = content.to_string();

        self.as_mut().set_loading(true);

        runtime.spawn(async move {
            let request = TodoCreateRequest {
                content: content_str,
            };

            match client.create_todo(request).await {
                Ok(todo) => {
                    tracing::info!("Created note: {}", todo.id);
                }
                Err(e) => {
                    tracing::error!("Failed to create note: {}", e);
                }
            }
        });
    }

    /// Toggle the done status of a note
    pub fn toggle_done(mut self: Pin<&mut Self>, index: i32) {
        let todos = &self.as_ref().rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return;
        }

        let note_id = todos[index as usize].id.clone();
        let current_done = todos[index as usize].done;

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return,
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        runtime.spawn(async move {
            let request = TodoUpdateRequest {
                content: None,
                done: Some(!current_done),
            };

            match client.update_todo(&note_id, request).await {
                Ok(_) => {
                    tracing::info!("Toggled note {} done status to {}", note_id, !current_done);
                }
                Err(e) => {
                    tracing::error!("Failed to update note: {}", e);
                }
            }
        });
    }

    /// Delete a note
    pub fn delete_todo(mut self: Pin<&mut Self>, index: i32) {
        let todos = &self.as_ref().rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return;
        }

        let note_id = todos[index as usize].id.clone();

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return,
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        runtime.spawn(async move {
            match client.delete_todo(&note_id).await {
                Ok(_) => {
                    tracing::info!("Deleted note {}", note_id);
                }
                Err(e) => {
                    tracing::error!("Failed to delete note: {}", e);
                }
            }
        });
    }

    /// Get the number of todos
    pub fn row_count(&self) -> i32 {
        self.rust().todos.len() as i32
    }

    /// Get note content at index
    pub fn get_content(&self, index: i32) -> QString {
        let todos = &self.rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return QString::from("");
        }
        QString::from(&todos[index as usize].content)
    }

    /// Get done status at index
    pub fn get_done(&self, index: i32) -> bool {
        let todos = &self.rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return false;
        }
        todos[index as usize].done
    }

    /// Get note ID at index
    pub fn get_id(&self, index: i32) -> QString {
        let todos = &self.rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return QString::from("");
        }
        QString::from(&todos[index as usize].id)
    }

    /// Get created_at timestamp formatted as string
    pub fn get_created_at(&self, index: i32) -> QString {
        let todos = &self.rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return QString::from("");
        }
        QString::from(todos[index as usize].created_at.format("%Y-%m-%d %H:%M").to_string())
    }
}
