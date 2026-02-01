# Google Gmail & Calendar Integration - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add full Gmail and Google Calendar integration with offline support, home page widgets, and unified account management.

**Architecture:** Two new crates (`myme-gmail`, `myme-calendar`) following existing service patterns. SQLite for offline cache. Channel-based async for non-blocking UI. Shared Google OAuth in `myme-auth`.

**Tech Stack:** Rust, reqwest, rusqlite, tokio, cxx-qt, QML

**Design Document:** `docs/plans/2026-02-01-google-integration-design.md`

---

## Phase 1: Google OAuth Authentication

### Task 1.1: Add Google OAuth Provider

**Files:**
- Create: `crates/myme-auth/src/google.rs`
- Modify: `crates/myme-auth/src/lib.rs`
- Test: `crates/myme-auth/src/google.rs` (inline tests)

**Step 1: Write the failing test**

Add to `crates/myme-auth/src/google.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_oauth_provider_creation() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        assert_eq!(provider.client_id, "test_client_id");
    }

    #[test]
    fn test_google_auth_url_contains_scopes() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        let (url, _state) = provider.authorization_url(8080);
        assert!(url.contains("scope="));
        assert!(url.contains("gmail"));
        assert!(url.contains("calendar"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p myme-auth google`
Expected: FAIL with "cannot find module `google`"

**Step 3: Write minimal implementation**

Create `crates/myme-auth/src/google.rs`:

```rust
//! Google OAuth2 provider for Gmail and Calendar access.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

// Scopes for Gmail and Calendar access
const GMAIL_SCOPE: &str = "https://www.googleapis.com/auth/gmail.modify";
const CALENDAR_SCOPE: &str = "https://www.googleapis.com/auth/calendar";
const USERINFO_SCOPE: &str = "https://www.googleapis.com/auth/userinfo.email";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserInfo {
    pub email: String,
    pub verified_email: bool,
    pub picture: Option<String>,
}

pub struct GoogleOAuth2Provider {
    pub client_id: String,
    pub client_secret: String,
}

impl GoogleOAuth2Provider {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }

    /// Generate authorization URL for OAuth flow.
    /// Returns (url, state) where state should be verified on callback.
    pub fn authorization_url(&self, port: u16) -> (String, String) {
        let state = uuid::Uuid::new_v4().to_string();
        let redirect_uri = format!("http://localhost:{}/callback", port);
        let scopes = format!("{} {} {}", GMAIL_SCOPE, CALENDAR_SCOPE, USERINFO_SCOPE);

        let url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&access_type=offline&prompt=consent",
            GOOGLE_AUTH_URL,
            urlencoding::encode(&self.client_id),
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&scopes),
            urlencoding::encode(&state),
        );

        (url, state)
    }

    /// Exchange authorization code for tokens.
    pub async fn exchange_code(&self, code: &str, port: u16) -> Result<GoogleTokenResponse> {
        let redirect_uri = format!("http://localhost:{}/callback", port);
        let client = reqwest::Client::new();

        let response = client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", &redirect_uri),
            ])
            .send()
            .await
            .context("Failed to send token request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Token exchange failed: {}", error_text);
        }

        response
            .json::<GoogleTokenResponse>()
            .await
            .context("Failed to parse token response")
    }

    /// Refresh an expired access token.
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<GoogleTokenResponse> {
        let client = reqwest::Client::new();

        let response = client
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .context("Failed to send refresh request")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("Token refresh failed: {}", error_text);
        }

        response
            .json::<GoogleTokenResponse>()
            .await
            .context("Failed to parse refresh response")
    }

    /// Get user info (email) from access token.
    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleUserInfo> {
        let client = reqwest::Client::new();

        let response = client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to fetch user info")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            anyhow::bail!("User info request failed: {}", error_text);
        }

        response
            .json::<GoogleUserInfo>()
            .await
            .context("Failed to parse user info")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_oauth_provider_creation() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        assert_eq!(provider.client_id, "test_client_id");
    }

    #[test]
    fn test_google_auth_url_contains_scopes() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        let (url, _state) = provider.authorization_url(8080);
        assert!(url.contains("scope="));
        assert!(url.contains("gmail"));
        assert!(url.contains("calendar"));
    }

    #[test]
    fn test_google_auth_url_contains_offline_access() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        let (url, _state) = provider.authorization_url(8080);
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
    }

    #[test]
    fn test_google_state_is_unique() {
        let provider = GoogleOAuth2Provider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
        );
        let (_, state1) = provider.authorization_url(8080);
        let (_, state2) = provider.authorization_url(8080);
        assert_ne!(state1, state2);
    }
}
```

