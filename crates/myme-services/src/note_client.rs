//! Unified note client supporting multiple backends.
//!
//! This module provides `NoteClient`, an enum that wraps both SQLite and HTTP
//! backends with a consistent async interface.

use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;

use crate::note_backend::NoteBackend;
use crate::note_store::SqliteNoteStore;
use crate::todo::{Todo, TodoClient, TodoCreateRequest, TodoUpdateRequest};

/// Unified note client supporting multiple storage backends.
///
/// Provides async methods for note operations regardless of the underlying
/// storage mechanism (SQLite or HTTP API).
#[derive(Clone)]
pub enum NoteClient {
    /// Local SQLite storage (default).
    Sqlite(Arc<Mutex<SqliteNoteStore>>),

    /// Remote HTTP API (Godo compatibility).
    Http(Arc<TodoClient>),
}

impl NoteClient {
    /// Create a new SQLite-backed note client.
    pub fn sqlite(store: SqliteNoteStore) -> Self {
        Self::Sqlite(Arc::new(Mutex::new(store)))
    }

    /// Create a new HTTP-backed note client (Godo API).
    pub fn http(client: TodoClient) -> Self {
        Self::Http(Arc::new(client))
    }

    /// Check if this client uses SQLite storage.
    pub fn is_sqlite(&self) -> bool {
        matches!(self, Self::Sqlite(_))
    }

    /// Check if this client uses HTTP API.
    pub fn is_http(&self) -> bool {
        matches!(self, Self::Http(_))
    }

    /// List all notes.
    ///
    /// Returns notes ordered by creation time (newest first).
    pub async fn list_todos(&self) -> Result<Vec<Todo>> {
        match self {
            Self::Sqlite(store) => {
                let store = store.clone();
                tokio::task::spawn_blocking(move || {
                    store.lock().list().map_err(|e| anyhow::anyhow!("{}", e))
                })
                .await?
            }
            Self::Http(client) => client.list_todos().await,
        }
    }

    /// Get a note by ID.
    ///
    /// Returns an error if the note doesn't exist (for API compatibility).
    pub async fn get_todo(&self, id: &str) -> Result<Todo> {
        match self {
            Self::Sqlite(store) => {
                let store = store.clone();
                let id = id.to_string();
                tokio::task::spawn_blocking(move || {
                    store
                        .lock()
                        .get(&id)
                        .map_err(|e| anyhow::anyhow!("{}", e))?
                        .ok_or_else(|| anyhow::anyhow!("Note not found: {}", id))
                })
                .await?
            }
            Self::Http(client) => client.get_todo(id).await,
        }
    }

    /// Create a new note.
    ///
    /// # Arguments
    /// * `request` - The create request containing note content.
    pub async fn create_todo(&self, request: TodoCreateRequest) -> Result<Todo> {
        match self {
            Self::Sqlite(store) => {
                let store = store.clone();
                tokio::task::spawn_blocking(move || {
                    store
                        .lock()
                        .create(&request.content)
                        .map_err(|e| anyhow::anyhow!("{}", e))
                })
                .await?
            }
            Self::Http(client) => client.create_todo(request).await,
        }
    }

    /// Update an existing note.
    ///
    /// # Arguments
    /// * `id` - The note ID.
    /// * `request` - The update request with optional content and done fields.
    pub async fn update_todo(&self, id: &str, request: TodoUpdateRequest) -> Result<Todo> {
        match self {
            Self::Sqlite(store) => {
                let store = store.clone();
                let id = id.to_string();
                tokio::task::spawn_blocking(move || {
                    store
                        .lock()
                        .update(&id, request.content, request.done)
                        .map_err(|e| anyhow::anyhow!("{}", e))
                })
                .await?
            }
            Self::Http(client) => client.update_todo(id, request).await,
        }
    }

    /// Delete a note.
    ///
    /// # Arguments
    /// * `id` - The note ID.
    pub async fn delete_todo(&self, id: &str) -> Result<()> {
        match self {
            Self::Sqlite(store) => {
                let store = store.clone();
                let id = id.to_string();
                tokio::task::spawn_blocking(move || {
                    store
                        .lock()
                        .delete(&id)
                        .map_err(|e| anyhow::anyhow!("{}", e))
                })
                .await?
            }
            Self::Http(client) => client.delete_todo(id).await,
        }
    }

