# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MyMe is a modular Rust desktop application using Qt/QML + Kirigami via cxx-qt that serves as a personal productivity and development hub. It integrates with external services (Godo note-taking app, GitHub, Google services) through a plugin-based architecture.

**Current Status**: Phase 1 complete (Foundation + Godo Integration). Phase 2 UI complete (Qt/CMake build working, GitHub OAuth2 UI ready). Async callback implementation pending.

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

# Test entire workspace
cargo test --workspace
```

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

3. **Async-First with tokio**: Non-blocking I/O for network calls. Async tasks are spawned from Qt bridge methods using `tokio::spawn()` to avoid blocking the UI thread.

4. **Plugin System (Trait-Based)**: The `PluginProvider` trait in `myme-core/src/plugin.rs` defines the interface for all plugins. Static compilation with feature flags (not dynamic loading).

5. **Service Client Pattern**: Each external service (Todo API, GitHub API, etc.) has its own async client in `myme-services`. Clients use `Arc<reqwest::Client>` for shared HTTP connections.

### Component Responsibilities

**myme-core**: Core application lifecycle (`App` struct), configuration management (TOML-based, cross-platform paths), plugin system traits (`PluginProvider`, `PluginContext`).

**myme-services**: HTTP clients for external APIs. Currently includes `TodoClient` with full CRUD operations. Each client is async and uses structured logging.

**myme-ui**: cxx-qt bridge layer. Contains QObject models (e.g., `NoteModel`, `RepoModel`) that expose Rust functionality to QML. The `build.rs` configures cxx-qt code generation. QML files are in `qml/` subdirectory.

**myme-auth** (Phase 2): OAuth2 flows (`oauth.rs`, `github.rs`), secure token storage using system keyring (`storage.rs`).

**myme-integrations** (Phase 2): GitHub API wrapper using octocrab (`github/`), local Git operations using git2 (`git/`).

### Data Flow: QML → Rust → API

```
NotePage.qml (user clicks button)
  ↓
NoteModel::fetch_notes() [QML invokable method]
  ↓
tokio::spawn(async move { ... }) [Spawn async task]
  ↓
TodoClient::list_todos() [HTTP GET to API]
  ↓
Parse JSON response
  ↓
Emit Qt signal to update UI
```

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

## Important Files

- [Cargo.toml](Cargo.toml) - Workspace configuration with shared dependencies
- [CMakeLists.txt](CMakeLists.txt) - Qt/C++ build system, links Rust library
- [crates/myme-core/src/config.rs](crates/myme-core/src/config.rs) - Configuration management
- [crates/myme-core/src/plugin.rs](crates/myme-core/src/plugin.rs) - Plugin system traits
- [crates/myme-services/src/todo.rs](crates/myme-services/src/todo.rs) - Example service client
- [crates/myme-ui/src/models/note_model.rs](crates/myme-ui/src/models/note_model.rs) - Example cxx-qt bridge
- [crates/myme-ui/build.rs](crates/myme-ui/build.rs) - cxx-qt build configuration
- [qt-main/main.cpp](qt-main/main.cpp) - C++ Qt application entry point
- [qml.qrc](qml.qrc) - Qt resource file for bundling QML

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

## Phase Roadmap

**Phase 1** (Complete): Foundation + Godo Integration
- Workspace structure, core application, Todo API client, cxx-qt bridge, QML UI

**Phase 2** (In Progress): GitHub + Local Git Management
- OAuth2 authentication, git2 integration, repository management UI

**Phase 3** (Planned): Google Email/Calendar Integration
- Gmail and Calendar API clients, email/calendar QML pages

**Phase 4** (Planned): Project Scaffolding + Plugin System
- Project templates (Laravel, Drupal, Node.js), scaffold wizard UI
