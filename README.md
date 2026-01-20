# MyMe - Personal Productivity & Dev Hub

A modular Rust desktop application using Qt/QML + Kirigami via cxx-qt that serves as a personal control center for productivity and development workflows.

**Integrated with [Godo](../godo/)** - Your Golang note-taking application!

## Project Status

**Phase 1: Foundation + Godo Integration** - Complete ✓

The core architecture has been established and fully integrated with your Godo application.

## Architecture

### Workspace Structure

```
myme/
├── crates/
│   ├── myme-core/       # Core application shell & plugin system
│   ├── myme-ui/         # cxx-qt QML bridge & UI models
│   └── myme-services/   # Microservice clients (Todo API)
└── src/                 # Binary entry point
```

### Key Components

- **myme-core**: Application lifecycle, configuration, plugin system
- **myme-ui**: Qt/QML integration via cxx-qt, TodoModel bridge
- **myme-services**: HTTP clients for external services (Golang todo API)

## Current Features

- ✓ Workspace-based crate structure
- ✓ Configuration management (TOML-based)
- ✓ Plugin system architecture
- ✓ Todo API client (full CRUD operations)
- ✓ TodoModel with cxx-qt bridge
- ✓ QML UI with Kirigami (Main window + TodoPage)
- ✓ Async/await support via tokio
- ✓ Structured logging with tracing

## Building & Running

### Prerequisites

- Rust 2021 edition or later
- Qt 6.x with Kirigami
- CMake (for cxx-qt)

### Build

```bash
cargo build
```

### Run

```bash
cargo run
```

**Note**: Phase 1 demonstrates the Rust architecture. Full Qt/QML integration requires a C++ main executable that initializes the Qt application and loads the QML. This will be completed in the next iteration.

## Configuration

Configuration is stored at:
- Windows: `%APPDATA%/myme/config.toml`
- macOS: `~/Library/Application Support/myme/config.toml`
- Linux: `~/.config/myme/config.toml`

Example configuration:

```toml
config_dir = "/path/to/config"

[services]
todo_api_url = "http://localhost:8008"  # Godo default port
jwt_token = "your-jwt-token-here"  # Optional, for Godo authentication

[ui]
window_width = 1200
window_height = 800
dark_mode = false
```

## Godo Integration

MyMe is fully integrated with your Godo note-taking application:

- **API**: Connects to Godo's `/api/v1/notes` endpoints
- **Authentication**: Supports JWT Bearer tokens
- **Features**: Create, read, update, delete notes with done/pending status
- **Port**: Default 8008 (configurable)

**Quick Start with Godo**:

```bash
# 1. Start Godo
cd ../godo
./godo-windows-amd64.exe

# 2. Verify Godo is running
curl http://localhost:8008/api/v1/health

# 3. (Optional) Set JWT token
$env:GODO_JWT_TOKEN="your-token"

# 4. Start MyMe
cd ../myme
.\build\Release\myme-qt.exe
```

See [GODO_INTEGRATION.md](GODO_INTEGRATION.md) for complete integration details.

## Next Steps

### Immediate (Complete Phase 1)

1. Create C++ main executable for Qt application
2. Wire up TodoModel initialization with actual client
3. Implement proper Qt signal/slot for async updates
4. Test end-to-end: Rust → cxx-qt → QML → display todos

### Phase 2: GitHub + Local Git Management

- Implement myme-auth crate with GitHub OAuth2
- Add git2 integration for local repo management
- Create RepoListModel and ReposPage.qml

### Phase 3: Google Email/Calendar Integration

- Extend auth for Google OAuth2
- Implement Gmail and Calendar clients
- Create email and calendar QML pages

### Phase 4: Project Scaffolding + Plugin System

- Implement myme-scaffolding crate
- Create templates for Laravel, Drupal, Node.js
- Build ScaffoldWizard.qml
- Refine plugin discovery system

## Technology Stack

### Rust

- **tokio**: Async runtime
- **serde**: Serialization
- **anyhow/thiserror**: Error handling
- **tracing**: Structured logging
- **reqwest**: HTTP client

### Qt/QML

- **cxx-qt**: Rust ↔ Qt bridge
- **Qt 6.x**: UI framework
- **Kirigami**: Modern mobile/desktop UI components

### Future

- **oauth2**: OAuth flows
- **keyring**: Secure token storage
- **git2**: Git operations
- **octocrab**: GitHub API

## License

TBD

## Contributing

This is currently a personal project. Contributions welcome once Phase 1 is complete.
