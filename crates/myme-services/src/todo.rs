//! Note (todo) types used by SQLite note storage and the unified NoteClient.
//! HTTP/Godo client has been removed; notes are SQLite-only.

use serde::{Deserialize, Serialize};

/// A single note (todo item).
/// Same shape as used by SQLite store with Keep-style extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: i64,
    pub content: String,
    pub done: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    pub color: Option<String>,
    pub pinned: bool,
    pub archived: bool,
    pub labels: Vec<String>,
    pub is_checklist: bool,
    pub reminder: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request to create a new note.
#[derive(Debug, Clone, Serialize)]
pub struct TodoCreateRequest {
    pub content: String,
    pub is_checklist: bool,
}

/// Request to update an existing note (partial update).
/// Use Option<Option<T>> for fields that can be "cleared" (Some(None)) vs "don't touch" (None).
#[derive(Debug, Clone, Serialize, Default)]
pub struct TodoUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Option<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_checklist: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reminder: Option<Option<chrono::DateTime<chrono::Utc>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_todo_serialization() {
        let todo = Todo {
            id: 1,
            content: "Test note".to_string(),
            done: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            color: None,
            pinned: false,
            archived: false,
            labels: vec![],
            is_checklist: false,
            reminder: None,
        };

        let json = serde_json::to_string(&todo).unwrap();
        assert!(json.contains("Test note"));
        assert!(json.contains("\"done\":false"));
    }

    #[test]
    fn test_create_request_serialization() {
        let req = TodoCreateRequest { content: "New note".to_string(), is_checklist: false };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("New note"));
        assert!(json.contains("is_checklist"));
    }

    #[test]
    fn test_update_request_partial() {
        let req = TodoUpdateRequest {
            content: None,
            color: None,
            pinned: None,
            archived: None,
            labels: None,
            is_checklist: None,
            reminder: None,
            done: Some(true),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"done":true}"#);
    }
}
