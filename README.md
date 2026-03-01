# MyMe - Developer Productivity Hub

A modular Rust desktop application using Qt/QML via cxx-qt that serves as a personal control center for developer productivity workflows. Consolidates notes, GitHub/git management, developer tools, weather, Google email/calendar, and project management into a single dashboard.

## Project Status

- **Phase 1**: Foundation + Notes (SQLite) - Complete
- **Phase 2**: GitHub + Local Git Management - Complete
- **Phase 3**: Google Email/Calendar Integration - Complete
- **Dev Tools**: JWT, encoding, UUID, hashing, JSON, time utilities - Complete
- **Weather**: Open-Meteo API with geolocation and caching - Complete
- **Projects/Kanban**: SQLite-backed project management with GitHub sync - In Progress
- **2026 Architectural Modernization**: 18-step refactoring - Complete

## Architecture

### Workspace Structure

```
myme/
├── crates/
│   ├── myme-core/          # Application lifecycle, config, error types
│   ├── myme-ui/            # cxx-qt bridge, QML models, dev tools models
│   ├── myme-services/      # HTTP API clients, project store (SQLite)
│   ├── myme-auth/          # OAuth2 flows, secure token storage (keyring)
│   ├── myme-integrations/  # GitHub API, Git operations (git2)
│   ├── myme-weather/       # Weather API (Open-Meteo), location services, SQLite cache
│   ├── myme-gmail/         # Gmail API client
│   └── myme-calendar/      # Google Calendar API client
├── qt-main/main.cpp        # C++ Qt application entry point
└── qml.qrc                 # Qt resource file for QML
```

### Key Technologies

- **Language**: Rust 2021 edition
- **UI**: Qt 6.x / QML
- **Bridge**: cxx-qt 0.8 (Rust <-> Qt FFI)
- **Build**: CMake + Cargo
- **Async**: tokio with full features
- **HTTP**: reqwest with retry logic

### Architectural Patterns

- **Channel-Based Async**: mpsc channels + QML Timer polling for non-blocking UI
- **AppServices Singleton**: Centralized service container with `parking_lot::RwLock`
- **Secure Token Storage**: System keyring (Windows Credential Manager, macOS Keychain, Linux Secret Service)
- **Operation Cancellation**: `CancellationToken` for long-running git operations
- **Graceful Shutdown**: Qt `aboutToQuit` signal triggers Rust cleanup via C FFI

## Building & Running

### Prerequisites

**All platforms:**
- Rust 2021 edition or later
- CMake 3.16+
- [Task](https://taskfile.dev) (task runner)

**Windows:**
- Qt 6.10.2 (or 6.x) with MSVC 2022 kit
- Visual Studio 2019+ with C++ workload

**Linux / WSL2 (Ubuntu/Debian):**
```bash
# Build dependencies
sudo apt install qt6-base-dev qt6-declarative-dev cmake build-essential g++ \
    libssl-dev libsecret-1-dev libxcb-cursor0

# QML runtime modules (required to run the app)
sudo apt install qml6-module-qtquick qml6-module-qtquick-controls \
    qml6-module-qtquick-layouts qml6-module-qtqml-workerscript
```
WSL2 with WSLg supports GUI apps natively.

### Build

```bash
task build          # Full build (Rust + Qt app)
task os:build:rust  # Rust crates only (Linux, no Qt required)
```

### Run

```bash
task run
```

### Test

```bash
task test   # All non-Qt crates

# Or individual crates
cargo test -p myme-core
cargo test -p myme-services
```

## Configuration

Configuration is stored at platform-specific locations:
- Windows: `%APPDATA%/myme/config.toml`
- macOS: `~/Library/Application Support/myme/config.toml`
- Linux: `~/.config/myme/config.toml`

Default config is created automatically on first run.

## License

MIT

## Contributing

This is currently a personal project. Contributions welcome in the future.