**Step 4: Update lib.rs to export module**

Add to `crates/myme-auth/src/lib.rs`:

```rust
pub mod google;
pub use google::{GoogleOAuth2Provider, GoogleTokenResponse, GoogleUserInfo};
```

**Step 5: Add uuid dependency**

Add to `crates/myme-auth/Cargo.toml`:

```toml
uuid = { version = "1.0", features = ["v4"] }
urlencoding = "2.1"
```

**Step 6: Run test to verify it passes**

Run: `cargo test -p myme-auth google`
Expected: PASS (4 tests)

**Step 7: Commit**

```bash
git add crates/myme-auth/
git commit -m "feat(auth): add Google OAuth2 provider

- Add GoogleOAuth2Provider with authorization URL generation
- Add token exchange and refresh methods
- Add user info endpoint for email retrieval
- Request Gmail, Calendar, and userinfo scopes
- Include offline access for refresh tokens"
```

---

### Task 1.2: Extend SecureStorage for Google Tokens

**Files:**
- Modify: `crates/myme-auth/src/storage.rs`
- Test: `crates/myme-auth/src/storage.rs` (inline tests)

**Step 1: Write the failing test**

Add to `crates/myme-auth/src/storage.rs` tests:

```rust
#[test]
fn test_google_token_storage_separate_from_github() {
    let storage = SecureStorage::new();

    // Store different tokens for each provider
    storage.store_token("github", "github_token").unwrap();
    storage.store_token("google", "google_token").unwrap();

    // Retrieve and verify they're separate
    assert_eq!(storage.get_token("github").unwrap(), Some("github_token".to_string()));
    assert_eq!(storage.get_token("google").unwrap(), Some("google_token".to_string()));

    // Delete one doesn't affect the other
    storage.delete_token("github").unwrap();
    assert_eq!(storage.get_token("github").unwrap(), None);
    assert_eq!(storage.get_token("google").unwrap(), Some("google_token".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p myme-auth storage`
Expected: FAIL (method signatures don't match or don't exist)

**Step 3: Update SecureStorage with provider parameter**

The current `SecureStorage` likely uses a fixed service name. Update to support multiple providers:

```rust
impl SecureStorage {
    const SERVICE_PREFIX: &'static str = "myme";

    fn service_name(provider: &str) -> String {
        format!("{}.{}", Self::SERVICE_PREFIX, provider)
    }

    pub fn store_token(&self, provider: &str, token: &str) -> Result<()> {
        let entry = keyring::Entry::new(&Self::service_name(provider), "access_token")?;
        entry.set_password(token)?;
        Ok(())
    }

    pub fn get_token(&self, provider: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(&Self::service_name(provider), "access_token")?;
        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete_token(&self, provider: &str) -> Result<()> {
        let entry = keyring::Entry::new(&Self::service_name(provider), "access_token")?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    // Add refresh token storage for Google
    pub fn store_refresh_token(&self, provider: &str, token: &str) -> Result<()> {
        let entry = keyring::Entry::new(&Self::service_name(provider), "refresh_token")?;
        entry.set_password(token)?;
        Ok(())
    }

    pub fn get_refresh_token(&self, provider: &str) -> Result<Option<String>> {
        let entry = keyring::Entry::new(&Self::service_name(provider), "refresh_token")?;
        match entry.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p myme-auth storage`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/myme-auth/src/storage.rs
git commit -m "feat(auth): extend SecureStorage for multiple providers

