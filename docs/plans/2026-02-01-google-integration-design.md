# Google Gmail & Calendar Integration Design

**Date:** 2026-02-01
**Status:** Approved
**Phase:** 3 (Google Email/Calendar Integration)

## Overview

Add Gmail and Google Calendar integration to MyMe with:
- Full email client (inbox, search, compose, reply)
- Full calendar experience (view, create, edit, respond)
- Home page widgets for at-a-glance summaries
- Unified "Connected Accounts" in Settings

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Gmail scope | Read + write | Full client experience |
| Calendar scope | Read + write | Full calendar experience |
| Offline mode | Full with sync queue | Best UX, queue actions when offline |
| Sync frequency | Hybrid (10 min poll + manual) | Balance between freshness and API usage |
| Auth models | Separate per provider | Different OAuth flows, easier to maintain |
| Home widgets | Both email + calendar | Equal prominence, side by side |

---

## 1. Authentication & Account Management

### OAuth2 Flow for Google

- Reuse existing OAuth2 pattern from `myme-auth` (dynamic port discovery 8080-8089)
- Request scopes:
  - `https://www.googleapis.com/auth/gmail.modify`
  - `https://www.googleapis.com/auth/calendar`
- Store tokens in system keyring via `SecureStorage` (separate entry from GitHub)
- Auto-refresh tokens before API calls (Google tokens expire in 1 hour)

### New Rust Components

**`myme-auth/src/google.rs`:**
```rust
pub struct GoogleOAuth2Provider {
    client_id: String,
    client_secret: String,
    redirect_port: u16,
}

impl GoogleOAuth2Provider {
    pub async fn authenticate(&self) -> Result<TokenResponse>;
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse>;
    pub fn sign_out(&self) -> Result<()>;
}
```

**`SecureStorage` extension:**
- Namespace keys by provider: `myme.github.token`, `myme.google.token`
- Store both access token and refresh token for Google

---

## 2. Gmail Integration

### New Crate: `myme-gmail`

```
myme-gmail/
├── src/
│   ├── lib.rs
│   ├── client.rs      # Gmail API client with retry logic
│   ├── types.rs       # Message, Thread, Label, Draft structs
│   ├── cache.rs       # SQLite local cache
│   ├── sync.rs        # Sync queue for offline actions
│   └── error.rs       # Unified error enum
```

### Core API Operations

```rust
impl GmailClient {
    pub async fn list_messages(&self, query: Option<&str>, page_token: Option<&str>) -> Result<MessageList>;
    pub async fn get_message(&self, id: &str) -> Result<Message>;
    pub async fn send_message(&self, to: &str, subject: &str, body: &str, reply_to: Option<&str>) -> Result<Message>;
    pub async fn modify_labels(&self, id: &str, add: &[&str], remove: &[&str]) -> Result<()>;
    pub async fn trash_message(&self, id: &str) -> Result<()>;
    pub async fn delete_message(&self, id: &str) -> Result<()>;
    pub async fn list_labels(&self) -> Result<Vec<Label>>;
}
```

### SQLite Schema

```sql
-- Messages cache
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    thread_id TEXT NOT NULL,
    snippet TEXT,
    sender TEXT NOT NULL,
    recipients TEXT NOT NULL,  -- JSON array
    subject TEXT,
    date INTEGER NOT NULL,     -- Unix timestamp
    labels TEXT NOT NULL,      -- JSON array
    body_cached TEXT,
    is_read INTEGER NOT NULL DEFAULT 0,
    is_starred INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_messages_date ON messages(date DESC);
CREATE INDEX idx_messages_thread ON messages(thread_id);

-- Offline action queue
CREATE TABLE sync_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action_type TEXT NOT NULL,  -- 'send', 'archive', 'trash', 'mark_read', etc.
    payload_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0
);

-- Sync metadata
CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Keys: 'last_history_id', 'last_full_sync'
```

### Sync Strategy

