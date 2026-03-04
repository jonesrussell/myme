# UI Bridge Specification

Covers `crates/myme-ui/src/` (Rust side only -- QObject models, services, bridge, build system).

## File Map

| File | Purpose |
|------|---------|
| `crates/myme-ui/src/lib.rs` | Module declarations, re-exports QObject models (NoteModel, GmailModel, etc.) |
| `crates/myme-ui/src/app_services.rs` | `AppServices` singleton: runtime, clients, service channels, shutdown |
| `crates/myme-ui/src/bridge.rs` | C FFI functions (`#[no_mangle] extern "C"`) for Qt initialization + shutdown |
| `crates/myme-ui/src/models/mod.rs` | 18 model modules (auth, calendar, encoding, gmail, google_auth, hash, json, jwt, kanban, note, organization, project, prospect, repo, time, uuid, weather, workflow) |
| `crates/myme-ui/src/models/note_model.rs` | NoteModel QObject: cxx-qt bridge, channel-based async for note CRUD |
| `crates/myme-ui/src/models/gmail_model.rs` | GmailModel QObject: Gmail message listing, actions |
| `crates/myme-ui/src/models/organization_model.rs` | OrganizationModel QObject: organization/prospect CRUD |
| `crates/myme-ui/src/services/mod.rs` | 11 service modules, message type re-exports |
| `crates/myme-ui/src/services/note_service.rs` | NoteServiceMessage enum, async note operations |
| `crates/myme-ui/src/services/organization_service.rs` | OrganizationServiceMessage enum, async org operations |
| `crates/myme-ui/src/error_mapping/mod.rs` | Maps service errors to `myme_core::AppError` for UI display |
| `crates/myme-ui/build.rs` | cxx-qt build configuration: registers all 18 model files |

## Interface Signatures

### AppServices Singleton (crates/myme-ui/src/app_services.rs)

