# Google Services Specification

Covers `crates/myme-gmail/` and `crates/myme-calendar/`.

## File Map

| File | Purpose |
|------|---------|
| `crates/myme-gmail/src/lib.rs` | Re-exports: `GmailClient`, `GmailCache`, `GmailError`, `SyncQueue`, `Message`, `Label` |
| `crates/myme-gmail/src/client.rs` | Gmail REST API client with auth and error handling |
| `crates/myme-gmail/src/cache.rs` | SQLite offline cache for messages and labels |
| `crates/myme-gmail/src/sync.rs` | Offline action queue (mark read, star, archive, etc.) |
| `crates/myme-gmail/src/types.rs` | `Message`, `Label`, `LabelType`, API response types |
| `crates/myme-gmail/src/error.rs` | `GmailError` enum with user messages and retry logic |
| `crates/myme-calendar/src/lib.rs` | Re-exports: `CalendarClient`, `CalendarCache`, `CalendarError`, `Event`, `Calendar` |
| `crates/myme-calendar/src/client.rs` | Google Calendar REST API client |
| `crates/myme-calendar/src/cache.rs` | SQLite offline cache for events and calendars |
| `crates/myme-calendar/src/types.rs` | `Event`, `Calendar`, `EventTime`, `Attendee`, API response types |
| `crates/myme-calendar/src/error.rs` | `CalendarError` enum with user messages and retry logic |

## Interface Signatures

### myme-gmail::client

```rust
pub struct GmailClient { client: reqwest::Client, access_token: String, base_url: String }
impl GmailClient {
    pub fn new(access_token: &str) -> Self;
    pub async fn list_message_ids(&self, query: Option<&str>, page_token: Option<&str>) -> Result<MessageListResponse, GmailError>;
    pub async fn get_message(&self, message_id: &str) -> Result<Message, GmailError>;
    pub async fn list_labels(&self) -> Result<Vec<Label>, GmailError>;
    pub async fn modify_labels(&self, message_id: &str, add_labels: &[&str], remove_labels: &[&str]) -> Result<(), GmailError>;
    pub async fn trash_message(&self, message_id: &str) -> Result<(), GmailError>;
    pub async fn send_message(&self, to: &str, subject: &str, body: &str, reply_to_id: Option<&str>) -> Result<Message, GmailError>;
    pub async fn mark_as_read(&self, message_id: &str) -> Result<(), GmailError>;
    pub async fn mark_as_unread(&self, message_id: &str) -> Result<(), GmailError>;
    pub async fn star_message(&self, message_id: &str) -> Result<(), GmailError>;
    pub async fn unstar_message(&self, message_id: &str) -> Result<(), GmailError>;
    pub async fn archive_message(&self, message_id: &str) -> Result<(), GmailError>;
}
```

### myme-gmail::types

```rust
pub struct Message {
    pub id: String, pub thread_id: String, pub from: String,
    pub to: Vec<String>, pub subject: String, pub snippet: String,
    pub date: DateTime<Utc>, pub labels: Vec<String>,
    pub is_unread: bool, pub is_starred: bool, pub body: Option<String>,
}
impl Message {
    pub fn from_api(api: ApiMessage) -> Self;  // extracts headers, parses internal_date
}

pub struct Label {
    pub id: String, pub name: String, pub label_type: LabelType,
    pub messages_total: Option<u32>, pub messages_unread: Option<u32>,
}
impl Label {
    pub fn is_system_label(id: &str) -> bool;  // INBOX, SENT, DRAFT, TRASH, SPAM, STARRED, etc.
}

pub enum LabelType { System, User }

pub struct MessageListResponse {
    pub messages: Vec<MessageRef>,
    pub next_page_token: Option<String>,
    pub result_size_estimate: Option<u32>,
}

pub struct MessageRef { pub id: String, pub thread_id: String }
```

### myme-gmail::cache

