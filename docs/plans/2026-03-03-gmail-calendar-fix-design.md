# Gmail & Calendar Fix — Design

Date: 2026-03-03

## Problem

Gmail and Calendar are broken on two fronts:
1. **Auth breaks** — tokens expire hourly, no auto-refresh, app becomes unusable
2. **Reading is unreliable** — message bodies never stored, only 20 messages fetched, full re-fetch every time, no background sync

## Scope

First pass focuses on **reliable reading + background sync**. Write operations (compose, event creation, sync queue wiring) are deferred.

## Approach

Build on existing architecture (which is solid — 42 tests, SQLite caching, async threading model). Add a shared auth middleware and incremental improvements.

## Design

### 1. Default Credentials & Config

Ship Google Cloud OAuth2 credentials as compile-time defaults in `myme-auth`:

```rust
// myme-auth/src/google.rs
const DEFAULT_GOOGLE_CLIENT_ID: &str = "xxx.apps.googleusercontent.com";
const DEFAULT_GOOGLE_CLIENT_SECRET: &str = "xxx";
```

Config.toml override (existing `[google]` section):
```toml
[google]
client_id = "optional-override"
client_secret = "optional-override"
sync_interval_secs = 300  # default 5 min
```

Resolution: config.toml > compiled default. Scopes: `gmail.modify`, `calendar`, `userinfo.email`.

Requires creating a Google Cloud project with OAuth consent screen and enabling Gmail + Calendar APIs.

### 2. GoogleApiClient — Auth Middleware

New struct in `myme-auth` wrapping `reqwest::Client` with transparent token lifecycle:

```rust
// myme-auth/src/google_api_client.rs
pub struct GoogleApiClient {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    storage: SecureStorage,
    token_state: RwLock<TokenState>,
}

struct TokenState {
    access_token: String,
    refresh_token: String,
    expires_at: DateTime<Utc>,
}
```

Behaviors:
- **Proactive refresh** — if token expires within 5 minutes, refresh before sending
- **Reactive retry** — on 401, refresh once and retry
- **Token persistence** — writes refreshed tokens back to system keyring
- **Thread-safe** — `RwLock` on token state for concurrent async access

Caller experience: `GmailClient::new(&api)` takes `Arc<GoogleApiClient>` instead of raw token string. Internal HTTP calls use `api.request(Method::GET, url)` — token handling is invisible.

### 3. Reliable Gmail Reading

**a) Message bodies.** Extract body from MIME parts in `get_message()` (text/plain preferred, text/html fallback). Store in existing `body` column in `gmail_cache.db`.

**b) Fetch limits.** Remove `take(20)` cap. Fetch up to 50 per page (Gmail API max). Support pagination via `nextPageToken` for loading more.

**c) Incremental sync.** Store `historyId` from message list responses in `sync_state`. Subsequent syncs use `history.list(startHistoryId=...)` to get only changes. Fall back to full fetch on 410 Gone (stale history).

### 4. Reliable Calendar Reading

**a) Incremental sync.** Store `nextSyncToken` from event list responses in `sync_state`. Subsequent fetches pass `syncToken=...` for delta updates. Fall back to full fetch on 410 Gone.

**b) Calendar list.** Fetch and cache user's calendars on first sync (names, colors). UI stays on "primary" for now.

### 5. Background Sync & UI

**a) Configurable timer.** Repeating sync timer in Rust bridge layer:
- Default 300s (5 min), configurable via `sync_interval_secs` in config.toml
- First sync on auth success, then repeats at interval
- Uses existing channel pattern for async results

**b) Sync status.** `syncing` bool property on models. Subtle indicator (small spinner/pulsing dot in header) during background sync — user keeps reading.

**c) Last-synced timestamp.** "Last synced: 2 min ago" in page header. Updated after each successful sync.

**d) Error recovery.** Network failure: non-blocking banner, retry at next interval. Auth failure: "Re-authenticate" prompt.

## Files Affected

| Area | Files |
|------|-------|
| Auth middleware | `crates/myme-auth/src/google_api_client.rs` (new), `google.rs` (defaults) |
| Config | `crates/myme-core/src/config.rs` (sync_interval_secs field) |
| Gmail client | `crates/myme-gmail/src/client.rs` (use GoogleApiClient, body extraction) |
| Gmail cache | `crates/myme-gmail/src/cache.rs` (historyId in sync_state) |
| Gmail types | `crates/myme-gmail/src/types.rs` (history API types) |
| Calendar client | `crates/myme-calendar/src/client.rs` (use GoogleApiClient, syncToken) |
| Calendar cache | `crates/myme-calendar/src/cache.rs` (syncToken in sync_state) |
| UI bridge | `crates/myme-ui/src/services/gmail_service.rs`, `calendar_service.rs`, `google_common.rs` |
| UI models | `crates/myme-ui/src/models/gmail_model.rs`, `calendar_model.rs` (sync timer, status props) |
| QML | `crates/myme-ui/qml/pages/GmailPage.qml`, `CalendarPage.qml` (sync indicator, last-synced) |

## Out of Scope

- Email compose/send UI
- Calendar event creation/editing
- Sync queue wiring (offline write actions)
- Multi-calendar UI
- Gmail label management
