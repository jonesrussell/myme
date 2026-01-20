# Godo Integration Guide

MyMe has been integrated with your **Godo** (Golang todo application) to provide seamless note management.

## What Changed

### API Alignment

MyMe now connects to Godo's actual API structure:

- **Endpoint**: `/api/v1/notes` (not `/api/todos`)
- **Port**: `8008` (Godo's default, configurable)
- **Authentication**: JWT Bearer token support
- **Data Model**: `Note` with `content` and `done` fields

### Updated Components

#### 1. myme-services ([crates/myme-services/src/todo.rs](crates/myme-services/src/todo.rs:1))

**New Structure**:
```rust
pub struct Todo {
    pub id: String,          // UUID from Godo
    pub content: String,     // Note content (1-1000 chars)
    pub done: bool,          // Completion status
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**New Methods**:
- `TodoClient::new(url, jwt_token)` - Accepts optional JWT token
- `set_jwt_token(token)` - Update JWT token
- `health_check()` - Check if Godo API is running
- `mark_done(id)`, `mark_undone(id)`, `toggle_done(id)` - Convenience methods

**API Calls**:
```rust
GET  /api/v1/notes           → list_todos()
POST /api/v1/notes           → create_todo(content)
GET  /api/v1/notes/{id}      → get_todo(id)
PATCH /api/v1/notes/{id}     → update_todo(id, content?, done?)
DELETE /api/v1/notes/{id}    → delete_todo(id)
GET  /api/v1/health          → health_check()
```

#### 2. myme-core Configuration ([crates/myme-core/src/config.rs](crates/myme-core/src/config.rs:1))

**New Fields**:
```toml
[services]
todo_api_url = "http://localhost:8008"  # Godo default port
jwt_token = "..."  # Optional, can be set via GODO_JWT_TOKEN env var
```

#### 3. TodoModel ([crates/myme-ui/src/models/todo_model.rs](crates/myme-ui/src/models/todo_model.rs:1))

**Updated Methods**:
- `add_todo(content)` - Single content field (no title/description split)
- `toggle_done(index)` - Toggle completion status
- `get_content(index)` - Get note content
- `get_done(index)` - Get completion status
- `get_created_at(index)` - Get formatted timestamp

#### 4. UI ([crates/myme-ui/qml/pages/TodoPage.qml](crates/myme-ui/qml/pages/TodoPage.qml:1))

**New Features**:
- Changed "Todos" to "Notes" throughout
- Single content field (1-1000 characters)
- Character count indicator
- Checkmark visual for done status
- Strikethrough for completed notes
- Click to toggle done status
- Ctrl+Enter shortcut in add dialog
- Footer with statistics (total, done, pending)
- Godo API connection indicator

## Setting Up Godo

### 1. Configure Godo

Edit `godo/config.yaml`:

```yaml
http:
  port: 8008  # Or any port you prefer

database:
  path: "$HOME/.config/godo/godo.db"

storage:
  type: "sqlite"  # Use local SQLite storage
```

### 2. Start Godo API Server

```bash
cd godo
task build
./godo-windows-amd64.exe  # Or appropriate binary for your OS
```

Godo will start on `http://localhost:8008`

### 3. Verify Godo is Running

```bash
curl http://localhost:8008/api/v1/health
```

Should return HTTP 200 OK.

## Authentication Setup

### Option 1: No Authentication (Development)

For local development, you can disable JWT auth in Godo or use the API without auth if configured.

Update `godo/config.yaml` if needed to allow unauthenticated access for development.

### Option 2: JWT Token (Production)

#### Generate JWT Token

Godo uses JWT tokens for authentication. You'll need to generate a token with the secret from `JWT_SECRET` environment variable.

**Set JWT_SECRET** (in Godo):
```bash
export JWT_SECRET="your-secret-key"
```

**Generate Token** (example with a tool or script):
```bash
# Use jwt.io or a JWT generation tool
# Payload: {"sub": "user_id", "exp": <future_timestamp>}
# Sign with your JWT_SECRET
```

#### Configure MyMe with Token

**Option A: Environment Variable**
```bash
export GODO_JWT_TOKEN="your-generated-jwt-token"
```

MyMe will automatically read this on startup.

**Option B: Configuration File**

Edit `%APPDATA%\myme\config.toml`:
```toml
[services]
todo_api_url = "http://localhost:8008"
jwt_token = "your-generated-jwt-token"
```

## Configuration Options

### MyMe Configuration

**Location**:
- Windows: `%APPDATA%\myme\config.toml`
- macOS: `~/Library/Application Support/myme/config.toml`
- Linux: `~/.config/myme/config.toml`

**Full Configuration**:
```toml
config_dir = "C:\\Users\\YourName\\AppData\\Roaming\\myme"

[services]
todo_api_url = "http://localhost:8008"
jwt_token = "optional-jwt-token"

[ui]
window_width = 1200
window_height = 800
dark_mode = false
```

### Environment Variables

MyMe respects these environment variables:
- `GODO_JWT_TOKEN` - JWT token for Godo API authentication
- `RUST_LOG` - Logging level (debug, info, warn, error)

## Usage

### Starting the Application

```bash
# 1. Start Godo
cd godo
./godo-windows-amd64.exe

# 2. Start MyMe (once built)
cd myme
.\build\Release\myme-qt.exe
```

### Adding Notes

1. Click "Add Note" button or use Ctrl+N (planned)
2. Enter note content (1-1000 characters)
3. Press "OK" or Ctrl+Enter
4. Note appears in the list

### Managing Notes

- **Toggle Done**: Click on a note or use swipe action
- **Delete**: Swipe and click delete icon
- **View Details**: Notes show creation time and status

### Keyboard Shortcuts

- **Ctrl+Enter**: Save note in add dialog
- **Click**: Toggle done status

## API Reference

### TodoClient Methods

```rust
// Create client
let client = TodoClient::new("http://localhost:8008", Some(token))?;

// Health check
let healthy = client.health_check().await?;

// List all notes
let notes = client.list_todos().await?;

// Create note
let note = client.create_todo(TodoCreateRequest {
    content: "Buy groceries".to_string(),
}).await?;

// Update note
let updated = client.update_todo(&note_id, TodoUpdateRequest {
    content: Some("Buy groceries and milk".to_string()),
    done: Some(true),
}).await?;

// Convenience methods
let done_note = client.mark_done(&note_id).await?;
let undone_note = client.mark_undone(&note_id).await?;
let toggled = client.toggle_done(&note_id).await?;

// Delete note
client.delete_todo(&note_id).await?;
```

### Response Format

**Note Object**:
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "content": "Buy groceries",
  "done": false,
  "created_at": "2026-01-19T10:30:00Z",
  "updated_at": "2026-01-19T10:30:00Z"
}
```

## Troubleshooting

### Connection Refused

**Problem**: MyMe can't connect to Godo

**Solutions**:
1. Verify Godo is running: `curl http://localhost:8008/api/v1/health`
2. Check port in config.toml matches Godo's port
3. Check firewall settings

