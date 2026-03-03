//! Gmail API types and data structures.

use base64::Engine as _;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Gmail message as stored locally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub thread_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    pub snippet: String,
    pub date: DateTime<Utc>,
    pub labels: Vec<String>,
    pub is_unread: bool,
    pub is_starred: bool,
    pub body: Option<String>,
}

/// Gmail API message response structure.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiMessage {
    pub id: String,
    pub thread_id: String,
    #[serde(default)]
    pub label_ids: Vec<String>,
    #[serde(default)]
    pub snippet: String,
    pub internal_date: Option<String>,
    pub payload: Option<MessagePayload>,
}

#[derive(Debug, Deserialize)]
pub struct MessagePayload {
    #[serde(default)]
    pub headers: Vec<Header>,
    pub body: Option<MessageBody>,
    #[serde(default)]
    pub parts: Vec<MessagePart>,
}

#[derive(Debug, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageBody {
    pub data: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePart {
    pub mime_type: String,
    pub body: Option<MessageBody>,
    #[serde(default)]
    pub parts: Vec<MessagePart>,
}

/// Decode base64url-encoded data (as used by Gmail API).
fn decode_base64url(data: &str) -> Option<String> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(data).ok()?;
    String::from_utf8(bytes).ok()
}

/// Extract body text from a list of MIME parts (recursive).
/// Prefers text/plain, falls back to text/html.
fn extract_body_from_parts(parts: &[MessagePart]) -> Option<String> {
    // First pass: look for text/plain
    for part in parts {
        if part.mime_type == "text/plain" {
            if let Some(data) = part.body.as_ref().and_then(|b| b.data.as_ref()) {
                if let Some(text) = decode_base64url(data) {
                    return Some(text);
                }
            }
        }
        // Recurse into nested parts
        if !part.parts.is_empty() {
            if let Some(text) = extract_body_from_parts(&part.parts) {
                return Some(text);
            }
        }
    }
    // Second pass: fall back to text/html
    for part in parts {
        if part.mime_type == "text/html" {
            if let Some(data) = part.body.as_ref().and_then(|b| b.data.as_ref()) {
                if let Some(text) = decode_base64url(data) {
                    return Some(text);
                }
            }
        }
        if !part.parts.is_empty() {
            if let Some(text) = extract_body_from_parts(&part.parts) {
                return Some(text);
            }
        }
    }
    None
}

/// Extract body from a message payload.
/// Handles both single-part (body.data on payload) and multipart (parts array) messages.
fn extract_body(payload: &MessagePayload) -> Option<String> {
    // Single-part message: body data directly on payload
    if let Some(data) = payload.body.as_ref().and_then(|b| b.data.as_ref()) {
        if let Some(text) = decode_base64url(data) {
            return Some(text);
        }
    }
    // Multipart message: search parts
    if !payload.parts.is_empty() {
        return extract_body_from_parts(&payload.parts);
    }
    None
}

impl Message {
    /// Convert API response to local Message.
    pub fn from_api(api: ApiMessage) -> Self {
        let headers = api.payload.as_ref().map(|p| &p.headers);

        let from = headers
            .and_then(|h| h.iter().find(|h| h.name.eq_ignore_ascii_case("from")))
            .map(|h| h.value.clone())
            .unwrap_or_default();

        let to = headers
            .and_then(|h| h.iter().find(|h| h.name.eq_ignore_ascii_case("to")))
            .map(|h| h.value.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let subject = headers
            .and_then(|h| h.iter().find(|h| h.name.eq_ignore_ascii_case("subject")))
            .map(|h| h.value.clone())
            .unwrap_or_default();

        let date = api
            .internal_date
            .as_ref()
            .and_then(|d| d.parse::<i64>().ok())
            .and_then(DateTime::from_timestamp_millis)
            .unwrap_or_default();

        let is_unread = api.label_ids.iter().any(|l| l == "UNREAD");
        let is_starred = api.label_ids.iter().any(|l| l == "STARRED");

        let body = api.payload.as_ref().and_then(extract_body);

        Self {
            id: api.id,
            thread_id: api.thread_id,
            from,
            to,
            subject,
            snippet: api.snippet,
            date,
            labels: api.label_ids,
            is_unread,
            is_starred,
            body,
        }
    }
}

/// Gmail label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub id: String,
    pub name: String,
    pub label_type: LabelType,
    pub messages_total: Option<u32>,
    pub messages_unread: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LabelType {
    System,
    User,
}

impl Label {
    const SYSTEM_LABELS: &'static [&'static str] = &[
        "INBOX",
        "SENT",
        "DRAFT",
        "TRASH",
        "SPAM",
        "STARRED",
        "IMPORTANT",
        "UNREAD",
        "CATEGORY_PERSONAL",
        "CATEGORY_SOCIAL",
        "CATEGORY_PROMOTIONS",
        "CATEGORY_UPDATES",
        "CATEGORY_FORUMS",
    ];