1. **Initial sync:** Fetch last 500 messages, store `historyId`
2. **Incremental sync:** Use `history.list` with stored `historyId`
3. **History expired fallback:** On 404, trigger full resync
4. **Poll interval:** Every 10 minutes when app is open
5. **Offline queue:** Process pending actions on reconnect with exponential backoff

---

## 3. Google Calendar Integration

### New Crate: `myme-calendar`

```
myme-calendar/
├── src/
│   ├── lib.rs
│   ├── client.rs      # Calendar API client with retry logic
│   ├── types.rs       # Event, Calendar, Attendee, Recurrence
│   ├── cache.rs       # SQLite local cache
│   ├── sync.rs        # Sync queue for offline actions
│   └── error.rs       # Unified error enum
```

### Core API Operations

```rust
impl CalendarClient {
    pub async fn list_calendars(&self) -> Result<Vec<Calendar>>;
    pub async fn list_events(&self, calendar_id: &str, time_min: DateTime, time_max: DateTime) -> Result<EventList>;
    pub async fn list_events_incremental(&self, calendar_id: &str, sync_token: &str) -> Result<EventList>;
    pub async fn get_event(&self, calendar_id: &str, event_id: &str) -> Result<Event>;
    pub async fn create_event(&self, calendar_id: &str, event: &NewEvent) -> Result<Event>;
    pub async fn update_event(&self, calendar_id: &str, event_id: &str, event: &Event) -> Result<Event>;
    pub async fn delete_event(&self, calendar_id: &str, event_id: &str) -> Result<()>;
    pub async fn respond_to_event(&self, calendar_id: &str, event_id: &str, response: ResponseStatus) -> Result<()>;
}
```

### SQLite Schema

```sql
-- Calendars
CREATE TABLE calendars (
    id TEXT PRIMARY KEY,
    summary TEXT NOT NULL,
    color TEXT,
    is_primary INTEGER NOT NULL DEFAULT 0,
    access_role TEXT NOT NULL  -- 'owner', 'writer', 'reader'
);

-- Events cache
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    calendar_id TEXT NOT NULL,
    summary TEXT,
    description TEXT,
    location TEXT,
    start_time INTEGER,        -- Unix timestamp (NULL for all-day)
    end_time INTEGER,
    is_all_day INTEGER NOT NULL DEFAULT 0,
    recurrence_rule TEXT,      -- RRULE string
    attendees_json TEXT,
    meet_link TEXT,
    status TEXT NOT NULL,      -- 'confirmed', 'tentative', 'cancelled'
    etag TEXT NOT NULL,        -- For conflict detection
    updated_at INTEGER NOT NULL,
    FOREIGN KEY (calendar_id) REFERENCES calendars(id)
);

CREATE INDEX idx_events_calendar ON events(calendar_id);
CREATE INDEX idx_events_start ON events(start_time);

-- Offline action queue
CREATE TABLE sync_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    action_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    retry_count INTEGER NOT NULL DEFAULT 0
);

-- Sync metadata
CREATE TABLE metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
-- Keys: 'sync_token', 'last_full_sync'
```

### Sync Strategy

1. **Initial sync:** Fetch 3 months (1 past + 2 future) with `singleEvents=true`, store `syncToken`
2. **Incremental sync:** Use only `syncToken` parameter (no filters allowed)
3. **Token expired fallback:** On 410 Gone, trigger full resync
4. **Conflict detection:** Use `etag` on updates; handle 412 Precondition Failed
5. **Poll interval:** Every 10 minutes when app is open

---

## 4. QML Pages & Rust Models

### New Rust Models

