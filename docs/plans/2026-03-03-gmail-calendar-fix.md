# Gmail & Calendar Fix — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix Gmail and Calendar so auth auto-refreshes, reading is reliable (message bodies, incremental sync), and background sync keeps data fresh.

**Architecture:** Add a `GoogleApiClient` auth middleware in `myme-auth` that manages token lifecycle (proactive refresh, 401 retry). Refactor `GmailClient`/`CalendarClient` to use it instead of raw token strings. Add incremental sync (Gmail historyId, Calendar syncToken), message body extraction, and background sync timers in the UI bridge.

**Tech Stack:** Rust, reqwest, tokio, rusqlite, cxx-qt, QML, Google Gmail/Calendar REST APIs.

**Design doc:** `docs/plans/2026-03-03-gmail-calendar-fix-design.md`

**Test command:** `cargo test -p myme-core -p myme-auth -p myme-gmail -p myme-calendar`

---

## Task 1: GoogleApiClient — Auth Middleware

**Files:**
- Create: `crates/myme-auth/src/google_api_client.rs`
- Modify: `crates/myme-auth/src/lib.rs` (add module + re-export)
- Test: inline `#[cfg(test)]` module

This is the foundation. A shared struct that manages Google OAuth token lifecycle: proactive refresh (5-min buffer), forced refresh on 401, and keyring persistence.

### Step 1: Write failing tests

Create `crates/myme-auth/src/google_api_client.rs` with test module and struct skeleton:

```rust
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::storage::{SecureStorage, TokenSet};

const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Error)]
pub enum GoogleApiError {
    #[error("No refresh token available")]
    NoRefreshToken,
    #[error("Token refresh failed: {0}")]
    RefreshFailed(String),
    #[error("Not authenticated")]
    NotAuthenticated,
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
}

struct TokenState {
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
}

impl TokenState {
    fn needs_refresh(&self) -> bool {
        Utc::now() >= self.expires_at - Duration::minutes(5)
    }
}

pub struct GoogleApiClient {
    http: Client,
    client_id: String,
    client_secret: String,
    token_state: RwLock<TokenState>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_state_needs_refresh_when_expiring_soon() {
        let state = TokenState {
            access_token: "tok".into(),
            refresh_token: "ref".into(),
            expires_at: Utc::now() + Duration::minutes(3),
        };
        assert!(state.needs_refresh());
    }

    #[test]
    fn test_token_state_no_refresh_when_fresh() {
        let state = TokenState {
            access_token: "tok".into(),
            refresh_token: "ref".into(),
            expires_at: Utc::now() + Duration::hours(1),
        };
        assert!(!state.needs_refresh());
    }

    #[tokio::test]
    async fn test_access_token_returns_valid_token() {
        let api = GoogleApiClient::new_for_test("test-token");
        let token = api.access_token().await.unwrap();
        assert_eq!(token, "test-token");
    }

    #[tokio::test]
    async fn test_access_token_detects_expired() {
        let api = GoogleApiClient::new_expired_for_test("old-token", "refresh-tok");
        // access_token() will try to refresh — no mock server, so it should fail
        let result = api.access_token().await;
        assert!(result.is_err());
    }
}
```

### Step 2: Run tests — verify they fail

```bash
cargo test -p myme-auth
```

Expected: compilation errors (methods not implemented yet).

### Step 3: Implement GoogleApiClient

Add to `google_api_client.rs` above the test module:

