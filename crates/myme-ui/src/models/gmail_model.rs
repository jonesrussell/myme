//! Gmail model for QML.
//!
//! Provides email listing, reading, and actions.

use core::pin::Pin;
use std::sync::mpsc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_auth::{SecureStorage, GoogleOAuth2Provider};
use myme_gmail::{GmailClient, GmailCache, Message};

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

/// Messages for async operations
enum GmailMessage {
    FetchDone(Result<Vec<Message>, String>),
    ActionDone(Result<String, String>), // message_id or error
}

#[derive(Default)]
pub struct GmailModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    unread_count: i32,
    message_count: i32,
    messages: Vec<Message>,
    rx: Option<mpsc::Receiver<GmailMessage>>,
}

impl GmailModelRust {
    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }

    fn get_access_token() -> Option<String> {
        let token_set = SecureStorage::retrieve_token("google").ok()?;

        if token_set.is_expired() {
            // Try to refresh
            if let Some(refresh_token) = &token_set.refresh_token {
                if let Some((client_id, client_secret)) = get_google_config() {
                    let rt = tokio::runtime::Runtime::new().ok()?;
                    let provider = GoogleOAuth2Provider::new(client_id, client_secret);

                    if let Ok(new_tokens) = rt.block_on(provider.refresh_token(refresh_token)) {
                        let expires_at = chrono::Utc::now().timestamp() + new_tokens.expires_in as i64;
                        let new_token_set = myme_auth::TokenSet {
                            access_token: new_tokens.access_token.clone(),
                            refresh_token: new_tokens.refresh_token.or(token_set.refresh_token.clone()),
                            expires_at,
                            scopes: new_tokens.scope.split(' ').map(|s| s.to_string()).collect(),
                        };
                        let _ = SecureStorage::store_token("google", &new_token_set);
                        return Some(new_tokens.access_token);
                    }
                }
            }
            return None;
        }

        Some(token_set.access_token)
    }

    fn get_cache_path() -> std::path::PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("myme")
            .join("gmail_cache.db")
    }
}

fn get_google_config() -> Option<(String, String)> {
    match myme_core::Config::load() {
        Ok(config) => {
            let client_id = config.google.as_ref()?.client_id.clone()?;
            let client_secret = config.google.as_ref()?.client_secret.clone()?;
            Some((client_id, client_secret))
        }
        Err(_) => None,
    }
}

impl qobject::GmailModel {
    /// Check if Google is authenticated
    pub fn check_auth(mut self: Pin<&mut Self>) {
        let is_authenticated = SecureStorage::has_token("google");
        self.as_mut().set_authenticated(is_authenticated);

        if is_authenticated {
            // Load cached message count
            if let Ok(cache) = GmailCache::new(GmailModelRust::get_cache_path()) {
                if let Ok(count) = cache.unread_count() {
                    self.as_mut().set_unread_count(count as i32);
                }
            }
        }
    }

    /// Fetch messages from Gmail (non-blocking)
    pub fn fetch_messages(mut self: Pin<&mut Self>) {
        let access_token = match GmailModelRust::get_access_token() {
            Some(t) => t,
            None => {
                self.as_mut().set_error_message(QString::from("Not authenticated"));
                self.as_mut().set_authenticated(false);
                return;
            }
        };

        let (tx, rx) = mpsc::channel();
        self.as_mut().rust_mut().rx = Some(rx);

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();

        let cache_path = GmailModelRust::get_cache_path();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let client = GmailClient::new(&access_token);

                // Fetch message list
                let list_response = client.list_message_ids(Some("in:inbox"), None).await
                    .map_err(|e| e.to_string())?;

                // Fetch full details for each message (limited to first 20)
                let mut messages = Vec::new();
                for msg_ref in list_response.messages.into_iter().take(20) {
                    match client.get_message(&msg_ref.id).await {
                        Ok(msg) => messages.push(msg),
                        Err(e) => tracing::warn!("Failed to fetch message {}: {}", msg_ref.id, e),
                    }
                }

                // Cache messages
                if let Ok(cache) = GmailCache::new(&cache_path) {
                    for msg in &messages {
                        let _ = cache.store_message(msg);
                    }
                }

                Ok(messages)
            });

            let _ = tx.send(GmailMessage::FetchDone(result));
        });
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

        QString::from(json.to_string().as_str())
    }

    /// Mark message as read
    pub fn mark_as_read(mut self: Pin<&mut Self>, message_id: QString) {
        let access_token = match GmailModelRust::get_access_token() {
            Some(t) => t,
            None => return,
        };

        let msg_id = message_id.to_string();
        let (tx, rx) = mpsc::channel();
        self.as_mut().rust_mut().rx = Some(rx);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let client = GmailClient::new(&access_token);
                client.mark_as_read(&msg_id).await
                    .map(|_| msg_id.clone())
                    .map_err(|e| e.to_string())
            });

            let _ = tx.send(GmailMessage::ActionDone(result));
        });
    }

    /// Archive message
    pub fn archive_message(mut self: Pin<&mut Self>, message_id: QString) {
        let access_token = match GmailModelRust::get_access_token() {
            Some(t) => t,
            None => return,
        };

        let msg_id = message_id.to_string();
        let (tx, rx) = mpsc::channel();
        self.as_mut().rust_mut().rx = Some(rx);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let client = GmailClient::new(&access_token);
                client.archive_message(&msg_id).await
                    .map(|_| msg_id.clone())
                    .map_err(|e| e.to_string())
            });

            let _ = tx.send(GmailMessage::ActionDone(result));
        });
    }

    /// Move message to trash
    pub fn trash_message(mut self: Pin<&mut Self>, message_id: QString) {
        let access_token = match GmailModelRust::get_access_token() {
            Some(t) => t,
            None => return,
        };

        let msg_id = message_id.to_string();
        let (tx, rx) = mpsc::channel();
        self.as_mut().rust_mut().rx = Some(rx);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let client = GmailClient::new(&access_token);
                client.trash_message(&msg_id).await
                    .map(|_| msg_id.clone())
                    .map_err(|e| e.to_string())
            });

            let _ = tx.send(GmailMessage::ActionDone(result));
        });
    }

    /// Poll for async operation results
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match self.as_ref().rust().rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            Some(m) => m,
            None => return,
        };

        match msg {
            GmailMessage::FetchDone(result) => {
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
            GmailMessage::ActionDone(result) => {
                match result {
                    Ok(msg_id) => {
                        self.as_mut().message_updated(QString::from(&msg_id));
                        // Refresh messages after action
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
