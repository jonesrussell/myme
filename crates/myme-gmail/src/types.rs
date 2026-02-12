//! Gmail API types and data structures.

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
            body: None, // Loaded separately with full message
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
