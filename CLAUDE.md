# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MyMe is a modular Rust desktop application using Qt/QML via cxx-qt that serves as a personal productivity and development hub. It integrates with external services (GitHub, Google services) and uses local SQLite for notes and projects.

**Current Status**: "Warm Forge" UI redesign complete. Amber/gold theme, Outfit custom font, persistent sidebar navigation, dashboard with live widgets, softer card styling, staggered animations across all pages.

## Build Commands

### Prerequisites
- Rust 2021 edition or later
- Qt 6.10.1 (or 6.x)
- CMake 3.16+
- C++ compiler (Visual Studio 2019+ on Windows)

### Building

**Windows (Recommended):**
```powershell
# All-in-one build script (builds Rust + Qt)
.\scripts\build.ps1
```

**Manual Build:**
```bash
# Build Rust crates (includes cxx-qt code generation)
cargo build --release

# Build Qt application
mkdir build-qt
cd build-qt
cmake ..
cmake --build . --config Release
```

### Running

```bash
# Run Rust binary (architecture test, no UI)
cargo run

# Run Qt application (full UI)
.\build\Release\myme-qt.exe
```

### Testing

```bash
# Test individual crates
cargo test -p myme-core
cargo test -p myme-services
cargo test -p myme-ui

# Test entire workspace (excludes myme-ui which requires Qt)
cargo test -p myme-core -p myme-services -p myme-auth -p myme-integrations -p myme-weather -p myme-gmail -p myme-calendar
```

**Test Coverage:**
- `myme-core`: 16 tests (config validation, error handling, state machine)
- `myme-services`: unit + integration tests (GitHubClient, NoteClient/SQLite, retry logic)
- `myme-auth`: 10 tests (OAuth for GitHub and Google, token storage)
- `myme-integrations`: 17 tests (git operations, repo discovery)
- `myme-gmail`: 35 tests (client, cache, sync queue)
- `myme-calendar`: 21 tests (client, cache)

### Debugging

```bash
# Rust debug output
$env:RUST_LOG="debug"
cargo run

# Qt debug output
$env:QT_LOGGING_RULES="*.debug=true"
.\build\Release\myme-qt.exe
```

## Architecture Overview

### Workspace Structure

This is a Rust workspace with 8 member crates:

```
myme/
├── crates/
│   ├── myme-core/          # Application lifecycle, config, error types
│   ├── myme-ui/            # cxx-qt bridge, 16 QML models (NoteModel, RepoModel, GmailModel, etc.)
│   ├── myme-services/      # HTTP API clients (Todo/Note API)
│   ├── myme-auth/          # OAuth2 flows, secure token storage
│   ├── myme-integrations/  # GitHub API, Git operations
│   ├── myme-weather/       # Weather API with platform geolocation (WinRT/D-Bus)
│   ├── myme-gmail/         # Gmail API client, SQLite cache
│   └── myme-calendar/      # Google Calendar API client, cache
├── src/main.rs             # Rust binary entry point
├── qt-main/main.cpp        # C++ Qt application entry point
└── qml.qrc                 # Qt resource file for QML
```

### Key Technologies

- **Language**: Rust 2021 edition
- **UI**: Qt 6.10.1 / QML (QtQuick Controls 2 with custom Theme.qml)
- **Bridge**: cxx-qt 0.8 (Rust ↔ Qt FFI with automatic codegen)
- **Build**: CMake + Cargo
- **Async**: tokio 1.42 with full features
- **HTTP**: reqwest 0.12

### Architectural Patterns

1. **Workspace Over Monolith**: Enables independent development and testing of components with clear separation of concerns.

2. **cxx-qt Bridge Pattern**: Automatic code generation for C++/Rust FFI. Qt QObjects are defined in Rust using `#[cxx_qt::bridge]` macros. Methods marked with `#[qinvokable]` are exposed to QML. Signals are emitted from Rust to update the UI.

3. **Channel-Based Async Pattern**: Non-blocking UI with channel polling:
   - Qt invokable methods send requests via `mpsc` channels
   - Background tokio tasks process requests and send results back
   - QML Timer (100ms) calls `poll_channel()` to check for results
   - No `block_on()` calls - UI never freezes during network operations
   - See `NoteModel`, `RepoModel` for implementation examples