    pub fn is_system_label(id: &str) -> bool {
        Self::SYSTEM_LABELS.contains(&id)
    }
}

/// API response for message list.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageListResponse {
    #[serde(default)]
    pub messages: Vec<MessageRef>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRef {
    pub id: String,
    pub thread_id: String,
}

/// API response for label list.
#[derive(Debug, Deserialize)]
pub struct LabelListResponse {
    #[serde(default)]
    pub labels: Vec<ApiLabel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiLabel {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub label_type: Option<String>,
    pub messages_total: Option<u32>,
    pub messages_unread: Option<u32>,
}

/// Gmail History API response for incremental sync.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryListResponse {
    #[serde(default)]
    pub history: Vec<HistoryRecord>,
    pub next_page_token: Option<String>,
    pub history_id: Option<String>,
}

/// A single history record from the Gmail History API.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub id: String,
    #[serde(default)]
    pub messages: Vec<HistoryMessage>,
    #[serde(default)]
    pub messages_added: Vec<HistoryMessage>,
    #[serde(default)]
    pub messages_deleted: Vec<HistoryMessage>,
    #[serde(default)]
    pub labels_added: Vec<HistoryLabelChange>,
    #[serde(default)]
    pub labels_removed: Vec<HistoryLabelChange>,
}

/// A message reference within a history record.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryMessage {
    pub message: MessageRef,
}

/// A label change within a history record.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryLabelChange {
    pub message: MessageRef,
    #[serde(default)]
    pub label_ids: Vec<String>,
}

