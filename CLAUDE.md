# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MyMe is a modular Rust desktop application using Qt/QML + Kirigami via cxx-qt that serves as a personal productivity and development hub. It integrates with external services (Godo note-taking app, GitHub, Google services) through a plugin-based architecture.

**Current Status**: Phase 2 complete (GitHub + Local Git Management). 2026 Architectural Modernization complete - all 18 steps implemented including: non-blocking async callbacks, secure keyring token storage, retry logic, graceful shutdown, and comprehensive test coverage (75+ tests).

## Build Commands

### Prerequisites
- Rust 2021 edition or later
- Qt 6.10.1 (or 6.x) with Kirigami
- CMake 3.16+
- C++ compiler (Visual Studio 2019+ on Windows)

### Building

**Windows (Recommended):**
```powershell
# All-in-one build script (builds Rust + Qt)
.\build-qt.ps1
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
cargo test -p myme-core -p myme-services -p myme-auth -p myme-integrations
```

**Test Coverage:**
- `myme-core`: 16 tests (config validation, error handling, state machine)
- `myme-services`: 18 unit + 20 integration tests (TodoClient, GitHubClient, retry logic)
- `myme-auth`: 4 tests (OAuth, token storage)
- `myme-integrations`: 17 tests (git operations, repo discovery)

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

This is a Rust workspace with 5 member crates:

```
myme/
├── crates/
│   ├── myme-core/          # Application lifecycle, config, plugin system
│   ├── myme-ui/            # cxx-qt bridge, QML models (NoteModel, RepoModel)
│   ├── myme-services/      # HTTP API clients (Todo/Note API)
│   ├── myme-auth/          # OAuth2 flows, secure token storage (Phase 2)
│   └── myme-integrations/  # GitHub API, Git operations (Phase 2)
├── src/main.rs             # Rust binary entry point
├── qt-main/main.cpp        # C++ Qt application entry point
└── qml.qrc                 # Qt resource file for QML
```

### Key Technologies

- **Language**: Rust 2021 edition
- **UI**: Qt 6.10.1 / QML / Kirigami
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
   - `TodoClient` for Godo API with retry logic
   - `GitHubClient` for GitHub API with retry logic
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

**myme-core**: Core application lifecycle (`App` struct), configuration management (TOML-based, cross-platform paths), plugin system traits (`PluginProvider`, `PluginContext`).

**myme-services**: HTTP clients for external APIs. Currently includes `TodoClient` with full CRUD operations. Each client is async and uses structured logging.

**myme-ui**: cxx-qt bridge layer. Contains QObject models (e.g., `NoteModel`, `RepoModel`) that expose Rust functionality to QML. The `build.rs` configures cxx-qt code generation. QML files are in `qml/` subdirectory.

**myme-auth**: OAuth2 flows (`oauth.rs`, `github.rs`), secure token storage using system keyring (`storage.rs`). Dynamic port discovery (8080-8089) for OAuth callback.

**myme-integrations**: GitHub API wrapper (`github/`), local Git operations using git2 (`git/`). Repository discovery and clone/pull with cancellation support.

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
TodoClient::list_todos() [HTTP GET with retry logic]
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
4. Add navigation action in `Main.qml` drawer
5. Wire up data model invokable methods in QML

### Adding Authentication

1. Extend `myme-auth` with new OAuth provider (see `github.rs` as example)
2. Use `SecureStorage` to store tokens in system keyring
3. Pass tokens to service clients via bearer authentication
4. Add auth flow UI in QML if needed

## QML UI Patterns

### Theme System
- `Theme.qml` singleton in `crates/myme-ui/qml/` provides centralized colors/spacing
- Pages import theme with `import ".."` to access `Theme.background`, `Theme.text`, etc.
- Theme supports `light`, `dark`, `auto` modes via `Theme.mode` property
- New QML files must be added to `qml.qrc` for bundling

