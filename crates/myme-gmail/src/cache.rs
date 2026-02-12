//! SQLite-based offline cache for Gmail messages and labels.

use anyhow::Result;
use rusqlite::{params, Connection};
use std::path::Path;

use crate::types::{Label, LabelType, Message};

/// SQLite cache for Gmail data.
pub struct GmailCache {
    conn: Connection,
}

impl GmailCache {
    /// Create a new cache at the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let cache = Self { conn };
        cache.init_schema()?;
        Ok(cache)
    }

    /// Create an in-memory cache (for testing).
    #[cfg(test)]
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let cache = Self { conn };
        cache.init_schema()?;
        Ok(cache)
    }

    /// Initialize the database schema.
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                thread_id TEXT NOT NULL,
                from_addr TEXT NOT NULL,
                to_addrs TEXT NOT NULL,
                subject TEXT NOT NULL,
                snippet TEXT NOT NULL,
                date_ms INTEGER NOT NULL,
                labels TEXT NOT NULL,
                is_unread INTEGER NOT NULL,
                is_starred INTEGER NOT NULL,
                body TEXT,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS labels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                label_type TEXT NOT NULL,
                messages_total INTEGER,
                messages_unread INTEGER,
                cached_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_messages_date ON messages(date_ms DESC);
            CREATE INDEX IF NOT EXISTS idx_messages_thread ON messages(thread_id);
            CREATE INDEX IF NOT EXISTS idx_messages_unread ON messages(is_unread);
            "#,
        )?;
        Ok(())
    }

    /// Store a message in the cache.
    pub fn store_message(&self, msg: &Message) -> Result<()> {
        let to_json = serde_json::to_string(&msg.to)?;
        let labels_json = serde_json::to_string(&msg.labels)?;
        let now = chrono::Utc::now().timestamp_millis();

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO messages
            (id, thread_id, from_addr, to_addrs, subject, snippet, date_ms, labels, is_unread, is_starred, body, cached_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                msg.id,
                msg.thread_id,
                msg.from,
                to_json,
                msg.subject,
                msg.snippet,
                msg.date.timestamp_millis(),
                labels_json,
                msg.is_unread as i32,
                msg.is_starred as i32,
                msg.body,
                now,
            ],
        )?;
        Ok(())
    }

    /// Get a message from the cache by ID.
    pub fn get_message(&self, id: &str) -> Result<Option<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, thread_id, from_addr, to_addrs, subject, snippet, date_ms, labels, is_unread, is_starred, body FROM messages WHERE id = ?1"
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Self::row_to_message(row)?))
        } else {
            Ok(None)
        }
    }

    /// List messages from cache, optionally filtered by label.
    pub fn list_messages(&self, label: Option<&str>, limit: u32) -> Result<Vec<Message>> {
        let sql = if label.is_some() {
            "SELECT id, thread_id, from_addr, to_addrs, subject, snippet, date_ms, labels, is_unread, is_starred, body
             FROM messages
             WHERE labels LIKE ?1
             ORDER BY date_ms DESC
             LIMIT ?2"
        } else {
            "SELECT id, thread_id, from_addr, to_addrs, subject, snippet, date_ms, labels, is_unread, is_starred, body
             FROM messages
             ORDER BY date_ms DESC
             LIMIT ?2"
        };

        let mut stmt = self.conn.prepare(sql)?;

        let rows = if let Some(lbl) = label {
            let pattern = format!("%\"{}\"%", lbl);
            stmt.query_map(params![pattern, limit], Self::row_to_message)?
        } else {
            stmt.query_map(params!["", limit], Self::row_to_message)?
        };

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to read messages: {}", e))
    }

    /// Delete a message from the cache.
    pub fn delete_message(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM messages WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Store a label in the cache.
    pub fn store_label(&self, label: &Label) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        let label_type = match label.label_type {
            LabelType::System => "system",
            LabelType::User => "user",
        };

        self.conn.execute(
            r#"
            INSERT OR REPLACE INTO labels
            (id, name, label_type, messages_total, messages_unread, cached_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                label.id,
                label.name,
                label_type,
                label.messages_total,
                label.messages_unread,
                now,
            ],
        )?;
        Ok(())
    }

    /// List all labels from cache.
    pub fn list_labels(&self) -> Result<Vec<Label>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, label_type, messages_total, messages_unread FROM labels ORDER BY name"
        )?;

        let rows = stmt.query_map([], |row| {
            let label_type_str: String = row.get(2)?;
            Ok(Label {
                id: row.get(0)?,
                name: row.get(1)?,
                label_type: if label_type_str == "system" {
                    LabelType::System
                } else {
                    LabelType::User
                },
                messages_total: row.get(3)?,
                messages_unread: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to read labels: {}", e))
    }

    /// Get unread message count.
    pub fn unread_count(&self) -> Result<u32> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE is_unread = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    /// Get the last sync timestamp.
    pub fn get_last_sync(&self) -> Result<Option<i64>> {
        let result: Result<i64, _> = self.conn.query_row(
            "SELECT value FROM sync_state WHERE key = 'last_sync'",
            [],
            |row| row.get::<_, String>(0).map(|s| s.parse().unwrap_or(0)),
        );
        match result {
            Ok(ts) => Ok(Some(ts)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Set the last sync timestamp.
    pub fn set_last_sync(&self, timestamp: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO sync_state (key, value) VALUES ('last_sync', ?1)",
            params![timestamp.to_string()],
        )?;
        Ok(())
    }

    /// Clear all cached data.
    pub fn clear(&self) -> Result<()> {
        self.conn
            .execute_batch("DELETE FROM messages; DELETE FROM labels; DELETE FROM sync_state;")?;
        Ok(())
    }

    fn row_to_message(row: &rusqlite::Row) -> rusqlite::Result<Message> {
        let to_json: String = row.get(3)?;
        let labels_json: String = row.get(7)?;
        let date_ms: i64 = row.get(6)?;

        Ok(Message {
            id: row.get(0)?,
            thread_id: row.get(1)?,
            from: row.get(2)?,
            to: serde_json::from_str(&to_json).unwrap_or_default(),
            subject: row.get(4)?,
            snippet: row.get(5)?,
            date: chrono::DateTime::from_timestamp_millis(date_ms).unwrap_or_default(),
            labels: serde_json::from_str(&labels_json).unwrap_or_default(),
            is_unread: row.get::<_, i32>(8)? != 0,
            is_starred: row.get::<_, i32>(9)? != 0,
            body: row.get(10)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_message(id: &str, is_unread: bool) -> Message {
        Message {
            id: id.to_string(),
            thread_id: format!("thread_{}", id),
            from: "sender@example.com".to_string(),
            to: vec!["me@example.com".to_string()],
            subject: format!("Test Subject {}", id),
            snippet: "Test snippet...".to_string(),
            date: Utc::now(),
            labels: vec!["INBOX".to_string()],
            is_unread,
            is_starred: false,
            body: Some("Test body".to_string()),
        }
    }

    #[test]
    fn test_store_and_get_message() {
        let cache = GmailCache::in_memory().unwrap();
        let msg = create_test_message("msg1", true);

        cache.store_message(&msg).unwrap();
        let retrieved = cache.get_message("msg1").unwrap().unwrap();

        assert_eq!(retrieved.id, "msg1");
        assert_eq!(retrieved.subject, "Test Subject msg1");
        assert!(retrieved.is_unread);
    }

    #[test]
    fn test_message_not_found() {
        let cache = GmailCache::in_memory().unwrap();
        let result = cache.get_message("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_messages() {
        let cache = GmailCache::in_memory().unwrap();

        cache
            .store_message(&create_test_message("msg1", true))
            .unwrap();
        cache
            .store_message(&create_test_message("msg2", false))
            .unwrap();
        cache
            .store_message(&create_test_message("msg3", true))
            .unwrap();

        let messages = cache.list_messages(None, 10).unwrap();
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_delete_message() {
        let cache = GmailCache::in_memory().unwrap();
        let msg = create_test_message("msg1", true);

        cache.store_message(&msg).unwrap();
        assert!(cache.get_message("msg1").unwrap().is_some());

        cache.delete_message("msg1").unwrap();
        assert!(cache.get_message("msg1").unwrap().is_none());
    }

    #[test]
    fn test_unread_count() {
        let cache = GmailCache::in_memory().unwrap();

        cache
            .store_message(&create_test_message("msg1", true))
            .unwrap();
        cache
            .store_message(&create_test_message("msg2", false))
            .unwrap();
        cache
            .store_message(&create_test_message("msg3", true))
            .unwrap();

        assert_eq!(cache.unread_count().unwrap(), 2);
    }

    #[test]
    fn test_store_and_list_labels() {
        let cache = GmailCache::in_memory().unwrap();

        let label1 = Label {
            id: "INBOX".to_string(),
            name: "Inbox".to_string(),
            label_type: LabelType::System,
            messages_total: Some(100),
            messages_unread: Some(5),
        };
        let label2 = Label {
            id: "Label_1".to_string(),
            name: "Work".to_string(),
            label_type: LabelType::User,
            messages_total: Some(50),
            messages_unread: None,
        };

        cache.store_label(&label1).unwrap();
        cache.store_label(&label2).unwrap();

        let labels = cache.list_labels().unwrap();
        assert_eq!(labels.len(), 2);
    }

    #[test]
    fn test_sync_state() {
        let cache = GmailCache::in_memory().unwrap();

        assert!(cache.get_last_sync().unwrap().is_none());

        let now = Utc::now().timestamp_millis();
        cache.set_last_sync(now).unwrap();

        assert_eq!(cache.get_last_sync().unwrap(), Some(now));
    }

    #[test]
    fn test_clear_cache() {
        let cache = GmailCache::in_memory().unwrap();

        cache
            .store_message(&create_test_message("msg1", true))
            .unwrap();
        cache.set_last_sync(12345).unwrap();

        cache.clear().unwrap();

        assert!(cache.get_message("msg1").unwrap().is_none());
        assert!(cache.get_last_sync().unwrap().is_none());
    }

    #[test]
    fn test_update_existing_message() {
        let cache = GmailCache::in_memory().unwrap();

        let mut msg = create_test_message("msg1", true);
        cache.store_message(&msg).unwrap();

        // Update the message
        msg.is_unread = false;
        msg.is_starred = true;
        cache.store_message(&msg).unwrap();

        let retrieved = cache.get_message("msg1").unwrap().unwrap();
        assert!(!retrieved.is_unread);
        assert!(retrieved.is_starred);
    }
}
