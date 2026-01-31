use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::{Todo as Note, TodoClient as NoteClient};

use crate::bridge;
use crate::services::{
    request_note_create, request_note_delete, request_note_fetch, request_note_toggle,
    NoteServiceMessage,
};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(QString, error_message)]
        type NoteModel = super::NoteModelRust;

        #[qinvokable]
        fn fetch_notes(self: Pin<&mut NoteModel>);

        #[qinvokable]
        fn add_note(self: Pin<&mut NoteModel>, content: &QString);

        #[qinvokable]
        fn toggle_done(self: Pin<&mut NoteModel>, index: i32);

        #[qinvokable]
        fn delete_note(self: Pin<&mut NoteModel>, index: i32);

        /// Poll for async operation results. Call this from a QML Timer.
        #[qinvokable]
        fn poll_channel(self: Pin<&mut NoteModel>);

        #[qinvokable]
        fn row_count(self: &NoteModel) -> i32;

        #[qinvokable]
        fn get_content(self: &NoteModel, index: i32) -> QString;

        #[qinvokable]
        fn get_done(self: &NoteModel, index: i32) -> bool;

        #[qinvokable]
        fn get_id(self: &NoteModel, index: i32) -> QString;

        #[qinvokable]
        fn get_created_at(self: &NoteModel, index: i32) -> QString;

        #[qsignal]
        fn notes_changed(self: Pin<&mut NoteModel>);

        #[qsignal]
        fn error_occurred(self: Pin<&mut NoteModel>);
    }
}

/// Operation state tracking to prevent concurrent operations
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum OpState {
    #[default]
    Idle,
    Fetching,
    Creating,
    Updating(usize),
    Deleting(usize),
}

#[derive(Default)]
pub struct NoteModelRust {
    loading: bool,
    error_message: QString,
    notes: Vec<Note>,
    client: Option<Arc<NoteClient>>,
    op_state: OpState,
}

impl NoteModelRust {
    pub fn initialize(&mut self, client: Arc<NoteClient>) {
        self.client = Some(client);
    }

    /// Auto-initialize from global services if not already initialized
    fn ensure_initialized(&mut self) {
        if self.client.is_some() {
            return;
        }

        match crate::bridge::get_todo_client_and_runtime() {
            Some((client, _runtime)) => {
                self.initialize(client);
                tracing::info!("NoteModel auto-initialized from global services");
            }
            None => {
                tracing::error!("Cannot auto-initialize NoteModel - global services not ready");
            }
        }
    }

    /// Get note at index if valid, returns None for invalid indices
    fn get_note(&self, index: i32) -> Option<&Note> {
        if index < 0 {
            return None;
        }
        self.notes.get(index as usize)
    }

    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }
}