```rust
pub struct AppServices {
    runtime: tokio::runtime::Runtime,
    shutdown_tx: broadcast::Sender<()>,
    note_client: RwLock<Option<Arc<NoteClient>>>,
    github_client: RwLock<Option<Arc<GitHubClient>>>,
    github_auth: RwLock<Option<Arc<GitHubAuth>>>,
    project_store: RwLock<Option<Arc<parking_lot::Mutex<ProjectStore>>>>,
    organization_store: RwLock<Option<Arc<parking_lot::Mutex<OrganizationStore>>>>,
    weather_provider: RwLock<Option<Arc<WeatherProvider>>>,
    weather_cache: RwLock<Option<parking_lot::Mutex<WeatherCache>>>,
    // 10 service channel pairs (tx + rx) for: repo, note, weather, auth, project, workflow, kanban, gmail, calendar, organization
    repo_cancel_token: RwLock<Option<Arc<CancellationToken>>>,
}

static SERVICES: OnceLock<Arc<AppServices>> = OnceLock::new();

impl AppServices {
    pub fn init() -> Arc<Self>;                    // singleton, creates tokio runtime
    pub fn runtime(&self) -> tokio::runtime::Handle;
    pub fn subscribe_shutdown(&self) -> broadcast::Receiver<()>;
    pub fn shutdown(&self);                        // broadcasts signal, clears all state

    // Client accessors
    pub fn note_client(&self) -> Option<Arc<NoteClient>>;
    pub fn set_note_client(&self, client: Option<Arc<NoteClient>>);
    pub fn init_note_client(&self) -> bool;
    pub fn github_client(&self) -> Option<Arc<GitHubClient>>;
    pub fn set_github_client(&self, client: Option<Arc<GitHubClient>>);
    pub fn init_github_client(&self) -> bool;      // reads token from keyring
    pub fn clear_github_client(&self);
    pub fn is_github_authenticated(&self) -> bool;
    pub fn github_auth(&self) -> Option<Arc<GitHubAuth>>;
    pub fn init_github_auth(&self) -> bool;        // reads config for client_id/secret
    pub fn project_store(&self) -> Option<Arc<parking_lot::Mutex<ProjectStore>>>;
    pub fn init_project_store(&self) -> bool;
    pub fn organization_store(&self) -> Option<Arc<parking_lot::Mutex<OrganizationStore>>>;
    pub fn init_organization_store(&self) -> bool;
    pub fn weather_provider(&self) -> Option<Arc<WeatherProvider>>;
    pub fn weather_cache(&self) -> Option<WeatherCache>;  // lazy-init on first use
    pub fn init_weather_services(&self) -> bool;

    // Service channel methods (macro-generated for each of 10 services)
    pub fn {svc}_service_tx(&self) -> Option<mpsc::Sender<{Svc}ServiceMessage>>;
    pub fn init_{svc}_service_channel(&self) -> bool;
    pub fn try_recv_{svc}_message(&self) -> Option<{Svc}ServiceMessage>;

    // Repo cancellation
    pub fn new_repo_cancel_token(&self) -> Arc<CancellationToken>;
    pub fn repo_cancel_token(&self) -> Option<Arc<CancellationToken>>;
    pub fn cancel_repo_operation(&self);
    pub fn clear_repo_cancel_token(&self);
}

// Convenience functions
pub fn services() -> Arc<AppServices>;
pub fn runtime() -> tokio::runtime::Handle;
pub fn note_client_and_runtime() -> Option<(Arc<NoteClient>, tokio::runtime::Handle)>;
pub fn note_client_or_init() -> Option<Arc<NoteClient>>;
pub fn github_client_and_runtime() -> Option<(Arc<GitHubClient>, tokio::runtime::Handle)>;
pub fn github_auth_and_runtime() -> Option<(Arc<GitHubAuth>, tokio::runtime::Handle)>;
pub fn project_store() -> Option<Arc<parking_lot::Mutex<ProjectStore>>>;
pub fn project_store_or_init() -> Option<Arc<parking_lot::Mutex<ProjectStore>>>;
pub fn organization_store_or_init() -> Option<Arc<parking_lot::Mutex<OrganizationStore>>>;
pub fn weather_services() -> Option<(Arc<WeatherProvider>, WeatherCache, tokio::runtime::Handle)>;
pub fn is_github_authenticated() -> bool;
pub fn get_repos_local_search_path() -> Option<(PathBuf, bool)>;
```

### C FFI Bridge (crates/myme-ui/src/bridge.rs)

```rust
// Called from qt-main/main.cpp before QML engine starts
#[no_mangle] pub extern "C" fn initialize_note_client() -> bool;
#[no_mangle] pub extern "C" fn initialize_weather_services() -> bool;
#[no_mangle] pub extern "C" fn initialize_github_client() -> bool;
#[no_mangle] pub extern "C" fn initialize_github_auth() -> bool;
#[no_mangle] pub extern "C" fn shutdown_app_services();  // connected to QCoreApplication::aboutToQuit

// Rust-internal bridge functions (not extern "C")
pub fn get_note_client_and_runtime() -> Option<(Arc<NoteClient>, tokio::runtime::Handle)>;
pub fn get_note_client_or_init() -> Option<Arc<NoteClient>>;
pub fn get_weather_services() -> Option<(Arc<WeatherProvider>, WeatherCache, tokio::runtime::Handle)>;
pub fn get_github_client_and_runtime() -> Option<(Arc<GitHubClient>, tokio::runtime::Handle)>;
pub fn get_project_store() -> Option<Arc<parking_lot::Mutex<ProjectStore>>>;
pub fn get_project_store_or_init() -> Option<Arc<parking_lot::Mutex<ProjectStore>>>;
pub fn get_organization_store_or_init() -> Option<Arc<parking_lot::Mutex<OrganizationStore>>>;
pub fn get_runtime() -> Option<tokio::runtime::Handle>;
pub fn is_github_authenticated() -> bool;
pub fn get_repos_local_search_path() -> Option<(PathBuf, bool)>;
pub fn reinitialize_github_client();              // after OAuth success
pub fn clear_github_client();                     // on sign-out
pub fn shutdown_services();
pub fn new_repo_cancel_token() -> Arc<CancellationToken>;
pub fn cancel_repo_operation();
pub fn clear_repo_cancel_token();

// Macro-generated for each of 10 services:
pub fn init_{svc}_service_channel() -> bool;
pub fn get_{svc}_service_tx() -> Option<mpsc::Sender<{Svc}ServiceMessage>>;
pub fn try_recv_{svc}_message() -> Option<{Svc}ServiceMessage>;
```

