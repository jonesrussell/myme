use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::{QString, QStringList};
use myme_services::{NoteClient, Todo as Note, TodoUpdateRequest};

use crate::bridge;
use crate::services::{
    request_note_create, request_note_delete, request_note_fetch_with_filter,
    request_note_toggle, request_note_update, NoteServiceFilter as ServiceFilter,
    NoteServiceMessage,
};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        include!("cxx-qt-lib/qstringlist.h");
        type QString = cxx_qt_lib::QString;
        type QStringList = cxx_qt_lib::QStringList;
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
        fn add_note_checklist(self: Pin<&mut NoteModel>, content: &QString);

        #[qinvokable]
        fn toggle_done(self: Pin<&mut NoteModel>, index: i32);

        #[qinvokable]
        fn delete_note(self: Pin<&mut NoteModel>, index: i32);

        #[qinvokable]
        fn update_content(self: Pin<&mut NoteModel>, index: i32, content: &QString);

        #[qinvokable]
        fn set_color(self: Pin<&mut NoteModel>, index: i32, color: &QString);

        #[qinvokable]
        fn set_pinned(self: Pin<&mut NoteModel>, index: i32, pinned: bool);

        #[qinvokable]
        fn archive_note(self: Pin<&mut NoteModel>, index: i32);

        #[qinvokable]
        fn unarchive_note(self: Pin<&mut NoteModel>, index: i32);

        #[qinvokable]
        fn set_reminder(self: Pin<&mut NoteModel>, index: i32, iso: &QString);

        #[qinvokable]
        fn set_filter(self: Pin<&mut NoteModel>, filter: &QString);

        #[qinvokable]
        fn add_label(self: Pin<&mut NoteModel>, index: i32, label: &QString);

        #[qinvokable]
        fn remove_label(self: Pin<&mut NoteModel>, index: i32, label: &QString);

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

        #[qinvokable]
        fn get_color(self: &NoteModel, index: i32) -> QString;

        #[qinvokable]
        fn get_pinned(self: &NoteModel, index: i32) -> bool;

        #[qinvokable]
        fn get_archived(self: &NoteModel, index: i32) -> bool;

        #[qinvokable]
        fn get_labels(self: &NoteModel, index: i32) -> QStringList;

        #[qinvokable]
        fn get_is_checklist(self: &NoteModel, index: i32) -> bool;

        #[qinvokable]
        fn get_reminder(self: &NoteModel, index: i32) -> QString;

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

#[derive(Clone, PartialEq, Eq)]
enum NoteFilter {
    All,
    Archived,
    Label(String),
}

impl Default for NoteFilter {
    fn default() -> Self {
        NoteFilter::All
    }
}

#[derive(Default)]
pub struct NoteModelRust {
    loading: bool,
    connected: bool,
    error_message: QString,
    notes: Vec<Note>,
    client: Option<Arc<NoteClient>>,
    op_state: OpState,
    filter: NoteFilter,
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

        match crate::bridge::get_note_client_and_runtime() {
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

        let service_filter = match &self.as_ref().rust().filter {
            NoteFilter::All => ServiceFilter::All,
            NoteFilter::Archived => ServiceFilter::Archived,
            NoteFilter::Label(label) => ServiceFilter::Label(label.clone()),
        };
        request_note_fetch_with_filter(&tx, client, service_filter);
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
        request_note_create(&tx, client, content_str, false);
    }