```rust
impl GoogleApiClient {
    /// Create from an existing TokenSet (typically loaded from keyring)
    pub fn from_token_set(
        client_id: String,
        client_secret: String,
        token_set: &TokenSet,
    ) -> Result<Self, GoogleApiError> {
        let refresh_token = token_set.refresh_token.clone()
            .ok_or(GoogleApiError::NoRefreshToken)?;
        Ok(Self {
            http: Client::new(),
            client_id,
            client_secret,
            token_state: RwLock::new(TokenState {
                access_token: token_set.access_token.clone(),
                refresh_token,
                expires_at: DateTime::from_timestamp(token_set.expires_at, 0)
                    .unwrap_or_else(Utc::now),
            }),
        })
    }

    /// Get a valid access token, refreshing proactively if within 5 min of expiry.
    pub async fn access_token(&self) -> Result<String, GoogleApiError> {
        {
            let state = self.token_state.read().await;
            if !state.needs_refresh() {
                return Ok(state.access_token.clone());
            }
        }
        self.refresh().await
    }

    /// Force a token refresh (call after receiving 401).
    pub async fn force_refresh(&self) -> Result<String, GoogleApiError> {
        self.refresh().await
    }

    /// Get the shared HTTP client for making API requests.
    pub fn http(&self) -> &Client {
        &self.http
    }

    async fn refresh(&self) -> Result<String, GoogleApiError> {
        let mut state = self.token_state.write().await;
        // Double-check: another task may have refreshed while we waited for write lock
        if !state.needs_refresh() {
            return Ok(state.access_token.clone());
        }

        let response = self.http
            .post(GOOGLE_TOKEN_URL)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("refresh_token", state.refresh_token.as_str()),
                ("grant_type", "refresh_token"),
            ])
            .send()
            .await
            .map_err(GoogleApiError::Network)?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(GoogleApiError::RefreshFailed(text));
        }

        let token_response: crate::google::GoogleTokenResponse = response.json().await
            .map_err(|e| GoogleApiError::RefreshFailed(e.to_string()))?;

        state.access_token = token_response.access_token.clone();
        state.expires_at = Utc::now() + Duration::seconds(token_response.expires_in as i64);

        // Persist to keyring (best-effort — don't fail the request if keyring write fails)
        let new_token_set = TokenSet {
            access_token: state.access_token.clone(),
            refresh_token: Some(state.refresh_token.clone()),
            expires_at: state.expires_at.timestamp(),
            scopes: vec![],
        };
        drop(state); // release lock before I/O
        let _ = SecureStorage.store_token("google", &new_token_set);

        Ok(new_token_set.access_token)
    }

    #[cfg(test)]
    pub fn new_for_test(access_token: &str) -> Self {
        Self {
            http: Client::new(),
            client_id: String::new(),
            client_secret: String::new(),
            token_state: RwLock::new(TokenState {
                access_token: access_token.to_string(),
                refresh_token: "test-refresh".to_string(),
                expires_at: Utc::now() + Duration::hours(1),
            }),
        }
    }

    #[cfg(test)]
    pub fn new_expired_for_test(access_token: &str, refresh_token: &str) -> Self {
        Self {
            http: Client::new(),
            client_id: "test-id".to_string(),
            client_secret: "test-secret".to_string(),
            token_state: RwLock::new(TokenState {
                access_token: access_token.to_string(),
                refresh_token: refresh_token.to_string(),
                expires_at: Utc::now() - Duration::hours(1), // already expired
            }),
        }
    }
}
```

### Step 4: Register module in lib.rs

In `crates/myme-auth/src/lib.rs`, add:

```rust
mod google_api_client;
pub use google_api_client::{GoogleApiClient, GoogleApiError};
```

### Step 5: Run tests — verify they pass

```bash
cargo test -p myme-auth
```

Expected: all 4 new tests pass. `test_access_token_detects_expired` returns error (no real Google server to refresh against).

### Step 6: Commit

```bash
git add crates/myme-auth/src/google_api_client.rs crates/myme-auth/src/lib.rs
git commit -m "feat(auth): add GoogleApiClient with proactive token refresh"
```

---

## Task 2: Default Credentials + Config

**Files:**
- Modify: `crates/myme-auth/src/google.rs:6-13` (add defaults)
- Modify: `crates/myme-core/src/config.rs:243-250` (add sync_interval_secs)

### Step 1: Add default credential constants

In `crates/myme-auth/src/google.rs`, add after existing constants (line ~13):

```rust
// Default Google Cloud project credentials for MyMe
// Users can override in ~/.config/myme/config.toml [google] section
pub const DEFAULT_GOOGLE_CLIENT_ID: &str = "PLACEHOLDER.apps.googleusercontent.com";
pub const DEFAULT_GOOGLE_CLIENT_SECRET: &str = "PLACEHOLDER";
```

Note: Replace `PLACEHOLDER` with actual values after creating the Google Cloud project. The code compiles with placeholders; auth will fail until real values are set.

Add a public helper to resolve credentials (config override > defaults):

```rust
impl GoogleOAuth2Provider {
    /// Create provider using config overrides or compiled defaults
    pub fn with_defaults(config_id: Option<&str>, config_secret: Option<&str>) -> Self {
        Self::new(
            config_id.filter(|s| !s.is_empty())
                .unwrap_or(DEFAULT_GOOGLE_CLIENT_ID),
            config_secret.filter(|s| !s.is_empty())
                .unwrap_or(DEFAULT_GOOGLE_CLIENT_SECRET),
        )
    }
}
```

### Step 2: Add sync_interval_secs to GoogleConfig

In `crates/myme-core/src/config.rs`, modify `GoogleConfig` (line ~243):

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GoogleConfig {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    /// Background sync interval in seconds. Default: 300 (5 minutes). 0 = disabled.
    #[serde(default = "default_sync_interval")]
    pub sync_interval_secs: u64,
}

fn default_sync_interval() -> u64 {
    300
}
```

### Step 3: Add test for credential resolution

In `crates/myme-auth/src/google.rs` test module, add:

```rust
#[test]
fn test_with_defaults_uses_config_override() {
    let provider = GoogleOAuth2Provider::with_defaults(
        Some("custom-id"), Some("custom-secret")
    );
    assert_eq!(provider.client_id, "custom-id");
}

#[test]
fn test_with_defaults_falls_back_to_compiled() {
    let provider = GoogleOAuth2Provider::with_defaults(None, None);
    assert_eq!(provider.client_id, DEFAULT_GOOGLE_CLIENT_ID);
}