```rust
pub struct GmailCache { conn: Connection }
impl GmailCache {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self>;
    pub fn store_message(&self, msg: &Message) -> Result<()>;
    pub fn get_message(&self, id: &str) -> Result<Option<Message>>;
    pub fn list_messages(&self, label: Option<&str>, limit: u32) -> Result<Vec<Message>>;
    pub fn delete_message(&self, id: &str) -> Result<()>;
    pub fn store_label(&self, label: &Label) -> Result<()>;
    pub fn list_labels(&self) -> Result<Vec<Label>>;
    pub fn unread_count(&self) -> Result<u32>;
    pub fn get_last_sync(&self) -> Result<Option<i64>>;
    pub fn set_last_sync(&self, timestamp: i64) -> Result<()>;
    pub fn clear(&self) -> Result<()>;
}
```

### myme-gmail::sync

```rust
pub enum SyncAction {
    MarkRead { message_id: String },
    MarkUnread { message_id: String },
    Star { message_id: String },
    Unstar { message_id: String },
    Archive { message_id: String },
    Trash { message_id: String },
    AddLabels { message_id: String, labels: Vec<String> },
    RemoveLabels { message_id: String, labels: Vec<String> },
}

pub struct QueuedAction { pub id: i64, pub action: SyncAction, pub created_at: i64, pub attempts: u32, pub last_error: Option<String> }

pub struct SyncQueue { conn: Connection }
impl SyncQueue {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self>;
    pub fn enqueue(&self, action: SyncAction) -> Result<i64>;
    pub fn peek(&self) -> Result<Option<QueuedAction>>;     // FIFO order
    pub fn list_pending(&self) -> Result<Vec<QueuedAction>>;
    pub fn complete(&self, id: i64) -> Result<()>;
    pub fn record_failure(&self, id: i64, error: &str) -> Result<()>;
    pub fn remove_failed(&self, max_attempts: u32) -> Result<u32>;
    pub fn pending_count(&self) -> Result<u32>;
    pub fn clear(&self) -> Result<()>;
    pub fn has_pending_for_message(&self, message_id: &str) -> Result<bool>;
}
```

### myme-gmail::error

```rust
pub enum GmailError {
    AuthRequired, TokenExpired, RateLimited(u64),
    MessageNotFound(String), LabelNotFound(String),
    InvalidMessageFormat, SendFailed(String),
    ApiError(String), CacheError(String), NetworkError(reqwest::Error),
}
impl GmailError {
    pub fn user_message(&self) -> String;
    pub fn should_refresh_token(&self) -> bool;  // TokenExpired | AuthRequired
    pub fn is_retryable(&self) -> bool;           // RateLimited | NetworkError
}
```

### myme-calendar::client

```rust
pub struct CalendarClient { client: reqwest::Client, access_token: String, base_url: String }
impl CalendarClient {
    pub fn new(access_token: &str) -> Self;
    pub async fn list_calendars(&self) -> Result<Vec<Calendar>, CalendarError>;
    pub async fn list_events(&self, calendar_id: &str, time_min: DateTime<Utc>, time_max: DateTime<Utc>, page_token: Option<&str>) -> Result<EventListResponse, CalendarError>;
    pub async fn get_event(&self, calendar_id: &str, event_id: &str) -> Result<Event, CalendarError>;
    pub async fn create_event(&self, calendar_id: &str, summary: &str, start: DateTime<Utc>, end: DateTime<Utc>, description: Option<&str>, location: Option<&str>) -> Result<Event, CalendarError>;
    pub async fn update_event(&self, calendar_id: &str, event_id: &str, summary: Option<&str>, start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>, description: Option<&str>, location: Option<&str>) -> Result<Event, CalendarError>;
    pub async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<(), CalendarError>;
    pub async fn quick_add(&self, calendar_id: &str, text: &str) -> Result<Event, CalendarError>;
}
```

### myme-calendar::types