**`myme-ui/src/models/gmail_model.rs`:**
```rust
#[cxx_qt::bridge]
mod ffi {
    extern "RustQt" {
        #[qobject]
        type GmailModel = super::GmailModelRust;

        // Properties
        #[qproperty]
        fn messages(self: &GmailModel) -> QVariantList;
        #[qproperty]
        fn loading(self: &GmailModel) -> bool;
        #[qproperty]
        fn error_message(self: &GmailModel) -> QString;
        #[qproperty]
        fn selected_message(self: &GmailModel) -> QString;
        #[qproperty]
        fn unread_count(self: &GmailModel) -> i32;
        #[qproperty]
        fn sync_status(self: &GmailModel) -> QString;  // 'idle', 'syncing', 'offline'
        #[qproperty]
        fn pending_actions(self: &GmailModel) -> i32;

        // Invokables
        #[qinvokable]
        fn fetch_messages(self: Pin<&mut GmailModel>);
        #[qinvokable]
        fn search(self: Pin<&mut GmailModel>, query: QString);
        #[qinvokable]
        fn get_message(self: Pin<&mut GmailModel>, id: QString);
        #[qinvokable]
        fn send_message(self: Pin<&mut GmailModel>, to: QString, subject: QString, body: QString);
        #[qinvokable]
        fn archive(self: Pin<&mut GmailModel>, id: QString);
        #[qinvokable]
        fn mark_read(self: Pin<&mut GmailModel>, id: QString);
        #[qinvokable]
        fn poll_channel(self: Pin<&mut GmailModel>);

        // Signals
        #[qsignal]
        fn messages_changed(self: Pin<&mut GmailModel>);
        #[qsignal]
        fn message_loaded(self: Pin<&mut GmailModel>);
        #[qsignal]
        fn send_complete(self: Pin<&mut GmailModel>, success: bool);
        #[qsignal]
        fn sync_status_changed(self: Pin<&mut GmailModel>);
        #[qsignal]
        fn pending_actions_changed(self: Pin<&mut GmailModel>);
    }
}
```

**`myme-ui/src/models/calendar_model.rs`:**
```rust
#[cxx_qt::bridge]
mod ffi {
    extern "RustQt" {
        #[qobject]
        type CalendarModel = super::CalendarModelRust;

        // Properties
        #[qproperty]
        fn events(self: &CalendarModel) -> QVariantList;
        #[qproperty]
        fn calendars(self: &CalendarModel) -> QVariantList;
        #[qproperty]
        fn loading(self: &CalendarModel) -> bool;
        #[qproperty]
        fn error_message(self: &CalendarModel) -> QString;
        #[qproperty]
        fn selected_date(self: &CalendarModel) -> QString;
        #[qproperty]
        fn view_mode(self: &CalendarModel) -> QString;  // 'month', 'week', 'day'
        #[qproperty]
        fn sync_status(self: &CalendarModel) -> QString;

        // Invokables
        #[qinvokable]
        fn fetch_events(self: Pin<&mut CalendarModel>, start: QString, end: QString);
        #[qinvokable]
        fn create_event(self: Pin<&mut CalendarModel>, calendar_id: QString, event_json: QString);
        #[qinvokable]
        fn update_event(self: Pin<&mut CalendarModel>, calendar_id: QString, event_id: QString, event_json: QString);
        #[qinvokable]
        fn delete_event(self: Pin<&mut CalendarModel>, event_id: QString);
        #[qinvokable]
        fn respond(self: Pin<&mut CalendarModel>, event_id: QString, status: QString);
        #[qinvokable]
        fn poll_channel(self: Pin<&mut CalendarModel>);

        // Signals
        #[qsignal]
        fn events_changed(self: Pin<&mut CalendarModel>);
        #[qsignal]
        fn calendars_changed(self: Pin<&mut CalendarModel>);
        #[qsignal]
        fn event_created(self: Pin<&mut CalendarModel>, event_id: QString);
        #[qsignal]
        fn event_updated(self: Pin<&mut CalendarModel>, event_id: QString);
        #[qsignal]
        fn sync_status_changed(self: Pin<&mut CalendarModel>);
    }
}
```