#[test]
fn test_with_defaults_ignores_empty_strings() {
    let provider = GoogleOAuth2Provider::with_defaults(Some(""), Some(""));
    assert_eq!(provider.client_id, DEFAULT_GOOGLE_CLIENT_ID);
}
```

### Step 4: Run tests

```bash
cargo test -p myme-auth -p myme-core
```

### Step 5: Commit

```bash
git add crates/myme-auth/src/google.rs crates/myme-core/src/config.rs
git commit -m "feat(auth): add default Google credentials and sync_interval_secs config"
```

---

## Task 3: Gmail Message Body Extraction

**Files:**
- Modify: `crates/myme-gmail/src/types.rs:68-109` (from_api body extraction)
- Test: inline tests in types.rs

### Step 1: Write failing test

Add to `crates/myme-gmail/src/types.rs` test module:

```rust
#[test]
fn test_message_body_extracted_from_single_part() {
    let body_text = "Hello, this is a test email body.";
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(body_text.as_bytes());

    let api = ApiMessage {
        id: "msg1".into(),
        thread_id: "t1".into(),
        label_ids: vec![],
        snippet: "Hello".into(),
        internal_date: Some("1700000000000".into()),
        payload: Some(MessagePayload {
            headers: vec![
                Header { name: "From".into(), value: "alice@test.com".into() },
                Header { name: "Subject".into(), value: "Test".into() },
            ],
            body: Some(MessageBody { data: Some(encoded), size: None }),
            parts: vec![],
        }),
    };
    let msg = Message::from_api(api);
    assert_eq!(msg.body.as_deref(), Some(body_text));
}

#[test]
fn test_message_body_extracted_from_multipart_text_plain() {
    let body_text = "Plain text body.";
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(body_text.as_bytes());

    let api = ApiMessage {
        id: "msg2".into(),
        thread_id: "t2".into(),
        label_ids: vec![],
        snippet: "Plain".into(),
        internal_date: Some("1700000000000".into()),
        payload: Some(MessagePayload {
            headers: vec![
                Header { name: "From".into(), value: "bob@test.com".into() },
                Header { name: "Subject".into(), value: "Multi".into() },
            ],
            body: None,
            parts: vec![
                MessagePart {
                    mime_type: "text/plain".into(),
                    body: Some(MessageBody { data: Some(encoded.clone()), size: None }),
                    parts: vec![],
                },
                MessagePart {
                    mime_type: "text/html".into(),
                    body: Some(MessageBody { data: Some("html-data".into()), size: None }),
                    parts: vec![],
                },
            ],
        }),
    };
    let msg = Message::from_api(api);
    assert_eq!(msg.body.as_deref(), Some(body_text));
}

#[test]
fn test_message_body_falls_back_to_html() {
    let html = "<p>HTML only</p>";
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(html.as_bytes());

    let api = ApiMessage {
        id: "msg3".into(),
        thread_id: "t3".into(),
        label_ids: vec![],
        snippet: "HTML".into(),
        internal_date: Some("1700000000000".into()),
        payload: Some(MessagePayload {
            headers: vec![],
            body: None,
            parts: vec![
                MessagePart {
                    mime_type: "text/html".into(),
                    body: Some(MessageBody { data: Some(encoded), size: None }),
                    parts: vec![],
                },
            ],
        }),
    };
    let msg = Message::from_api(api);
    assert_eq!(msg.body.as_deref(), Some(html));
}
```

### Step 2: Run tests — verify they fail

```bash
cargo test -p myme-gmail -- test_message_body
```

Expected: FAIL — `body` is always `None`.

### Step 3: Implement body extraction

Add a helper function in `types.rs` (before `impl Message`):

```rust
use base64::Engine;

fn decode_base64url(data: &str) -> Option<String> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
}

fn extract_body_from_parts(parts: &[MessagePart], mime_type: &str) -> Option<String> {
    for part in parts {
        if part.mime_type == mime_type {
            if let Some(body) = &part.body {
                if let Some(data) = &body.data {
                    if let Some(decoded) = decode_base64url(data) {
                        return Some(decoded);
                    }
                }
            }
        }
        // Recurse into nested parts
        if let Some(found) = extract_body_from_parts(&part.parts, mime_type) {
            return Some(found);
        }
    }
    None
}

