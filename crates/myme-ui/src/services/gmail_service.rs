//! Gmail backend: async operations using the shared runtime and channel pattern.
//! All network work runs off the UI thread; results sent via mpsc.

use std::path::PathBuf;

use myme_gmail::{GmailCache, GmailClient, Message};

use crate::bridge;

/// Messages sent from async operations back to the UI thread.
#[derive(Debug)]
pub enum GmailServiceMessage {
    /// Result of fetching messages.
    FetchDone(Result<Vec<Message>, String>),
    /// Result of an action (mark read, archive, trash); carries message_id or error.
    ActionDone(Result<String, String>),
}

/// Request to fetch messages asynchronously.
pub fn request_fetch(
    tx: &std::sync::mpsc::Sender<GmailServiceMessage>,
    access_token: String,
    cache_path: PathBuf,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(GmailServiceMessage::FetchDone(Err(
                "Runtime not available".to_string(),
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let client = GmailClient::new(&access_token);

        let result = async {
            let list_response = client
                .list_message_ids(Some("in:inbox"), None)
                .await
                .map_err(|e| e.to_string())?;

            let mut messages = Vec::new();
            for msg_ref in list_response.messages.into_iter().take(20) {
                match client.get_message(&msg_ref.id).await {
                    Ok(msg) => messages.push(msg),
                    Err(e) => tracing::warn!("Failed to fetch message {}: {}", msg_ref.id, e),
                }
            }

            if let Ok(cache) = GmailCache::new(&cache_path) {
                for msg in &messages {
                    let _ = cache.store_message(msg);
                }
            }

            Ok(messages)
        }
        .await;

        let _ = tx.send(GmailServiceMessage::FetchDone(result));
    });
}

/// Request to mark a message as read.
pub fn request_mark_as_read(
    tx: &std::sync::mpsc::Sender<GmailServiceMessage>,
    access_token: String,
    message_id: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(GmailServiceMessage::ActionDone(Err(
                "Runtime not available".to_string(),
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let client = GmailClient::new(&access_token);
        let result = client
            .mark_as_read(&message_id)
            .await
            .map(|_| message_id)
            .map_err(|e| e.to_string());
        let _ = tx.send(GmailServiceMessage::ActionDone(result));
    });
}

/// Request to archive a message.
pub fn request_archive(
    tx: &std::sync::mpsc::Sender<GmailServiceMessage>,
    access_token: String,
    message_id: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(GmailServiceMessage::ActionDone(Err(
                "Runtime not available".to_string(),
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let client = GmailClient::new(&access_token);
        let result = client
            .archive_message(&message_id)
            .await
            .map(|_| message_id)
            .map_err(|e| e.to_string());
        let _ = tx.send(GmailServiceMessage::ActionDone(result));
    });
}

/// Request to move a message to trash.
pub fn request_trash(
    tx: &std::sync::mpsc::Sender<GmailServiceMessage>,
    access_token: String,
    message_id: String,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(GmailServiceMessage::ActionDone(Err(
                "Runtime not available".to_string(),
            )));
            return;
        }
    };

    runtime.spawn(async move {
        let client = GmailClient::new(&access_token);
        let result = client
            .trash_message(&message_id)
            .await
            .map(|_| message_id)
            .map_err(|e| e.to_string());
        let _ = tx.send(GmailServiceMessage::ActionDone(result));
    });
}