impl From<ApiLabel> for Label {
    fn from(api: ApiLabel) -> Self {
        Self {
            id: api.id.clone(),
            name: api.name,
            label_type: if Label::is_system_label(&api.id) {
                LabelType::System
            } else {
                LabelType::User
            },
            messages_total: api.messages_total,
            messages_unread: api.messages_unread,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_message_from_api_response() {
        let json = r#"{
            "id": "abc123",
            "threadId": "thread456",
            "labelIds": ["INBOX", "UNREAD"],
            "snippet": "Hello world...",
            "internalDate": "1706745600000",
            "payload": {
                "headers": [
                    {"name": "From", "value": "sender@example.com"},
                    {"name": "To", "value": "me@example.com"},
                    {"name": "Subject", "value": "Test Subject"}
                ]
            }
        }"#;

        let api_msg: ApiMessage = serde_json::from_str(json).unwrap();
        let msg = Message::from_api(api_msg);

        assert_eq!(msg.id, "abc123");
        assert_eq!(msg.thread_id, "thread456");
        assert_eq!(msg.from, "sender@example.com");
        assert_eq!(msg.subject, "Test Subject");
        assert!(msg.is_unread);
    }

    #[test]
    fn test_label_system_labels() {
        assert!(Label::is_system_label("INBOX"));
        assert!(Label::is_system_label("SENT"));
        assert!(!Label::is_system_label("MyCustomLabel"));
    }

    #[test]
    fn test_message_starred_flag() {
        let api_msg = ApiMessage {
            id: "test".into(),
            thread_id: "thread".into(),
            label_ids: vec!["STARRED".into()],
            snippet: "".into(),
            internal_date: None,
            payload: None,
        };
        let msg = Message::from_api(api_msg);
        assert!(msg.is_starred);
        assert!(!msg.is_unread);
    }

    #[test]
    fn test_label_from_api() {
        let api_label = ApiLabel {
            id: "INBOX".into(),
            name: "Inbox".into(),
            label_type: Some("system".into()),
            messages_total: Some(100),
            messages_unread: Some(5),
        };
        let label = Label::from(api_label);
        assert_eq!(label.id, "INBOX");
        assert_eq!(label.label_type, LabelType::System);
        assert_eq!(label.messages_unread, Some(5));
    }

    #[test]
    fn test_user_label_from_api() {
        let api_label = ApiLabel {
            id: "Label_123".into(),
            name: "My Custom Label".into(),
            label_type: Some("user".into()),
            messages_total: Some(10),
            messages_unread: None,
        };
        let label = Label::from(api_label);
        assert_eq!(label.label_type, LabelType::User);
    }

    #[test]
    fn test_body_extraction_single_part() {
        // "Hello World" base64url-encoded
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"Hello World");
        let json = format!(
            r#"{{
            "id": "msg1",
            "threadId": "thread1",
            "labelIds": ["INBOX"],
            "snippet": "Hello...",
            "payload": {{
                "headers": [],
                "body": {{ "data": "{}" }},
                "parts": []
            }}
        }}"#,
            encoded
        );

        let api_msg: ApiMessage = serde_json::from_str(&json).unwrap();
        let msg = Message::from_api(api_msg);
        assert_eq!(msg.body, Some("Hello World".to_string()));
    }

    #[test]
    fn test_body_extraction_multipart_prefers_plain() {
        let plain = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"Plain text body");
        let html = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"<p>HTML body</p>");
        let json = format!(
            r#"{{
            "id": "msg2",
            "threadId": "thread2",
            "labelIds": [],
            "snippet": "",
            "payload": {{
                "headers": [],
                "parts": [
                    {{ "mimeType": "text/html", "body": {{ "data": "{}" }}, "parts": [] }},
                    {{ "mimeType": "text/plain", "body": {{ "data": "{}" }}, "parts": [] }}
                ]
            }}
        }}"#,
            html, plain
        );

        let api_msg: ApiMessage = serde_json::from_str(&json).unwrap();
        let msg = Message::from_api(api_msg);
        assert_eq!(msg.body, Some("Plain text body".to_string()));
    }

    #[test]
    fn test_body_extraction_html_fallback() {
        let html = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"<p>HTML only</p>");
        let json = format!(
            r#"{{
            "id": "msg3",
            "threadId": "thread3",
            "labelIds": [],
            "snippet": "",
            "payload": {{
                "headers": [],
                "parts": [
                    {{ "mimeType": "text/html", "body": {{ "data": "{}" }}, "parts": [] }}
                ]
            }}
        }}"#,
            html
        );

        let api_msg: ApiMessage = serde_json::from_str(&json).unwrap();
        let msg = Message::from_api(api_msg);
        assert_eq!(msg.body, Some("<p>HTML only</p>".to_string()));
    }

    #[test]
    fn test_history_list_response_parsing() {
        let json = r#"{
            "history": [
                {
                    "id": "12345",
                    "messages": [{"message": {"id": "msg1", "threadId": "t1"}}],
                    "messagesAdded": [{"message": {"id": "msg2", "threadId": "t2"}}],
                    "messagesDeleted": [],
                    "labelsAdded": [{"message": {"id": "msg1", "threadId": "t1"}, "labelIds": ["INBOX"]}],
                    "labelsRemoved": []
                }
            ],
            "historyId": "67890"
        }"#;

        let resp: HistoryListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.history.len(), 1);
        assert_eq!(resp.history_id, Some("67890".to_string()));
        assert_eq!(resp.history[0].messages_added.len(), 1);
        assert_eq!(resp.history[0].labels_added.len(), 1);
    }

    #[test]
    fn test_message_list_response_parsing() {
        let json = r#"{
            "messages": [
                {"id": "msg1", "threadId": "thread1"},
                {"id": "msg2", "threadId": "thread2"}
            ],
            "nextPageToken": "token123",
            "resultSizeEstimate": 100
        }"#;

        let response: MessageListResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.messages.len(), 2);
        assert_eq!(response.next_page_token, Some("token123".into()));
    }
}
