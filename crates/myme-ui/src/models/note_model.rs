use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_services::{Todo as Note, TodoClient as NoteClient, TodoCreateRequest as NoteCreateRequest, TodoUpdateRequest as NoteUpdateRequest};

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
    error_message: QString,
    notes: Vec<Note>,
    client: Option<Arc<NoteClient>>,
    runtime: Option<tokio::runtime::Handle>,
}

impl NoteModelRust {
    pub fn initialize(&mut self, client: Arc<NoteClient>, runtime: tokio::runtime::Handle) {
        self.client = Some(client);
        self.runtime = Some(runtime);
    }

    /// Auto-initialize from global services if not already initialized
    fn ensure_initialized(&mut self) {
        if self.client.is_some() && self.runtime.is_some() {
            return;
        }

        match crate::bridge::get_todo_client_and_runtime() {
            Some((client, runtime)) => {
                self.initialize(client, runtime);
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
}

impl qobject::NoteModel {
    pub fn fetch_notes(mut self: Pin<&mut Self>) {
        // Auto-initialize if needed
        self.as_mut().rust_mut().ensure_initialized();

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut().set_error_message(QString::from("Not initialized"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        // Use block_on to make this synchronous for simplicity
        // TODO: Implement proper async callbacks with signals
        match runtime.block_on(async { client.list_todos().await }) {
            Ok(notes) => {
                tracing::info!("Successfully fetched {} notes", notes.len());
                self.as_mut().rust_mut().notes = notes;
                self.as_mut().set_loading(false);
                self.notes_changed();
            }
            Err(e) => {
                tracing::error!("Failed to fetch notes: {}", e);
                self.as_mut().set_error_message(QString::from(format!("Failed to fetch notes: {}", e)));
                self.as_mut().set_loading(false);
            }
        }
    }

    pub fn add_note(mut self: Pin<&mut Self>, content: &QString) {
        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut().set_error_message(QString::from("Not initialized"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        let content_str = content.to_string();
        self.as_mut().set_loading(true);

        let request = NoteCreateRequest { content: content_str };
        match runtime.block_on(async { client.create_todo(request).await }) {
            Ok(note) => {
                tracing::info!("Created note: {}", note.id);
                self.as_mut().rust_mut().notes.push(note);
                self.as_mut().set_loading(false);
                self.notes_changed();
            }
            Err(e) => {
                tracing::error!("Failed to create note: {}", e);
                self.as_mut().set_error_message(QString::from(format!("Failed to create note: {}", e)));
                self.as_mut().set_loading(false);
            }
        }
    }

    pub fn toggle_done(mut self: Pin<&mut Self>, index: i32) {
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

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        let index_usize = index as usize;

        let request = NoteUpdateRequest {
            content: None,
            done: Some(!current_done),
        };

        match runtime.block_on(async { client.update_todo(&note_id, request).await }) {
            Ok(_) => {
                tracing::info!("Toggled note {} done status", note_id);
                if index_usize < self.rust().notes.len() {
                    self.as_mut().rust_mut().notes[index_usize].done = !current_done;
                    self.notes_changed();
                }
            }
            Err(e) => {
                tracing::error!("Failed to update note: {}", e);
                self.as_mut().set_error_message(QString::from(format!("Failed to toggle note: {}", e)));
            }
        }
    }

    pub fn delete_note(mut self: Pin<&mut Self>, index: i32) {
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

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        let index_usize = index as usize;

        match runtime.block_on(async { client.delete_todo(&note_id).await }) {
            Ok(_) => {
                tracing::info!("Deleted note {}", note_id);
                if index_usize < self.rust().notes.len() {
                    self.as_mut().rust_mut().notes.remove(index_usize);
                    self.notes_changed();
                }
            }
            Err(e) => {
                tracing::error!("Failed to delete note: {}", e);
                self.as_mut().set_error_message(QString::from(format!("Failed to delete note: {}", e)));
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
