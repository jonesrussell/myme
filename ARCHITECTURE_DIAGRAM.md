# MyMe Architecture Diagram

## Layer Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Interface                          │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    QML/Kirigami UI                       │  │
│  │                                                          │  │
│  │  Main.qml              TodoPage.qml                     │  │
│  │  ├─ ApplicationWindow   ├─ ListView                     │  │
│  │  ├─ GlobalDrawer        ├─ TodoItem delegates           │  │
│  │  └─ Navigation          └─ Add/Complete/Delete actions  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↕ QML bindings
┌─────────────────────────────────────────────────────────────────┐
│                      Qt/C++ Bridge Layer                        │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                   cxx-qt Generated                       │  │
│  │                                                          │  │
│  │  QObject wrappers      Qt MOC integration               │  │
│  │  Signal/Slot system    QML type registration            │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↕ FFI (cxx-qt bridge)
┌─────────────────────────────────────────────────────────────────┐
│                      Rust UI Layer (myme-ui)                   │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                     TodoModel                            │  │
│  │  ┌────────────────────────────────────────────────────┐  │  │
│  │  │ #[qobject] struct TodoModelRust                    │  │  │
│  │  │                                                    │  │  │
│  │  │ Properties:                                        │  │  │
│  │  │  - loading: bool                                   │  │  │
│  │  │  - error_message: QString                          │  │  │
│  │  │  - todos: Vec<Todo>                                │  │  │
│  │  │                                                    │  │  │
│  │  │ Methods (QInvokable):                              │  │  │
│  │  │  - fetchTodos()                                    │  │  │
│  │  │  - addTodo(title, desc)                            │  │  │
│  │  │  - completeTodo(index)                             │  │  │
│  │  │  - deleteTodo(index)                               │  │  │
│  │  │  - getTitle/Description/Status/Id(index)           │  │  │
│  │  └────────────────────────────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↕ Async calls (tokio)
┌─────────────────────────────────────────────────────────────────┐
│                   Rust Service Layer (myme-services)           │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                     TodoClient                           │  │
│  │                                                          │  │
│  │  base_url: Url                                           │  │
│  │  client: Arc<reqwest::Client>                            │  │
│  │                                                          │  │
│  │  async fn listTodos() -> Result<Vec<Todo>>              │  │
│  │  async fn getTodo(id) -> Result<Todo>                    │  │
│  │  async fn createTodo(req) -> Result<Todo>                │  │
│  │  async fn updateTodo(id, req) -> Result<Todo>            │  │
│  │  async fn deleteTodo(id) -> Result<()>                   │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ↕ HTTP/JSON
┌─────────────────────────────────────────────────────────────────┐
│                      External Services                         │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Golang Todo API (Port 8080)                 │  │
│  │                                                          │  │
│  │  GET    /api/todos           → List all                  │  │
│  │  GET    /api/todos/:id       → Get one                   │  │
│  │  POST   /api/todos           → Create                    │  │
│  │  PUT    /api/todos/:id       → Update                    │  │
│  │  DELETE /api/todos/:id       → Delete                    │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Core Application Layer

```
┌─────────────────────────────────────────────────────────────────┐
│                  Rust Core Layer (myme-core)                   │
│                                                                 │
│  ┌────────────────────────────────────────────────────────┐    │
│  │                      App                               │    │
│  │  ┌──────────────────────────────────────────────────┐  │    │
│  │  │ config: Arc<Config>                              │  │    │
│  │  │ plugins: Vec<Box<dyn PluginProvider>>            │  │    │
│  │  │                                                  │  │    │
│  │  │ fn initialize()                                  │  │    │
│  │  │ fn shutdown()                                    │  │    │
│  │  │ fn registerPlugin()                              │  │    │
│  │  └──────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌────────────────────────────────────────────────────────┐    │
│  │                    Config                              │    │
│  │  ┌──────────────────────────────────────────────────┐  │    │
│  │  │ config_dir: PathBuf                              │  │    │
│  │  │ services: ServiceConfig                          │  │    │
│  │  │ ui: UiConfig                                     │  │    │
│  │  │                                                  │  │    │
│  │  │ fn load() -> Result<Config>                      │  │    │
│  │  │ fn save() -> Result<()>                          │  │    │
│  │  └──────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                 │
│  ┌────────────────────────────────────────────────────────┐    │
│  │              Plugin System (Traits)                    │    │
│  │  ┌──────────────────────────────────────────────────┐  │    │
│  │  │ trait PluginProvider                             │  │    │
│  │  │   fn id() -> &str                                │  │    │
│  │  │   fn name() -> &str                              │  │    │
│  │  │   fn initialize(&mut PluginContext)              │  │    │
│  │  │   fn shutdown()                                  │  │    │
│  │  │   fn uiComponents() -> Vec<UiComponent>          │  │    │
│  │  └──────────────────────────────────────────────────┘  │    │
│  └────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow: Add Todo Example

```
User clicks "Add Todo" button in QML
         ↓
QML calls todoModel.addTodo(title, description)
         ↓
[cxx-qt bridge FFI transition]
         ↓