```rust
pub struct Event {
    pub id: String, pub calendar_id: String, pub summary: String,
    pub description: Option<String>, pub location: Option<String>,
    pub start: EventTime, pub end: EventTime, pub all_day: bool,
    pub attendees: Vec<Attendee>, pub organizer: Option<String>,
    pub status: EventStatus, pub html_link: Option<String>, pub etag: Option<String>,
}
impl Event {
    pub fn from_api(api: ApiEvent, calendar_id: &str) -> Self;
}

pub enum EventTime { DateTime(DateTime<Utc>), Date(NaiveDate) }
impl EventTime {
    pub fn as_datetime(&self) -> DateTime<Utc>;  // Date -> midnight UTC
}

pub enum EventStatus { Confirmed, Tentative, Cancelled }
pub struct Attendee { pub email: String, pub display_name: Option<String>, pub response_status: ResponseStatus, pub is_organizer: bool }
pub enum ResponseStatus { NeedsAction, Declined, Tentative, Accepted }
pub enum AccessRole { Owner, Writer, Reader, FreeBusyReader }

pub struct Calendar {
    pub id: String, pub summary: String, pub description: Option<String>,
    pub time_zone: Option<String>, pub background_color: Option<String>,
    pub foreground_color: Option<String>, pub is_primary: bool, pub access_role: AccessRole,
}
```

### myme-calendar::cache

```rust
pub struct CalendarCache { conn: Connection }
impl CalendarCache {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self>;
    pub fn store_calendar(&self, calendar: &Calendar) -> Result<()>;
    pub fn list_calendars(&self) -> Result<Vec<Calendar>>;
    pub fn store_event(&self, event: &Event) -> Result<()>;
    pub fn get_event(&self, calendar_id: &str, event_id: &str) -> Result<Option<Event>>;
    pub fn list_events(&self, calendar_id: &str, time_min: DateTime<Utc>, time_max: DateTime<Utc>) -> Result<Vec<Event>>;
    pub fn get_today_events(&self, calendar_id: &str) -> Result<Vec<Event>>;
    pub fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()>;
    pub fn upcoming_event_count(&self, calendar_id: &str, hours: i64) -> Result<u32>;
    pub fn clear(&self) -> Result<()>;
}
```

### myme-calendar::error

```rust
pub enum CalendarError {
    AuthRequired, TokenExpired, RateLimited(u64),
    EventNotFound(String), CalendarNotFound(String),
    InvalidEventData(String), Conflict,
    ApiError(String), CacheError(String), NetworkError(reqwest::Error),
}
impl CalendarError {
    pub fn user_message(&self) -> String;
    pub fn should_refresh_token(&self) -> bool;
    pub fn is_retryable(&self) -> bool;
}
```

## Data Flow

### Gmail Message Fetch

1. `GmailClient::list_message_ids(query, page_token)` -> GET `/gmail/v1/users/me/messages?q=...&maxResults=50`
2. Returns `MessageListResponse` with `Vec<MessageRef>` (id + thread_id only)
3. For each message ref: `GmailClient::get_message(id)` -> GET `/gmail/v1/users/me/messages/{id}?format=full`
4. API returns `ApiMessage` with payload/headers; `Message::from_api()` extracts From, To, Subject from headers
5. `GmailCache::store_message(msg)` persists to SQLite for offline access

### Gmail Offline Sync Queue

1. User action while offline (e.g., mark as read): `SyncQueue::enqueue(SyncAction::MarkRead { message_id })`
2. When online: `SyncQueue::peek()` gets oldest action (FIFO)
3. Execute action via `GmailClient` (e.g., `mark_as_read()`)
4. On success: `SyncQueue::complete(id)` removes from queue
5. On failure: `SyncQueue::record_failure(id, error)` increments attempts
6. Periodic cleanup: `SyncQueue::remove_failed(max_attempts=3)` purges stuck actions

### Calendar Event Listing

1. `CalendarClient::list_events(calendar_id, time_min, time_max, page_token)`
2. GET `/calendars/{id}/events?timeMin=...&timeMax=...&singleEvents=true&orderBy=startTime&maxResults=50`
3. Returns `EventListResponse` with `Vec<ApiEvent>` + pagination token
4. `Event::from_api(api_event, calendar_id)` converts: parses RFC3339 datetimes or dates, maps attendee statuses
5. `CalendarCache::store_event(event)` persists for offline access

## Storage / Schema

### Gmail Cache (`~/.config/myme/gmail_cache.db`)

