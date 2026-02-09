//! Gmail model for QML.
//!
//! Provides email listing, reading, and actions.
//! Uses the shared AppServices runtime and channel pattern (no block_on).

use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_auth::SecureStorage;
use myme_gmail::{GmailCache, Message};

use crate::bridge;
use crate::services::google_common::{get_google_access_token, get_google_cache_path};
use crate::services::{
    request_gmail_archive, request_gmail_fetch, request_gmail_mark_as_read, request_gmail_trash,
    GmailServiceMessage,
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
        #[qproperty(bool, authenticated)]
        #[qproperty(QString, error_message)]
        #[qproperty(i32, unread_count)]
        #[qproperty(i32, message_count)]
        type GmailModel = super::GmailModelRust;

        #[qinvokable]
        fn check_auth(self: Pin<&mut GmailModel>);

        #[qinvokable]
        fn fetch_messages(self: Pin<&mut GmailModel>);

        #[qinvokable]
        fn get_message(self: Pin<&mut GmailModel>, index: i32) -> QString;

        #[qinvokable]
        fn mark_as_read(self: Pin<&mut GmailModel>, message_id: QString);

        #[qinvokable]
        fn archive_message(self: Pin<&mut GmailModel>, message_id: QString);

        #[qinvokable]
        fn trash_message(self: Pin<&mut GmailModel>, message_id: QString);

        /// Poll for async operation results. Call this from a QML Timer.
        #[qinvokable]
        fn poll_channel(self: Pin<&mut GmailModel>);

        #[qsignal]
        fn messages_changed(self: Pin<&mut GmailModel>);

        #[qsignal]
        fn message_updated(self: Pin<&mut GmailModel>, message_id: QString);
    }
}

#[derive(Default)]
pub struct GmailModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    unread_count: i32,
    message_count: i32,
    messages: Vec<Message>,
}

impl GmailModelRust {
    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }

    fn get_access_token() -> Option<String> {
        get_google_access_token()
    }

    fn get_cache_path() -> std::path::PathBuf {
        get_google_cache_path("gmail_cache.db")
    }
}

impl qobject::GmailModel {
    /// Check if Google is authenticated
    pub fn check_auth(mut self: Pin<&mut Self>) {
        let is_authenticated = SecureStorage::has_token("google");
        self.as_mut().set_authenticated(is_authenticated);

        if is_authenticated {
            if let Ok(cache) = GmailCache::new(GmailModelRust::get_cache_path()) {
                if let Ok(count) = cache.unread_count() {
                    self.as_mut().set_unread_count(count as i32);
                }
            }
        }
    }

    /// Fetch messages from Gmail (non-blocking, uses shared runtime)
    pub fn fetch_messages(mut self: Pin<&mut Self>) {
        let access_token = match GmailModelRust::get_access_token() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Not authenticated"));
                self.as_mut().set_authenticated(false);
                return;
            }
        };

        bridge::init_gmail_service_channel();
        let tx = match bridge::get_gmail_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();

        let cache_path = GmailModelRust::get_cache_path();
        request_gmail_fetch(&tx, access_token, cache_path);
    }

    /// Get message at index as JSON
    pub fn get_message(self: Pin<&mut Self>, index: i32) -> QString {
        let rust = self.rust();
        if index < 0 || index as usize >= rust.messages.len() {
            return QString::from("{}");
        }

        let msg = &rust.messages[index as usize];
        let json = serde_json::json!({
            "id": msg.id,
            "from": msg.from,
            "subject": msg.subject,
            "snippet": msg.snippet,
            "date": msg.date.to_rfc3339(),
            "isUnread": msg.is_unread,
            "isStarred": msg.is_starred,
        });

        let s = json.to_string();
        QString::from(s.as_str())
    }

    /// Mark message as read
    pub fn mark_as_read(self: Pin<&mut Self>, message_id: QString) {
        let access_token = match GmailModelRust::get_access_token() {
            Some(t) => t,
            None => return,
        };

        bridge::init_gmail_service_channel();
        let tx = match bridge::get_gmail_service_tx() {
            Some(t) => t,
            None => return,
        };

        let msg_id = message_id.to_string();
        request_gmail_mark_as_read(&tx, access_token, msg_id);
    }

    /// Archive message
    pub fn archive_message(self: Pin<&mut Self>, message_id: QString) {
        let access_token = match GmailModelRust::get_access_token() {
            Some(t) => t,
            None => return,
        };

        bridge::init_gmail_service_channel();
        let tx = match bridge::get_gmail_service_tx() {
            Some(t) => t,
            None => return,
        };

        let msg_id = message_id.to_string();
        request_gmail_archive(&tx, access_token, msg_id);
    }

    /// Move message to trash
    pub fn trash_message(self: Pin<&mut Self>, message_id: QString) {
        let access_token = match GmailModelRust::get_access_token() {
            Some(t) => t,
            None => return,
        };

        bridge::init_gmail_service_channel();
        let tx = match bridge::get_gmail_service_tx() {
            Some(t) => t,
            None => return,
        };

        let msg_id = message_id.to_string();
        request_gmail_trash(&tx, access_token, msg_id);
    }

    /// Poll for async operation results
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_gmail_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            GmailServiceMessage::FetchDone(result) => {
                self.as_mut().set_loading(false);

                match result {
                    Ok(messages) => {
                        let unread = messages.iter().filter(|m| m.is_unread).count();
                        self.as_mut().set_unread_count(unread as i32);
                        self.as_mut().set_message_count(messages.len() as i32);
                        self.as_mut().rust_mut().messages = messages;
                        self.as_mut().rust_mut().clear_error();
                        self.as_mut().messages_changed();
                    }
                    Err(e) => {
                        self.as_mut().rust_mut().set_error(&e);
                    }
                }
            }
            GmailServiceMessage::ActionDone(result) => {
                match result {
                    Ok(msg_id) => {
                        self.as_mut().message_updated(QString::from(&msg_id));
                        self.fetch_messages();
                    }
                    Err(e) => {
                        self.as_mut().rust_mut().set_error(&e);
                    }
                }
            }
        }
    }
}
