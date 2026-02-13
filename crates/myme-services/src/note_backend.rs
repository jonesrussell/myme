//! Note storage backend trait and error types.
//!
//! This module defines the `NoteBackend` trait that abstracts over different
//! storage implementations (SQLite, HTTP API).

use crate::todo::{Todo, TodoUpdateRequest};
use thiserror::Error;

/// Errors that can occur during note backend operations.
#[derive(Debug, Error)]
pub enum NoteBackendError {
    /// Note was not found.
    #[error("Note not found: {0}")]
    NotFound(String),

    /// Validation error (e.g., empty content, content too long).
    #[error("Validation error: {0}")]
    Validation(String),

    /// Storage error (database, network, etc.).
    #[error("Storage error: {0}")]
    Storage(String),

    /// Generic error wrapper.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl NoteBackendError {
    /// Create a not found error.
    pub fn not_found(id: impl Into<String>) -> Self {
        Self::NotFound(id.into())
    }

    /// Create a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    /// Create a storage error.
    pub fn storage(message: impl Into<String>) -> Self {
        Self::Storage(message.into())
    }
}

/// Result type for note backend operations.
pub type NoteBackendResult<T> = Result<T, NoteBackendError>;

/// Trait for note storage backends.
///
/// This trait abstracts over different storage implementations (SQLite, HTTP API)
/// and provides a common interface for note CRUD operations.
///
/// Note: Implementations don't need to be Sync - the NoteClient wrapper handles
/// thread-safe access via Mutex.
pub trait NoteBackend: Send {
    /// List all non-archived notes.
    ///
    /// Returns notes ordered by pinned DESC, updated_at DESC.
    fn list(&self) -> NoteBackendResult<Vec<Todo>>;

    /// List archived notes.
    fn list_archived(&self) -> NoteBackendResult<Vec<Todo>>;

    /// Get a note by ID.
    ///
    /// Returns `None` if the note doesn't exist.
    fn get(&self, id: i64) -> NoteBackendResult<Option<Todo>>;

    /// Create a new note.
    ///
    /// # Arguments
    /// * `content` - The note content (1-1000 characters).
    /// * `is_checklist` - Whether the note is a checklist.
    ///
    /// # Errors
    /// Returns `NoteBackendError::Validation` if content is empty or too long.
    fn create(&self, content: &str, is_checklist: bool) -> NoteBackendResult<Todo>;

    /// Update an existing note.
    ///
    /// # Arguments
    /// * `id` - The note ID.
    /// * `request` - Partial update request; only Some fields are applied.
    ///
    /// # Errors
    /// Returns `NoteBackendError::NotFound` if the note doesn't exist.
    /// Returns `NoteBackendError::Validation` if content is invalid.
    fn update(&self, id: i64, request: TodoUpdateRequest) -> NoteBackendResult<Todo>;

    /// Delete a note.
    ///
    /// # Errors
    /// Returns `NoteBackendError::NotFound` if the note doesn't exist.
    fn delete(&self, id: i64) -> NoteBackendResult<()>;

    /// Toggle the done status of a note.
    ///
    /// Default implementation fetches the note and updates with inverted done status.
    fn toggle_done(&self, id: i64) -> NoteBackendResult<Todo> {
        let note = self
            .get(id)?
            .ok_or_else(|| NoteBackendError::not_found(id.to_string()))?;
        let mut req = TodoUpdateRequest::default();
        req.done = Some(!note.done);
        self.update(id, req)
    }

    /// Mark a note as done.
    fn mark_done(&self, id: i64) -> NoteBackendResult<Todo> {
        let mut req = TodoUpdateRequest::default();
        req.done = Some(true);
        self.update(id, req)
    }

    /// Mark a note as not done.
    fn mark_undone(&self, id: i64) -> NoteBackendResult<Todo> {
        let mut req = TodoUpdateRequest::default();
        req.done = Some(false);
        self.update(id, req)
    }
}

/// Maximum content length for notes (matches Godo validation).
pub const MAX_CONTENT_LENGTH: usize = 1000;

/// Validate note content.
///
/// # Errors
/// Returns `NoteBackendError::Validation` if:
/// - Content is empty or whitespace-only.
/// - Content exceeds `MAX_CONTENT_LENGTH` characters.
pub fn validate_content(content: &str) -> NoteBackendResult<()> {
    let trimmed = content.trim();

    if trimmed.is_empty() {
        return Err(NoteBackendError::validation("Content cannot be empty"));
    }

    if content.len() > MAX_CONTENT_LENGTH {
        return Err(NoteBackendError::validation(format!(
            "Content exceeds maximum length of {} characters",
            MAX_CONTENT_LENGTH
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_content_empty() {
        let result = validate_content("");
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_validate_content_whitespace() {
        let result = validate_content("   ");
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_validate_content_too_long() {
        let long_content = "a".repeat(MAX_CONTENT_LENGTH + 1);
        let result = validate_content(&long_content);
        assert!(matches!(result, Err(NoteBackendError::Validation(_))));
    }

    #[test]
    fn test_validate_content_max_length() {
        let max_content = "a".repeat(MAX_CONTENT_LENGTH);
        let result = validate_content(&max_content);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_content_valid() {
        let result = validate_content("Valid note content");
        assert!(result.is_ok());
    }
}
