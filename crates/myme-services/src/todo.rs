//! Note (todo) types used by SQLite note storage and the unified NoteClient.
//! HTTP/Godo client has been removed; notes are SQLite-only.

use serde::{Deserialize, Serialize};

/// A single note (todo item).
/// Same shape as used by SQLite store and previously by Godo API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub id: String,
    pub content: String,
    pub done: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a new note.
#[derive(Debug, Clone, Serialize)]
pub struct TodoCreateRequest {
    pub content: String,
}

/// Request to update an existing note (partial update).
#[derive(Debug, Clone, Serialize)]
pub struct TodoUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_serialization() {
        let todo = Todo {
            id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
            content: "Test note".to_string(),
            done: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&todo).unwrap();
        assert!(json.contains("Test note"));
        assert!(json.contains("\"done\":false"));
    }

    #[test]
    fn test_create_request_serialization() {
        let req = TodoCreateRequest {
            content: "New note".to_string(),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"content":"New note"}"#);
    }

    #[test]
    fn test_update_request_partial() {
        let req = TodoUpdateRequest {
            content: None,
            done: Some(true),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert_eq!(json, r#"{"done":true}"#);
    }
}