### Navigation Best Practices
- Sidebar should be outside StackView (sibling in RowLayout) to prevent reload on page changes
- Use `stackView.replace()` with `clip: true` and fade transitions to avoid visual glitches
- Track current page with a property (e.g., `currentPage: "notes"`) for sidebar highlighting

### QML-Only Changes
- QML changes don't require rebuild - just restart the application
- Only Rust bridge changes (cxx-qt) require `cargo build` + CMake rebuild

### QML JavaScript
- Qt 6.x QML supports modern ES6+ JavaScript: arrow functions, template literals, `let`/`const`, destructuring, etc.
- Use modern syntax for cleaner, more readable code

### QML Formatting
- qmlformat location: `/mnt/c/Qt/6.10.1/msvc2022_64/bin/qmlformat.exe -i <file>`

### QML Singletons
- Theme/Icons defined in `crates/myme-ui/qml/Theme.qml` and `Icons.qml`
- Registered in `crates/myme-ui/qml/qmldir`
- Phosphor Icons font used for UI icons (`crates/myme-ui/qml/fonts/Phosphor.ttf`)

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
- [crates/myme-core/src/plugin.rs](crates/myme-core/src/plugin.rs) - Plugin system traits (deferred)

### Service Layer
- [crates/myme-services/src/todo.rs](crates/myme-services/src/todo.rs) - Godo API client with retry logic
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

### Integration Tests
- [crates/myme-services/tests/todo_integration.rs](crates/myme-services/tests/todo_integration.rs) - TodoClient mock tests
- [crates/myme-services/tests/github_integration.rs](crates/myme-services/tests/github_integration.rs) - GitHubClient tests

## Integration with Godo

MyMe integrates with the Godo note-taking application (separate Golang project):

- **API Endpoints**: `/api/v1/notes` for CRUD operations
- **Authentication**: JWT Bearer tokens (optional)
- **Default Port**: 8008 (configurable in `config.toml`)
- **Client**: `TodoClient` in `myme-services/src/todo.rs`

To test with Godo:
```bash
# Terminal 1: Start Godo
cd ../godo
./godo-windows-amd64.exe

# Terminal 2: Verify Godo is running
curl http://localhost:8008/api/v1/health

# Terminal 3: Start MyMe
cd ../myme
.\build\Release\myme-qt.exe
```

## Windows-Specific Notes

**Linker Issue**: Windows builds may fail with "cannot find link.exe" due to PATH conflicts. Solutions:

1. Use "Developer Command Prompt for VS" from Start Menu
2. Configure `.cargo/config.toml` with correct linker path
3. Use WSL2 for Linux environment
4. See [WINDOWS_BUILD_FIX.md](WINDOWS_BUILD_FIX.md) for details

**Qt Path**: CMakeLists.txt currently hardcodes Qt path to `C:/Qt/6.10.1/msvc2022_64`. Update this if your Qt installation differs.

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
- `TodoClient`: `list_todos`, `get_todo`, `create_todo`, `update_todo`, `delete_todo`
- `GitHubClient`: `list_repos`, `get_repo`, `create_repo`, `list_issues`, `create_issue`, `update_issue`
- `GitOperations`: `discover_repositories`, `clone_repository`, `fetch`, `pull`, `push`
- `OAuth2Provider`: `authenticate`, `exchange_code`

**Expected Performance Baselines:**
| Operation | Typical Duration | Notes |
|-----------|------------------|-------|
| `list_todos` | 50-200ms | Depends on Godo server |
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

## Phase Roadmap

**Phase 1** (Complete): Foundation + Godo Integration
- Workspace structure, core application, Todo API client, cxx-qt bridge, QML UI

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
- Created comprehensive integration test suite (75+ tests)
- Added `#[tracing::instrument]` for performance observability
- Implemented graceful shutdown via Qt signal

**Phase 3** (Planned): Google Email/Calendar Integration
- Gmail and Calendar API clients, email/calendar QML pages

**Phase 4** (Planned): Project Scaffolding + Plugin System
- Project templates (Laravel, Drupal, Node.js), scaffold wizard UI
