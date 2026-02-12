//! Offline action queue for Gmail operations.
//!
//! Stores pending actions when offline and syncs them when connectivity is restored.

use anyhow::Result;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Types of actions that can be queued for offline sync.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncAction {
    /// Mark a message as read
    MarkRead { message_id: String },
    /// Mark a message as unread
    MarkUnread { message_id: String },
    /// Star a message
    Star { message_id: String },
    /// Unstar a message
    Unstar { message_id: String },
    /// Archive a message (remove from INBOX)
    Archive { message_id: String },
    /// Move message to trash
    Trash { message_id: String },
    /// Add labels to a message
    AddLabels {
        message_id: String,
        labels: Vec<String>,
    },
    /// Remove labels from a message
    RemoveLabels {
        message_id: String,
        labels: Vec<String>,
    },
}

/// A queued action with metadata.
#[derive(Debug, Clone)]
pub struct QueuedAction {
    pub id: i64,
    pub action: SyncAction,
    pub created_at: i64,
    pub attempts: u32,
    pub last_error: Option<String>,
}

/// Sync queue backed by SQLite.
pub struct SyncQueue {
    conn: Connection,
}

impl SyncQueue {
    /// Create a new sync queue at the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let queue = Self { conn };
        queue.init_schema()?;
        Ok(queue)
    }

    /// Create an in-memory queue (for testing).
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let queue = Self { conn };
        queue.init_schema()?;
        Ok(queue)
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sync_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                action_json TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                attempts INTEGER NOT NULL DEFAULT 0,
                last_error TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_sync_queue_created ON sync_queue(created_at);
            "#,
        )?;
        Ok(())
    }

    /// Add an action to the queue.
    pub fn enqueue(&self, action: SyncAction) -> Result<i64> {
        let action_json = serde_json::to_string(&action)?;
        let now = chrono::Utc::now().timestamp_millis();

        self.conn.execute(
            "INSERT INTO sync_queue (action_json, created_at) VALUES (?1, ?2)",
            params![action_json, now],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get the next pending action (FIFO order).
    pub fn peek(&self) -> Result<Option<QueuedAction>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, action_json, created_at, attempts, last_error FROM sync_queue ORDER BY created_at ASC LIMIT 1"
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let action_json: String = row.get(1)?;
            let action: SyncAction = serde_json::from_str(&action_json)?;

            Ok(Some(QueuedAction {
                id: row.get(0)?,
                action,
                created_at: row.get(2)?,
                attempts: row.get(3)?,
                last_error: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all pending actions.
    pub fn list_pending(&self) -> Result<Vec<QueuedAction>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, action_json, created_at, attempts, last_error FROM sync_queue ORDER BY created_at ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            let action_json: String = row.get(1)?;
            let action: SyncAction = serde_json::from_str(&action_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            Ok(QueuedAction {
                id: row.get(0)?,
                action,
                created_at: row.get(2)?,
                attempts: row.get(3)?,
                last_error: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to read queue: {}", e))
    }

    /// Mark an action as completed and remove it from the queue.
    pub fn complete(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM sync_queue WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Record a failed attempt for an action.
    pub fn record_failure(&self, id: i64, error: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE sync_queue SET attempts = attempts + 1, last_error = ?1 WHERE id = ?2",
            params![error, id],
        )?;
        Ok(())
    }

    /// Remove actions that have exceeded max attempts.
    pub fn remove_failed(&self, max_attempts: u32) -> Result<u32> {
        let count = self.conn.execute(
            "DELETE FROM sync_queue WHERE attempts >= ?1",
            params![max_attempts],
        )?;
        Ok(count as u32)
    }

    /// Get the number of pending actions.
    pub fn pending_count(&self) -> Result<u32> {
        let count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM sync_queue", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Clear all pending actions.
    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM sync_queue", [])?;
        Ok(())
    }

    /// Check if there are any pending actions for a specific message.
    pub fn has_pending_for_message(&self, message_id: &str) -> Result<bool> {
        let pattern = format!("%\"message_id\":\"{}\"%", message_id);
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM sync_queue WHERE action_json LIKE ?1",
            params![pattern],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_and_peek() {
        let queue = SyncQueue::in_memory().unwrap();

        let action = SyncAction::MarkRead {
            message_id: "msg1".to_string(),
        };
        let id = queue.enqueue(action.clone()).unwrap();

        let queued = queue.peek().unwrap().unwrap();
        assert_eq!(queued.id, id);
        assert_eq!(queued.action, action);
        assert_eq!(queued.attempts, 0);
    }

    #[test]
    fn test_fifo_order() {
        let queue = SyncQueue::in_memory().unwrap();

        queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg1".to_string(),
            })
            .unwrap();
        queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg2".to_string(),
            })
            .unwrap();
        queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg3".to_string(),
            })
            .unwrap();

        let first = queue.peek().unwrap().unwrap();
        assert!(
            matches!(first.action, SyncAction::MarkRead { message_id } if message_id == "msg1")
        );
    }

    #[test]
    fn test_complete_removes_action() {
        let queue = SyncQueue::in_memory().unwrap();

        let id = queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg1".to_string(),
            })
            .unwrap();
        assert_eq!(queue.pending_count().unwrap(), 1);

        queue.complete(id).unwrap();
        assert_eq!(queue.pending_count().unwrap(), 0);
    }

    #[test]
    fn test_record_failure() {
        let queue = SyncQueue::in_memory().unwrap();

        let id = queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg1".to_string(),
            })
            .unwrap();

        queue.record_failure(id, "Network error").unwrap();
        queue.record_failure(id, "Timeout").unwrap();

        let queued = queue.peek().unwrap().unwrap();
        assert_eq!(queued.attempts, 2);
        assert_eq!(queued.last_error, Some("Timeout".to_string()));
    }

    #[test]
    fn test_remove_failed() {
        let queue = SyncQueue::in_memory().unwrap();

        let id1 = queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg1".to_string(),
            })
            .unwrap();
        let _id2 = queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg2".to_string(),
            })
            .unwrap();

        // Fail id1 three times
        queue.record_failure(id1, "error").unwrap();
        queue.record_failure(id1, "error").unwrap();
        queue.record_failure(id1, "error").unwrap();

        let removed = queue.remove_failed(3).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(queue.pending_count().unwrap(), 1);
    }

    #[test]
    fn test_list_pending() {
        let queue = SyncQueue::in_memory().unwrap();

        queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg1".to_string(),
            })
            .unwrap();
        queue
            .enqueue(SyncAction::Star {
                message_id: "msg2".to_string(),
            })
            .unwrap();
        queue
            .enqueue(SyncAction::Trash {
                message_id: "msg3".to_string(),
            })
            .unwrap();

        let pending = queue.list_pending().unwrap();
        assert_eq!(pending.len(), 3);
    }

    #[test]
    fn test_has_pending_for_message() {
        let queue = SyncQueue::in_memory().unwrap();

        queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg1".to_string(),
            })
            .unwrap();
        queue
            .enqueue(SyncAction::Star {
                message_id: "msg2".to_string(),
            })
            .unwrap();

        assert!(queue.has_pending_for_message("msg1").unwrap());
        assert!(queue.has_pending_for_message("msg2").unwrap());
        assert!(!queue.has_pending_for_message("msg3").unwrap());
    }

    #[test]
    fn test_clear() {
        let queue = SyncQueue::in_memory().unwrap();

        queue
            .enqueue(SyncAction::MarkRead {
                message_id: "msg1".to_string(),
            })
            .unwrap();
        queue
            .enqueue(SyncAction::Star {
                message_id: "msg2".to_string(),
            })
            .unwrap();

        queue.clear().unwrap();
        assert_eq!(queue.pending_count().unwrap(), 0);
    }

    #[test]
    fn test_action_serialization() {
        // Test that all action types serialize/deserialize correctly
        let actions = vec![
            SyncAction::MarkRead {
                message_id: "m1".to_string(),
            },
            SyncAction::MarkUnread {
                message_id: "m2".to_string(),
            },
            SyncAction::Star {
                message_id: "m3".to_string(),
            },
            SyncAction::Unstar {
                message_id: "m4".to_string(),
            },
            SyncAction::Archive {
                message_id: "m5".to_string(),
            },
            SyncAction::Trash {
                message_id: "m6".to_string(),
            },
            SyncAction::AddLabels {
                message_id: "m7".to_string(),
                labels: vec!["Label1".to_string()],
            },
            SyncAction::RemoveLabels {
                message_id: "m8".to_string(),
                labels: vec!["Label2".to_string()],
            },
        ];

        let queue = SyncQueue::in_memory().unwrap();

        for action in actions {
            let id = queue.enqueue(action.clone()).unwrap();
            let queued = queue.peek().unwrap().unwrap();
            assert_eq!(queued.action, action);
            queue.complete(id).unwrap();
        }
    }
}