    /// Add a new checklist note asynchronously (non-blocking)
    pub fn add_note_checklist(mut self: Pin<&mut Self>, content: &QString) {
        self.as_mut().rust_mut().ensure_initialized();
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            return;
        }
        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return,
        };
        bridge::init_note_service_channel();
        let tx = match bridge::get_note_service_tx() {
            Some(t) => t,
            None => return,
        };
        let content_str = content.to_string();
        if content_str.is_empty() {
            return;
        }
        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::Creating;
        request_note_create(&tx, client, content_str, true);
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

    /// Helper: send update request for note at index
    fn send_update(mut self: Pin<&mut Self>, index: i32, req: TodoUpdateRequest) -> bool {
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            return false;
        }
        let binding = self.as_ref();
        let notes = &binding.rust().notes;
        if index < 0 || index >= notes.len() as i32 {
            return false;
        }
        let note_id = notes[index as usize].id;
        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return false,
        };
        bridge::init_note_service_channel();
        let tx = match bridge::get_note_service_tx() {
            Some(t) => t,
            None => return false,
        };
        let index_usize = index as usize;
        self.as_mut().rust_mut().op_state = OpState::Updating(index_usize);
        request_note_update(&tx, client, index_usize, note_id, req);
        true
    }

    /// Update note content asynchronously
    pub fn update_content(mut self: Pin<&mut Self>, index: i32, content: &QString) {
        let content_str = content.to_string();
        let mut req = TodoUpdateRequest::default();
        req.content = Some(content_str);
        self.as_mut().send_update(index, req);
    }

    /// Set note color (empty string = clear)
    pub fn set_color(mut self: Pin<&mut Self>, index: i32, color: &QString) {
        let color_str = color.to_string();
        let mut req = TodoUpdateRequest::default();
        req.color = Some(if color_str.trim().is_empty() {
            None
        } else {
            Some(color_str)
        });
        self.as_mut().send_update(index, req);
    }

    /// Set pinned status
    pub fn set_pinned(mut self: Pin<&mut Self>, index: i32, pinned: bool) {
        let mut req = TodoUpdateRequest::default();
        req.pinned = Some(pinned);
        self.as_mut().send_update(index, req);
    }

    /// Archive a note
    pub fn archive_note(mut self: Pin<&mut Self>, index: i32) {
        let mut req = TodoUpdateRequest::default();
        req.archived = Some(true);
        self.as_mut().send_update(index, req);
    }

    /// Unarchive a note
    pub fn unarchive_note(mut self: Pin<&mut Self>, index: i32) {
        let mut req = TodoUpdateRequest::default();
        req.archived = Some(false);
        self.as_mut().send_update(index, req);
    }

    /// Set reminder (empty string = clear)
    pub fn set_reminder(mut self: Pin<&mut Self>, index: i32, iso: &QString) {
        let iso_str = iso.to_string();
        let mut req = TodoUpdateRequest::default();
        req.reminder = Some(if iso_str.trim().is_empty() {
            None
        } else {
            chrono::DateTime::parse_from_rfc3339(&iso_str)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
        self.as_mut().send_update(index, req);
    }

    /// Set filter and refetch
    /// Add a label to a note
    pub fn add_label(mut self: Pin<&mut Self>, index: i32, label: &QString) {
        let label_str = label.to_string().trim().to_string();
        if label_str.is_empty() {
            return;
        }
        let note = match self.rust().get_note(index) {
            Some(n) => n.clone(),
            None => return,
        };
        if note.labels.contains(&label_str) {
            return;
        }
        let mut labels = note.labels.clone();
        labels.push(label_str);
        let mut req = TodoUpdateRequest::default();
        req.labels = Some(labels);
        self.as_mut().send_update(index, req);
    }

    /// Remove a label from a note
    pub fn remove_label(mut self: Pin<&mut Self>, index: i32, label: &QString) {
        let label_str = label.to_string();
        let note = match self.rust().get_note(index) {
            Some(n) => n.clone(),
            None => return,
        };
        let labels: Vec<String> = note.labels.into_iter().filter(|l| l != &label_str).collect();
        let mut req = TodoUpdateRequest::default();
        req.labels = Some(labels);
        self.as_mut().send_update(index, req);
    }

    pub fn set_filter(mut self: Pin<&mut Self>, filter: &QString) {
        let f = filter.to_string();
        let new_filter = if f.starts_with("label:") {
            NoteFilter::Label(f.strip_prefix("label:").unwrap_or("").to_string())
        } else {
            match f.as_str() {
                "archived" => NoteFilter::Archived,
                _ => NoteFilter::All,
            }
        };
        self.as_mut().rust_mut().filter = new_filter.clone();

        let client = match &self.as_ref().rust().client {
            Some(c) => c.clone(),
            None => return,
        };
        bridge::init_note_service_channel();
        let tx = match bridge::get_note_service_tx() {
            Some(t) => t,
            None => return,
        };
        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().op_state = OpState::Fetching;
        let service_filter = match &new_filter {
            NoteFilter::All => ServiceFilter::All,
            NoteFilter::Archived => ServiceFilter::Archived,
            NoteFilter::Label(label) => ServiceFilter::Label(label.clone()),
        };
        request_note_fetch_with_filter(&tx, client, service_filter);
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
                        self.as_mut().set_connected(true);
                        self.as_mut().notes_changed();
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch notes: {}", e);
                        let msg = myme_core::AppError::from(e).user_message();
                        self.as_mut().rust_mut().set_error(msg);
                        self.as_mut().set_connected(false);
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
                        let msg = myme_core::AppError::from(e).user_message();
                        self.as_mut().rust_mut().set_error(msg);
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
                            let filter = &self.as_ref().rust().filter;
                            let should_remove = (matches!(filter, NoteFilter::All) && updated_note.archived)
                                || (matches!(filter, NoteFilter::Archived) && !updated_note.archived);
                            if should_remove {
                                self.as_mut().rust_mut().notes.remove(index);
                            } else {
                                self.as_mut().rust_mut().notes[index] = updated_note;
                            }
                            self.as_mut().notes_changed();
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to update note: {}", e);
                        let msg = myme_core::AppError::from(e).user_message();
                        self.as_mut().rust_mut().set_error(msg);
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
                        let msg = myme_core::AppError::from(e).user_message();
                        self.as_mut().rust_mut().set_error(msg);
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
            .map(|note| QString::from(note.id.to_string()))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_created_at(&self, index: i32) -> QString {
        self.rust()
            .get_note(index)
            .map(|note| QString::from(note.created_at.format("%Y-%m-%d %H:%M").to_string()))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_color(&self, index: i32) -> QString {
        self.rust()
            .get_note(index)
            .and_then(|note| note.color.as_ref())
            .map(|s| QString::from(s.as_str()))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_pinned(&self, index: i32) -> bool {
        self.rust()
            .get_note(index)
            .map(|note| note.pinned)
            .unwrap_or(false)
    }

    pub fn get_archived(&self, index: i32) -> bool {
        self.rust()
            .get_note(index)
            .map(|note| note.archived)
            .unwrap_or(false)
    }

    pub fn get_labels(&self, index: i32) -> QStringList {
        self.rust()
            .get_note(index)
            .map(|note| {
                let mut list = QStringList::default();
                for label in &note.labels {
                    list.append(QString::from(label.as_str()));
                }
                list
            })
            .unwrap_or_else(QStringList::default)
    }

    pub fn get_is_checklist(&self, index: i32) -> bool {
        self.rust()
            .get_note(index)
            .map(|note| note.is_checklist)
            .unwrap_or(false)
    }

    pub fn get_reminder(&self, index: i32) -> QString {
        self.rust()
            .get_note(index)
            .and_then(|note| note.reminder.as_ref())
            .map(|dt| QString::from(dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()))
            .unwrap_or_else(|| QString::from(""))
    }
}