4. **AppServices Singleton**: Centralized mutable service container (`app_services.rs`):
   - Replaces `OnceLock` pattern to allow runtime state changes
   - Uses `parking_lot::RwLock` for service references
   - Supports reinitializing clients after auth changes
   - Provides `shutdown()` for graceful cleanup
   - Channel senders/receivers stored here for model communication

5. **Service Client Pattern**: Each external service has its own async client:
   - `GitHubClient` for GitHub API with retry logic
   - `NoteClient` for local SQLite notes (no HTTP backend)
   - Retry with exponential backoff (100ms, 200ms, 400ms...)
   - Retries: timeouts, 5xx errors, 429 rate limits
   - No retry: 4xx client errors (auth failures, not found)

6. **Secure Token Storage**: System keyring for OAuth tokens:
   - Windows: Windows Credential Manager
   - macOS: Keychain
   - Linux: Secret Service (libsecret)
   - Automatic migration from legacy plaintext files
   - See `myme-auth/src/storage.rs`

7. **Operation Cancellation**: Support for cancelling long-running operations:
   - `CancellationToken` from `tokio_util` for git clone/pull
   - Cancel button in UI (RepoCard.qml)
   - Token checked before and during operations

8. **Graceful Shutdown**: Clean application exit:
   - `shutdown_app_services()` C FFI function called from Qt's `aboutToQuit` signal
   - Cancels in-flight async operations
   - Clears service references and channels
   - Prevents resource leaks on exit

### Component Responsibilities

**myme-core**: Core application lifecycle (`App` struct), configuration management (TOML-based, cross-platform paths), error type hierarchy.

**myme-services**: HTTP clients for external APIs (e.g. GitHub) and local stores (NoteClient/SQLite, ProjectStore). Each async client uses structured logging and retry where applicable.

**myme-ui**: cxx-qt bridge layer. Contains QObject models (e.g., `NoteModel`, `RepoModel`) that expose Rust functionality to QML. The `build.rs` configures cxx-qt code generation. QML files are in `qml/` subdirectory.

**myme-auth**: OAuth2 flows (`oauth.rs`, `github.rs`), secure token storage using system keyring (`storage.rs`). Dynamic port discovery (8080-8089) for OAuth callback.

**myme-integrations**: GitHub API wrapper (`github/`), local Git operations using git2 (`git/`). Repository discovery and clone/pull with cancellation support.

**myme-weather**: Weather data provider with platform-native geolocation. Uses WinRT Geolocation APIs on Windows (`windows` crate) and D-Bus location services on Linux (`zbus`). Includes weather cache and WeatherModel for QML.

### Data Flow: QML → Rust → API (Channel Pattern)

```
NotePage.qml: user clicks refresh button
  ↓
noteModel.fetch_notes() [QML invokable - snake_case!]
  ↓
NoteModel sends FetchNotes request via channel
  ↓
Background tokio task receives request
  ↓
NoteClient (SQLite) or GitHubClient [HTTP with retry logic]
  ↓
Task sends result back via channel
  ↓
QML Timer triggers: noteModel.poll_channel()
  ↓
NoteModel receives result, updates state, emits signal
  ↓
QML reacts to signal, updates ListView
```

**Key points:**
- No `block_on()` - UI stays responsive
- Timer polls every 100ms while loading
- Loading/error states shown in QML

### Build System Integration

**CMakeLists.txt** orchestrates the build:
1. Detects build type (Debug/Release) and sets Rust build flags
2. Finds Qt6 with required components
3. Executes `cargo build` with Qt environment variables set
4. Collects cxx-qt generated C++ files from `target/{debug,release}/build/`
5. Links Rust static library (`myme_ui.lib/a`) with C++ application
6. Includes cxx-qt generated headers
7. Sets QML import paths for runtime

**Important**: Always run `cargo build` before `cmake` to ensure cxx-qt generates the necessary C++ bridge code.

### Configuration System

Configuration is stored at platform-specific locations:
- Windows: `%APPDATA%\myme\config.toml`
- macOS: `~/Library/Application Support/myme/config.toml`
- Linux: `~/.config/myme/config.toml`

