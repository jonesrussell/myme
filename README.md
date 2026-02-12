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

- Rust 2021 edition or later
- Qt 6.x
- CMake 3.16+
- C++ compiler (Visual Studio 2019+ on Windows)

### Build

```powershell
# All-in-one build (Windows)
.\build-qt.ps1

# Manual build
cargo build --release
mkdir build-qt && cd build-qt
cmake .. && cmake --build . --config Release
```

### Run

```bash
.\build\Release\myme-qt.exe
```

### Test

```bash
# Test individual crates
cargo test -p myme-core
cargo test -p myme-services

# Test all non-Qt crates
cargo test -p myme-core -p myme-services -p myme-auth -p myme-integrations
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