fn extract_body(payload: &MessagePayload) -> Option<String> {
    // 1. Try direct body (single-part messages)
    if let Some(body) = &payload.body {
        if let Some(data) = &body.data {
            if let Some(decoded) = decode_base64url(data) {
                return Some(decoded);
            }
        }
    }
    // 2. Try text/plain in parts
    if let Some(plain) = extract_body_from_parts(&payload.parts, "text/plain") {
        return Some(plain);
    }
    // 3. Fall back to text/html
    extract_body_from_parts(&payload.parts, "text/html")
}
```

Then update `Message::from_api()` — change `body: None` to:

```rust
body: api.payload.as_ref().and_then(extract_body),
```

### Step 4: Run tests — verify they pass

```bash
cargo test -p myme-gmail
```

Expected: all existing tests + 3 new body tests pass.

### Step 5: Commit

```bash
git add crates/myme-gmail/src/types.rs
git commit -m "feat(gmail): extract message body from MIME parts"
```

---

## Task 4: Gmail Incremental Sync (History API)

**Files:**
- Modify: `crates/myme-gmail/src/types.rs` (add history types, historyId to MessageListResponse)
- Modify: `crates/myme-gmail/src/client.rs` (add `list_history` method)
- Modify: `crates/myme-gmail/src/cache.rs` (add `get_history_id`/`set_history_id`)
- Modify: `crates/myme-gmail/src/error.rs` (add `HistoryExpired` variant)

### Step 1: Add history API types

In `crates/myme-gmail/src/types.rs`, add:

```rust
/// Response from Gmail messages.list — add historyId field
// Modify existing MessageListResponse to include history_id:
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageListResponse {
    #[serde(default)]
    pub messages: Vec<MessageRef>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u32>,
    pub history_id: Option<String>,  // NEW — latest historyId
}

/// History list response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryListResponse {
    #[serde(default)]
    pub history: Vec<HistoryRecord>,
    pub next_page_token: Option<String>,
    pub history_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryRecord {
    pub id: String,
    #[serde(default)]
    pub messages_added: Vec<HistoryMessage>,
    #[serde(default)]
    pub messages_deleted: Vec<HistoryMessage>,
    #[serde(default)]
    pub labels_added: Vec<HistoryLabelChange>,
    #[serde(default)]
    pub labels_removed: Vec<HistoryLabelChange>,
}

#[derive(Debug, Deserialize)]
pub struct HistoryMessage {
    pub message: MessageRef,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryLabelChange {
    pub message: MessageRef,
    pub label_ids: Vec<String>,
}
```

Check whether `MessageListResponse` already has a `history_id` field. If not, add it. The Gmail API returns `historyId` in message list responses.

### Step 2: Add `HistoryExpired` error variant

In `crates/myme-gmail/src/error.rs`, add:

```rust
#[error("History expired, full sync required")]
HistoryExpired,
```

And update `is_retryable()` to return `false` for it.

### Step 3: Add `list_history` to GmailClient

In `crates/myme-gmail/src/client.rs`, add:

```rust
pub async fn list_history(
    &self,
    start_history_id: &str,
    page_token: Option<&str>,
) -> Result<HistoryListResponse, GmailError> {
    let mut params = vec![
        format!("startHistoryId={}", start_history_id),
        "historyTypes=messageAdded,messageDeleted,labelAdded,labelRemoved".to_string(),
        "maxResults=100".to_string(),
    ];
    if let Some(pt) = page_token {
        params.push(format!("pageToken={}", pt));
    }
    let url = format!(
        "{}/gmail/v1/users/me/history?{}",
        self.base_url,
        params.join("&")
    );
    let response = self.client
        .get(&url)
        .header("Authorization", self.auth_header())
        .send()
        .await?;

    // 410 Gone = historyId too old
    if response.status().as_u16() == 410 {
        return Err(GmailError::HistoryExpired);
    }

    self.handle_response(response).await
}
```

### Step 4: Add history_id cache methods

In `crates/myme-gmail/src/cache.rs`, add:

```rust
pub fn get_history_id(&self) -> Option<String> {
    self.conn
        .query_row(
            "SELECT value FROM sync_state WHERE key = 'history_id'",
            [],
            |row| row.get(0),
        )
        .ok()
}

pub fn set_history_id(&self, history_id: &str) -> Result<()> {
    self.conn.execute(
        "INSERT OR REPLACE INTO sync_state (key, value) VALUES ('history_id', ?1)",
        [history_id],
    )?;
    Ok(())
}
```

### Step 5: Write tests

**Cache tests** in `cache.rs`:
```rust
#[test]
fn test_history_id_roundtrip() {
    let cache = GmailCache::in_memory().unwrap();
    assert!(cache.get_history_id().is_none());
    cache.set_history_id("12345").unwrap();
    assert_eq!(cache.get_history_id().unwrap(), "12345");
}
```

**Client test** in `client.rs`:
```rust
#[tokio::test]
async fn test_list_history() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/history"))
        .and(query_param("startHistoryId", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "history": [{
                "id": "101",
                "messagesAdded": [{"message": {"id": "m1", "threadId": "t1"}}],
                "messagesDeleted": [],
                "labelsAdded": [],
                "labelsRemoved": []
            }],
            "historyId": "102"
        })))
        .mount(&mock_server)
        .await;

    let client = GmailClient::new_with_base_url("token", &mock_server.uri());
    let result = client.list_history("100", None).await.unwrap();
    assert_eq!(result.history.len(), 1);
    assert_eq!(result.history_id, "102");
}

