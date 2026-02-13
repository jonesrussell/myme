//! Gmail API client with retry logic.

use base64::Engine;
use tracing::instrument;

use crate::error::GmailError;
use crate::types::*;

const GMAIL_API_BASE: &str = "https://gmail.googleapis.com";

pub struct GmailClient {
    client: reqwest::Client,
    access_token: String,
    base_url: String,
}

impl GmailClient {
    pub fn new(access_token: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token: access_token.to_string(),
            base_url: GMAIL_API_BASE.to_string(),
        }
    }

    #[cfg(test)]
    pub fn new_with_base_url(access_token: &str, base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            access_token: access_token.to_string(),
            base_url: base_url.to_string(),
        }
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    /// List message IDs (metadata only, not full messages).
    #[instrument(skip(self), level = "info")]
    pub async fn list_message_ids(
        &self,
        query: Option<&str>,
        page_token: Option<&str>,
    ) -> Result<MessageListResponse, GmailError> {
        let mut url = format!("{}/gmail/v1/users/me/messages", self.base_url);
        let mut params = vec![];

        if let Some(q) = query {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if let Some(pt) = page_token {
            params.push(format!("pageToken={}", pt));
        }
        params.push("maxResults=50".to_string());

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        let response =
            self.client.get(&url).header("Authorization", self.auth_header()).send().await?;

        self.handle_response(response).await
    }

    /// Get a single message with full details.
    #[instrument(skip(self), level = "info")]
    pub async fn get_message(&self, message_id: &str) -> Result<Message, GmailError> {
        let url =
            format!("{}/gmail/v1/users/me/messages/{}?format=full", self.base_url, message_id);

        let response =
            self.client.get(&url).header("Authorization", self.auth_header()).send().await?;

        let api_msg: ApiMessage = self.handle_response(response).await?;
        Ok(Message::from_api(api_msg))
    }

    /// List all labels.
    #[instrument(skip(self), level = "info")]
    pub async fn list_labels(&self) -> Result<Vec<Label>, GmailError> {
        let url = format!("{}/gmail/v1/users/me/labels", self.base_url);

        let response =
            self.client.get(&url).header("Authorization", self.auth_header()).send().await?;

        let resp: LabelListResponse = self.handle_response(response).await?;
        Ok(resp.labels.into_iter().map(Label::from).collect())
    }

    /// Modify message labels (archive, mark read, star, etc.).
    #[instrument(skip(self), level = "info")]
    pub async fn modify_labels(
        &self,
        message_id: &str,
        add_labels: &[&str],
        remove_labels: &[&str],
    ) -> Result<(), GmailError> {
        let url = format!("{}/gmail/v1/users/me/messages/{}/modify", self.base_url, message_id);

        let body = serde_json::json!({
            "addLabelIds": add_labels,
            "removeLabelIds": remove_labels,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(GmailError::ApiError(format!("{}: {}", status, text)))
        }
    }

    /// Move message to trash.
    #[instrument(skip(self), level = "info")]
    pub async fn trash_message(&self, message_id: &str) -> Result<(), GmailError> {
        let url = format!("{}/gmail/v1/users/me/messages/{}/trash", self.base_url, message_id);

        let response =
            self.client.post(&url).header("Authorization", self.auth_header()).send().await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            Err(GmailError::ApiError(format!("{}: {}", status, text)))
        }
    }

    /// Send a new email or reply.
    #[instrument(skip(self, body), level = "info")]
    pub async fn send_message(
        &self,
        to: &str,
        subject: &str,
        body: &str,
        reply_to_id: Option<&str>,
    ) -> Result<Message, GmailError> {
        let url = format!("{}/gmail/v1/users/me/messages/send", self.base_url);

        // Build RFC 2822 message
        let mut headers = format!(
            "To: {}\r\nSubject: {}\r\nContent-Type: text/plain; charset=utf-8\r\n",
            to, subject
        );

        if let Some(reply_id) = reply_to_id {
            headers.push_str(&format!("In-Reply-To: {}\r\nReferences: {}\r\n", reply_id, reply_id));
        }

        let raw_message = format!("{}\r\n{}", headers, body);
        let encoded =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw_message.as_bytes());

        let request_body = serde_json::json!({
            "raw": encoded,
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .json(&request_body)
            .send()
            .await?;

        let api_msg: ApiMessage = self.handle_response(response).await?;
        Ok(Message::from_api(api_msg))
    }

    /// Mark a message as read.
    pub async fn mark_as_read(&self, message_id: &str) -> Result<(), GmailError> {
        self.modify_labels(message_id, &[], &["UNREAD"]).await
    }

    /// Mark a message as unread.
    pub async fn mark_as_unread(&self, message_id: &str) -> Result<(), GmailError> {
        self.modify_labels(message_id, &["UNREAD"], &[]).await
    }

    /// Star a message.
    pub async fn star_message(&self, message_id: &str) -> Result<(), GmailError> {
        self.modify_labels(message_id, &["STARRED"], &[]).await
    }

    /// Unstar a message.
    pub async fn unstar_message(&self, message_id: &str) -> Result<(), GmailError> {
        self.modify_labels(message_id, &[], &["STARRED"]).await
    }

    /// Archive a message (remove from INBOX).
    pub async fn archive_message(&self, message_id: &str) -> Result<(), GmailError> {
        self.modify_labels(message_id, &[], &["INBOX"]).await
    }

    /// Helper to handle API responses and errors.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, GmailError> {
        let status = response.status();

        if status.is_success() {
            response
                .json()
                .await
                .map_err(|e| GmailError::ApiError(format!("JSON parse error: {}", e)))
        } else if status.as_u16() == 401 {
            Err(GmailError::TokenExpired)
        } else if status.as_u16() == 403 {
            Err(GmailError::AuthRequired)
        } else if status.as_u16() == 404 {
            let text = response.text().await.unwrap_or_default();
            Err(GmailError::MessageNotFound(text))
        } else if status.as_u16() == 429 {
            // Extract retry-after if available
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            Err(GmailError::RateLimited(retry_after))
        } else {
            let text = response.text().await.unwrap_or_default();
            Err(GmailError::ApiError(format!("{}: {}", status, text)))
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_messages() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages"))
            .and(header("Authorization", "Bearer test_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "messages": [
                    {"id": "msg1", "threadId": "thread1"},
                    {"id": "msg2", "threadId": "thread2"}
                ],
                "resultSizeEstimate": 2
            })))
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("test_token", &mock_server.uri());
        let result = client.list_message_ids(None, None).await.unwrap();

        assert_eq!(result.messages.len(), 2);
        assert_eq!(result.messages[0].id, "msg1");
    }

    #[tokio::test]
    async fn test_get_message() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages/msg123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg123",
                "threadId": "thread123",
                "labelIds": ["INBOX"],
                "snippet": "Test message",
                "payload": {
                    "headers": [
                        {"name": "From", "value": "test@example.com"},
                        {"name": "Subject", "value": "Test Subject"}
                    ]
                }
            })))
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("test_token", &mock_server.uri());
        let msg = client.get_message("msg123").await.unwrap();

        assert_eq!(msg.id, "msg123");
        assert_eq!(msg.subject, "Test Subject");
    }

    #[tokio::test]
    async fn test_list_labels() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/labels"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "labels": [
                    {"id": "INBOX", "name": "Inbox", "messagesTotal": 100, "messagesUnread": 5},
                    {"id": "Label_1", "name": "Work", "messagesTotal": 50}
                ]
            })))
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("test_token", &mock_server.uri());
        let labels = client.list_labels().await.unwrap();

        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].id, "INBOX");
        assert_eq!(labels[0].label_type, LabelType::System);
        assert_eq!(labels[1].label_type, LabelType::User);
    }

    #[tokio::test]
    async fn test_token_expired_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages"))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("expired_token", &mock_server.uri());
        let result = client.list_message_ids(None, None).await;

        assert!(matches!(result, Err(GmailError::TokenExpired)));
    }

    #[tokio::test]
    async fn test_rate_limited() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages"))
            .respond_with(ResponseTemplate::new(429).append_header("Retry-After", "30"))
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("token", &mock_server.uri());
        let result = client.list_message_ids(None, None).await;

        assert!(matches!(result, Err(GmailError::RateLimited(30))));
    }

    #[tokio::test]
    async fn test_modify_labels() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/gmail/v1/users/me/messages/msg123/modify"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg123",
                "threadId": "thread123",
                "labelIds": ["STARRED"]
            })))
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("test_token", &mock_server.uri());
        let result = client.modify_labels("msg123", &["STARRED"], &["UNREAD"]).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trash_message() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/gmail/v1/users/me/messages/msg123/trash"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg123",
                "threadId": "thread123",
                "labelIds": ["TRASH"]
            })))
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("test_token", &mock_server.uri());
        let result = client.trash_message("msg123").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_messages_with_query() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "messages": [{"id": "msg1", "threadId": "thread1"}],
                "resultSizeEstimate": 1
            })))
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("test_token", &mock_server.uri());
        let result = client.list_message_ids(Some("is:unread"), None).await.unwrap();

        assert_eq!(result.messages.len(), 1);
    }
}
