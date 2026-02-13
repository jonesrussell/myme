//! Note backend: async CRUD operations for notes/todos.
//! All network work runs off the UI thread; results sent via mpsc.

use std::sync::Arc;

use myme_services::{NoteClient, Todo as Note, TodoCreateRequest, TodoUpdateRequest};

use crate::bridge;

/// Error type for note operations
#[derive(Debug, Clone)]
pub enum NoteError {
    Network(String),
    NotInitialized,
    InvalidIndex,
}

impl std::fmt::Display for NoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoteError::Network(s) => write!(f, "Network error: {}", s),
            NoteError::NotInitialized => write!(f, "Note service not initialized"),
            NoteError::InvalidIndex => write!(f, "Invalid note index"),
        }
    }
}

impl std::error::Error for NoteError {}

/// Messages sent from async operations back to the UI thread
#[derive(Debug)]
pub enum NoteServiceMessage {
    /// Result of fetching all notes
    FetchDone(Result<Vec<Note>, NoteError>),
    /// Result of creating a new note
    CreateDone(Result<Note, NoteError>),
    /// Result of updating a note (toggle done, edit content)
    UpdateDone {
        index: usize,
        result: Result<Note, NoteError>,
    },
    /// Result of deleting a note
    DeleteDone {
        index: usize,
        result: Result<(), NoteError>,
    },
}

/// Filter mode for note listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoteFilter {
    All,
    Archived,
    Pinned,
    Reminders,
    Label(String),
}

/// Request to fetch notes asynchronously.
/// Sends `FetchDone` on the channel when complete.
pub fn request_fetch(tx: &std::sync::mpsc::Sender<NoteServiceMessage>, client: Arc<NoteClient>) {
    request_fetch_with_filter(tx, client, NoteFilter::All);
}

/// Request to fetch notes with filter.
pub fn request_fetch_with_filter(
    tx: &std::sync::mpsc::Sender<NoteServiceMessage>,
    client: Arc<NoteClient>,
    filter: NoteFilter,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::FetchDone(Err(
                NoteError::NotInitialized,
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let result = match filter {
            NoteFilter::All | NoteFilter::Pinned | NoteFilter::Reminders => {
                client.list_todos().await
            }
            NoteFilter::Archived => client.list_archived().await,
            NoteFilter::Label(ref label) => client.list_by_label(label).await,
        };
        let result = result.map_err(|e| NoteError::Network(e.to_string()));
        let _ = tx.send(NoteServiceMessage::FetchDone(result));
    });
}

/// Request to create a new note asynchronously.
/// Sends `CreateDone` on the channel when complete.
pub fn request_create(
    tx: &std::sync::mpsc::Sender<NoteServiceMessage>,
    client: Arc<NoteClient>,
    content: String,
    is_checklist: bool,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::CreateDone(Err(
                NoteError::NotInitialized,
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let request = TodoCreateRequest {
            content,
            is_checklist,
        };
        let result = client
            .create_todo(request)
            .await
            .map_err(|e| NoteError::Network(e.to_string()));
        let _ = tx.send(NoteServiceMessage::CreateDone(result));
    });
}

/// Request to update a note asynchronously.
/// Sends `UpdateDone` on the channel when complete.
pub fn request_update(
    tx: &std::sync::mpsc::Sender<NoteServiceMessage>,
    client: Arc<NoteClient>,
    index: usize,
    note_id: i64,
    request: TodoUpdateRequest,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::UpdateDone {
                index,
                result: Err(NoteError::NotInitialized),
            });
            return;
        }
    };

    runtime.spawn(async move {
        let result = client
            .update_todo(note_id, request)
            .await
            .map_err(|e| NoteError::Network(e.to_string()));
        let _ = tx.send(NoteServiceMessage::UpdateDone { index, result });
    });
}

/// Request to toggle a note's done status asynchronously.
/// Sends `UpdateDone` on the channel when complete.
pub fn request_toggle_done(
    tx: &std::sync::mpsc::Sender<NoteServiceMessage>,
    client: Arc<NoteClient>,
    index: usize,
    note_id: i64,
    current_done: bool,
) {
    let mut req = TodoUpdateRequest::default();
    req.done = Some(!current_done);
    request_update(tx, client, index, note_id, req);
}

/// Request to delete a note asynchronously.
/// Sends `DeleteDone` on the channel when complete.
pub fn request_delete(
    tx: &std::sync::mpsc::Sender<NoteServiceMessage>,
    client: Arc<NoteClient>,
    index: usize,
    note_id: i64,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::DeleteDone {
                index,
                result: Err(NoteError::NotInitialized),
            });
            return;
        }
    };

    runtime.spawn(async move {
        let result = client
            .delete_todo(note_id)
            .await
            .map(|_| ())
            .map_err(|e| NoteError::Network(e.to_string()));
        let _ = tx.send(NoteServiceMessage::DeleteDone { index, result });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_error_display() {
        assert!(format!("{}", NoteError::Network("timeout".into())).contains("Network"));
        assert!(format!("{}", NoteError::NotInitialized).contains("not initialized"));
        assert!(format!("{}", NoteError::InvalidIndex).contains("Invalid"));
    }

    #[test]
    fn note_service_message_variants() {
        // Verify we can construct and match all message variants
        let _fetch_ok: NoteServiceMessage = NoteServiceMessage::FetchDone(Ok(vec![]));
        let _fetch_err: NoteServiceMessage =
            NoteServiceMessage::FetchDone(Err(NoteError::NotInitialized));
        let _create: NoteServiceMessage =
            NoteServiceMessage::CreateDone(Err(NoteError::Network("x".into())));
        let _update: NoteServiceMessage = NoteServiceMessage::UpdateDone {
            index: 0,
            result: Err(NoteError::InvalidIndex),
        };
        let _delete: NoteServiceMessage = NoteServiceMessage::DeleteDone {
            index: 1,
            result: Ok(()),
        };
    }
}