#[tokio::test]
async fn test_list_history_expired() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/gmail/v1/users/me/history"))
        .respond_with(ResponseTemplate::new(410))
        .mount(&mock_server)
        .await;

    let client = GmailClient::new_with_base_url("token", &mock_server.uri());
    let result = client.list_history("1", None).await;
    assert!(matches!(result, Err(GmailError::HistoryExpired)));
}
```

### Step 6: Run tests

```bash
cargo test -p myme-gmail
```

### Step 7: Commit

```bash
git add crates/myme-gmail/src/
git commit -m "feat(gmail): add history API for incremental sync"
```

---

## Task 5: Calendar Incremental Sync (syncToken)

**Files:**
- Modify: `crates/myme-calendar/src/client.rs:52-76` (add sync_token param to list_events)
- Modify: `crates/myme-calendar/src/cache.rs` (add get/set_sync_token)
- Modify: `crates/myme-calendar/src/error.rs` (add SyncTokenExpired)

### Step 1: Add SyncTokenExpired error

In `crates/myme-calendar/src/error.rs`:

```rust
#[error("Sync token expired, full sync required")]
SyncTokenExpired,
```

### Step 2: Modify list_events signature

In `crates/myme-calendar/src/client.rs`, change `list_events` to accept an optional sync token. When `sync_token` is present, omit `timeMin`/`timeMax`/`singleEvents`/`orderBy` (Google rejects them with syncToken):

```rust
pub async fn list_events(
    &self,
    calendar_id: &str,
    time_min: Option<DateTime<Utc>>,
    time_max: Option<DateTime<Utc>>,
    sync_token: Option<&str>,
    page_token: Option<&str>,
) -> Result<EventListResponse, CalendarError> {
    let mut params = vec!["maxResults=50".to_string()];

    if let Some(st) = sync_token {
        // Incremental sync — no time bounds allowed
        params.push(format!("syncToken={}", st));
    } else {
        // Full sync
        if let Some(min) = time_min {
            params.push(format!("timeMin={}", min.to_rfc3339()));
        }
        if let Some(max) = time_max {
            params.push(format!("timeMax={}", max.to_rfc3339()));
        }
        params.push("singleEvents=true".to_string());
        params.push("orderBy=startTime".to_string());
    }

    if let Some(pt) = page_token {
        params.push(format!("pageToken={}", pt));
    }

    let url = format!(
        "{}/calendars/{}/events?{}",
        self.base_url,
        urlencoding::encode(calendar_id),
        params.join("&")
    );
    let response = self.client
        .get(&url)
        .header("Authorization", self.auth_header())
        .send()
        .await?;

    if response.status().as_u16() == 410 {
        return Err(CalendarError::SyncTokenExpired);
    }

    self.handle_response(response).await
}
```

**Important:** Update ALL existing callers of `list_events` to pass the new `sync_token` param as `None`. This includes:
- `crates/myme-ui/src/services/calendar_service.rs` (line ~62)
- Any existing tests in `client.rs`

### Step 3: Add sync_token cache methods

In `crates/myme-calendar/src/cache.rs`:

```rust
pub fn get_sync_token(&self, calendar_id: &str) -> Option<String> {
    let key = format!("sync_token:{}", calendar_id);
    self.conn
        .query_row(
            "SELECT value FROM sync_state WHERE key = ?1",
            [&key],
            |row| row.get(0),
        )
        .ok()
}

pub fn set_sync_token(&self, calendar_id: &str, token: &str) -> Result<()> {
    let key = format!("sync_token:{}", calendar_id);
    self.conn.execute(
        "INSERT OR REPLACE INTO sync_state (key, value) VALUES (?1, ?2)",
        [&key, token],
    )?;
    Ok(())
}
```

### Step 4: Write tests

**Cache test:**
```rust
#[test]
fn test_sync_token_roundtrip() {
    let cache = CalendarCache::in_memory().unwrap();
    assert!(cache.get_sync_token("primary").is_none());
    cache.set_sync_token("primary", "token123").unwrap();
    assert_eq!(cache.get_sync_token("primary").unwrap(), "token123");
}

#[test]
fn test_sync_token_per_calendar() {
    let cache = CalendarCache::in_memory().unwrap();
    cache.set_sync_token("primary", "tok1").unwrap();
    cache.set_sync_token("work", "tok2").unwrap();
    assert_eq!(cache.get_sync_token("primary").unwrap(), "tok1");
    assert_eq!(cache.get_sync_token("work").unwrap(), "tok2");
}
```

**Client test:**
```rust
#[tokio::test]
async fn test_list_events_with_sync_token() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(r"/calendars/.*/events"))
        .and(query_param("syncToken", "abc123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [],
            "nextSyncToken": "abc124"
        })))
        .mount(&mock_server)
        .await;

    let client = CalendarClient::new_with_base_url("token", &mock_server.uri());
    let result = client.list_events("primary", None, None, Some("abc123"), None).await.unwrap();
    assert_eq!(result.next_sync_token.unwrap(), "abc124");
}

