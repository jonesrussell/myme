//! Unified note client (SQLite backend only).
//!
//! Provides async methods for note operations against local SQLite storage.

use std::sync::Arc;

use anyhow::Result;
use parking_lot::Mutex;

use crate::note_backend::NoteBackend;
use crate::note_store::SqliteNoteStore;
use crate::todo::{Todo, TodoCreateRequest, TodoUpdateRequest};

/// Note client wrapping SQLite storage.
#[derive(Clone)]
pub struct NoteClient(Arc<Mutex<SqliteNoteStore>>);

impl NoteClient {
    /// Create a new SQLite-backed note client.
    pub fn sqlite(store: SqliteNoteStore) -> Self {
        Self(Arc::new(Mutex::new(store)))
    }

    /// List all non-archived notes (pinned first, then by updated_at DESC).
    pub async fn list_todos(&self) -> Result<Vec<Todo>> {
        let store = self.0.clone();
        tokio::task::spawn_blocking(move || {
            store.lock().list().map_err(|e| anyhow::anyhow!("{}", e))
        })
        .await?
    }

    /// List archived notes.
    pub async fn list_archived(&self) -> Result<Vec<Todo>> {
        let store = self.0.clone();
        tokio::task::spawn_blocking(move || {
            store.lock().list_archived().map_err(|e| anyhow::anyhow!("{}", e))
        })
        .await?
    }

    /// List notes filtered by label.
    pub async fn list_by_label(&self, label: &str) -> Result<Vec<Todo>> {
        let store = self.0.clone();
        let label = label.to_string();
        tokio::task::spawn_blocking(move || {
            store
                .lock()
                .list_by_label(&label)
                .map_err(|e| anyhow::anyhow!("{}", e))
        })
        .await?
    }

    /// List notes with reminders.
    pub async fn list_with_reminders(&self) -> Result<Vec<Todo>> {
        let store = self.0.clone();
        tokio::task::spawn_blocking(move || {
            store
                .lock()
                .list_with_reminders()
                .map_err(|e| anyhow::anyhow!("{}", e))
        })
        .await?
    }

    /// Get a note by ID.
    pub async fn get_todo(&self, id: i64) -> Result<Todo> {
        let store = self.0.clone();
        tokio::task::spawn_blocking(move || {
            store
                .lock()
                .get(id)
                .map_err(|e| anyhow::anyhow!("{}", e))?
                .ok_or_else(|| anyhow::anyhow!("Note not found: {}", id))
        })
        .await?
    }

    /// Create a new note.
    pub async fn create_todo(&self, request: TodoCreateRequest) -> Result<Todo> {
        let store = self.0.clone();
        tokio::task::spawn_blocking(move || {
            store
                .lock()
                .create(&request.content, request.is_checklist)
                .map_err(|e| anyhow::anyhow!("{}", e))
        })
        .await?
    }

    /// Update an existing note.
    pub async fn update_todo(&self, id: i64, request: TodoUpdateRequest) -> Result<Todo> {
        let store = self.0.clone();
        tokio::task::spawn_blocking(move || {
            store
                .lock()
                .update(id, request)
                .map_err(|e| anyhow::anyhow!("{}", e))
        })
        .await?
    }

    /// Delete a note.
    pub async fn delete_todo(&self, id: i64) -> Result<()> {
        let store = self.0.clone();
        tokio::task::spawn_blocking(move || {
            store
                .lock()
                .delete(id)
                .map_err(|e| anyhow::anyhow!("{}", e))
        })
        .await?
    }

    /// Mark a note as done.
    pub async fn mark_done(&self, id: i64) -> Result<Todo> {
        let mut req = TodoUpdateRequest::default();
        req.done = Some(true);
        self.update_todo(id, req).await
    }

    /// Mark a note as not done.
    pub async fn mark_undone(&self, id: i64) -> Result<Todo> {
        let mut req = TodoUpdateRequest::default();
        req.done = Some(false);
        self.update_todo(id, req).await
    }

    /// Toggle the done status of a note.
    pub async fn toggle_done(&self, id: i64) -> Result<Todo> {
        let store = self.0.clone();
        tokio::task::spawn_blocking(move || {
            store
                .lock()
                .toggle_done(id)
                .map_err(|e| anyhow::anyhow!("{}", e))
        })
        .await?
    }

    /// Health check (always true for local store).
    pub async fn health_check(&self) -> Result<bool> {
        Ok(true)
    }

    /// Get the underlying SQLite store.
    pub fn sqlite_store(&self) -> Arc<Mutex<SqliteNoteStore>> {
        self.0.clone()
    }
}

impl std::fmt::Debug for NoteClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NoteClient").finish()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn create_client() -> NoteClient {
        let store = SqliteNoteStore::in_memory().expect("in-memory store");
        NoteClient::sqlite(store)
    }

    #[tokio::test]
    async fn test_create_and_list() {
        let client = create_client();

        let note = client
            .create_todo(TodoCreateRequest {
                content: "Test note".to_string(),
                is_checklist: false,
            })
            .await
            .unwrap();

        assert_eq!(note.content, "Test note");
        assert!(!note.done);

        let notes = client.list_todos().await.unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, note.id);
    }

    #[tokio::test]
    async fn test_update() {
        let client = create_client();

        let note = client
            .create_todo(TodoCreateRequest {
                content: "Original".to_string(),
                is_checklist: false,
            })
            .await
            .unwrap();

        let mut req = TodoUpdateRequest::default();
        req.content = Some("Updated".to_string());
        req.done = Some(true);

        let updated = client.update_todo(note.id, req).await.unwrap();

        assert_eq!(updated.content, "Updated");
        assert!(updated.done);
    }

    #[tokio::test]
    async fn test_delete() {
        let client = create_client();

        let note = client
            .create_todo(TodoCreateRequest {
                content: "To delete".to_string(),
                is_checklist: false,
            })
            .await
            .unwrap();

        client.delete_todo(note.id).await.unwrap();

        let notes = client.list_todos().await.unwrap();
        assert!(notes.is_empty());
    }

    #[tokio::test]
    async fn test_toggle_done() {
        let client = create_client();

        let note = client
            .create_todo(TodoCreateRequest {
                content: "Test".to_string(),
                is_checklist: false,
            })
            .await
            .unwrap();

        assert!(!note.done);

        let toggled = client.toggle_done(note.id).await.unwrap();
        assert!(toggled.done);

        let toggled_back = client.toggle_done(note.id).await.unwrap();
        assert!(!toggled_back.done);
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = create_client();
        assert!(client.health_check().await.unwrap());
    }
}