Default config is created automatically on first run. Configuration is loaded using the `dirs` crate for cross-platform path resolution.

**Configuration Validation**: Use `Config::load_validated()` for validation with warnings:
- URL validation (must be valid http/https)
- Port validation (1-65535)
- Path validation (directories must exist)
- Returns errors for critical issues, warnings for non-critical

### Error Handling

**Error Type Hierarchy** (`myme-core/src/error.rs`):
- `AppError` - Application-level errors with user-friendly messages
- `AuthError` - Authentication failures
- `GitHubError` - GitHub API errors
- All errors implement `user_message()` for UI display

**HTTP Retry Logic** (`myme-services/src/retry.rs`):
- Exponential backoff: 100ms, 200ms, 400ms (up to 5s max)
- Retries: timeouts, 5xx server errors, 429 rate limits
- No retry: 4xx client errors (prevents retry loops on bad requests)

## Development Workflow

### Making Rust Changes

1. Edit Rust source in `crates/*/src/`
2. Run `cargo build --release`
3. Rebuild Qt app from `build/` directory: `cmake --build . --config Release`
4. Run `.\build\Release\myme-qt.exe`

### Making QML Changes

1. Edit QML files in `crates/myme-ui/qml/`
2. No rebuild needed - QML is loaded at runtime
3. Just restart the application

### Adding New Service Clients

1. Create new client in `crates/myme-services/src/`
2. Implement async methods with proper error handling (`anyhow::Result`)
3. Use structured logging (`tracing` macros)
4. Export from `lib.rs`
5. If needed, create corresponding QObject model in `myme-ui/src/models/`

### Adding New UI Pages

1. Create `crates/myme-ui/qml/pages/NewPage.qml`
2. Create corresponding QObject in `crates/myme-ui/src/models/new_model.rs`
3. Use `#[cxx_qt::bridge]` macro and `#[qinvokable]` for methods
4. Register the model file in `crates/myme-ui/build.rs` (`.file("src/models/new_model.rs")`)
5. Add the QML file to `qml.qrc`
6. Add navigation action in `Main.qml` drawer
7. Wire up data model invokable methods in QML

### Adding Authentication

1. Extend `myme-auth` with new OAuth provider (see `github.rs` as example)
2. Use `SecureStorage` to store tokens in system keyring
3. Pass tokens to service clients via bearer authentication
4. Add auth flow UI in QML if needed

## QML UI Patterns

### Theme System ("Warm Forge")
- `Theme.qml` singleton in `crates/myme-ui/qml/` provides centralized colors/spacing
- Pages import theme with `import ".."` to access `Theme.background`, `Theme.text`, etc.
- Theme supports `light`, `dark`, `auto` modes via `Theme.mode` property
- **Colors**: Amber/gold primary (`#e5a54b` dark, `#c08832` light), warm neutrals, dark-first
- **Typography**: Outfit variable font (`fonts/Outfit-Regular.ttf`), loaded via single `FontLoader`; use `font.weight: Font.Bold` etc. for weight variants
- **Cards**: `cardRadius: 10`, `cardPadding: 20`, `buttonRadius: 8`; near-invisible borders: `border.color: Theme.isDark ? "#ffffff08" : "#00000008"`
- **Error banners**: Softer styling with `border.color: "transparent"` / `border.width: 0`
- New QML files must be added to `qml.qrc` for bundling

### Navigation — Persistent Sidebar
- `Sidebar.qml` component: collapsible (220px expanded / 60px collapsed), 8 nav items
- Sidebar is sibling of StackView in `RowLayout` (not inside StackView) to prevent reload on page changes
- StackView uses slide-fade transitions (opacity 0->1 + x offset 20->0, 200ms OutCubic)
- Track current page via `root.currentPage` and `AppContext.currentPage` for sidebar highlighting
- Keyboard shortcuts: `Ctrl+1` through `Ctrl+8` for nav, `Ctrl+,` for Settings, `Ctrl+B` to toggle sidebar

### Staggered List Animations
- List delegates start `opacity: 0` and animate to 1 on `Component.onCompleted`
- Stagger via `PauseAnimation { duration: index * 30 }` before fade
- Subtle y shift (8px down -> 0) with `Easing.OutCubic`
- Applied to: NotePage, GmailPage, CalendarPage, RepoCard, ProjectsPage, WorkflowsPage