#[tokio::test]
async fn test_list_events_sync_token_expired() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path_regex(r"/calendars/.*/events"))
        .respond_with(ResponseTemplate::new(410))
        .mount(&mock_server)
        .await;

    let client = CalendarClient::new_with_base_url("token", &mock_server.uri());
    let result = client.list_events("primary", None, None, Some("old"), None).await;
    assert!(matches!(result, Err(CalendarError::SyncTokenExpired)));
}
```

### Step 5: Fix existing tests

Update any existing `list_events` test calls to include `None` for the new `sync_token` parameter. Search for `list_events(` in the calendar test files.

### Step 6: Run tests

```bash
cargo test -p myme-calendar
```

### Step 7: Commit

```bash
git add crates/myme-calendar/src/
git commit -m "feat(calendar): add syncToken for incremental sync"
```

---

## Task 6: Refactor GmailClient to Use GoogleApiClient

**Files:**
- Modify: `crates/myme-gmail/src/client.rs` (replace access_token with Arc\<GoogleApiClient\>)
- Modify: `crates/myme-gmail/src/lib.rs` (if exports change)

### Step 1: Change GmailClient struct

Replace the struct fields (lines 11-15):

```rust
use std::sync::Arc;
use myme_auth::GoogleApiClient;

pub struct GmailClient {
    api: Arc<GoogleApiClient>,
    base_url: String,
}
```

### Step 2: Add execute helper for 401 retry

```rust
impl GmailClient {
    /// Execute a request with automatic 401 retry after token refresh.
    async fn execute<F>(&self, build: F) -> Result<reqwest::Response, GmailError>
    where
        F: Fn(&reqwest::Client, &str) -> reqwest::RequestBuilder,
    {
        let token = self.api.access_token().await
            .map_err(|e| GmailError::ApiError(e.to_string()))?;
        let auth = format!("Bearer {}", token);

        let response = build(self.api.http(), &auth)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            let token = self.api.force_refresh().await
                .map_err(|_| GmailError::TokenExpired)?;
            let auth = format!("Bearer {}", token);
            return build(self.api.http(), &auth)
                .send()
                .await
                .map_err(Into::into);
        }

        Ok(response)
    }
}
```

### Step 3: Update constructors

```rust
impl GmailClient {
    pub fn new(api: Arc<GoogleApiClient>) -> Self {
        Self {
            api,
            base_url: GMAIL_API_BASE.to_string(),
        }
    }

    #[cfg(test)]
    pub fn new_with_base_url(api: Arc<GoogleApiClient>, base_url: &str) -> Self {
        Self {
            api,
            base_url: base_url.to_string(),
        }
    }
}
```

### Step 4: Update each method to use execute()

Example for `list_message_ids`:

```rust
pub async fn list_message_ids(
    &self,
    query: Option<&str>,
    page_token: Option<&str>,
) -> Result<MessageListResponse, GmailError> {
    let mut params = vec!["maxResults=50".to_string()];
    if let Some(q) = query {
        params.push(format!("q={}", urlencoding::encode(q)));
    }
    if let Some(pt) = page_token {
        params.push(format!("pageToken={}", pt));
    }
    let url = format!(
        "{}/gmail/v1/users/me/messages?{}",
        self.base_url,
        params.join("&")
    );

    let response = self.execute(|http, auth| {
        http.get(&url).header("Authorization", auth)
    }).await?;

    self.handle_response(response).await
}
```

Apply the same pattern to: `get_message`, `list_labels`, `modify_labels`, `trash_message`, `send_message`, `list_history`. For POST methods, the closure captures the body:

```rust
// Example for modify_labels
let body = serde_json::json!({ "addLabelIds": add, "removeLabelIds": remove });
let response = self.execute(|http, auth| {
    http.post(&url)
        .header("Authorization", auth)
        .json(&body)
}).await?;
```

Remove the old `auth_header()` method — it's replaced by `execute()`.

### Step 5: Update tests to use new_for_test

All tests create `GmailClient::new_with_base_url("token", &mock_server.uri())`. Change to:

```rust
let api = Arc::new(GoogleApiClient::new_for_test("token"));
let client = GmailClient::new_with_base_url(api, &mock_server.uri());
```

Add `use std::sync::Arc;` and `use myme_auth::GoogleApiClient;` to test imports.

### Step 6: Run tests

```bash
cargo test -p myme-gmail
```

### Step 7: Commit

```bash
git add crates/myme-gmail/src/
git commit -m "refactor(gmail): use GoogleApiClient for automatic token refresh"
```

---

## Task 7: Refactor CalendarClient to Use GoogleApiClient

**Files:**
- Modify: `crates/myme-calendar/src/client.rs`

Exact same pattern as Task 6 but for CalendarClient. Steps:

### Step 1: Change struct to use Arc\<GoogleApiClient\>

Replace `client: reqwest::Client` + `access_token: String` with `api: Arc<GoogleApiClient>`.

### Step 2: Add execute() helper (same pattern as Gmail)

### Step 3: Update constructors

### Step 4: Update each method to use execute()

Methods: `list_calendars`, `list_events`, `get_event`, `create_event`, `update_event`, `delete_event`, `quick_add`.

### Step 5: Update tests

Same pattern: `Arc::new(GoogleApiClient::new_for_test("token"))`.

### Step 6: Run tests

```bash
cargo test -p myme-calendar
```

### Step 7: Commit

```bash
git add crates/myme-calendar/src/
git commit -m "refactor(calendar): use GoogleApiClient for automatic token refresh"
```

---

## Task 8: UI Bridge — GoogleApiClient Integration + Background Sync

**Files:**
- Modify: `crates/myme-ui/src/services/google_common.rs` (replace get_google_access_token with create_google_api_client)
- Modify: `crates/myme-ui/src/services/gmail_service.rs` (accept Arc\<GoogleApiClient\>)
- Modify: `crates/myme-ui/src/services/calendar_service.rs` (accept Arc\<GoogleApiClient\>)
- Modify: `crates/myme-ui/src/models/gmail_model.rs` (background sync, syncing/last_synced props)
- Modify: `crates/myme-ui/src/models/calendar_model.rs` (same)
- Modify: `crates/myme-ui/qml/pages/GmailPage.qml` (sync timer, status indicator)
- Modify: `crates/myme-ui/qml/pages/CalendarPage.qml` (same)

### Step 1: Update google_common.rs

Replace `get_google_access_token() -> Option<String>` with:

```rust
use std::sync::Arc;
use myme_auth::{GoogleApiClient, GoogleOAuth2Provider, SecureStorage};
use myme_auth::google::{DEFAULT_GOOGLE_CLIENT_ID, DEFAULT_GOOGLE_CLIENT_SECRET};
use myme_core::config::Config;