impl qobject::NoteModel {
    /// Fetch all notes asynchronously (non-blocking)
    pub fn fetch_notes(mut self: Pin<&mut Self>) {
        // Auto-initialize if needed
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("fetch_notes: operation already in progress");
            return;
        }

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Not initialized"));
                self.as_mut().error_occurred();
                return;
            }
        };

        // Initialize channel if needed
        bridge::init_note_service_channel();
        let tx = match bridge::get_note_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                self.as_mut().error_occurred();
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::Fetching;

        // Spawn async operation (non-blocking)
        request_note_fetch(&tx, client);
    }

    /// Add a new note asynchronously (non-blocking)
    pub fn add_note(mut self: Pin<&mut Self>, content: &QString) {
        // Auto-initialize if needed
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("add_note: operation already in progress");
            return;
        }

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Not initialized"));
                self.as_mut().error_occurred();
                return;
            }
        };

        // Initialize channel if needed
        bridge::init_note_service_channel();
        let tx = match bridge::get_note_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                self.as_mut().error_occurred();
                return;
            }
        };

        let content_str = content.to_string();
        if content_str.is_empty() {
            return;
        }

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::Creating;

        // Spawn async operation (non-blocking)
        request_note_create(&tx, client, content_str);
    }

    /// Toggle a note's done status asynchronously (non-blocking)
    pub fn toggle_done(mut self: Pin<&mut Self>, index: i32) {
        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("toggle_done: operation already in progress");
            return;
        }

        let binding = self.as_ref();
        let notes = &binding.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return;
        }

        let note_id = notes[index as usize].id.clone();
        let current_done = notes[index as usize].done;

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return,
        };

        // Initialize channel if needed
        bridge::init_note_service_channel();
        let tx = match bridge::get_note_service_tx() {
            Some(t) => t,
            None => return,
        };

        let index_usize = index as usize;
        self.as_mut().rust_mut().op_state = OpState::Updating(index_usize);

        // Spawn async operation (non-blocking)
        request_note_toggle(&tx, client, index_usize, note_id, current_done);
    }

    /// Delete a note asynchronously (non-blocking)
    pub fn delete_note(mut self: Pin<&mut Self>, index: i32) {
        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("delete_note: operation already in progress");
            return;
        }

        let binding = self.as_ref();
        let notes = &binding.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return;
        }

        let note_id = notes[index as usize].id.clone();

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return,
        };

        // Initialize channel if needed
        bridge::init_note_service_channel();
        let tx = match bridge::get_note_service_tx() {
            Some(t) => t,
            None => return,
        };

        let index_usize = index as usize;
        self.as_mut().rust_mut().op_state = OpState::Deleting(index_usize);

        // Spawn async operation (non-blocking)
        request_note_delete(&tx, client, index_usize, note_id);
    }

    /// Poll for async operation results. Call this from a QML Timer (e.g., every 100ms).
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_note_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            NoteServiceMessage::FetchDone(result) => {
                self.as_mut().set_loading(false);
                self.as_mut().rust_mut().op_state = OpState::Idle;
                match result {
                    Ok(notes) => {
                        tracing::info!("Successfully fetched {} notes", notes.len());
                        self.as_mut().rust_mut().clear_error();
                        self.as_mut().rust_mut().notes = notes;
                        self.as_mut().notes_changed();
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch notes: {}", e);
                        self.as_mut().rust_mut().set_error(&format!("Failed to fetch notes: {}", e));
                        self.as_mut().error_occurred();
                    }
                }
            }
            NoteServiceMessage::CreateDone(result) => {
                self.as_mut().set_loading(false);
                self.as_mut().rust_mut().op_state = OpState::Idle;
                match result {
                    Ok(note) => {
                        tracing::info!("Created note: {}", note.id);
                        self.as_mut().rust_mut().clear_error();
                        self.as_mut().rust_mut().notes.push(note);
                        self.as_mut().notes_changed();
                    }
                    Err(e) => {
                        tracing::error!("Failed to create note: {}", e);
                        self.as_mut().rust_mut().set_error(&format!("Failed to create note: {}", e));
                        self.as_mut().error_occurred();
                    }
                }
            }
            NoteServiceMessage::UpdateDone { index, result } => {
                self.as_mut().rust_mut().op_state = OpState::Idle;
                match result {
                    Ok(updated_note) => {
                        tracing::info!("Updated note at index {}", index);
                        self.as_mut().rust_mut().clear_error();
                        if index < self.as_ref().rust().notes.len() {
                            self.as_mut().rust_mut().notes[index] = updated_note;
                            self.as_mut().notes_changed();
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to update note: {}", e);
                        self.as_mut().rust_mut().set_error(&format!("Failed to toggle note: {}", e));
                        self.as_mut().error_occurred();
                    }
                }
            }
            NoteServiceMessage::DeleteDone { index, result } => {
                self.as_mut().rust_mut().op_state = OpState::Idle;
                match result {
                    Ok(_) => {
                        tracing::info!("Deleted note at index {}", index);
                        self.as_mut().rust_mut().clear_error();
                        if index < self.as_ref().rust().notes.len() {
                            self.as_mut().rust_mut().notes.remove(index);
                            self.as_mut().notes_changed();
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete note: {}", e);
                        self.as_mut().rust_mut().set_error(&format!("Failed to delete note: {}", e));
                        self.as_mut().error_occurred();
                    }
                }
            }
        }
    }

    pub fn row_count(&self) -> i32 {
        self.rust().notes.len() as i32
    }

    pub fn get_content(&self, index: i32) -> QString {
        self.rust()
            .get_note(index)
            .map(|note| QString::from(&note.content))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_done(&self, index: i32) -> bool {
        self.rust()
            .get_note(index)
            .map(|note| note.done)
            .unwrap_or(false)
    }

    pub fn get_id(&self, index: i32) -> QString {
        self.rust()
            .get_note(index)
            .map(|note| QString::from(&note.id))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_created_at(&self, index: i32) -> QString {
        self.rust()
            .get_note(index)
            .map(|note| QString::from(note.created_at.format("%Y-%m-%d %H:%M").to_string()))
            .unwrap_or_else(|| QString::from(""))
    }
}
