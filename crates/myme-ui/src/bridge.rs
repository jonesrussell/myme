//! C FFI bridge for Qt/QML initialization.
//!
//! This module provides the C-callable functions that Qt/QML uses to initialize
//! the Rust services. Internally, it delegates to `AppServices` for state management.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Arc;

use myme_auth::GitHubAuth;
use myme_services::{GitHubClient, NoteClient, ProjectStore, TodoClient};
use myme_weather::{WeatherCache, WeatherProvider};

use crate::app_services::{self, AppServices};

// =========== C FFI Initialization Functions ===========

/// Initialize the NoteModel with a TodoClient
/// Must be called before QML tries to access it
#[no_mangle]
pub extern "C" fn initialize_note_model(base_url: *const c_char) -> bool {
    // Initialize tracing if not already done
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    // Convert C string to Rust string
    let base_url_str = unsafe {
        if base_url.is_null() {
            "http://localhost:8080"
        } else {
            match CStr::from_ptr(base_url).to_str() {
                Ok(s) => s,
                Err(_) => {
                    tracing::error!("Invalid UTF-8 in base_url");
                    return false;
                }
            }
        }
    };

    tracing::info!("Initializing NoteModel with base_url: {}", base_url_str);

    // Get JWT token and cert config from config file or environment variable
    let (jwt_token, allow_invalid_certs) = match myme_core::Config::load() {
        Ok(config) => {
            let token = if let Some(token) = config.services.jwt_token {
                tracing::info!("Using JWT token from config file");
                Some(token)
            } else if let Ok(token) = std::env::var("GODO_JWT_TOKEN") {
                tracing::info!("Using JWT token from GODO_JWT_TOKEN environment variable");
                Some(token)
            } else {
                tracing::warn!(
                    "No JWT token configured - API calls may fail if authentication is required"
                );
                None
            };
            (token, config.services.allow_invalid_certs)
        }
        Err(e) => {
            tracing::warn!(
                "Failed to load config: {}. Checking environment variable.",
                e
            );
            (std::env::var("GODO_JWT_TOKEN").ok(), false)
        }
    };

    // Initialize via AppServices
    let services = AppServices::init();
    let success = services.init_todo_client(base_url_str, jwt_token, allow_invalid_certs);

    if success {
        tracing::info!("NoteModel services initialized successfully");
    }

    success
}

/// Get the initialized TodoClient and runtime for use by NoteModels (legacy).
/// Prefer using `get_note_client_and_runtime()` for new code.
pub fn get_todo_client_and_runtime() -> Option<(Arc<TodoClient>, tokio::runtime::Handle)> {
    app_services::todo_client_and_runtime()
}

/// Initialize unified note client from configuration.
///
/// This is the preferred initialization method. It reads the config to determine
/// whether to use SQLite (default) or HTTP (legacy Godo) backend.
///
/// Must be called before QML tries to access NoteModel.
#[no_mangle]
pub extern "C" fn initialize_note_client() -> bool {
    // Initialize tracing if not already done
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .try_init();

    tracing::info!("Initializing unified note client from configuration");

    let services = AppServices::init();
    let success = services.init_note_client();

    if success {
        tracing::info!("Unified note client initialized successfully");
    }

    success
}

/// Get the unified note client and runtime for use by NoteModels.
pub fn get_note_client_and_runtime() -> Option<(Arc<NoteClient>, tokio::runtime::Handle)> {
    app_services::note_client_and_runtime()
}

/// Get unified note client, initializing if needed.
pub fn get_note_client_or_init() -> Option<Arc<NoteClient>> {
    app_services::note_client_or_init()
}

/// Initialize weather services
/// Must be called before QML tries to access WeatherModel
#[no_mangle]
pub extern "C" fn initialize_weather_services() -> bool {
    let services = AppServices::init();
    services.init_weather_services()
}

/// Get the initialized weather services for use by WeatherModels
pub fn get_weather_services(
) -> Option<(Arc<WeatherProvider>, WeatherCache, tokio::runtime::Handle)> {
    app_services::weather_services()
}

/// Initialize GitHub client and project store
/// Must be called before QML tries to access ProjectModel
#[no_mangle]
pub extern "C" fn initialize_github_client() -> bool {
    let services = AppServices::init();

    // Initialize GitHub client from secure storage
    let github_ok = services.init_github_client();

    // Initialize project store (always, even without GitHub)
    let store_ok = services.init_project_store();

    if github_ok {
        tracing::info!("GitHub client and project store initialized");
    } else if store_ok {
        tracing::info!("Project store initialized (GitHub client not available)");
    }

    github_ok
}

