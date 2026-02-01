use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::Todo as Note;

use crate::bridge;
use crate::services::{
    request_create, request_delete, request_fetch, request_health_check, request_toggle,
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
        #[qproperty(bool, connected)]
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

        #[qinvokable]
        fn poll_channel(self: Pin<&mut NoteModel>);

        #[qinvokable]
        fn check_connection(self: Pin<&mut NoteModel>);

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
    }
}

#[derive(Default)]
pub struct NoteModelRust {
    loading: bool,
    connected: bool,
    error_message: QString,
    notes: Vec<Note>,
    channel_initialized: bool,
}

impl NoteModelRust {
    /// Ensure the service channel is initialized
    fn ensure_channel(&mut self) {
        if !self.channel_initialized {
            bridge::init_note_service_channel();
            self.channel_initialized = true;
            tracing::info!("NoteModel channel initialized");
        }
    }

    /// Get note at index if valid, returns None for invalid indices
    fn get_note(&self, index: i32) -> Option<&Note> {
        if index < 0 {
            return None;
        }
        self.notes.get(index as usize)
    }
}

impl qobject::NoteModel {
    /// Fetch notes from the Godo API (async, non-blocking)
    pub fn fetch_notes(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_channel();

        let tx = match bridge::get_note_service_tx() {
            Some(tx) => tx,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service not initialized"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        request_fetch(&tx);
        tracing::debug!("Requested note fetch");
    }

    /// Add a new note (async, non-blocking)
    pub fn add_note(mut self: Pin<&mut Self>, content: &QString) {
        self.as_mut().rust_mut().ensure_channel();

        let tx = match bridge::get_note_service_tx() {
            Some(tx) => tx,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service not initialized"));
                return;
            }
        };

        let content_str = content.to_string();
        if content_str.trim().is_empty() {
            return;
        }

        self.as_mut().set_loading(true);
        request_create(&tx, content_str);
        tracing::debug!("Requested note create");
    }

    /// Toggle done status for a note (async, non-blocking)
    pub fn toggle_done(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().rust_mut().ensure_channel();

        let tx = match bridge::get_note_service_tx() {
            Some(tx) => tx,
            None => return,
        };

        let binding = self.as_ref();
        let notes = &binding.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return;
        }

        let note_id = notes[index as usize].id.clone();
        let new_done = !notes[index as usize].done;

        request_toggle(&tx, index as usize, note_id, new_done);
        tracing::debug!("Requested toggle for index {}", index);
    }

    /// Delete a note (async, non-blocking)
    pub fn delete_note(mut self: Pin<&mut Self>, index: i32) {
        self.as_mut().rust_mut().ensure_channel();

        let tx = match bridge::get_note_service_tx() {
            Some(tx) => tx,
            None => return,
        };

        let binding = self.as_ref();
        let notes = &binding.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return;
        }

        let note_id = notes[index as usize].id.clone();
        self.as_mut().set_loading(true);

        request_delete(&tx, index as usize, note_id);
        tracing::debug!("Requested delete for index {}", index);
    }

    /// Check API connectivity (async, non-blocking)
    pub fn check_connection(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_channel();

        let tx = match bridge::get_note_service_tx() {
            Some(tx) => tx,
            None => {
                self.as_mut().set_connected(false);
                return;
            }
        };

        request_health_check(&tx);
        tracing::debug!("Requested health check");
    }

    /// Poll the message channel for async results.
    /// Call this from a QML Timer (e.g., every 100ms).
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        // Process all available messages
        while let Some(msg) = bridge::try_recv_note_message() {
            match msg {
                NoteServiceMessage::FetchDone(result) => {
                    self.as_mut().set_loading(false);
                    match result {
                        Ok(notes) => {
                            tracing::info!("Received {} notes", notes.len());
                            self.as_mut().rust_mut().notes = notes;
                            self.as_mut().set_connected(true);
                            self.as_mut().set_error_message(QString::from(""));
                            self.as_mut().notes_changed();
                        }
                        Err(e) => {
                            tracing::error!("Fetch failed: {}", e);
                            self.as_mut().set_error_message(QString::from(format!(
                                "Failed to fetch notes: {}",
                                e
                            )));
                            self.as_mut().set_connected(false);
                        }
                    }
                }
                NoteServiceMessage::CreateDone(result) => {
                    self.as_mut().set_loading(false);
                    match result {
                        Ok(note) => {
                            tracing::info!("Created note: {}", note.id);
                            self.as_mut().rust_mut().notes.push(note);
                            self.as_mut().notes_changed();
                        }
                        Err(e) => {
                            tracing::error!("Create failed: {}", e);
                            self.as_mut().set_error_message(QString::from(format!(
                                "Failed to create note: {}",
                                e
                            )));
                        }
                    }
                }
                NoteServiceMessage::ToggleDone { index, result } => match result {
                    Ok(note) => {
                        tracing::info!("Toggled note at index {}", index);
                        if index < self.rust().notes.len() {
                            self.as_mut().rust_mut().notes[index].done = note.done;
                            self.as_mut().notes_changed();
                        }
                    }
                    Err(e) => {
                        tracing::error!("Toggle failed: {}", e);
                        self.as_mut().set_error_message(QString::from(format!(
                            "Failed to toggle note: {}",
                            e
                        )));
                    }
                },
                NoteServiceMessage::DeleteDone { index, result } => {
                    self.as_mut().set_loading(false);
                    match result {
                        Ok(()) => {
                            tracing::info!("Deleted note at index {}", index);
                            if index < self.rust().notes.len() {
                                self.as_mut().rust_mut().notes.remove(index);
                                self.as_mut().notes_changed();
                            }
                        }
                        Err(e) => {
                            tracing::error!("Delete failed: {}", e);
                            self.as_mut().set_error_message(QString::from(format!(
                                "Failed to delete note: {}",
                                e
                            )));
                        }
                    }
                }
                NoteServiceMessage::HealthCheckDone(healthy) => {
                    tracing::debug!("Health check result: {}", healthy);
                    self.as_mut().set_connected(healthy);
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