- Add provider parameter to token methods (github, google)
- Add refresh token storage for OAuth providers
- Use namespaced service names: myme.github, myme.google"
```

---

## Phase 2: Gmail Crate Foundation

### Task 2.1: Create myme-gmail Crate Structure

**Files:**
- Create: `crates/myme-gmail/Cargo.toml`
- Create: `crates/myme-gmail/src/lib.rs`
- Create: `crates/myme-gmail/src/error.rs`
- Modify: `Cargo.toml` (workspace)

**Step 1: Create Cargo.toml**

Create `crates/myme-gmail/Cargo.toml`:

```toml
[package]
name = "myme-gmail"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1.42", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.22"
tracing = "0.1"
anyhow = "1.0"
thiserror = "1.0"
chrono = { version = "0.4", features = ["serde"] }
myme-auth = { path = "../myme-auth" }

[dev-dependencies]
tempfile = "3.10"
wiremock = "0.6"
tokio = { version = "1.42", features = ["rt-multi-thread", "macros"] }
```

**Step 2: Create error.rs**

Create `crates/myme-gmail/src/error.rs`:

```rust
//! Gmail-specific error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GmailError {
    #[error("Authentication required")]
    AuthRequired,

    #[error("Token expired")]
    TokenExpired,

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("Message not found: {0}")]
    MessageNotFound(String),

    #[error("Label not found: {0}")]
    LabelNotFound(String),

    #[error("Invalid message format")]
    InvalidMessageFormat,

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

impl GmailError {
    /// User-friendly error message for UI display.
    pub fn user_message(&self) -> String {
        match self {
            Self::AuthRequired => "Please sign in to your Google account".to_string(),
            Self::TokenExpired => "Your session has expired. Please sign in again.".to_string(),
            Self::RateLimited(secs) => format!("Too many requests. Please wait {} seconds.", secs),
            Self::MessageNotFound(_) => "Message not found".to_string(),
            Self::LabelNotFound(_) => "Label not found".to_string(),
            Self::InvalidMessageFormat => "Invalid email format".to_string(),
            Self::SendFailed(msg) => format!("Failed to send email: {}", msg),
            Self::ApiError(msg) => format!("Gmail error: {}", msg),
            Self::CacheError(_) => "Local cache error".to_string(),
            Self::NetworkError(_) => "Network error. Check your connection.".to_string(),
        }
    }

    /// Whether this error should trigger a token refresh.
    pub fn should_refresh_token(&self) -> bool {
        matches!(self, Self::TokenExpired | Self::AuthRequired)
    }

    /// Whether this error is retryable.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited(_) | Self::NetworkError(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_user_messages() {
        let err = GmailError::AuthRequired;
        assert!(err.user_message().contains("sign in"));

        let err = GmailError::RateLimited(30);
        assert!(err.user_message().contains("30"));
    }

    #[test]
    fn test_should_refresh_token() {
        assert!(GmailError::TokenExpired.should_refresh_token());
        assert!(GmailError::AuthRequired.should_refresh_token());
        assert!(!GmailError::MessageNotFound("x".into()).should_refresh_token());
    }

    #[test]
    fn test_is_retryable() {
        assert!(GmailError::RateLimited(10).is_retryable());
        assert!(!GmailError::MessageNotFound("x".into()).is_retryable());
    }
}
```

**Step 3: Create lib.rs**

Create `crates/myme-gmail/src/lib.rs`:

```rust
//! Gmail integration for MyMe.
//!
//! Provides Gmail API client, offline caching, and sync queue.

pub mod error;

pub use error::GmailError;

// TODO: Add in subsequent tasks
// pub mod types;
// pub mod client;
// pub mod cache;
// pub mod sync;
```

**Step 4: Add to workspace**

Add to root `Cargo.toml` members:

```toml
members = [
    "crates/myme-core",
    "crates/myme-ui",
    "crates/myme-services",
    "crates/myme-auth",
    "crates/myme-integrations",
    "crates/myme-weather",
    "crates/myme-gmail",  # Add this
]
```

**Step 5: Verify build**

Run: `cargo build -p myme-gmail`
Expected: Build succeeds

**Step 6: Run tests**

Run: `cargo test -p myme-gmail`
Expected: PASS (3 tests in error.rs)

**Step 7: Commit**

```bash
git add crates/myme-gmail/ Cargo.toml
git commit -m "feat(gmail): create myme-gmail crate with error types