/// Get GitHub client and runtime
pub fn get_github_client_and_runtime() -> Option<(Arc<GitHubClient>, tokio::runtime::Handle)> {
    app_services::github_client_and_runtime()
}

/// Get project store (legacy API with std::sync::Mutex)
///
/// This wraps the parking_lot::Mutex store for backward compatibility.
/// New code should use `app_services::project_store()` directly.
pub fn get_project_store() -> Option<Arc<std::sync::Mutex<ProjectStore>>> {
    // For backward compatibility, we keep a separate std::sync::Mutex store
    // This will be removed when all models are updated to use parking_lot
    static LEGACY_STORE: std::sync::OnceLock<Arc<std::sync::Mutex<ProjectStore>>> =
        std::sync::OnceLock::new();

    // If we already have a legacy store, return it
    if let Some(store) = LEGACY_STORE.get() {
        return Some(store.clone());
    }

    // Otherwise, create one from the same database
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("myme");
    let db_path = config_dir.join("projects.db");

    // Ensure directory exists
    std::fs::create_dir_all(&config_dir).ok()?;

    match ProjectStore::open(&db_path) {
        Ok(store) => {
            let arc = Arc::new(std::sync::Mutex::new(store));
            let _ = LEGACY_STORE.set(arc.clone());
            tracing::info!("Legacy project store initialized (std::sync::Mutex)");
            Some(arc)
        }
        Err(e) => {
            tracing::error!("Failed to open legacy project store: {}", e);
            None
        }
    }
}

/// Get project store (new API using parking_lot)
pub fn get_project_store_parking_lot() -> Option<Arc<parking_lot::Mutex<ProjectStore>>> {
    app_services::project_store()
}

/// Check if GitHub is authenticated
pub fn is_github_authenticated() -> bool {
    app_services::is_github_authenticated()
}

/// Get the runtime handle (always available after any initialization)
pub fn get_runtime() -> Option<tokio::runtime::Handle> {
    Some(app_services::runtime())
}

/// Get project store, initializing if needed (legacy API with std::sync::Mutex)
///
/// This wraps the parking_lot::Mutex store for backward compatibility.
/// New code should use `app_services::project_store_or_init()` directly.
pub fn get_project_store_or_init() -> Option<Arc<std::sync::Mutex<ProjectStore>>> {
    // Also ensure the parking_lot version is initialized
    let _ = app_services::project_store_or_init();
    // Return the legacy std::sync::Mutex version
    get_project_store()
}

/// Get project store, initializing if needed (new API using parking_lot)
pub fn get_project_store_or_init_parking_lot() -> Option<Arc<parking_lot::Mutex<ProjectStore>>> {
    app_services::project_store_or_init()
}

/// Initialize GitHub OAuth provider
/// Must be called before QML tries to use AuthModel
#[no_mangle]
pub extern "C" fn initialize_github_auth() -> bool {
    let services = AppServices::init();
    services.init_github_auth()
}

/// Get GitHub auth provider and runtime for use by AuthModel
pub fn get_github_auth_and_runtime() -> Option<(Arc<GitHubAuth>, tokio::runtime::Handle)> {
    app_services::github_auth_and_runtime()
}

/// Get effective repos local search path and whether config path was invalid.
/// Returns (effective_path, config_path_invalid).
pub fn get_repos_local_search_path() -> Option<(std::path::PathBuf, bool)> {
    app_services::get_repos_local_search_path()
}

/// Initialize repo service channel. Call once when RepoModel is first created.
/// Returns true if initialized (or already initialized).
pub fn init_repo_service_channel() -> bool {
    AppServices::init().init_repo_service_channel()
}

/// Get repo service sender for request_* calls. None if init_repo_service_channel not called yet.
pub fn get_repo_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::RepoServiceMessage>> {
    AppServices::init().repo_service_tx()
}

/// Non-blocking recv from repo service channel. Called by RepoModel::poll_channel.
pub fn try_recv_repo_message() -> Option<crate::services::RepoServiceMessage> {
    AppServices::init().try_recv_repo_message()
}

/// Initialize note service channel. Call once when NoteModel is first created.
/// Returns true if initialized (or already initialized).
pub fn init_note_service_channel() -> bool {
    AppServices::init().init_note_service_channel()
}

/// Get note service sender for request_* calls. None if init_note_service_channel not called yet.
pub fn get_note_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::NoteServiceMessage>> {
    AppServices::init().note_service_tx()
}