### Service Message Types (crates/myme-ui/src/services/mod.rs)

```rust
// Each service has a message enum with request and response variants.
// The pattern is consistent across all services:

pub enum NoteServiceMessage {
    FetchResult(Result<Vec<Todo>, NoteError>),
    CreateResult(Result<Todo, NoteError>),
    UpdateResult(Result<Todo, NoteError>),
    DeleteResult(Result<(), NoteError>),
    ToggleDoneResult(Result<Todo, NoteError>),
}

pub enum RepoServiceMessage { /* RefreshResult, CloneResult, PullResult */ }
pub enum WeatherServiceMessage { /* FetchResult */ }
pub enum AuthServiceMessage { /* AuthResult */ }
pub enum ProjectServiceMessage { /* FetchRepoResult */ }
pub enum WorkflowServiceMessage { /* FetchResult */ }
pub enum KanbanServiceMessage { /* SyncResult, CreateResult, UpdateResult */ }
pub enum GmailServiceMessage { /* FetchResult, MarkReadResult, ArchiveResult, TrashResult */ }
pub enum CalendarServiceMessage { /* FetchEventsResult, FetchTodayResult */ }
pub enum OrganizationServiceMessage { /* FetchResult, CreateOrgResult, ... */ }
```

### Error Mapping (crates/myme-ui/src/error_mapping/mod.rs)

```
Submodules: auth, calendar, gmail, kanban, note, project, repo, weather, workflow
Each maps its service-specific error to myme_core::AppError for user_message() display.
```

### Build Configuration (crates/myme-ui/build.rs)

```rust
CxxQtBuilder::new_qml_module(QmlModule::new("myme_ui"))
    .file("src/models/auth_model.rs")
    .file("src/models/calendar_model.rs")
    .file("src/models/encoding_model.rs")
    .file("src/models/gmail_model.rs")
    .file("src/models/google_auth_model.rs")
    .file("src/models/hash_model.rs")
    .file("src/models/json_model.rs")
    .file("src/models/jwt_model.rs")
    .file("src/models/kanban_model.rs")
    .file("src/models/note_model.rs")
    .file("src/models/organization_model.rs")
    .file("src/models/project_model.rs")
    .file("src/models/prospect_model.rs")
    .file("src/models/repo_model.rs")
    .file("src/models/workflow_model.rs")
    .file("src/models/time_model.rs")
    .file("src/models/uuid_model.rs")
    .file("src/models/weather_model.rs")
    .build();
```

### ProspectModel Invokables (crates/myme-ui/src/models/prospect_model.rs)

```rust
// All methods are #[qinvokable] — called from QML with snake_case names.

impl ProspectModelRust {
    // Per-row field accessors
    pub fn get_prospect_source_url(&self, index: i32) -> QString;    // "" if index out of range or field absent
    pub fn get_prospect_closing_date(&self, index: i32) -> QString;  // "" if index out of range or field absent

    // Sorting / filtering
    pub fn lead_prospects_by_urgency(&self) -> QString;  // JSON array of Lead prospect indices sorted by closing_date ASC, None last

    // Organization notes
    pub fn get_org_notes(&self) -> QString;              // notes for the current organization
    pub fn set_org_notes(&self, notes: &QString);        // save notes for the current organization
}
```

