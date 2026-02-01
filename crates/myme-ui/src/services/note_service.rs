//! Note service: async API calls with channel-based responses.
//! All HTTP requests run off the UI thread; results sent via mpsc.

use myme_services::{Todo as Note, TodoCreateRequest, TodoUpdateRequest};

use crate::bridge;

#[derive(Debug, Clone)]
pub enum NoteError {
    Api(String),
    Config(String),
}

impl std::fmt::Display for NoteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NoteError::Api(s) => write!(f, "API: {}", s),
            NoteError::Config(s) => write!(f, "Config: {}", s),
        }
    }
}

#[derive(Debug)]
pub enum NoteServiceMessage {
    FetchDone(Result<Vec<Note>, NoteError>),
    CreateDone(Result<Note, NoteError>),
    ToggleDone {
        index: usize,
        result: Result<Note, NoteError>,
    },
    DeleteDone {
        index: usize,
        result: Result<(), NoteError>,
    },
    HealthCheckDone(bool),
}

/// Request to fetch all notes from Godo API.
/// Sends `FetchDone` on the channel when complete.
pub fn request_fetch(tx: &std::sync::mpsc::Sender<NoteServiceMessage>) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::FetchDone(Err(NoteError::Config(
                "Runtime not initialized".into(),
            ))));
            return;
        }
    };

    let (client, _) = match bridge::get_todo_client_and_runtime() {
        Some(c) => c,
        None => {
            let _ = tx.send(NoteServiceMessage::FetchDone(Err(NoteError::Config(
                "TodoClient not initialized".into(),
            ))));
            return;
        }
    };

    runtime.spawn(async move {
        match client.list_todos().await {
            Ok(notes) => {
                tracing::info!("Fetched {} notes", notes.len());
                let _ = tx.send(NoteServiceMessage::FetchDone(Ok(notes)));
            }
            Err(e) => {
                tracing::error!("Failed to fetch notes: {}", e);
                let _ = tx.send(NoteServiceMessage::FetchDone(Err(NoteError::Api(
                    e.to_string(),
                ))));
            }
        }
    });
}

/// Request to create a new note.
/// Sends `CreateDone` on the channel when complete.
pub fn request_create(tx: &std::sync::mpsc::Sender<NoteServiceMessage>, content: String) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::CreateDone(Err(NoteError::Config(
                "Runtime not initialized".into(),
            ))));
            return;
        }
    };

    let (client, _) = match bridge::get_todo_client_and_runtime() {
        Some(c) => c,
        None => {
            let _ = tx.send(NoteServiceMessage::CreateDone(Err(NoteError::Config(
                "TodoClient not initialized".into(),
            ))));
            return;
        }
    };

    runtime.spawn(async move {
        let request = TodoCreateRequest { content };
        match client.create_todo(request).await {
            Ok(note) => {
                tracing::info!("Created note: {}", note.id);
                let _ = tx.send(NoteServiceMessage::CreateDone(Ok(note)));
            }
            Err(e) => {
                tracing::error!("Failed to create note: {}", e);
                let _ = tx.send(NoteServiceMessage::CreateDone(Err(NoteError::Api(
                    e.to_string(),
                ))));
            }
        }
    });
}