- Add GmailError enum with user-friendly messages
- Add helper methods: should_refresh_token, is_retryable
- Set up crate structure with dependencies"
```

---

### Task 2.2: Gmail Types

**Files:**
- Create: `crates/myme-gmail/src/types.rs`
- Modify: `crates/myme-gmail/src/lib.rs`

**Step 1: Write the failing test**

Create `crates/myme-gmail/src/types.rs` with tests first:

```rust
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
}
```

**Step 2: Write implementation**

```rust
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
pub struct MessagePart {
    #[serde(rename = "mimeType")]
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
            .map(|ms| DateTime::from_timestamp_millis(ms).unwrap_or_default())
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
        "INBOX", "SENT", "DRAFT", "TRASH", "SPAM", "STARRED", "IMPORTANT", "UNREAD",
        "CATEGORY_PERSONAL", "CATEGORY_SOCIAL", "CATEGORY_PROMOTIONS", "CATEGORY_UPDATES", "CATEGORY_FORUMS",
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
}
```

**Step 3: Update lib.rs**

```rust
pub mod error;
pub mod types;

pub use error::GmailError;
pub use types::{Label, LabelType, Message};
```

**Step 4: Run tests**

Run: `cargo test -p myme-gmail`
Expected: PASS (6 tests)

**Step 5: Commit**

```bash
git add crates/myme-gmail/src/
git commit -m "feat(gmail): add Gmail types and API response parsing

- Add Message, Label types with serde support
- Add API response types for Gmail REST API
- Implement Message::from_api for response conversion
- Parse headers, dates, labels correctly"
```

---

### Task 2.3: Gmail Client

**Files:**
- Create: `crates/myme-gmail/src/client.rs`
- Modify: `crates/myme-gmail/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

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
}
```

**Step 2: Write implementation**

Create `crates/myme-gmail/src/client.rs`:

```rust
//! Gmail API client with retry logic.

use anyhow::{Context, Result};
use reqwest::Client;
use tracing::instrument;

use crate::error::GmailError;
use crate::types::*;

const GMAIL_API_BASE: &str = "https://gmail.googleapis.com";

pub struct GmailClient {
    client: Client,
    access_token: String,
    base_url: String,
}

impl GmailClient {
    pub fn new(access_token: &str) -> Self {
        Self {
            client: Client::new(),
            access_token: access_token.to_string(),
            base_url: GMAIL_API_BASE.to_string(),
        }
    }