## Data Flow

### Channel-Based Async Pattern (used by all models)

```
1. QML calls model.some_action()          [#[qinvokable], snake_case]
2. Model gets sender: bridge::get_{svc}_service_tx()
3. Model spawns tokio task on runtime:
     runtime.spawn(async move {
         let result = do_async_work().await;
         tx.send({Svc}ServiceMessage::ActionResult(result));
     });
4. QML Timer (100ms) calls model.poll_channel()  [#[qinvokable]]
5. Model calls bridge::try_recv_{svc}_message()
6. On Some(message):
   - Update model state (properties)
   - Emit signal to notify QML
7. QML reacts to signal, updates UI
```

### Initialization Sequence

```
1. qt-main/main.cpp starts Qt app
2. Calls extern "C" functions:
   - initialize_note_client()     -> opens SQLite, creates NoteClient
   - initialize_weather_services() -> creates WeatherProvider + cache
   - initialize_github_client()   -> reads keyring token, creates GitHubClient + ProjectStore
   - initialize_github_auth()     -> reads config, creates GitHubAuth provider
3. QML engine starts, models created
4. Each model in Component.onCompleted:
   - Calls init_{svc}_service_channel() (lazy channel creation)
   - Starts 100ms polling Timer
   - Calls initial fetch (e.g., fetch_notes(), fetch_weather())
```

### Shutdown Sequence

```
1. Qt aboutToQuit signal fires
2. main.cpp calls shutdown_app_services() [extern "C"]
3. AppServices::shutdown():
   a. Broadcasts shutdown via broadcast::channel
   b. Clears all client references (note_client, github_client, etc.)
   c. Clears all service channel senders/receivers (10 pairs)
   d. Cancels active repo operations via CancellationToken
4. Tokio runtime drops on process exit
```

## Storage / Schema

No direct storage in myme-ui. All persistence delegated to:
- `myme-services`: NoteClient (SQLite), ProjectStore (SQLite)
- `myme-organizations`: OrganizationStore (SQLite)
- `myme-gmail`: GmailCache (SQLite), SyncQueue (SQLite)
- `myme-calendar`: CalendarCache (SQLite)
- `myme-weather`: WeatherCache (JSON file)
- `myme-auth`: SecureStorage (system keyring)

## Configuration

AppServices reads config via `myme_core::Config::load_cached()`:
- `notes.sqlite_path` -> NoteClient database path
- `github.client_id/secret` -> GitHubAuth provider
- `weather.temperature_unit` -> WeatherProvider unit
- `repos.local_search_path` -> repo discovery path

## Edge Cases

- **OnceLock singleton**: `AppServices::init()` is idempotent; second call returns same Arc
- **Runtime creation failure**: `expect()` panics -- this is considered fatal, no recovery
- **Service channel not initialized**: `get_{svc}_service_tx()` returns `None`; model silently skips operation
- **Poll with no channel**: `try_recv_{svc}_message()` returns `None` harmlessly
- **Token expired mid-session**: `init_github_client()` checks `token_set.is_expired()` before creating client
- **Reinitialize after OAuth**: `reinitialize_github_client()` clears old client, creates new from keyring
- **Concurrent access**: All RwLock fields use `parking_lot::RwLock` for better performance than std
- **Macro-generated methods**: `service_channel_methods!` and `service_channel_bridge!` macros generate identical patterns for all 10 services
- **cxx-qt naming**: Rust methods exposed with snake_case names (no camelCase conversion); QML must use `model.fetch_notes()` not `model.fetchNotes()`
- **Model registration order**: All 18 model files must be listed in `build.rs`; missing files cause compile errors in generated C++ code
- **No block_on()**: All async operations go through channels; UI thread never blocks
- **Shutdown signal fan-out**: `broadcast::channel(16)` supports up to 16 concurrent subscribers
