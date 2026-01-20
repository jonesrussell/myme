use std::pin::Pin;
use std::sync::Arc;

use myme_services::{Todo, TodoClient, TodoCreateRequest, TodoStatus};

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

        /// Add a new todo
        #[qinvokable]
        fn add_todo(self: Pin<&mut TodoModel>, title: QString, description: QString);

        /// Mark a todo as completed
        #[qinvokable]
        fn complete_todo(self: Pin<&mut TodoModel>, index: i32);

        /// Delete a todo
        #[qinvokable]
        fn delete_todo(self: Pin<&mut TodoModel>, index: i32);

        /// Get the number of todos
        #[qinvokable]
        fn row_count(self: &TodoModel) -> i32;

        /// Get todo title at index
        #[qinvokable]
        fn get_title(self: &TodoModel, index: i32) -> QString;

        /// Get todo description at index
        #[qinvokable]
        fn get_description(self: &TodoModel, index: i32) -> QString;

        /// Get todo status at index (0=pending, 1=in_progress, 2=completed)
        #[qinvokable]
        fn get_status(self: &TodoModel, index: i32) -> i32;

        /// Get todo ID at index
        #[qinvokable]
        fn get_id(self: &TodoModel, index: i32) -> u64;

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
                        tracing::info!("Successfully fetched {} todos", todos.len());
                        // Note: In a real implementation, we'd need to send these
                        // back to the Qt thread using a channel or signal
                        // For now, this demonstrates the pattern
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch todos: {}", e);
                    }
                }
            }
        });
    }

    /// Add a new todo
    pub fn add_todo(mut self: Pin<&mut Self>, title: QString, description: QString) {
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

        let title_str = title.to_string();
        let desc_str = description.to_string();
        let desc = if desc_str.is_empty() {
            None
        } else {
            Some(desc_str)
        };

        self.as_mut().set_loading(true);

        runtime.spawn(async move {
            let request = TodoCreateRequest {
                title: title_str,
                description: desc,
            };

            match client.create_todo(request).await {
                Ok(todo) => {
                    tracing::info!("Created todo: {:?}", todo.id);
                }
                Err(e) => {
                    tracing::error!("Failed to create todo: {}", e);
                }
            }
        });
    }

    /// Mark a todo as completed
    pub fn complete_todo(mut self: Pin<&mut Self>, index: i32) {
        let todos = &self.as_ref().rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return;
        }

        let todo_id = match todos[index as usize].id {
            Some(id) => id,
            None => return,
        };

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return,
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        runtime.spawn(async move {
            use myme_services::TodoUpdateRequest;
            let request = TodoUpdateRequest {
                title: None,
                description: None,
                status: Some(TodoStatus::Completed),
            };

            match client.update_todo(todo_id, request).await {
                Ok(_) => {
                    tracing::info!("Marked todo {} as completed", todo_id);
                }
                Err(e) => {
                    tracing::error!("Failed to update todo: {}", e);
                }
            }
        });
    }

    /// Delete a todo
    pub fn delete_todo(mut self: Pin<&mut Self>, index: i32) {
        let todos = &self.as_ref().rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return;
        }

        let todo_id = match todos[index as usize].id {
            Some(id) => id,
            None => return,
        };

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return,
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        runtime.spawn(async move {
            match client.delete_todo(todo_id).await {
                Ok(_) => {
                    tracing::info!("Deleted todo {}", todo_id);
                }
                Err(e) => {
                    tracing::error!("Failed to delete todo: {}", e);
                }
            }
        });
    }

    /// Get the number of todos
    pub fn row_count(&self) -> i32 {
        self.rust().todos.len() as i32
    }

    /// Get todo title at index
    pub fn get_title(&self, index: i32) -> QString {
        let todos = &self.rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return QString::from("");
        }
        QString::from(&todos[index as usize].title)
    }

    /// Get todo description at index
    pub fn get_description(&self, index: i32) -> QString {
        let todos = &self.rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return QString::from("");
        }
        QString::from(todos[index as usize].description.as_deref().unwrap_or(""))
    }

    /// Get todo status at index
    pub fn get_status(&self, index: i32) -> i32 {
        let todos = &self.rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return 0;
        }
        match todos[index as usize].status {
            TodoStatus::Pending => 0,
            TodoStatus::InProgress => 1,
            TodoStatus::Completed => 2,
        }
    }

    /// Get todo ID at index
    pub fn get_id(&self, index: i32) -> u64 {
        let todos = &self.rust().todos;
        if index < 0 || index >= todos.len() as i32 {
            return 0;
        }
        todos[index as usize].id.unwrap_or(0)
    }
}
