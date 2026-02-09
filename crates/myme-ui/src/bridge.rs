//! C FFI bridge for Qt/QML initialization.
//!
//! This module provides the C-callable functions that Qt/QML uses to initialize
//! the Rust services. Internally, it delegates to `AppServices` for state management.

use std::sync::Arc;

use myme_auth::GitHubAuth;
use myme_services::{GitHubClient, NoteClient, ProjectStore};
use myme_weather::{WeatherCache, WeatherProvider};

use crate::app_services::{self, AppServices};

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

/// Initialize workflow service channel. Call once when WorkflowModel is first created.
pub fn init_workflow_service_channel() -> bool {
    AppServices::init().init_workflow_service_channel()
}

/// Get workflow service sender for request_fetch_workflows. None if channel not initialized yet.
pub fn get_workflow_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::WorkflowServiceMessage>> {
    AppServices::init().workflow_service_tx()
}

/// Non-blocking recv from workflow service channel. Called by WorkflowModel::poll_channel.
pub fn try_recv_workflow_message() -> Option<crate::services::WorkflowServiceMessage> {
    AppServices::init().try_recv_workflow_message()
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

/// Initialize Gmail service channel. Call once when GmailModel is first created.
pub fn init_gmail_service_channel() -> bool {
    AppServices::init().init_gmail_service_channel()
}

/// Get Gmail service sender for request_* calls.
pub fn get_gmail_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::GmailServiceMessage>> {
    AppServices::init().gmail_service_tx()
}

/// Non-blocking recv from Gmail service channel. Called by GmailModel::poll_channel.
pub fn try_recv_gmail_message() -> Option<crate::services::GmailServiceMessage> {
    AppServices::init().try_recv_gmail_message()
}

/// Initialize Calendar service channel. Call once when CalendarModel is first created.
pub fn init_calendar_service_channel() -> bool {
    AppServices::init().init_calendar_service_channel()
}

/// Get Calendar service sender for request_* calls.
pub fn get_calendar_service_tx(
) -> Option<std::sync::mpsc::Sender<crate::services::CalendarServiceMessage>> {
    AppServices::init().calendar_service_tx()
}

/// Non-blocking recv from Calendar service channel. Called by CalendarModel::poll_channel.
pub fn try_recv_calendar_message() -> Option<crate::services::CalendarServiceMessage> {
    AppServices::init().try_recv_calendar_message()
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