### QML-Only Changes
- QML changes don't require rebuild - just restart the application
- Only Rust bridge changes (cxx-qt) require `cargo build` + CMake rebuild

### QML JavaScript
- Qt 6.x QML supports modern ES6+ JavaScript: arrow functions, template literals, `let`/`const`, destructuring, etc.
- Use modern syntax for cleaner, more readable code

### QML Formatting
- qmlformat location: `/mnt/c/Qt/6.10.1/msvc2022_64/bin/qmlformat.exe -i <file>`

### QML Singletons
- Theme/Icons/AppContext defined in `crates/myme-ui/qml/Theme.qml`, `Icons.qml`, `AppContext.qml`
- Registered in `crates/myme-ui/qml/qmldir`
- Phosphor Icons font for UI icons (`crates/myme-ui/qml/fonts/Phosphor.ttf`)
- Outfit variable font for text (`crates/myme-ui/qml/fonts/Outfit-Regular.ttf`)

### cxx-qt Invokable Naming
- cxx-qt exposes Rust methods to QML using the **exact snake_case names** from Rust (no camelCase conversion)
- In QML, always call invokable methods with snake_case: `model.check_auth()`, `model.fetch_repos()`, `model.poll_channel()` — not `checkAuth`, `fetchRepos`, or `pollChannel`

### Dev Tools Page
The Dev Tools page (`DevToolsPage.qml`) provides utility tools for developers:
- **JWT Generator**: Generate and verify JSON Web Tokens
- **Encoding Hub**: Encode/decode Base64, Hex, URL strings
- **UUID Generator**: Generate UUIDs v1, v4, v5, v7
- **JSON Toolkit**: Format, validate, minify, convert JSON
- **Hash Generator**: Generate MD5, SHA-1, SHA-256, SHA-512 hashes
- **Time Toolkit**: Parse timestamps, convert timezones
- **Text Chunker**: Split large text into ≤10,000 char chunks for AI tools with character limits

Tools follow a consistent pattern: add entry to `tools` array, create `Component`, register in `Loader` switch. QML-only tools (no Rust backend) are preferred for simple utilities.

## Important Files

### Core Infrastructure
- [Cargo.toml](Cargo.toml) - Workspace configuration with shared dependencies
- [CMakeLists.txt](CMakeLists.txt) - Qt/C++ build system, links Rust library
- [qt-main/main.cpp](qt-main/main.cpp) - C++ Qt application entry point with shutdown handler
- [qml.qrc](qml.qrc) - Qt resource file for bundling QML

### Configuration & Error Handling
- [crates/myme-core/src/config.rs](crates/myme-core/src/config.rs) - Configuration management with validation
- [crates/myme-core/src/error.rs](crates/myme-core/src/error.rs) - Error type hierarchy with user messages

### Service Layer
- [crates/myme-services/src/todo.rs](crates/myme-services/src/todo.rs) - Todo/Note data types (Todo, TodoCreateRequest, TodoUpdateRequest)
- [crates/myme-services/src/note_client.rs](crates/myme-services/src/note_client.rs) - SQLite-backed NoteClient
- [crates/myme-services/src/github.rs](crates/myme-services/src/github.rs) - GitHub API client with retry logic
- [crates/myme-services/src/retry.rs](crates/myme-services/src/retry.rs) - Exponential backoff retry utility

### UI Bridge
- [crates/myme-ui/src/app_services.rs](crates/myme-ui/src/app_services.rs) - AppServices singleton (replaces OnceLock)
- [crates/myme-ui/src/bridge.rs](crates/myme-ui/src/bridge.rs) - C FFI functions for Qt/Rust bridge
- [crates/myme-ui/src/models/note_model.rs](crates/myme-ui/src/models/note_model.rs) - Example cxx-qt bridge with channel pattern
- [crates/myme-ui/build.rs](crates/myme-ui/build.rs) - cxx-qt build configuration