/// Create a GoogleApiClient from stored keyring credentials.
/// Returns None if not authenticated.
pub fn create_google_api_client() -> Option<Arc<GoogleApiClient>> {
    let token_set = SecureStorage.retrieve_token("google").ok()?;
    let (client_id, client_secret) = get_google_credentials();

    let api = GoogleApiClient::from_token_set(client_id, client_secret, &token_set).ok()?;
    Some(Arc::new(api))
}

/// Get sync interval from config (default 300s)
pub fn get_sync_interval_secs() -> u64 {
    Config::load().ok()
        .and_then(|c| c.google)
        .map(|g| g.sync_interval_secs)
        .unwrap_or(300)
}

fn get_google_credentials() -> (String, String) {
    let config = Config::load().ok();
    let google = config.and_then(|c| c.google);
    let client_id = google.as_ref()
        .and_then(|g| g.client_id.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_GOOGLE_CLIENT_ID.to_string());
    let client_secret = google.as_ref()
        .and_then(|g| g.client_secret.clone())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| DEFAULT_GOOGLE_CLIENT_SECRET.to_string());
    (client_id, client_secret)
}
```

This eliminates the `block_on()` call that was blocking the Qt thread during token refresh.

### Step 2: Update gmail_service.rs

Change `request_gmail_fetch` to accept `Arc<GoogleApiClient>` instead of `String`:

```rust
use std::sync::Arc;
use myme_auth::GoogleApiClient;