/// Non-blocking recv from note service channel. Called by NoteModel::poll_channel.
pub fn try_recv_note_message() -> Option<crate::services::NoteServiceMessage> {
    AppServices::init().try_recv_note_message()
}

/// Reinitialize GitHub client after successful OAuth
/// Call this after authentication completes to refresh the client with new token
///
/// Unlike the old OnceLock-based implementation, this now properly replaces
/// the existing client with a new one using the fresh token.
pub fn reinitialize_github_client() {
    tracing::info!("Reinitializing GitHub client after OAuth...");

    let services = AppServices::init();

    // Clear old client first
    services.clear_github_client();

    // Initialize with new token from secure storage
    if services.init_github_client() {
        tracing::info!("GitHub client reinitialized with new token");
    } else {
        tracing::warn!("Failed to reinitialize GitHub client");
    }
}

/// Clear GitHub client (e.g., on sign-out)
/// New function that wasn't possible with OnceLock
pub fn clear_github_client() {
    AppServices::init().clear_github_client();
}

/// Shutdown all services gracefully
/// Call this when the application is about to quit
pub fn shutdown_services() {
    AppServices::init().shutdown();
}

/// C FFI: Shutdown all services gracefully
/// Hook this to QCoreApplication::aboutToQuit signal
#[no_mangle]
pub extern "C" fn shutdown_app_services() {
    shutdown_services();
}

/// Initialize weather service channel. Call once when WeatherModel is first created.
/// Returns true if initialized (or already initialized).
pub fn init_weather_service_channel() -> bool {
    AppServices::init().init_weather_service_channel()
}

/// Get weather service sender for request_* calls. None if init_weather_service_channel not called yet.
pub fn get_weather_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::WeatherServiceMessage>> {
    AppServices::init().weather_service_tx()
}

/// Non-blocking recv from weather service channel. Called by WeatherModel::poll_channel.
pub fn try_recv_weather_message() -> Option<crate::services::WeatherServiceMessage> {
    AppServices::init().try_recv_weather_message()
}

/// Initialize auth service channel. Call once when AuthModel is first created.
/// Returns true if initialized (or already initialized).
pub fn init_auth_service_channel() -> bool {
    AppServices::init().init_auth_service_channel()
}

/// Get auth service sender for request_* calls. None if init_auth_service_channel not called yet.
pub fn get_auth_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::AuthServiceMessage>> {
    AppServices::init().auth_service_tx()
}

/// Non-blocking recv from auth service channel. Called by AuthModel::poll_channel.
pub fn try_recv_auth_message() -> Option<crate::services::AuthServiceMessage> {
    AppServices::init().try_recv_auth_message()
}

/// Initialize project service channel. Call once when ProjectModel is first created.
/// Returns true if initialized (or already initialized).
pub fn init_project_service_channel() -> bool {
    AppServices::init().init_project_service_channel()
}

/// Get project service sender for request_* calls. None if init_project_service_channel not called yet.
pub fn get_project_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::ProjectServiceMessage>> {
    AppServices::init().project_service_tx()
}

/// Non-blocking recv from project service channel. Called by ProjectModel::poll_channel.
pub fn try_recv_project_message() -> Option<crate::services::ProjectServiceMessage> {
    AppServices::init().try_recv_project_message()
}

/// Initialize kanban service channel. Call once when KanbanModel is first created.
/// Returns true if initialized (or already initialized).
pub fn init_kanban_service_channel() -> bool {
    AppServices::init().init_kanban_service_channel()
}

/// Get kanban service sender for request_* calls. None if init_kanban_service_channel not called yet.
pub fn get_kanban_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::KanbanServiceMessage>> {
    AppServices::init().kanban_service_tx()
}

/// Non-blocking recv from kanban service channel. Called by KanbanModel::poll_channel.
pub fn try_recv_kanban_message() -> Option<crate::services::KanbanServiceMessage> {
    AppServices::init().try_recv_kanban_message()
}

/// Create a new cancellation token for repo operations.
pub fn new_repo_cancel_token() -> std::sync::Arc<tokio_util::sync::CancellationToken> {
    AppServices::init().new_repo_cancel_token()
}

/// Cancel any active repo operation.
pub fn cancel_repo_operation() {
    AppServices::init().cancel_repo_operation()
}

/// Clear the repo cancellation token after operation completes.
pub fn clear_repo_cancel_token() {
    AppServices::init().clear_repo_cancel_token()
}