```sql
CREATE TABLE messages (
    id TEXT PRIMARY KEY, thread_id TEXT NOT NULL, from_addr TEXT NOT NULL,
    to_addrs TEXT NOT NULL,          -- JSON array of strings
    subject TEXT NOT NULL, snippet TEXT NOT NULL,
    date_ms INTEGER NOT NULL,        -- Unix milliseconds
    labels TEXT NOT NULL,            -- JSON array of label IDs
    is_unread INTEGER NOT NULL, is_starred INTEGER NOT NULL,
    body TEXT, cached_at INTEGER NOT NULL
);
CREATE TABLE labels (
    id TEXT PRIMARY KEY, name TEXT NOT NULL, label_type TEXT NOT NULL,
    messages_total INTEGER, messages_unread INTEGER, cached_at INTEGER NOT NULL
);
CREATE TABLE sync_state (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE INDEX idx_messages_date ON messages(date_ms DESC);
CREATE INDEX idx_messages_thread ON messages(thread_id);
CREATE INDEX idx_messages_unread ON messages(is_unread);
```

### Gmail Sync Queue (`~/.config/myme/gmail_sync.db`)

```sql
CREATE TABLE sync_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action_json TEXT NOT NULL,       -- JSON-serialized SyncAction
    created_at INTEGER NOT NULL,     -- Unix milliseconds
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT
);
CREATE INDEX idx_sync_queue_created ON sync_queue(created_at);
```

### Calendar Cache (`~/.config/myme/calendar_cache.db`)

```sql
CREATE TABLE calendars (
    id TEXT PRIMARY KEY, summary TEXT NOT NULL, description TEXT,
    time_zone TEXT, background_color TEXT, foreground_color TEXT,
    is_primary INTEGER NOT NULL, access_role TEXT NOT NULL, cached_at INTEGER NOT NULL
);
CREATE TABLE events (
    id TEXT NOT NULL, calendar_id TEXT NOT NULL,
    summary TEXT NOT NULL, description TEXT, location TEXT,
    start_ms INTEGER NOT NULL, end_ms INTEGER NOT NULL,
    all_day INTEGER NOT NULL,
    attendees_json TEXT NOT NULL,     -- JSON array of Attendee
    organizer TEXT, status TEXT NOT NULL,
    html_link TEXT, etag TEXT, cached_at INTEGER NOT NULL,
    PRIMARY KEY (id, calendar_id)
);
CREATE TABLE sync_state (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE INDEX idx_events_calendar ON events(calendar_id);
CREATE INDEX idx_events_start ON events(start_ms);
```

## Configuration

Both services use Google OAuth tokens stored in the system keyring under service name `"google"`. Configuration in `config.toml`:

```toml
[google]
client_id = "YOUR_CLIENT_ID.apps.googleusercontent.com"
client_secret = "YOUR_CLIENT_SECRET"
```

API base URLs:
- Gmail: `https://gmail.googleapis.com`
- Calendar: `https://www.googleapis.com/calendar/v3`

## Edge Cases

- **Token expired (401)**: Both clients return specific error (`GmailError::TokenExpired`, `CalendarError::TokenExpired`); `should_refresh_token()` returns true, triggering UI to refresh via `GoogleOAuth2Provider::refresh_token()`
- **Rate limited (429)**: Extracts `Retry-After` header (defaults to 60s); `is_retryable()` returns true
- **403 Forbidden**: Maps to `AuthRequired`, not retryable
- **404 Not Found**: Gmail returns `MessageNotFound`, Calendar returns `EventNotFound`
- **409 Conflict (Calendar only)**: `CalendarError::Conflict` when event was modified externally; UI should refresh
- **All-day events**: `EventTime::Date(NaiveDate)` vs `EventTime::DateTime(DateTime<Utc>)`; `as_datetime()` converts date to midnight UTC
- **System vs user labels**: Gmail labels with IDs like `INBOX`, `SENT`, `STARRED` are classified as `LabelType::System`; all others as `LabelType::User`
- **Message body not in list**: `list_message_ids()` returns only id/threadId; `get_message()` needed for full content
- **Sync queue FIFO**: Actions processed in creation order; failed actions stay in queue with incremented `attempts`
- **Cache label filtering**: `list_messages(label, limit)` uses SQL `LIKE` on JSON labels column; pattern `%"LABEL_ID"%`
- **Send message encoding**: Email body encoded as URL-safe base64 (RFC 2822 format) before sending
- **Calendar event creation**: Returns created event with server-assigned ID; `quick_add()` uses natural language parsing
