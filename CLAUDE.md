# CLAUDE.md

Guidance for Claude Code working with the MyMe codebase.

## Project Overview

MyMe is a modular Rust desktop app using Qt/QML via cxx-qt — a personal productivity hub integrating GitHub, Google services (Gmail/Calendar), weather, and local SQLite storage. "Warm Forge" amber/gold theme with Outfit font and persistent sidebar nav.

## Build & Run

```bash
task build          # Full build (Rust + Qt)
task run            # Run Qt app
task os:build:rust  # Rust-only (no Qt headers needed)
```

**Testing** (excludes myme-ui which requires Qt):
```bash
cargo test -p myme-core -p myme-services -p myme-auth -p myme-integrations -p myme-weather -p myme-gmail -p myme-calendar
```

**Prerequisites**: Rust 2021+, CMake 3.16+, Qt 6.x, [Task](https://taskfile.dev). Linux deps: `qt6-base-dev qt6-declarative-dev cmake build-essential g++ libssl-dev libsecret-1-dev libxcb-cursor0` plus QML runtime modules (`qml6-module-qtquick*`).

**Windows**: May need "Developer Command Prompt for VS" for linker. See [WINDOWS_BUILD_FIX.md](WINDOWS_BUILD_FIX.md). Set `CMAKE_PREFIX_PATH` if Qt not on PATH.

**Debugging**: `RUST_LOG=debug cargo run` (Rust) / `QT_LOGGING_RULES="*.debug=true"` (Qt)

## Orchestration

| File pattern | Specialist skill | Spec |
|---|---|---|
| `crates/myme-core/*`, `crates/myme-auth/*` | myme-core-auth | `docs/specs/core-auth.md` |
| `crates/myme-services/*`, `crates/myme-integrations/*`, `crates/myme-organizations/*` | myme-data-services | `docs/specs/data-services.md` |
| `crates/myme-gmail/*`, `crates/myme-calendar/*` | myme-google-services | `docs/specs/google-services.md` |
| `crates/myme-weather/*` | myme-weather | `docs/specs/weather.md` |
| `crates/myme-ui/src/*` | myme-ui-bridge | `docs/specs/ui-bridge.md` |
| `crates/myme-ui/qml/*` | myme-qml-ui | `docs/specs/qml-ui.md` |

## Architecture Layers

```
Layer 0 (Foundation): myme-core — lifecycle, config (TOML), error types
Layer 1 (Auth):       myme-auth — OAuth2 (GitHub + Google), system keyring storage
Layer 2 (Data):       myme-services, myme-integrations, myme-organizations, myme-gmail, myme-calendar, myme-weather
Layer 3 (Bridge):     myme-ui Rust — cxx-qt QObject models (19), AppServices singleton, service channels
Layer 4 (UI):         QML — 13 pages, 9 components, Theme/Icons/AppContext singletons
```

Rule: packages import only from their layer or lower.

## Workspace Structure

```
crates/
├── myme-core/          # App lifecycle, config, error types
├── myme-auth/          # OAuth2 flows, system keyring storage
├── myme-services/      # HTTP clients, SQLite notes/projects, retry
├── myme-integrations/  # GitHub API, git2 operations
├── myme-organizations/ # Organizations & prospect pipeline (SQLite)
├── myme-gmail/         # Gmail API, SQLite cache, sync queue
├── myme-calendar/      # Google Calendar API, SQLite cache
├── myme-weather/       # Weather API, platform geolocation (WinRT/D-Bus)
└── myme-ui/            # cxx-qt bridge (19 models) + QML pages/components
```

## Common Operations

### Adding a new page
1. Create `crates/myme-ui/qml/pages/NewPage.qml`
2. Create QObject model in `crates/myme-ui/src/models/new_model.rs`
3. Create service in `crates/myme-ui/src/services/new_service.rs`
4. Register model in `crates/myme-ui/build.rs` (`.file("src/models/new_model.rs")`)
5. Add QML file to `qml.qrc`
6. Add nav entry in `Main.qml` StackView + `Sidebar.qml`

### Adding a new SQLite store
1. Create `store.rs` with `Store::open(path)` running CREATE TABLE IF NOT EXISTS
2. UUID TEXT primary keys (`uuid::Uuid::new_v4().to_string()`)
3. Timestamps as ISO 8601 TEXT, enums via `serde_json`, arrays as JSON TEXT
4. Test with `tempfile::tempdir()` for fresh databases per test

### Adding a new OAuth provider
1. Implement provider in `myme-auth/` (see `github.rs`, `google.rs`)
2. Use `SecureStorage` for tokens in system keyring
3. Dynamic port discovery on 8080-8089 for OAuth callback
4. Add `[provider]` section to `~/.config/myme/config.toml`

### Making changes
- **QML only**: Edit files, restart app (no rebuild needed)
- **Rust bridge**: `cargo build --release` then `cmake --build build-qt`
- **New QML files**: Must add to `qml.qrc` or they won't load

## Critical Gotchas

1. **Never `block_on()` the Qt thread** — three threads: Qt Main (UI), Tokio Runtime (async), mpsc channels between them. Model sends request via mpsc, Timer polls `poll_channel()` at 100ms, model receives result and emits signal
2. **cxx-qt snake_case** — QML calls Rust methods with exact snake_case: `model.fetch_data()` not `fetchData()`
3. **Register new models in build.rs** — `.file("src/models/x.rs")` in cxx-qt build config
4. **Add QML files to qml.qrc** — new files won't load without resource registration
5. **Google tokens expire hourly** — must auto-refresh; Gmail uses `historyId`, Calendar uses `syncToken`
6. **Sidebar is StackView sibling** — in `RowLayout`, not inside StackView (prevents page reload on nav)
7. **Run `cargo build` before `cmake`** — cxx-qt generates C++ bridge code that cmake needs
8. **Enum serialization** — use `#[serde(rename_all = "lowercase")]`; `serde_json::to_string()` on bare enums produces quoted strings

## Testing

120+ tests across all crates. Key patterns:
- **SQLite stores**: `tempfile::tempdir()` + fresh DB per test
- **HTTP clients**: `wiremock` mock server with `MockServer::start().await`
- **Retry logic**: configurable attempt counts in tests
- **No Qt tests**: myme-ui excluded from cargo test (needs Qt runtime)

## QML Conventions

- **Theme**: `Theme.qml` singleton — colors, spacing, fonts. Import via `import ".."`
- **Icons**: Phosphor font via `Icons.qml` singleton
- **Fonts**: Outfit variable font; use `font.weight: Font.Bold` for variants
- **Cards**: `cardRadius: 10`, `cardPadding: 20`, borders `#ffffff08` (dark) / `#00000008` (light)
- **Animations**: Staggered delegates: `PauseAnimation { duration: index * 30 }` + opacity 0→1
- **Keyboard**: Ctrl+1-9 nav, Ctrl+B sidebar toggle, Ctrl+, settings
- **ES6+**: Qt 6.x QML supports arrow functions, template literals, destructuring
- **Format**: `qmlformat -i <file>` (Qt 6 tool)

## Configuration

Platform paths: `~/.config/myme/config.toml` (Linux), `%APPDATA%\myme\config.toml` (Windows)

Key sections: `[github]` (client_id, client_secret), `[google]` (client_id, client_secret), `[notes]` (sqlite_path). Use `Config::load_validated()` for validation with warnings.

## Key Infrastructure Files

- `Cargo.toml` — workspace config with shared deps
- `CMakeLists.txt` — Qt/C++ build, links Rust static library
- `qt-main/main.cpp` — C++ entry point with shutdown handler
- `qml.qrc` — Qt resource file for QML bundling
- `crates/myme-ui/build.rs` — cxx-qt code generation config
- `crates/myme-ui/src/app_services.rs` — AppServices singleton (parking_lot::RwLock)
- `crates/myme-ui/src/bridge.rs` — C FFI functions for Qt/Rust bridge
- `.github/workflows/release.yml` — Automated releases on `v*` tags

## CI/CD

Release workflow: push `v*` tag → builds Windows (ZIP + Inno Setup installer) and Linux (tarball with bundled Qt) → publishes as GitHub Release.

## Spec Maintenance

When refactoring a subsystem, update the corresponding `docs/specs/` file. Run `tools/drift-detector.sh` to check which specs may need updates based on recent git changes.
