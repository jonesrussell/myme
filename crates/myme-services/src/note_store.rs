//! SQLite-based note storage implementation.
//!
//! This module provides `SqliteNoteStore`, a local SQLite implementation of
//! the `NoteBackend` trait. The schema matches Godo's database for easy migration.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;
use uuid::Uuid;

use crate::note_backend::{
    validate_content, NoteBackend, NoteBackendError, NoteBackendResult,
};
use crate::todo::Todo;

/// SQLite-based note storage.
///
/// Stores notes in a local SQLite database with a schema compatible with Godo.
pub struct SqliteNoteStore {
    conn: Connection,
}

impl SqliteNoteStore {
    /// Create a new note store at the given path.
    ///
    /// Creates the database file and schema if they don't exist.
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Create an in-memory note store (for testing).
    #[cfg(test)]
    pub fn in_memory() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Initialize the database schema.
    ///
    /// Schema matches Godo for easy migration:
    /// - id: UUID v4 primary key
    /// - content: Note text (max 1000 chars)
    /// - done: Completion status (0/1)
    /// - created_at: RFC3339 timestamp
    /// - updated_at: RFC3339 timestamp
    fn init_schema(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                done INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at DESC);
            "#,
        )?;
        Ok(())
    }

    /// Convert a database row to a Todo.
    fn row_to_todo(row: &rusqlite::Row) -> rusqlite::Result<Todo> {
        let id: String = row.get(0)?;
        let content: String = row.get(1)?;
        let done: i32 = row.get(2)?;
        let created_at_str: String = row.get(3)?;
        let updated_at_str: String = row.get(4)?;

        // Parse timestamps, falling back to now if invalid
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        Ok(Todo {
            id,
            content,
            done: done != 0,
            created_at,
            updated_at,
        })
    }

    /// Insert a note into the database.
    ///
    /// This is used by both create and migration operations.
    pub fn insert_note(&self, note: &Todo) -> anyhow::Result<()> {
        let created_at_str = note.created_at.to_rfc3339();
        let updated_at_str = note.updated_at.to_rfc3339();

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO notes (id, content, done, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                note.id,
                note.content,
                note.done as i32,
                created_at_str,
                updated_at_str,
            ],
        )?;

        Ok(())
    }

    /// Check if a note exists by ID.
    pub fn exists(&self, id: &str) -> anyhow::Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get the note count.
    pub fn count(&self) -> anyhow::Result<usize> {
        let count: i64 =
            self.conn
                .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

impl NoteBackend for SqliteNoteStore {
    fn list(&self) -> NoteBackendResult<Vec<Todo>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, content, done, created_at, updated_at FROM notes ORDER BY created_at DESC",
            )
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        let rows = stmt
            .query_map([], Self::row_to_todo)
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| NoteBackendError::storage(e.to_string()))
    }

    fn get(&self, id: &str) -> NoteBackendResult<Option<Todo>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, content, done, created_at, updated_at FROM notes WHERE id = ?1")
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        let mut rows = stmt
            .query(params![id])
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        match rows.next().map_err(|e| NoteBackendError::storage(e.to_string()))? {
            Some(row) => Ok(Some(
                Self::row_to_todo(row).map_err(|e| NoteBackendError::storage(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    fn create(&self, content: &str) -> NoteBackendResult<Todo> {
        // Validate content
        validate_content(content)?;

        let now = Utc::now();
        let note = Todo {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            done: false,
            created_at: now,
            updated_at: now,
        };

        self.insert_note(&note)
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        tracing::debug!("Created note with ID: {}", note.id);
        Ok(note)
    }

    fn update(
        &self,
        id: &str,
        content: Option<String>,
        done: Option<bool>,
    ) -> NoteBackendResult<Todo> {
        // Get existing note
        let mut note = self
            .get(id)?
            .ok_or_else(|| NoteBackendError::not_found(id))?;

        // Update fields
        if let Some(new_content) = content {
            validate_content(&new_content)?;
            note.content = new_content;
        }

        if let Some(new_done) = done {
            note.done = new_done;
        }

        // Update timestamp
        note.updated_at = Utc::now();

        // Save to database
        let updated_at_str = note.updated_at.to_rfc3339();

        self.conn
            .execute(
                "UPDATE notes SET content = ?1, done = ?2, updated_at = ?3 WHERE id = ?4",
                params![note.content, note.done as i32, updated_at_str, id],
            )
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        tracing::debug!("Updated note: {}", id);
        Ok(note)
    }

    fn delete(&self, id: &str) -> NoteBackendResult<()> {
        // Check if note exists
        if !self
            .exists(id)
            .map_err(|e| NoteBackendError::storage(e.to_string()))?
        {
            return Err(NoteBackendError::not_found(id));
        }

        self.conn
            .execute("DELETE FROM notes WHERE id = ?1", params![id])
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        tracing::debug!("Deleted note: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_store() -> SqliteNoteStore {
        SqliteNoteStore::in_memory().expect("Failed to create in-memory store")
    }

    #[test]
    fn test_create_and_get_note() {
        let store = create_test_store();

        let note = store.create("Test note content").unwrap();
        assert!(!note.id.is_empty());
        assert_eq!(note.content, "Test note content");
        assert!(!note.done);

        let retrieved = store.get(&note.id).unwrap().unwrap();
        assert_eq!(retrieved.id, note.id);
        assert_eq!(retrieved.content, note.content);
    }

    #[test]
    fn test_list_notes() {
        let store = create_test_store();

        store.create("Note 1").unwrap();
        store.create("Note 2").unwrap();
        store.create("Note 3").unwrap();

        let notes = store.list().unwrap();
        assert_eq!(notes.len(), 3);

        // Newest first
        assert_eq!(notes[0].content, "Note 3");
        assert_eq!(notes[1].content, "Note 2");
        assert_eq!(notes[2].content, "Note 1");
    }

    #[test]
    fn test_update_content() {
        let store = create_test_store();

        let note = store.create("Original content").unwrap();
        let updated = store
            .update(&note.id, Some("Updated content".to_string()), None)
            .unwrap();

        assert_eq!(updated.content, "Updated content");
        assert!(!updated.done);
        assert!(updated.updated_at > note.updated_at);
    }

    #[test]
    fn test_update_done_status() {
        let store = create_test_store();

        let note = store.create("Test note").unwrap();
        assert!(!note.done);

        let updated = store.update(&note.id, None, Some(true)).unwrap();
        assert!(updated.done);

        let toggled = store.toggle_done(&note.id).unwrap();
        assert!(!toggled.done);
    }

    #[test]
    fn test_delete_note() {
        let store = create_test_store();

        let note = store.create("To be deleted").unwrap();
        assert!(store.get(&note.id).unwrap().is_some());

        store.delete(&note.id).unwrap();
        assert!(store.get(&note.id).unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let store = create_test_store();

        let result = store.delete("nonexistent-id");
        assert!(matches!(result, Err(NoteBackendError::NotFound(_))));
    }

    #[test]
    fn test_get_nonexistent() {
        let store = create_test_store();

        let result = store.get("nonexistent-id").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_nonexistent() {
        let store = create_test_store();

        let result = store.update("nonexistent-id", Some("content".to_string()), None);
        assert!(matches!(result, Err(NoteBackendError::NotFound(_))));
    }

    #[test]
    fn test_create_empty_content() {
        let store = create_test_store();

        let result = store.create("");
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));

        let result = store.create("   ");
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_create_content_too_long() {
        let store = create_test_store();

        let long_content = "a".repeat(1001);
        let result = store.create(&long_content);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_update_invalid_content() {
        let store = create_test_store();

        let note = store.create("Valid content").unwrap();

        // Empty content
        let result = store.update(&note.id, Some("".to_string()), None);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));

        // Too long content
        let long_content = "a".repeat(1001);
        let result = store.update(&note.id, Some(long_content), None);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_count() {
        let store = create_test_store();

        assert_eq!(store.count().unwrap(), 0);

        store.create("Note 1").unwrap();
        assert_eq!(store.count().unwrap(), 1);

        store.create("Note 2").unwrap();
        assert_eq!(store.count().unwrap(), 2);
    }

    #[test]
    fn test_mark_done_undone() {
        let store = create_test_store();

        let note = store.create("Test note").unwrap();
        assert!(!note.done);

        let done = store.mark_done(&note.id).unwrap();
        assert!(done.done);

        let undone = store.mark_undone(&note.id).unwrap();
        assert!(!undone.done);
    }

    #[test]
    fn test_insert_preserves_id_and_timestamps() {
        let store = create_test_store();

        let custom_id = "custom-uuid-123";
        let custom_time = DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let note = Todo {
            id: custom_id.to_string(),
            content: "Imported note".to_string(),
            done: true,
            created_at: custom_time,
            updated_at: custom_time,
        };

        store.insert_note(&note).unwrap();

        let retrieved = store.get(custom_id).unwrap().unwrap();
        assert_eq!(retrieved.id, custom_id);
        assert_eq!(retrieved.content, "Imported note");
        assert!(retrieved.done);
        assert_eq!(retrieved.created_at, custom_time);
    }
}