/// Request to toggle done status for a note.
/// Sends `ToggleDone { index, result }` on the channel when complete.
pub fn request_toggle(
    tx: &std::sync::mpsc::Sender<NoteServiceMessage>,
    index: usize,
    note_id: String,
    new_done: bool,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::ToggleDone {
                index,
                result: Err(NoteError::Config("Runtime not initialized".into())),
            });
            return;
        }
    };

    let (client, _) = match bridge::get_todo_client_and_runtime() {
        Some(c) => c,
        None => {
            let _ = tx.send(NoteServiceMessage::ToggleDone {
                index,
                result: Err(NoteError::Config("TodoClient not initialized".into())),
            });
            return;
        }
    };

    runtime.spawn(async move {
        let request = TodoUpdateRequest {
            content: None,
            done: Some(new_done),
        };
        match client.update_todo(&note_id, request).await {
            Ok(note) => {
                tracing::info!("Toggled note {} to done={}", note_id, new_done);
                let _ = tx.send(NoteServiceMessage::ToggleDone {
                    index,
                    result: Ok(note),
                });
            }
            Err(e) => {
                tracing::error!("Failed to toggle note {}: {}", note_id, e);
                let _ = tx.send(NoteServiceMessage::ToggleDone {
                    index,
                    result: Err(NoteError::Api(e.to_string())),
                });
            }
        }
    });
}

/// Request to delete a note.
/// Sends `DeleteDone { index, result }` on the channel when complete.
pub fn request_delete(
    tx: &std::sync::mpsc::Sender<NoteServiceMessage>,
    index: usize,
    note_id: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::DeleteDone {
                index,
                result: Err(NoteError::Config("Runtime not initialized".into())),
            });
            return;
        }
    };

    let (client, _) = match bridge::get_todo_client_and_runtime() {
        Some(c) => c,
        None => {
            let _ = tx.send(NoteServiceMessage::DeleteDone {
                index,
                result: Err(NoteError::Config("TodoClient not initialized".into())),
            });
            return;
        }
    };

    runtime.spawn(async move {
        match client.delete_todo(&note_id).await {
            Ok(()) => {
                tracing::info!("Deleted note {}", note_id);
                let _ = tx.send(NoteServiceMessage::DeleteDone {
                    index,
                    result: Ok(()),
                });
            }
            Err(e) => {
                tracing::error!("Failed to delete note {}: {}", note_id, e);
                let _ = tx.send(NoteServiceMessage::DeleteDone {
                    index,
                    result: Err(NoteError::Api(e.to_string())),
                });
            }
        }
    });
}

/// Request a health check to verify API connectivity.
/// Sends `HealthCheckDone(bool)` on the channel when complete.
pub fn request_health_check(tx: &std::sync::mpsc::Sender<NoteServiceMessage>) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(NoteServiceMessage::HealthCheckDone(false));
            return;
        }
    };

    let (client, _) = match bridge::get_todo_client_and_runtime() {
        Some(c) => c,
        None => {
            let _ = tx.send(NoteServiceMessage::HealthCheckDone(false));
            return;
        }
    };

    runtime.spawn(async move {
        match client.health_check().await {
            Ok(healthy) => {
                tracing::debug!("Health check result: {}", healthy);
                let _ = tx.send(NoteServiceMessage::HealthCheckDone(healthy));
            }
            Err(e) => {
                tracing::warn!("Health check failed: {}", e);
                let _ = tx.send(NoteServiceMessage::HealthCheckDone(false));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_error_display() {
        assert!(format!("{}", NoteError::Api("x".into())).contains("API"));
        assert!(format!("{}", NoteError::Config("y".into())).contains("Config"));
    }

    #[test]
    fn note_service_message_variants() {
        // Verify we can construct and match all message variants
        let _fetch_ok: NoteServiceMessage = NoteServiceMessage::FetchDone(Ok(vec![]));
        let _fetch_err: NoteServiceMessage =
            NoteServiceMessage::FetchDone(Err(NoteError::Config("x".into())));
        let _create_err: NoteServiceMessage =
            NoteServiceMessage::CreateDone(Err(NoteError::Api("e".into())));
        let _toggle: NoteServiceMessage = NoteServiceMessage::ToggleDone {
            index: 0,
            result: Err(NoteError::Api("e".into())),
        };
        let _delete: NoteServiceMessage = NoteServiceMessage::DeleteDone {
            index: 1,
            result: Ok(()),
        };
        let _health: NoteServiceMessage = NoteServiceMessage::HealthCheckDone(true);
    }
}