### Authentication
- [crates/myme-auth/src/storage.rs](crates/myme-auth/src/storage.rs) - System keyring token storage
- [crates/myme-auth/src/oauth.rs](crates/myme-auth/src/oauth.rs) - OAuth2 flow with dynamic port discovery
- [crates/myme-auth/src/github.rs](crates/myme-auth/src/github.rs) - GitHub OAuth provider
- [crates/myme-auth/src/google.rs](crates/myme-auth/src/google.rs) - Google OAuth provider for Gmail/Calendar

### Gmail (Phase 3)
- [crates/myme-gmail/src/client.rs](crates/myme-gmail/src/client.rs) - Gmail API client with full CRUD operations
- [crates/myme-gmail/src/cache.rs](crates/myme-gmail/src/cache.rs) - SQLite offline cache for messages and labels
- [crates/myme-gmail/src/sync.rs](crates/myme-gmail/src/sync.rs) - Offline action sync queue
- [crates/myme-gmail/src/types.rs](crates/myme-gmail/src/types.rs) - Message, Label, and API response types

### Calendar (Phase 3)
- [crates/myme-calendar/src/client.rs](crates/myme-calendar/src/client.rs) - Google Calendar API client
- [crates/myme-calendar/src/cache.rs](crates/myme-calendar/src/cache.rs) - SQLite offline cache for events
- [crates/myme-calendar/src/types.rs](crates/myme-calendar/src/types.rs) - Event, Calendar, and API response types

### Weather
- [crates/myme-weather/src/provider.rs](crates/myme-weather/src/provider.rs) - Weather data provider
- [crates/myme-weather/src/cache.rs](crates/myme-weather/src/cache.rs) - Weather data cache
- [crates/myme-weather/src/location.rs](crates/myme-weather/src/location.rs) - Platform geolocation (WinRT/D-Bus)

### Integration Tests
- [crates/myme-services/tests/github_integration.rs](crates/myme-services/tests/github_integration.rs) - GitHubClient tests

## Notes (SQLite)

Notes are stored locally in SQLite. Configuration is in `[notes]` in `config.toml` (e.g. `sqlite_path`). The `NoteClient` in `myme-services` wraps the SQLite store; there is no HTTP/API backend for notes.

## Integration with Google Services

MyMe integrates with Gmail and Google Calendar using OAuth2:

### Setup

1. Create a Google Cloud project at https://console.cloud.google.com/
2. Enable Gmail API and Google Calendar API
3. Create OAuth 2.0 credentials (Desktop app type)
4. Add credentials to `~/.config/myme/config.toml`:

```toml
[google]
client_id = "YOUR_CLIENT_ID.apps.googleusercontent.com"
client_secret = "YOUR_CLIENT_SECRET"
```

### Architecture

- **OAuth Flow**: Uses `GoogleOAuth2Provider` in `myme-auth/src/google.rs`
- **Token Storage**: Stored securely in system keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- **Offline Cache**: SQLite databases in `~/.config/myme/`:
  - `gmail_cache.db` - Messages, labels, sync state
  - `calendar_cache.db` - Events, calendars
- **Sync Queue**: Offline actions queued and synced when online

### Scopes Requested

- Gmail: `https://www.googleapis.com/auth/gmail.readonly`, `https://www.googleapis.com/auth/gmail.modify`
- Calendar: `https://www.googleapis.com/auth/calendar.readonly`, `https://www.googleapis.com/auth/calendar.events`

## Windows-Specific Notes

**Linker Issue**: Windows builds may fail with "cannot find link.exe" due to PATH conflicts. Solutions:

1. Use "Developer Command Prompt for VS" from Start Menu
2. Configure `.cargo/config.toml` with correct linker path
3. Use WSL2 for Linux environment
4. See [WINDOWS_BUILD_FIX.md](WINDOWS_BUILD_FIX.md) for details

**Qt Path**: CMakeLists.txt auto-detects Qt via `find_package(Qt6)`. Ensure Qt is on your PATH or set `CMAKE_PREFIX_PATH`.

### Threading Model

**Qt Main Thread**: QML UI, Qt event loop
**Tokio Runtime**: Async HTTP requests, background processing
**Communication**: `std::sync::mpsc` channels between Qt and Tokio