### 401 Unauthorized

**Problem**: API returns 401 when fetching notes

**Solutions**:
1. JWT token is required - set `GODO_JWT_TOKEN` environment variable
2. Token may be expired - generate a new token
3. Token secret doesn't match - verify `JWT_SECRET` in Godo

### No Notes Appearing

**Problem**: UI shows "No notes yet" but Godo has notes

**Solutions**:
1. Check logs: `$env:RUST_LOG="debug"; cargo run`
2. Verify JWT token is valid
3. Try the refresh button
4. Check Godo logs for errors

### Character Limit Error

**Problem**: Can't add notes longer than 1000 characters

**Solution**: This is Godo's validation limit. Keep notes concise or split into multiple notes.

## Development Tips

### Testing Without Godo

```rust
// In tests, mock the client
#[tokio::test]
async fn test_without_godo() {
    // Use a mock server or test double
}
```

### Inspecting API Calls

```bash
# Enable debug logging
$env:RUST_LOG="myme_services=debug"
cargo run

# You'll see all HTTP requests/responses
```

### Using with Different Godo Instance

Update config.toml:
```toml
[services]
todo_api_url = "https://your-godo-server.com"
jwt_token = "production-token"
```

## Architecture Benefits

### Type Safety

Rust's type system ensures:
- UUIDs are always valid strings
- Timestamps are properly formatted
- JWT tokens are handled securely

### Error Handling

All API calls return `Result<T>` with descriptive errors:
```rust
match client.list_todos().await {
    Ok(todos) => // Handle success
    Err(e) => tracing::error!("Failed to fetch: {}", e)
}
```

### Async Performance

- Non-blocking I/O with tokio
- Multiple requests can run concurrently
- UI remains responsive during API calls

## Future Enhancements

Planned improvements for Godo integration:

### Phase 1 Completion
- [ ] Real-time UI updates after async operations
- [ ] Proper error display in UI
- [ ] Retry logic with exponential backoff

### Phase 2+
- [ ] Offline support with local caching
- [ ] Sync conflict resolution
- [ ] Batch operations
- [ ] Full-text search
- [ ] Tags and categories
- [ ] Note attachments

## Related Documentation

- [Godo README](../godo/README.md) - Godo application documentation
- [BUILD.md](BUILD.md) - MyMe build instructions
- [ARCHITECTURE_SUMMARY.md](ARCHITECTURE_SUMMARY.md) - Technical architecture details
- [DEVELOPMENT.md](DEVELOPMENT.md) - Development guide

## Support

### Godo Issues

For Godo-specific problems, refer to:
- Godo configuration: `godo/config.yaml`
- Godo logs: `logs/godo.log`
- Godo API docs: `godo/README.md`

### MyMe Issues

For MyMe integration issues:
- Check MyMe logs in console output
- Enable debug logging: `$env:RUST_LOG="debug"`
- Review this integration guide
- Check [PROJECT_STATUS.md](PROJECT_STATUS.md) for known issues

---

## Quick Reference

**Start Godo**:
```bash
cd godo && ./godo-windows-amd64.exe
```

**Start MyMe**:
```bash
cd myme && .\build\Release\myme-qt.exe
```

**Set JWT Token**:
```bash
$env:GODO_JWT_TOKEN="your-token"
```

**Check Health**:
```bash
curl http://localhost:8008/api/v1/health
```

**Default Config Location**:
```
%APPDATA%\myme\config.toml
```

Happy note-taking with MyMe + Godo! 📝
