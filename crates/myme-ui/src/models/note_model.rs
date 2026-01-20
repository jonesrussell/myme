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
}

impl qobject::NoteModel {
    pub fn fetch_notes(mut self: Pin<&mut Self>) {
        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut().set_error_message(&QString::from("Not initialized"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(&QString::from(""));

        runtime.spawn(async move {
            match client.list_todos().await {
                Ok(notes) => {
                    tracing::info!("Successfully fetched {} notes", notes.len());
                }
                Err(e) => {
                    tracing::error!("Failed to fetch notes: {}", e);
                }
            }
        });
    }

    pub fn add_note(mut self: Pin<&mut Self>, content: &QString) {
        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => {
                self.as_mut().set_error_message(&QString::from("Not initialized"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        let content_str = content.to_string();
        self.as_mut().set_loading(true);

        runtime.spawn(async move {
            let request = NoteCreateRequest { content: content_str };

            match client.create_todo(request).await {
                Ok(note) => {
                    tracing::info!("Created note: {}", note.id);
                }
                Err(e) => {
                    tracing::error!("Failed to create note: {}", e);
                }
            }
        });
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

        runtime.spawn(async move {
            let request = NoteUpdateRequest {
                content: None,
                done: Some(!current_done),
            };

            match client.update_todo(&note_id, request).await {
                Ok(_) => {
                    tracing::info!("Toggled note {} done status", note_id);
                }
                Err(e) => {
                    tracing::error!("Failed to update note: {}", e);
                }
            }
        });
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

        runtime.spawn(async move {
            match client.delete_todo(&note_id).await {
                Ok(_) => {
                    tracing::info!("Deleted note {}", note_id);
                }
                Err(e) => {
                    tracing::error!("Failed to delete note: {}", e);
                }
            }
        });
    }

    pub fn row_count(&self) -> i32 {
        self.rust().notes.len() as i32
    }

    pub fn get_content(&self, index: i32) -> QString {
        let notes = &self.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return QString::from("");
        }
        QString::from(&notes[index as usize].content)
    }

    pub fn get_done(&self, index: i32) -> bool {
        let notes = &self.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return false;
        }
        notes[index as usize].done
    }

    pub fn get_id(&self, index: i32) -> QString {
        let notes = &self.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return QString::from("");
        }
        QString::from(&notes[index as usize].id)
    }

    pub fn get_created_at(&self, index: i32) -> QString {
        let notes = &self.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return QString::from("");
        }
        QString::from(notes[index as usize].created_at.format("%Y-%m-%d %H:%M").to_string())
    }
}