**`myme-ui/src/models/google_auth_model.rs`:**
```rust
#[cxx_qt::bridge]
mod ffi {
    extern "RustQt" {
        #[qobject]
        type GoogleAuthModel = super::GoogleAuthModelRust;

        #[qproperty]
        fn authenticated(self: &GoogleAuthModel) -> bool;
        #[qproperty]
        fn loading(self: &GoogleAuthModel) -> bool;
        #[qproperty]
        fn error_message(self: &GoogleAuthModel) -> QString;
        #[qproperty]
        fn account_email(self: &GoogleAuthModel) -> QString;

        #[qinvokable]
        fn authenticate(self: Pin<&mut GoogleAuthModel>);
        #[qinvokable]
        fn sign_out(self: Pin<&mut GoogleAuthModel>);
        #[qinvokable]
        fn check_auth(self: Pin<&mut GoogleAuthModel>);
        #[qinvokable]
        fn poll_channel(self: Pin<&mut GoogleAuthModel>);

        #[qsignal]
        fn auth_changed(self: Pin<&mut GoogleAuthModel>);
    }
}
```

### New QML Pages

**`GmailPage.qml`:**
- Three-pane layout: labels sidebar | message list | message viewer
- Search bar at top of message list
- Floating compose button (FAB)
- Sync indicator in header (spinner/checkmark/warning)
- Pull-to-refresh gesture support

**`CalendarPage.qml`:**
- Header: Month/Week/Day toggle, nav arrows, Today button
- Main area: Calendar grid with color-coded events
- Side panel: Upcoming events, mini month picker
- Event detail popup with edit/delete/respond actions
- All-day badge for full-day events

---

## 5. Home Page Widgets

### New Page: `HomePage.qml`

Replaces current "welcome" state as the default landing page.

**Layout:**
- Two-column grid on desktop (>800px width)
- Single column on narrow windows
- Left: Email widget
- Right: Calendar widget
- Fade-in animation on load/refresh

### Email Widget (`components/EmailWidget.qml`)

```
┌─────────────────────────────────────┐
│ 📧 Gmail              [sync] ⟳  ➜  │
├─────────────────────────────────────┤
│ 3 unread                            │
├─────────────────────────────────────┤
│ • John Smith                   2m   │
│   Project update - Here's the...    │
├─────────────────────────────────────┤
│ • Alice Johnson               15m   │
│   Re: Meeting notes - Thanks...     │
└─────────────────────────────────────┘
```

- Header: Icon, title, sync indicator, refresh, open arrow
- Unread count badge
- Latest 5 emails (sender, subject snippet, relative time)
- Click email → opens GmailPage with message selected
- Empty state: "No new mail"
- Offline indicator when showing cached data

### Calendar Widget (`components/CalendarWidget.qml`)

```
┌─────────────────────────────────────┐
│ 📅 Today              [sync] ⟳  ➜  │
│    Saturday, Feb 1                  │
├─────────────────────────────────────┤
│ 🔵 9:00 AM  Team standup      30m  │
│ 🟢 11:00 AM Design review      1h  │
│ 🔵 2:00 PM  Client call        1h  │
├─────────────────────────────────────┤
│ Tomorrow                            │
│ 🟠 10:00 AM Dentist appt       1h  │
└─────────────────────────────────────┘
```

- Header: Icon, "Today" + date, sync indicator, refresh, open arrow
- Today's events with calendar color dots
- Time, title, duration
- All-day badge for all-day events
- Divider before "Tomorrow" section
- Empty state: "No events today" (then show tomorrow)
- Click event → opens CalendarPage to that day

### Navigation Updates

- Add "Home" as first sidebar item (house icon)
- Default `currentPage: "home"`
- Accent highlight for Home when selected

---

## 6. Settings - Connected Accounts

### Updated Section Design

Rename to "Connected Accounts" with unified card list:

```
┌─────────────────────────────────────────────────────┐
│ Connected Accounts                                  │
├─────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────┐ │
│ │ [GitHub]  GitHub                                │ │
│ │           Connected as @jonesrussell           │ │
│ │                                    [Disconnect] │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│ ┌─────────────────────────────────────────────────┐ │
│ │ [Google]  Google                                │ │
│ │           Connected as jones@gmail.com          │ │
│ │                                    [Disconnect] │ │
│ └─────────────────────────────────────────────────┘ │
│                                                     │
│ ℹ️ Connected accounts enable project tracking,      │
│   email access, and calendar integration.           │
└─────────────────────────────────────────────────────┘
```

### Reusable Component: `AccountCard.qml`

**Props:**
- `provider`: string (github/google)
- `connected`: bool
- `loading`: bool
- `accountName`: string
- `errorMessage`: string

**States:**
- `disconnected`: Neutral border, "Connect" button
- `connecting`: Spinner overlay, disabled button
- `connected`: Green border, "Disconnect" button
- `error`: Red border, error message, "Retry" button

**Visual cues:**
- Provider icon (GitHub logo, Google logo)
- Border color indicates state
- Loading spinner during OAuth flow

### Model Architecture

Keep separate models (recommended):
- `AuthModel` for GitHub (existing)
- `GoogleAuthModel` for Google (new)

Settings page imports both and renders two `AccountCard` instances.

---

## File Summary

### New Crates

| Crate | Purpose |
|-------|---------|
| `myme-gmail` | Gmail API client, types, cache, sync |
| `myme-calendar` | Calendar API client, types, cache, sync |

### New/Modified Files

**Rust (`crates/`):**
- `myme-auth/src/google.rs` - Google OAuth provider
- `myme-gmail/src/*` - New crate
- `myme-calendar/src/*` - New crate
- `myme-ui/src/models/gmail_model.rs`
- `myme-ui/src/models/calendar_model.rs`
- `myme-ui/src/models/google_auth_model.rs`
- `myme-ui/src/services/gmail_service.rs`
- `myme-ui/src/services/calendar_service.rs`

**QML (`crates/myme-ui/qml/`):**
- `pages/HomePage.qml` - New
- `pages/GmailPage.qml` - New
- `pages/CalendarPage.qml` - New
- `pages/SettingsPage.qml` - Modified (Connected Accounts)
- `components/EmailWidget.qml` - New
- `components/CalendarWidget.qml` - New
- `components/AccountCard.qml` - New
- `Main.qml` - Modified (navigation)

**Config:**
- `Cargo.toml` - Add new crate members
- `qml.qrc` - Add new QML files

---

## Dependencies

### New Cargo Dependencies

```toml
# myme-gmail/Cargo.toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1.42", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
base64 = "0.22"
tracing = "0.1"
anyhow = "1.0"
myme-auth = { path = "../myme-auth" }

# myme-calendar/Cargo.toml
[dependencies]
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1.42", features = ["full"] }
rusqlite = { version = "0.32", features = ["bundled"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
anyhow = "1.0"
myme-auth = { path = "../myme-auth" }
```

---

## Testing Strategy

### Unit Tests

- Gmail client: Mock HTTP responses, verify request formatting
- Calendar client: Mock HTTP responses, verify date handling
- Cache: Test SQLite operations, sync queue processing
- OAuth: Test token refresh, scope validation

### Integration Tests

- Full OAuth flow with mock server
- Incremental sync with history/syncToken
- Offline queue processing
- Conflict resolution with etag

### QML Tests

- Widget rendering with mock data
- Empty states
- Error states
- Navigation deep-linking

---

## Implementation Order

1. **Google OAuth** - `myme-auth/src/google.rs`, `SecureStorage` extension
2. **Gmail crate** - Client, types, cache, sync
3. **Calendar crate** - Client, types, cache, sync
4. **Rust models** - GmailModel, CalendarModel, GoogleAuthModel
5. **Settings UI** - Connected Accounts, AccountCard component
6. **Gmail page** - Full email client UI
7. **Calendar page** - Full calendar UI
8. **Home page** - Widgets, navigation updates
9. **Polish** - Animations, empty states, error handling