pub fn request_gmail_fetch(
    tx: &Sender<GmailServiceMessage>,
    api: Arc<GoogleApiClient>,
    cache_path: Option<String>,
) {
    let tx = tx.clone();
    let Some(runtime) = bridge::get_runtime() else { return };

    runtime.spawn(async move {
        let client = GmailClient::new(api);
        // ... rest unchanged, but remove take(20) limit
        let list = client.list_message_ids(Some("in:inbox"), None).await;
        // Fetch ALL message IDs from first page (up to 50), not just 20
        // ...
    });
}
```

Same for `request_mark_as_read`, `request_archive`, `request_trash`.

Also update `calendar_service.rs` with the same pattern.

### Step 3: Add syncing/last_synced properties to GmailModel

In `crates/myme-ui/src/models/gmail_model.rs`, add properties:

```rust
#[qproperty(bool, syncing)]        // true during background refresh
#[qproperty(QString, last_synced)] // "2 min ago" display string
#[qproperty(i32, sync_interval)]   // seconds, from config
```

Add to the Rust struct:
```rust
syncing: bool,
last_synced: QString,
sync_interval: i32,
last_synced_at: Option<DateTime<Utc>>,  // internal, not exposed to QML
```

Add a `background_sync` method that sets `syncing = true` (not `loading`) and triggers fetch.

Update `poll_channel` to handle results for both `loading` and `syncing` states:
```rust
pub fn poll_channel(&mut self) {
    if let Some(msg) = bridge::try_recv_gmail_message() {
        match msg {
            GmailServiceMessage::FetchDone(result) => {
                self.set_loading(false);
                self.set_syncing(false);
                match result {
                    Ok(messages) => {
                        // ... update messages, counts
                        self.last_synced_at = Some(Utc::now());
                        self.set_last_synced(QString::from("Just now"));
                    }
                    Err(e) => {
                        // Only show error banner during loading, not background sync
                        if self.loading() {
                            self.set_error_message(QString::from(&e));
                        }
                    }
                }
            }
            // ... rest unchanged
        }
    }

    // Update relative "last synced" timestamp
    if let Some(at) = self.last_synced_at {
        let ago = Utc::now() - at;
        let text = if ago.num_seconds() < 60 {
            "Just now".to_string()
        } else if ago.num_minutes() < 60 {
            format!("{} min ago", ago.num_minutes())
        } else {
            format!("{} hr ago", ago.num_hours())
        };
        self.set_last_synced(QString::from(&text));
    }
}
```

Initialize `sync_interval` from config in `check_auth`:
```rust
pub fn check_auth(&mut self) {
    // ... existing auth check
    let interval = google_common::get_sync_interval_secs();
    self.set_sync_interval(interval as i32);
}
```

Apply the same changes to `CalendarModel`.

### Step 4: Update fetch_messages to use GoogleApiClient

```rust
pub fn fetch_messages(&mut self) {
    let api = match google_common::create_google_api_client() {
        Some(api) => api,
        None => {
            self.set_error_message(QString::from("Not authenticated"));
            return;
        }
    };
    // ... init channel, get tx
    self.set_loading(true);
    request_gmail_fetch(&tx, api, cache_path);
}

pub fn background_sync(&mut self) {
    let api = match google_common::create_google_api_client() {
        Some(api) => api,
        None => return, // silent fail for background
    };
    // ... init channel, get tx
    self.set_syncing(true);
    request_gmail_fetch(&tx, api, cache_path);
}
```

### Step 5: Update GmailPage.qml

Add background sync timer and status indicator:

```qml
// Sync timer — fires at configurable interval
Timer {
    id: syncTimer
    interval: gmailModel.sync_interval * 1000
    running: gmailModel.authenticated && !gmailModel.loading && gmailModel.sync_interval > 0
    repeat: true
    onTriggered: gmailModel.background_sync()
}

// Update poll timer to also run during syncing
Timer {
    id: pollTimer
    interval: 100
    running: gmailModel.loading || gmailModel.syncing
    repeat: true
    onTriggered: gmailModel.poll_channel()
}
```

In the header area, add last-synced and sync indicator:

```qml
// After the Refresh button, add:
Label {
    text: gmailModel.last_synced
    font.pixelSize: 11
    color: Theme.textSecondary
    visible: gmailModel.last_synced !== ""
}

// Small sync indicator (pulsing dot)
Rectangle {
    width: 8
    height: 8
    radius: 4
    color: Theme.primary
    visible: gmailModel.syncing
    SequentialAnimation on opacity {
        running: gmailModel.syncing
        loops: Animation.Infinite
        NumberAnimation { from: 1.0; to: 0.3; duration: 600 }
        NumberAnimation { from: 0.3; to: 1.0; duration: 600 }
    }
}
```

Apply identical changes to `CalendarPage.qml`.

### Step 6: Build and test

```bash
cargo test -p myme-core -p myme-auth -p myme-gmail -p myme-calendar
task build  # Full build including Qt
```

Note: UI changes can't be unit-tested (requires Qt runtime). Manual verification by running the app.

### Step 7: Commit

```bash
git add crates/myme-ui/src/ crates/myme-ui/qml/pages/
git commit -m "feat(ui): wire GoogleApiClient and add background sync"
```

---

## Task Dependency Graph

```
Task 1 (GoogleApiClient)
  ├── Task 6 (Refactor GmailClient) ──┐
  └── Task 7 (Refactor CalendarClient) ├── Task 8 (UI Bridge + Sync)
                                        │
Task 2 (Default creds + config) ────────┘
Task 3 (Gmail body extraction)  — independent
Task 4 (Gmail history API)      — independent
Task 5 (Calendar syncToken)     — independent
```

**Parallel-safe groups:**
- Group A: Tasks 2, 3, 4, 5 (all independent)
- Group B: Tasks 6, 7 (both depend on Task 1, independent of each other)
- Group C: Task 8 (depends on Tasks 2, 6, 7)

## Post-Implementation

After all tasks pass:
1. **Set up Google Cloud project** — create project, enable Gmail + Calendar APIs, configure OAuth consent screen, get client_id/client_secret
2. **Replace PLACEHOLDER credentials** in `google.rs` with real values
3. **Manual QA** — `task run`, test Gmail/Calendar pages, verify auto-refresh works across token expiry
4. Use `superpowers:finishing-a-development-branch` to wrap up
