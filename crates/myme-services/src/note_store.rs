//! SQLite-based note storage implementation.
//!
//! This module provides `SqliteNoteStore`, a local SQLite implementation of
//! the `NoteBackend` trait. Schema supports Keep-style notes with color, pin, archive, labels, checklists, reminders.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::path::Path;

use crate::note_backend::{validate_content, NoteBackend, NoteBackendError, NoteBackendResult};
use crate::todo::{Todo, TodoUpdateRequest};

/// SQLite-based note storage.
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
    /// Detects old schema (TEXT id or missing pinned column) and migrates by
    /// dropping the table and recreating. Data loss is acceptable (single test note).
    fn init_schema(&self) -> anyhow::Result<()> {
        let needs_migration = self.detect_old_schema()?;

        if needs_migration {
            self.conn
                .execute_batch("DROP TABLE IF EXISTS notes;")
                .map_err(|e| anyhow::anyhow!("Failed to drop notes table: {}", e))?;
        }

        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS notes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                done INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                color TEXT,
                pinned INTEGER NOT NULL DEFAULT 0,
                archived INTEGER NOT NULL DEFAULT 0,
                labels TEXT NOT NULL DEFAULT '[]',
                is_checklist INTEGER NOT NULL DEFAULT 0,
                reminder TEXT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_notes_archived ON notes(archived);
            CREATE INDEX IF NOT EXISTS idx_notes_pinned_updated ON notes(pinned DESC, updated_at DESC);
            "#,
        )?;
        Ok(())
    }

    /// Detect if we have the old schema (TEXT id or missing pinned column).
    fn detect_old_schema(&self) -> anyhow::Result<bool> {
        let table_exists: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='notes'",
            [],
            |row| row.get(0),
        )?;
        if table_exists == 0 {
            return Ok(false);
        }

        let table_info: Vec<(String, String)> = self
            .conn
            .prepare("PRAGMA table_info(notes)")?
            .query_map([], |row| Ok((row.get::<_, String>(1)?, row.get::<_, String>(2)?)))?
            .collect::<Result<Vec<_>, _>>()?;

        let has_pinned = table_info.iter().any(|(name, _)| name == "pinned");
        let id_type =
            table_info.iter().find(|(name, _)| name == "id").map(|(_, t)| t.as_str()).unwrap_or("");

        Ok(!has_pinned || id_type == "TEXT")
    }

    /// Convert a database row to a Todo.
    fn row_to_todo(row: &rusqlite::Row) -> rusqlite::Result<Todo> {
        let id: i64 = row.get(0)?;
        let content: String = row.get(1)?;
        let done: i32 = row.get(2)?;
        let created_at_str: String = row.get(3)?;
        let updated_at_str: String = row.get(4)?;
        let color: Option<String> = row.get(5)?;
        let pinned: i32 = row.get(6)?;
        let archived: i32 = row.get(7)?;
        let labels_str: String = row.get(8)?;
        let is_checklist: i32 = row.get(9)?;
        let reminder_str: Option<String> = row.get(10)?;

        let created_at = DateTime::parse_from_rfc3339(&created_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());
        let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now());

        let labels: Vec<String> = serde_json::from_str(&labels_str).unwrap_or_default();

        let reminder = reminder_str
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));

        Ok(Todo {
            id,
            content,
            done: done != 0,
            created_at,
            updated_at,
            color,
            pinned: pinned != 0,
            archived: archived != 0,
            labels,
            is_checklist: is_checklist != 0,
            reminder,
        })
    }

    /// Check if a note exists by ID.
    pub fn exists(&self, id: i64) -> anyhow::Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM notes WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Get the note count.
    pub fn count(&self) -> anyhow::Result<usize> {
        let count: i64 = self.conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

impl NoteBackend for SqliteNoteStore {
    fn list(&self) -> NoteBackendResult<Vec<Todo>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, content, done, created_at, updated_at, color, pinned, archived, labels, is_checklist, reminder
                 FROM notes
                 WHERE archived = 0
                 ORDER BY pinned DESC, updated_at DESC",
            )
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        let rows = stmt
            .query_map([], Self::row_to_todo)
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| NoteBackendError::storage(e.to_string()))
    }

    fn list_archived(&self) -> NoteBackendResult<Vec<Todo>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, content, done, created_at, updated_at, color, pinned, archived, labels, is_checklist, reminder
                 FROM notes
                 WHERE archived = 1
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        let rows = stmt
            .query_map([], Self::row_to_todo)
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(|e| NoteBackendError::storage(e.to_string()))
    }

    fn list_by_label(&self, label: &str) -> NoteBackendResult<Vec<Todo>> {
        let notes = self.list()?;
        Ok(notes.into_iter().filter(|n| n.labels.iter().any(|l| l == label)).collect())
    }

    fn list_with_reminders(&self) -> NoteBackendResult<Vec<Todo>> {
        let notes = self.list()?;
        Ok(notes.into_iter().filter(|n| n.reminder.is_some()).collect())
    }

    fn get(&self, id: i64) -> NoteBackendResult<Option<Todo>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, content, done, created_at, updated_at, color, pinned, archived, labels, is_checklist, reminder
                 FROM notes WHERE id = ?1",
            )
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        let mut rows =
            stmt.query(params![id]).map_err(|e| NoteBackendError::storage(e.to_string()))?;

        match rows.next().map_err(|e| NoteBackendError::storage(e.to_string()))? {
            Some(row) => Ok(Some(
                Self::row_to_todo(&row).map_err(|e| NoteBackendError::storage(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    fn create(&self, content: &str, is_checklist: bool) -> NoteBackendResult<Todo> {
        validate_content(content)?;

        let now = Utc::now();
        let created_at_str = now.to_rfc3339();
        let updated_at_str = now.to_rfc3339();

        self.conn
            .execute(
                r#"
                INSERT INTO notes (content, done, created_at, updated_at, color, pinned, archived, labels, is_checklist, reminder)
                VALUES (?1, 0, ?2, ?3, NULL, 0, 0, '[]', ?4, NULL)
                "#,
                params![content, created_at_str, updated_at_str, is_checklist as i32],
            )
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        let id = self.conn.last_insert_rowid();
        tracing::debug!("Created note with ID: {}", id);

        Ok(Todo {
            id,
            content: content.to_string(),
            done: false,
            created_at: now,
            updated_at: now,
            color: None,
            pinned: false,
            archived: false,
            labels: vec![],
            is_checklist,
            reminder: None,
        })
    }

    fn update(&self, id: i64, request: TodoUpdateRequest) -> NoteBackendResult<Todo> {
        let mut note = self.get(id)?.ok_or_else(|| NoteBackendError::not_found(id.to_string()))?;

        if let Some(ref new_content) = request.content {
            validate_content(new_content)?;
            note.content = new_content.clone();
        }
        if let Some(color_opt) = request.color {
            note.color = color_opt;
        }
        if let Some(pinned) = request.pinned {
            note.pinned = pinned;
        }
        if let Some(archived) = request.archived {
            note.archived = archived;
        }
        if let Some(labels) = request.labels {
            note.labels = labels;
        }
        if let Some(is_checklist) = request.is_checklist {
            note.is_checklist = is_checklist;
        }
        if let Some(reminder_opt) = request.reminder {
            note.reminder = reminder_opt;
        }
        if let Some(done) = request.done {
            note.done = done;
        }

        note.updated_at = Utc::now();
        let updated_at_str = note.updated_at.to_rfc3339();
        let labels_str = serde_json::to_string(&note.labels).unwrap_or_else(|_| "[]".to_string());
        let reminder_str = note.reminder.map(|dt| dt.to_rfc3339());

        self.conn
            .execute(
                r#"
                UPDATE notes
                SET content = ?1, done = ?2, updated_at = ?3, color = ?4, pinned = ?5, archived = ?6, labels = ?7, is_checklist = ?8, reminder = ?9
                WHERE id = ?10
                "#,
                params![
                    note.content,
                    note.done as i32,
                    updated_at_str,
                    note.color,
                    note.pinned as i32,
                    note.archived as i32,
                    labels_str,
                    note.is_checklist as i32,
                    reminder_str,
                    id,
                ],
            )
            .map_err(|e| NoteBackendError::storage(e.to_string()))?;

        tracing::debug!("Updated note: {}", id);
        Ok(note)
    }

    fn delete(&self, id: i64) -> NoteBackendResult<()> {
        if !self.exists(id).map_err(|e| NoteBackendError::storage(e.to_string()))? {
            return Err(NoteBackendError::not_found(id.to_string()));
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
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    fn create_test_store() -> SqliteNoteStore {
        SqliteNoteStore::in_memory().expect("Failed to create in-memory store")
    }

    #[test]
    fn test_create_and_get_note() {
        let store = create_test_store();

        let note = store.create("Test note content", false).unwrap();
        assert!(note.id > 0);
        assert_eq!(note.content, "Test note content");
        assert!(!note.done);

        let retrieved = store.get(note.id).unwrap().unwrap();
        assert_eq!(retrieved.id, note.id);
        assert_eq!(retrieved.content, note.content);
    }

    #[test]
    fn test_list_notes() {
        let store = create_test_store();

        store.create("Note 1", false).unwrap();
        store.create("Note 2", false).unwrap();
        store.create("Note 3", false).unwrap();

        let notes = store.list().unwrap();
        assert_eq!(notes.len(), 3);

        assert_eq!(notes[0].content, "Note 3");
        assert_eq!(notes[1].content, "Note 2");
        assert_eq!(notes[2].content, "Note 1");
    }

    #[test]
    fn test_update_content() {
        let store = create_test_store();

        let note = store.create("Original content", false).unwrap();
        let mut req = TodoUpdateRequest::default();
        req.content = Some("Updated content".to_string());
        let updated = store.update(note.id, req).unwrap();

        assert_eq!(updated.content, "Updated content");
        assert!(!updated.done);
        assert!(updated.updated_at > note.updated_at);
    }

    #[test]
    fn test_update_done_status() {
        let store = create_test_store();

        let note = store.create("Test note", false).unwrap();
        assert!(!note.done);

        let mut req = TodoUpdateRequest::default();
        req.done = Some(true);
        let updated = store.update(note.id, req).unwrap();
        assert!(updated.done);

        let toggled = store.toggle_done(note.id).unwrap();
        assert!(!toggled.done);
    }

    #[test]
    fn test_delete_note() {
        let store = create_test_store();

        let note = store.create("To be deleted", false).unwrap();
        assert!(store.get(note.id).unwrap().is_some());

        store.delete(note.id).unwrap();
        assert!(store.get(note.id).unwrap().is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let store = create_test_store();

        let result = store.delete(99999);
        assert!(matches!(result, Err(NoteBackendError::NotFound(_))));
    }

    #[test]
    fn test_get_nonexistent() {
        let store = create_test_store();

        let result = store.get(99999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_nonexistent() {
        let store = create_test_store();

        let mut req = TodoUpdateRequest::default();
        req.content = Some("content".to_string());
        let result = store.update(99999, req);
        assert!(matches!(result, Err(NoteBackendError::NotFound(_))));
    }

    #[test]
    fn test_create_empty_content() {
        let store = create_test_store();

        let result = store.create("", false);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));

        let result = store.create("   ", false);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_create_content_too_long() {
        let store = create_test_store();

        let long_content = "a".repeat(1001);
        let result = store.create(&long_content, false);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_update_invalid_content() {
        let store = create_test_store();

        let note = store.create("Valid content", false).unwrap();

        let mut req = TodoUpdateRequest::default();
        req.content = Some("".to_string());
        let result = store.update(note.id, req);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));

        let long_content = "a".repeat(1001);
        let mut req2 = TodoUpdateRequest::default();
        req2.content = Some(long_content);
        let result = store.update(note.id, req2);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_count() {
        let store = create_test_store();

        assert_eq!(store.count().unwrap(), 0);

        store.create("Note 1", false).unwrap();
        assert_eq!(store.count().unwrap(), 1);

        store.create("Note 2", false).unwrap();
        assert_eq!(store.count().unwrap(), 2);
    }

    #[test]
    fn test_mark_done_undone() {
        let store = create_test_store();

        let note = store.create("Test note", false).unwrap();
        assert!(!note.done);

        let done = store.mark_done(note.id).unwrap();
        assert!(done.done);

        let undone = store.mark_undone(note.id).unwrap();
        assert!(!undone.done);
    }

    #[test]
    fn test_create_checklist_note() {
        let store = create_test_store();

        let note = store.create("- [ ] item", true).unwrap();
        assert!(note.is_checklist);
    }
}
