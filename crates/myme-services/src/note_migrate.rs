//! Migration utility for importing notes from Godo database.
//!
//! This module provides functionality to migrate notes from a Godo SQLite
//! database to a myme SQLite database.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use std::path::Path;

use crate::note_store::SqliteNoteStore;
use crate::todo::Todo;

/// Result of a migration operation.
#[derive(Debug, Clone, Default)]
pub struct MigrationResult {
    /// Number of notes successfully migrated.
    pub migrated: usize,
    /// Number of notes skipped (already exist).
    pub skipped: usize,
    /// Number of notes that failed to migrate.
    pub failed: usize,
    /// Error messages for failed notes.
    pub errors: Vec<String>,
}

impl MigrationResult {
    /// Total number of notes processed.
    pub fn total(&self) -> usize {
        self.migrated + self.skipped + self.failed
    }

    /// Check if all notes were successfully processed (none failed).
    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

impl std::fmt::Display for MigrationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Migration complete: {} migrated, {} skipped, {} failed (total: {})",
            self.migrated,
            self.skipped,
            self.failed,
            self.total()
        )
    }
}

/// Migrate notes from a Godo database to a myme SqliteNoteStore.
///
/// # Arguments
/// * `godo_path` - Path to the Godo SQLite database.
/// * `myme_store` - The myme note store to migrate into.
///
/// # Behavior
/// - Notes with existing IDs in myme are skipped (no overwrite).
/// - Original UUIDs, timestamps, and done status are preserved.
/// - Migration is idempotent - safe to run multiple times.
///
/// # Example
/// ```ignore
/// use myme_services::{SqliteNoteStore, migrate_from_godo};
///
/// let store = SqliteNoteStore::new("~/.config/myme/notes.db")?;
/// let result = migrate_from_godo("~/.config/godo/godo.db", &store)?;
/// println!("{}", result);
/// ```
pub fn migrate_from_godo<P: AsRef<Path>>(
    godo_path: P,
    myme_store: &SqliteNoteStore,
) -> Result<MigrationResult> {
    let godo_path = godo_path.as_ref();

    // Check if Godo database exists
    if !godo_path.exists() {
        return Err(anyhow::anyhow!(
            "Godo database not found: {}",
            godo_path.display()
        ));
    }

    tracing::info!("Starting migration from Godo database: {}", godo_path.display());

    // Open Godo database read-only
    let godo_conn = Connection::open_with_flags(
        godo_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .context("Failed to open Godo database")?;

    // Read all notes from Godo
    let notes = read_godo_notes(&godo_conn)?;
    tracing::info!("Found {} notes in Godo database", notes.len());

    let mut result = MigrationResult::default();

    for note in notes {
        // Check if note already exists in myme
        match myme_store.exists(&note.id) {
            Ok(true) => {
                tracing::debug!("Skipping existing note: {}", note.id);
                result.skipped += 1;
            }
            Ok(false) => {
                // Insert note
                match myme_store.insert_note(&note) {
                    Ok(()) => {
                        tracing::debug!("Migrated note: {}", note.id);
                        result.migrated += 1;
                    }
                    Err(e) => {
                        let error_msg = format!("Failed to migrate note {}: {}", note.id, e);
                        tracing::warn!("{}", error_msg);
                        result.errors.push(error_msg);
                        result.failed += 1;
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to check if note {} exists: {}", note.id, e);
                tracing::warn!("{}", error_msg);
                result.errors.push(error_msg);
                result.failed += 1;
            }
        }
    }

    tracing::info!("{}", result);
    Ok(result)
}

/// Read all notes from a Godo database connection.
fn read_godo_notes(conn: &Connection) -> Result<Vec<Todo>> {
    // Godo schema:
    // CREATE TABLE notes (
    //     id TEXT PRIMARY KEY,
    //     content TEXT NOT NULL,
    //     done INTEGER NOT NULL DEFAULT 0,
    //     created_at TEXT NOT NULL,
    //     updated_at TEXT NOT NULL
    // );

    let mut stmt = conn.prepare(
        "SELECT id, content, done, created_at, updated_at FROM notes ORDER BY created_at"
    )?;

    let rows = stmt.query_map([], |row| {
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
    })?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Failed to read notes from Godo database: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::note_backend::NoteBackend;
    use rusqlite::params;
    use tempfile::tempdir;

    fn create_godo_db(path: &Path, notes: Vec<Todo>) -> Result<()> {
        let conn = Connection::open(path)?;

        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                done INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;

        for note in notes {
            conn.execute(
                "INSERT INTO notes (id, content, done, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    note.id,
                    note.content,
                    note.done as i32,
                    note.created_at.to_rfc3339(),
                    note.updated_at.to_rfc3339(),
                ],
            )?;
        }

        Ok(())
    }

    fn create_test_note(id: &str, content: &str, done: bool) -> Todo {
        let now = Utc::now();
        Todo {
            id: id.to_string(),
            content: content.to_string(),
            done,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_migrate_from_godo() {
        let dir = tempdir().unwrap();
        let godo_path = dir.path().join("godo.db");
        let myme_path = dir.path().join("myme.db");

        // Create Godo database with test notes
        let godo_notes = vec![
            create_test_note("note-1", "First note", false),
            create_test_note("note-2", "Second note", true),
            create_test_note("note-3", "Third note", false),
        ];
        create_godo_db(&godo_path, godo_notes).unwrap();

        // Create myme store
        let myme_store = SqliteNoteStore::new(&myme_path).unwrap();

        // Run migration
        let result = migrate_from_godo(&godo_path, &myme_store).unwrap();

        assert_eq!(result.migrated, 3);
        assert_eq!(result.skipped, 0);
        assert_eq!(result.failed, 0);
        assert!(result.is_success());

        // Verify notes were migrated
        assert_eq!(myme_store.count().unwrap(), 3);

        // Check content and status preserved
        let note1 = myme_store.get("note-1").unwrap().unwrap();
        assert_eq!(note1.content, "First note");
        assert!(!note1.done);

        let note2 = myme_store.get("note-2").unwrap().unwrap();
        assert!(note2.done);
    }

    #[test]
    fn test_migrate_skips_existing() {
        let dir = tempdir().unwrap();
        let godo_path = dir.path().join("godo.db");
        let myme_path = dir.path().join("myme.db");

        // Create Godo database with test notes
        let godo_notes = vec![
            create_test_note("note-1", "First note", false),
            create_test_note("note-2", "Second note", true),
        ];
        create_godo_db(&godo_path, godo_notes).unwrap();

        // Create myme store with one existing note
        let myme_store = SqliteNoteStore::new(&myme_path).unwrap();
        let existing = create_test_note("note-1", "Already exists", false);
        myme_store.insert_note(&existing).unwrap();

        // Run migration
        let result = migrate_from_godo(&godo_path, &myme_store).unwrap();

        assert_eq!(result.migrated, 1);  // Only note-2
        assert_eq!(result.skipped, 1);   // note-1 was skipped
        assert_eq!(result.failed, 0);
        assert!(result.is_success());

        // Verify existing note was NOT overwritten
        let note1 = myme_store.get("note-1").unwrap().unwrap();
        assert_eq!(note1.content, "Already exists");  // Not "First note"
    }

    #[test]
    fn test_migrate_idempotent() {
        let dir = tempdir().unwrap();
        let godo_path = dir.path().join("godo.db");
        let myme_path = dir.path().join("myme.db");

        // Create Godo database
        let godo_notes = vec![create_test_note("note-1", "Test", false)];
        create_godo_db(&godo_path, godo_notes).unwrap();

        // Create myme store
        let myme_store = SqliteNoteStore::new(&myme_path).unwrap();

        // Run migration twice
        let result1 = migrate_from_godo(&godo_path, &myme_store).unwrap();
        let result2 = migrate_from_godo(&godo_path, &myme_store).unwrap();

        assert_eq!(result1.migrated, 1);
        assert_eq!(result1.skipped, 0);

        assert_eq!(result2.migrated, 0);
        assert_eq!(result2.skipped, 1);  // Skipped on second run

        // Still only one note
        assert_eq!(myme_store.count().unwrap(), 1);
    }

    #[test]
    fn test_migrate_nonexistent_godo_db() {
        let dir = tempdir().unwrap();
        let godo_path = dir.path().join("nonexistent.db");
        let myme_path = dir.path().join("myme.db");

        let myme_store = SqliteNoteStore::new(&myme_path).unwrap();

        let result = migrate_from_godo(&godo_path, &myme_store);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_migration_result_display() {
        let result = MigrationResult {
            migrated: 5,
            skipped: 2,
            failed: 1,
            errors: vec!["test error".to_string()],
        };

        let display = format!("{}", result);
        assert!(display.contains("5 migrated"));
        assert!(display.contains("2 skipped"));
        assert!(display.contains("1 failed"));
        assert!(display.contains("total: 8"));
    }
}
