//! C FFI bridge for Qt/QML initialization.
//!
//! This module provides the C-callable functions that Qt/QML uses to initialize
//! the Rust services. Internally, it delegates to `AppServices` for state management.

use std::sync::Arc;

use myme_auth::GitHubAuth;
use myme_services::{GitHubClient, NoteClient, ProjectStore};
use myme_weather::{WeatherCache, WeatherProvider};

use crate::app_services::{self, AppServices};

/// Generate bridge functions for service channels. List must match app_services.
macro_rules! service_channel_bridge {
    ($($svc:ident : $msg:ty),* $(,)?) => {
        $(
            paste::paste! {
                /// Initialize service channel. Call once when model is first created. Returns true if initialized (or already initialized).
                pub fn [<init_ $svc _service_channel>]() -> bool {
                    AppServices::init().[<init_ $svc _service_channel>]()
                }
                /// Get service sender for request_* calls.
                pub fn [<get_ $svc _service_tx>]() -> Option<std::sync::mpsc::Sender<$msg>> {
                    AppServices::init().[<$svc _service_tx>]()
                }
                /// Non-blocking recv from service channel. Called by model poll_channel.
                pub fn [<try_recv_ $svc _message>]() -> Option<$msg> {
                    AppServices::init().[<try_recv_ $svc _message>]()
                }
            }
        )*
    };
}

// =========== C FFI Initialization Functions ===========

/// Initialize note client from configuration (SQLite).
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
pub fn get_weather_services() -> Option<(Arc<WeatherProvider>, WeatherCache, tokio::runtime::Handle)>
{
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

/// Get project store if initialized.
pub fn get_project_store() -> Option<Arc<parking_lot::Mutex<ProjectStore>>> {
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

/// Get project store, initializing if needed.
pub fn get_project_store_or_init() -> Option<Arc<parking_lot::Mutex<ProjectStore>>> {
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

// Service channel bridge (list must match app_services)
service_channel_bridge!(
    repo: crate::services::RepoServiceMessage,
    note: crate::services::NoteServiceMessage,
    weather: crate::services::WeatherServiceMessage,
    auth: crate::services::AuthServiceMessage,
    project: crate::services::ProjectServiceMessage,
    workflow: crate::services::WorkflowServiceMessage,
    kanban: crate::services::KanbanServiceMessage,
    gmail: crate::services::GmailServiceMessage,
    calendar: crate::services::CalendarServiceMessage,
);

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