    /// Mark a note as done.
    pub async fn mark_done(&self, id: &str) -> Result<Todo> {
        self.update_todo(
            id,
            TodoUpdateRequest {
                content: None,
                done: Some(true),
            },
        )
        .await
    }

    /// Mark a note as not done.
    pub async fn mark_undone(&self, id: &str) -> Result<Todo> {
        self.update_todo(
            id,
            TodoUpdateRequest {
                content: None,
                done: Some(false),
            },
        )
        .await
    }

    /// Toggle the done status of a note.
    pub async fn toggle_done(&self, id: &str) -> Result<Todo> {
        match self {
            Self::Sqlite(store) => {
                let store = store.clone();
                let id = id.to_string();
                tokio::task::spawn_blocking(move || {
                    store
                        .lock()
                        .toggle_done(&id)
                        .map_err(|e| anyhow::anyhow!("{}", e))
                })
                .await?
            }
            Self::Http(client) => client.toggle_done(id).await,
        }
    }

    /// Check API health (HTTP backend only).
    ///
    /// For SQLite backend, always returns true.
    pub async fn health_check(&self) -> Result<bool> {
        match self {
            Self::Sqlite(_) => Ok(true),
            Self::Http(client) => client.health_check().await,
        }
    }

    /// Get the underlying SQLite store (if using SQLite backend).
    ///
    /// Returns `None` if using HTTP backend.
    pub fn sqlite_store(&self) -> Option<Arc<Mutex<SqliteNoteStore>>> {
        match self {
            Self::Sqlite(store) => Some(store.clone()),
            Self::Http(_) => None,
        }
    }

    /// Get the underlying HTTP client (if using HTTP backend).
    ///
    /// Returns `None` if using SQLite backend.
    pub fn http_client(&self) -> Option<Arc<TodoClient>> {
        match self {
            Self::Sqlite(_) => None,
            Self::Http(client) => Some(client.clone()),
        }
    }
}

impl std::fmt::Debug for NoteClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(_) => f.debug_tuple("NoteClient::Sqlite").finish(),
            Self::Http(_) => f.debug_tuple("NoteClient::Http").finish(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_sqlite_client() -> NoteClient {
        let store = SqliteNoteStore::in_memory().expect("Failed to create in-memory store");
        NoteClient::sqlite(store)
    }

    #[tokio::test]
    async fn test_sqlite_client_create_and_list() {
        let client = create_sqlite_client();

        // Create a note
        let note = client
            .create_todo(TodoCreateRequest {
                content: "Test note".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(note.content, "Test note");
        assert!(!note.done);

        // List notes
        let notes = client.list_todos().await.unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, note.id);
    }

    #[tokio::test]
    async fn test_sqlite_client_update() {
        let client = create_sqlite_client();

        let note = client
            .create_todo(TodoCreateRequest {
                content: "Original".to_string(),
            })
            .await
            .unwrap();

        let updated = client
            .update_todo(
                &note.id,
                TodoUpdateRequest {
                    content: Some("Updated".to_string()),
                    done: Some(true),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.content, "Updated");
        assert!(updated.done);
    }

    #[tokio::test]
    async fn test_sqlite_client_delete() {
        let client = create_sqlite_client();

        let note = client
            .create_todo(TodoCreateRequest {
                content: "To delete".to_string(),
            })
            .await
            .unwrap();

        client.delete_todo(&note.id).await.unwrap();

        let notes = client.list_todos().await.unwrap();
        assert!(notes.is_empty());
    }

    #[tokio::test]
    async fn test_sqlite_client_toggle_done() {
        let client = create_sqlite_client();

        let note = client
            .create_todo(TodoCreateRequest {
                content: "Test".to_string(),
            })
            .await
            .unwrap();

        assert!(!note.done);

        let toggled = client.toggle_done(&note.id).await.unwrap();
        assert!(toggled.done);

        let toggled_back = client.toggle_done(&note.id).await.unwrap();
        assert!(!toggled_back.done);
    }

    #[tokio::test]
    async fn test_sqlite_client_health_check() {
        let client = create_sqlite_client();
        assert!(client.health_check().await.unwrap());
    }

    #[test]
    fn test_client_type_detection() {
        let sqlite_client = create_sqlite_client();
        assert!(sqlite_client.is_sqlite());
        assert!(!sqlite_client.is_http());
        assert!(sqlite_client.sqlite_store().is_some());
        assert!(sqlite_client.http_client().is_none());
    }
}