**Never block Qt thread**: Use channel pattern instead of `block_on()`:
```rust
// BAD - blocks UI
let result = runtime.block_on(async { client.fetch().await });

// GOOD - non-blocking
let tx = get_service_tx();
tx.send(Request::Fetch);
// Later, in poll_channel():
if let Some(Response::FetchDone(result)) = try_recv() { ... }
```

### Performance Observability

Key operations are instrumented with `#[tracing::instrument]` for performance monitoring.

**Enable timing logs:**
```bash
$env:RUST_LOG="info"  # Shows operation start/end with durations
$env:RUST_LOG="debug" # Adds detailed operation internals
```

**Instrumented Operations:**
- `GitHubClient`: `list_repos`, `get_repo`, `create_repo`, `list_issues`, `create_issue`, `update_issue`
- `GitOperations`: `discover_repositories`, `clone_repository`, `fetch`, `pull`, `push`
- `OAuth2Provider`: `authenticate`, `exchange_code`

**Expected Performance Baselines:**
| Operation | Typical Duration | Notes |
|-----------|------------------|-------|
| `list_repos` | 100-500ms | GitHub API, includes 100 repos |
| `list_issues` | 100-300ms | Per repository |
| `discover_repositories` | 50-500ms | Depends on disk speed and repo count |
| `clone_repository` | 1-60s | Depends on repo size |
| `pull` | 100ms-10s | Depends on changes to fetch |
| `authenticate` (OAuth) | 5-30s | User interaction required |

**HTTP Retry Logic:**
- Retries: 3 attempts with exponential backoff (100ms, 200ms, 400ms)
- Retryable: Timeouts, 5xx errors, 429 rate limit
- Not retried: 4xx client errors (bad requests, auth failures)

## CI/CD & Build Scripts

- `.github/workflows/release.yml` - Automated Windows releases on version tags (`v*`)
- `scripts\build.ps1` - Full build (Rust + CMake + windeployqt) → myme-qt.exe
- `scripts\build-rust.ps1` - Rust-only build with VS Developer environment auto-detection → myme.exe
- `installer/myme.iss` - Inno Setup 6 installer script for Windows

## Phase Roadmap

**Phase 1** (Complete): Foundation
- Workspace structure, core application, SQLite notes, cxx-qt bridge, QML UI

**Phase 2** (Complete): GitHub + Local Git Management
- OAuth2 authentication, git2 integration, repository management UI
- Secure token storage, retry logic, graceful shutdown

**2026 Architectural Modernization** (Complete): 18-step refactoring
- Eliminated all `block_on()` calls (13 total) with channel-based async
- Replaced `OnceLock` with mutable `AppServices` for runtime state changes
- Migrated from plaintext tokens to system keyring storage
- Added `parking_lot` for better mutex performance
- Implemented HTTP retry with exponential backoff
- Added configuration validation with errors/warnings
- Created comprehensive integration test suite (120+ tests including Gmail/Calendar)
- Added `#[tracing::instrument]` for performance observability
- Implemented graceful shutdown via Qt signal

**Phase 3** (Complete): Google Email/Calendar Integration
- Gmail API client with offline SQLite cache and sync queue
- Google Calendar API client with SQLite cache
- OAuth2 authentication via GoogleOAuth2Provider
- GmailPage and CalendarPage QML pages
- Dashboard widgets (EmailWidget, CalendarWidget)
- Unified Google account management in Settings

**Warm Forge UI Redesign** (Complete): Cohesive visual identity
- Amber/gold primary color replacing generic purple, warm neutral palette
- Outfit variable font (Google Fonts, SIL OFL license)
- Persistent collapsible sidebar replacing mobile hamburger drawer
- Dashboard WelcomePage with time-based greeting, stat cards, widget grid
- Softer card borders, refined error banners, staggered list animations
- Keyboard shortcuts (Ctrl+1-8 nav, Ctrl+B sidebar toggle, Ctrl+, settings)
- 4 new files + 20 modified QML files, no Rust changes

**In Progress**: GitHub Workflows Integration
- WorkflowModel and WorkflowsPage for GitHub Actions management
- AppContext QML singleton for global app state

**Phase 4** (Planned): Project Scaffolding
- Project templates (Laravel, Drupal, Node.js), scaffold wizard UI