    #[cfg(test)]
    pub fn new_with_base_url(access_token: &str, base_url: &str) -> Self {
        Self {
            client: Client::new(),
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

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a single message with full details.
    #[instrument(skip(self), level = "info")]
    pub async fn get_message(&self, message_id: &str) -> Result<Message, GmailError> {
        let url = format!(
            "{}/gmail/v1/users/me/messages/{}?format=full",
            self.base_url, message_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        let api_msg: ApiMessage = self.handle_response(response).await?;
        Ok(Message::from_api(api_msg))
    }

    /// List all labels.
    #[instrument(skip(self), level = "info")]
    pub async fn list_labels(&self) -> Result<Vec<Label>, GmailError> {
        let url = format!("{}/gmail/v1/users/me/labels", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

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
        let url = format!(
            "{}/gmail/v1/users/me/messages/{}/modify",
            self.base_url, message_id
        );

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
        let url = format!(
            "{}/gmail/v1/users/me/messages/{}/trash",
            self.base_url, message_id
        );

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
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
        let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(raw_message.as_bytes());

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

// Need base64 engine
use base64::Engine;

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

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
            .respond_with(
                ResponseTemplate::new(429)
                    .append_header("Retry-After", "30")
            )
            .mount(&mock_server)
            .await;

        let client = GmailClient::new_with_base_url("token", &mock_server.uri());
        let result = client.list_message_ids(None, None).await;

        assert!(matches!(result, Err(GmailError::RateLimited(30))));
    }
}
```

**Step 3: Update lib.rs**

```rust
pub mod client;
pub mod error;
pub mod types;

pub use client::GmailClient;
pub use error::GmailError;
pub use types::{Label, LabelType, Message};
```

**Step 4: Run tests**

Run: `cargo test -p myme-gmail`
Expected: PASS (10+ tests)

**Step 5: Commit**

```bash
git add crates/myme-gmail/src/
git commit -m "feat(gmail): add Gmail API client with retry handling

- Add GmailClient with all core operations
- Implement list_message_ids, get_message, list_labels
- Implement modify_labels, trash_message, send_message
- Handle 401/403/404/429 status codes appropriately
- Add comprehensive mock tests"
```

---

## Phase 3: Gmail Cache & Sync

### Task 3.1: SQLite Cache

**Files:**
- Create: `crates/myme-gmail/src/cache.rs`
- Modify: `crates/myme-gmail/src/lib.rs`

(Detailed implementation following same TDD pattern...)

---

### Task 3.2: Sync Queue

**Files:**
- Create: `crates/myme-gmail/src/sync.rs`
- Modify: `crates/myme-gmail/src/lib.rs`

(Detailed implementation following same TDD pattern...)

---

## Phase 4: Calendar Crate

### Task 4.1: Create myme-calendar Crate Structure

(Mirror Phase 2 structure for Calendar...)

### Task 4.2: Calendar Types

### Task 4.3: Calendar Client

### Task 4.4: Calendar Cache & Sync

---

## Phase 5: UI Models

### Task 5.1: GoogleAuthModel (cxx-qt)

**Files:**
- Create: `crates/myme-ui/src/models/google_auth_model.rs`
- Modify: `crates/myme-ui/src/models/mod.rs`
- Modify: `crates/myme-ui/build.rs`

### Task 5.2: GmailModel (cxx-qt)

### Task 5.3: CalendarModel (cxx-qt)

---

## Phase 6: QML Pages

### Task 6.1: AccountCard Component

**Files:**
- Create: `crates/myme-ui/qml/components/AccountCard.qml`
- Modify: `crates/myme-ui/qml/components/qmldir`
- Modify: `qml.qrc`

### Task 6.2: Update SettingsPage (Connected Accounts)

**Files:**
- Modify: `crates/myme-ui/qml/pages/SettingsPage.qml`

### Task 6.3: GmailPage

**Files:**
- Create: `crates/myme-ui/qml/pages/GmailPage.qml`
- Modify: `qml.qrc`
- Modify: `crates/myme-ui/qml/Main.qml`

### Task 6.4: CalendarPage

**Files:**
- Create: `crates/myme-ui/qml/pages/CalendarPage.qml`
- Modify: `qml.qrc`
- Modify: `crates/myme-ui/qml/Main.qml`

---

## Phase 7: Home Page

### Task 7.1: EmailWidget Component

**Files:**
- Create: `crates/myme-ui/qml/components/EmailWidget.qml`
- Modify: `crates/myme-ui/qml/components/qmldir`
- Modify: `qml.qrc`

### Task 7.2: CalendarWidget Component

**Files:**
- Create: `crates/myme-ui/qml/components/CalendarWidget.qml`

### Task 7.3: HomePage

**Files:**
- Create: `crates/myme-ui/qml/pages/HomePage.qml`
- Modify: `crates/myme-ui/qml/Main.qml` (navigation, default page)

---

## Phase 8: Integration & Polish

### Task 8.1: Wire Up Background Sync

### Task 8.2: Add Icons to Icons.qml

### Task 8.3: End-to-End Testing

### Task 8.4: Update CLAUDE.md

---

## Test Commands Reference

```bash
# Unit tests by crate
cargo test -p myme-auth
cargo test -p myme-gmail
cargo test -p myme-calendar

# All tests (excludes myme-ui)
cargo test -p myme-core -p myme-services -p myme-auth -p myme-integrations -p myme-gmail -p myme-calendar

# Full build
cargo build --release

# Qt build (after cargo build)
cd build-qt && cmake --build . --config Release
```