TodoModel::addTodo() in Rust
         ↓
Creates TodoCreateRequest { title, description }
         ↓
Spawns tokio async task
         ↓
Calls client.createTodo(request).await
         ↓
TodoClient sends POST /api/todos with JSON body
         ↓
[HTTP request]
         ↓
Golang Todo API processes request
         ↓
Returns created Todo with ID
         ↓
[HTTP response]
         ↓
TodoClient deserializes response
         ↓
Result returned from async task
         ↓
(Future: emit Qt signal to update UI)
         ↓
QML ListView refreshes to show new todo
```

## Workspace Structure

```
myme/
├── Cargo.toml                    ← Workspace root
│   └── [workspace]
│       ├── members = [...]
│       └── [workspace.dependencies]
│
├── src/
│   └── main.rs                   ← Binary entry point
│
├── crates/
│   ├── myme-core/                ← Core application
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs
│   │       ├── config.rs
│   │       └── plugin.rs
│   │
│   ├── myme-services/            ← Service clients
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── todo.rs
│   │
│   └── myme-ui/                  ← Qt/QML bridge
│       ├── Cargo.toml
│       ├── build.rs              ← cxx-qt codegen
│       ├── src/
│       │   ├── lib.rs
│       │   └── models/
│       │       ├── mod.rs
│       │       └── todo_model.rs
│       └── qml/
│           ├── Main.qml
│           └── pages/
│               └── TodoPage.qml
│
├── qt-main/
│   └── main.cpp                  ← Qt C++ entry point
│
└── CMakeLists.txt                ← Qt build system
```

## Dependency Graph

```
Binary (main.rs)
    ├─ depends on → myme-core
    └─ depends on → myme-ui

myme-ui
    ├─ depends on → myme-services
    ├─ depends on → cxx-qt
    └─ depends on → tokio

myme-services
    ├─ depends on → reqwest (HTTP)
    ├─ depends on → tokio (async)
    └─ depends on → serde (JSON)

myme-core
    ├─ depends on → serde (config)
    ├─ depends on → toml (config)
    └─ depends on → tracing (logging)

Qt Application (main.cpp)
    ├─ links → libmyme_ui.a (Rust static lib)
    ├─ depends on → Qt6Core
    ├─ depends on → Qt6Quick
    └─ loads → Main.qml
```

## Future Phases (Preview)

```
┌─────────────────────────────────────────────────────────────┐
│ Phase 2: GitHub + Git                                       │
│                                                             │
│ myme-auth       → OAuth2, token storage                     │
│ myme-integrations                                           │
│   ├─ github/    → GitHub API (octocrab)                     │
│   └─ git/       → Local repos (git2)                        │
│ RepoListModel   → QML bridge for repos                      │
│ ReposPage.qml   → UI for repo management                    │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Phase 3: Google Integration                                 │
│                                                             │
│ myme-auth       → Google OAuth2 (Gmail, Calendar)           │
│ myme-integrations                                           │
│   └─ google/    → Gmail + Calendar clients                  │
│ EmailListModel  → QML bridge for emails                     │
│ EmailPage.qml   → UI for email viewing                      │
│ CalendarPage.qml → UI for calendar events                   │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ Phase 4: Project Scaffolding                                │
│                                                             │
│ myme-scaffolding                                            │
│   ├─ template.rs     → ProjectTemplate trait                │
│   ├─ executor.rs     → CLI command runner                   │
│   └─ templates/      → Laravel, Drupal, Node.js             │
│ ScaffoldWizard.qml   → UI for project creation              │
└─────────────────────────────────────────────────────────────┘
```

## Technology Stack Summary

```
┌──────────────┬─────────────────────────────────────────────┐
│ Layer        │ Technologies                                │
├──────────────┼─────────────────────────────────────────────┤
│ UI           │ QML, Kirigami, Qt Quick Controls           │
│ Bridge       │ cxx-qt, Qt MOC, C++ FFI                    │
│ Logic        │ Rust (2021), tokio, async/await            │
│ HTTP Client  │ reqwest, serde_json                        │
│ Config       │ TOML, dirs (cross-platform paths)          │
│ Logging      │ tracing, tracing-subscriber                │
│ Error        │ anyhow, thiserror                          │
│ Build        │ Cargo (Rust), CMake (Qt), cxx-qt-build     │
│ Platform     │ Windows 11, macOS, Linux (cross-platform)  │
└──────────────┴─────────────────────────────────────────────┘
```

## Key Design Patterns

1. **Repository Pattern**: TodoClient abstracts API access
2. **Bridge Pattern**: cxx-qt bridges Rust ↔ Qt
3. **Observer Pattern**: Qt signals/slots for UI updates
4. **Plugin Pattern**: Trait-based extensibility
5. **Factory Pattern**: Template system for project creation
6. **Singleton Pattern**: Arc-wrapped clients for sharing
7. **Async Pattern**: tokio runtime for non-blocking I/O
8. **Builder Pattern**: Config loading with defaults

---

This architecture provides a solid foundation for a full-featured productivity application while maintaining clean separation of concerns and extensibility for future features.
