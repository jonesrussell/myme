//! Centralized application services with mutable state support.
//!
//! This module provides a single `AppServices` struct that holds all shared
//! services (runtime, clients, stores) with proper mutability support via RwLock.
//!
//! Unlike OnceLock, this design allows:
//! - Reinitializing clients after authentication changes
//! - Clearing clients on sign-out
//! - Graceful shutdown coordination

use std::sync::{Arc, OnceLock};

use parking_lot::RwLock;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use myme_auth::GitHubAuth;
use myme_services::{GitHubClient, NoteClient, ProjectStore, SqliteNoteStore};
use myme_weather::{WeatherCache, WeatherProvider};

/// Message types for the repo service channel
pub use crate::services::RepoServiceMessage;

/// Message types for the note service channel
pub use crate::services::NoteServiceMessage;

/// Message types for the weather service channel
pub use crate::services::WeatherServiceMessage;

/// Message types for the auth service channel
pub use crate::services::AuthServiceMessage;

/// Message types for the project service channel
pub use crate::services::ProjectServiceMessage;

/// Message types for the workflow service channel
pub use crate::services::WorkflowServiceMessage;

/// Message types for the kanban service channel
pub use crate::services::KanbanServiceMessage;

/// Message types for the Gmail service channel
pub use crate::services::GmailServiceMessage;

/// Message types for the Calendar service channel
pub use crate::services::CalendarServiceMessage;

/// Generate shutdown clear lines for service channels. Pass `self` so the macro can refer to the receiver.
macro_rules! service_channel_shutdown {
    ($self_expr:expr; $($svc:ident : $msg:ty),* $(,)?) => {
        $(
            paste::paste! {
                *$self_expr.[<$svc _service_tx>].write() = None;
                *$self_expr.[<$svc _service_rx>].write() = None;
            }
        )*
    };
}

/// Generate getter, init_channel, and try_recv methods for one service channel.
macro_rules! service_channel_methods {
    ($($svc:ident : $msg:ty),* $(,)?) => {
        $(
            paste::paste! {
                /// Get service sender.
                pub fn [<$svc _service_tx>](&self) -> Option<std::sync::mpsc::Sender<$msg>> {
                    self.[<$svc _service_tx>].read().clone()
                }

                /// Initialize service channel.
                pub fn [<init_ $svc _service_channel>](&self) -> bool {
                    if self.[<$svc _service_tx>].read().is_some() {
                        return true;
                    }
                    let (tx, rx) = std::sync::mpsc::channel();
                    *self.[<$svc _service_tx>].write() = Some(tx);
                    *self.[<$svc _service_rx>].write() = Some(parking_lot::Mutex::new(rx));
                    tracing::info!("{} service channel initialized", stringify!($svc));
                    true
                }

                /// Try to receive a message from the service channel (non-blocking).
                pub fn [<try_recv_ $svc _message>](&self) -> Option<$msg> {
                    let guard = self.[<$svc _service_rx>].read();
                    let rx_mutex = guard.as_ref()?;
                    let result = { rx_mutex.lock().try_recv().ok() };
                    result
                }
            }
        )*
    };
}

/// Global application services container.
///
/// This is initialized once at application startup and provides mutable
/// access to all shared services through RwLock.
pub struct AppServices {
    /// Tokio runtime for async operations
    runtime: tokio::runtime::Runtime,

    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<()>,

    /// Note client (SQLite backend)
    note_client: RwLock<Option<Arc<NoteClient>>>,

    /// GitHub API client (requires authentication)
    github_client: RwLock<Option<Arc<GitHubClient>>>,

    /// GitHub OAuth provider
    github_auth: RwLock<Option<Arc<GitHubAuth>>>,

    /// Project store (SQLite database)
    project_store: RwLock<Option<Arc<parking_lot::Mutex<ProjectStore>>>>,

    /// Weather provider
    weather_provider: RwLock<Option<Arc<WeatherProvider>>>,

    /// Weather cache
    weather_cache: RwLock<Option<parking_lot::Mutex<WeatherCache>>>,

    /// Repo service channel sender
    repo_service_tx: RwLock<Option<std::sync::mpsc::Sender<RepoServiceMessage>>>,
    /// Repo service channel receiver
    repo_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<RepoServiceMessage>>>>,
    /// Note service channel sender
    note_service_tx: RwLock<Option<std::sync::mpsc::Sender<NoteServiceMessage>>>,
    /// Note service channel receiver
    note_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<NoteServiceMessage>>>>,
    /// Weather service channel sender
    weather_service_tx: RwLock<Option<std::sync::mpsc::Sender<WeatherServiceMessage>>>,
    /// Weather service channel receiver
    weather_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<WeatherServiceMessage>>>>,
    /// Auth service channel sender
    auth_service_tx: RwLock<Option<std::sync::mpsc::Sender<AuthServiceMessage>>>,
    /// Auth service channel receiver
    auth_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<AuthServiceMessage>>>>,
    /// Project service channel sender
    project_service_tx: RwLock<Option<std::sync::mpsc::Sender<ProjectServiceMessage>>>,
    /// Project service channel receiver
    project_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<ProjectServiceMessage>>>>,
    /// Workflow service channel sender
    workflow_service_tx: RwLock<Option<std::sync::mpsc::Sender<WorkflowServiceMessage>>>,
    /// Workflow service channel receiver
    workflow_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<WorkflowServiceMessage>>>>,
    /// Kanban service channel sender
    kanban_service_tx: RwLock<Option<std::sync::mpsc::Sender<KanbanServiceMessage>>>,
    /// Kanban service channel receiver
    kanban_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<KanbanServiceMessage>>>>,
    /// Gmail service channel sender
    gmail_service_tx: RwLock<Option<std::sync::mpsc::Sender<GmailServiceMessage>>>,
    /// Gmail service channel receiver
    gmail_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<GmailServiceMessage>>>>,
    /// Calendar service channel sender
    calendar_service_tx: RwLock<Option<std::sync::mpsc::Sender<CalendarServiceMessage>>>,
    /// Calendar service channel receiver
    calendar_service_rx:
        RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<CalendarServiceMessage>>>>,

    /// Cancellation token for repo operations (clone, pull)
    repo_cancel_token: RwLock<Option<Arc<CancellationToken>>>,
}

/// Global singleton for application services
static SERVICES: OnceLock<Arc<AppServices>> = OnceLock::new();

impl AppServices {
    /// Initialize the application services.
    ///
    /// This should be called once at application startup. Subsequent calls
    /// return the existing instance.
    pub fn init() -> Arc<Self> {
        SERVICES
            .get_or_init(|| {
                // Runtime creation failure is fatal; no recovery.
                #[allow(clippy::expect_used)]
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .thread_name("myme-tokio")
                    .build()
                    .expect("Failed to create tokio runtime");

                let (shutdown_tx, _) = broadcast::channel(16);

                Arc::new(Self {
                    runtime,
                    shutdown_tx,
                    note_client: RwLock::new(None),
                    github_client: RwLock::new(None),
                    github_auth: RwLock::new(None),
                    project_store: RwLock::new(None),
                    weather_provider: RwLock::new(None),
                    weather_cache: RwLock::new(None),
                    repo_service_tx: RwLock::new(None),
                    repo_service_rx: RwLock::new(None),
                    note_service_tx: RwLock::new(None),
                    note_service_rx: RwLock::new(None),
                    weather_service_tx: RwLock::new(None),
                    weather_service_rx: RwLock::new(None),
                    auth_service_tx: RwLock::new(None),
                    auth_service_rx: RwLock::new(None),
                    project_service_tx: RwLock::new(None),
                    project_service_rx: RwLock::new(None),
                    workflow_service_tx: RwLock::new(None),
                    workflow_service_rx: RwLock::new(None),
                    kanban_service_tx: RwLock::new(None),
                    kanban_service_rx: RwLock::new(None),
                    gmail_service_tx: RwLock::new(None),
                    gmail_service_rx: RwLock::new(None),
                    calendar_service_tx: RwLock::new(None),
                    calendar_service_rx: RwLock::new(None),
                    repo_cancel_token: RwLock::new(None),
                })
            })
            .clone()
    }

    /// Get the tokio runtime handle.
    pub fn runtime(&self) -> tokio::runtime::Handle {
        self.runtime.handle().clone()
    }

    /// Subscribe to shutdown notifications.
    pub fn subscribe_shutdown(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Signal application shutdown.
    ///
    /// This broadcasts a shutdown signal to all subscribers and clears
    /// all service references.
    pub fn shutdown(&self) {
        tracing::info!("AppServices shutdown initiated");

        // Broadcast shutdown signal
        let _ = self.shutdown_tx.send(());

        // Clear all mutable state
        *self.note_client.write() = None;
        *self.github_client.write() = None;
        *self.github_auth.write() = None;
        *self.project_store.write() = None;
        *self.weather_provider.write() = None;
        *self.weather_cache.write() = None;
        service_channel_shutdown!(
            self;
            repo: RepoServiceMessage,
            note: NoteServiceMessage,
            weather: WeatherServiceMessage,
            auth: AuthServiceMessage,
            project: ProjectServiceMessage,
            workflow: WorkflowServiceMessage,
            kanban: KanbanServiceMessage,
            gmail: GmailServiceMessage,
            calendar: CalendarServiceMessage,
        );

        // Cancel any active repo operations
        if let Some(token) = self.repo_cancel_token.write().take() {
            token.cancel();
        }

        tracing::info!("AppServices shutdown complete");
    }

    // =========== Note Client ===========

    /// Get the unified note client if initialized.
    pub fn note_client(&self) -> Option<Arc<NoteClient>> {
        self.note_client.read().clone()
    }

    /// Set or update the unified note client.
    pub fn set_note_client(&self, client: Option<Arc<NoteClient>>) {
        *self.note_client.write() = client;
    }

    /// Initialize note client from configuration (SQLite only).
    ///
    /// Returns `true` if the client was initialized or was already initialized.
    /// Returns `false` only on creation failure.
    pub fn init_note_client(&self) -> bool {
        if self.note_client.read().is_some() {
            return true;
        }

        let config = myme_core::Config::load_cached();
        let db_path = config.notes.sqlite_path();

        if let Some(parent) = db_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::error!("Failed to create notes database directory: {}", e);
                return false;
            }
        }

        let store = match SqliteNoteStore::new(&db_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to create SQLite note store at {:?}: {}", db_path, e);
                return false;
            }
        };

        tracing::info!("SQLite note store opened at {:?}", db_path);
        self.set_note_client(Some(Arc::new(NoteClient::sqlite(store))));
        true
    }

    // =========== GitHub Client ===========

    /// Get the GitHub client if initialized.
    pub fn github_client(&self) -> Option<Arc<GitHubClient>> {
        self.github_client.read().clone()
    }

    /// Set or update the GitHub client.
    pub fn set_github_client(&self, client: Option<Arc<GitHubClient>>) {
        *self.github_client.write() = client;
    }

    /// Check if GitHub client is available (authenticated).
    pub fn is_github_authenticated(&self) -> bool {
        self.github_client.read().is_some()
    }

    /// Initialize GitHub client from secure storage token.
    ///
    /// Returns true if client was successfully initialized.
    pub fn init_github_client(&self) -> bool {
        // Get token from secure storage
        let token = match myme_auth::SecureStorage::retrieve_token("github") {
            Ok(token_set) => {
                if token_set.is_expired() {
                    tracing::warn!(
                        "GitHub token is expired (expires_at: {})",
                        token_set.expires_at
                    );
                    return false;
                }
                tracing::info!("Retrieved valid GitHub token from secure storage");
                token_set.access_token
            }
            Err(e) => {
                tracing::warn!("Failed to retrieve GitHub token: {}", e);
                return false;
            }
        };

        // Create GitHub client
        match GitHubClient::new(token) {
            Ok(client) => {
                self.set_github_client(Some(Arc::new(client)));
                tracing::info!("GitHub client initialized");
                true
            }
            Err(e) => {
                tracing::error!("Failed to create GitHub client: {}", e);
                false
            }
        }
    }

    /// Clear GitHub client (e.g., on sign-out).
    pub fn clear_github_client(&self) {
        self.set_github_client(None);
        tracing::info!("GitHub client cleared");
    }

    // =========== GitHub Auth Provider ===========

    /// Get the GitHub auth provider if initialized.
    pub fn github_auth(&self) -> Option<Arc<GitHubAuth>> {
        self.github_auth.read().clone()
    }

    /// Set or update the GitHub auth provider.
    pub fn set_github_auth(&self, auth: Option<Arc<GitHubAuth>>) {
        *self.github_auth.write() = auth;
    }

    /// Initialize GitHub auth provider from configuration.
    pub fn init_github_auth(&self) -> bool {
        let config = myme_core::Config::load_cached();

        if !config.github.is_configured() {
            tracing::info!("GitHub OAuth not configured");
            return false;
        }

        let provider = Arc::new(GitHubAuth::new(
            config.github.client_id.clone(),
            config.github.client_secret.clone(),
        ));

        self.set_github_auth(Some(provider));
        tracing::info!("GitHub OAuth provider initialized");
        true
    }

    // =========== Project Store ===========

    /// Get the project store if initialized.
    pub fn project_store(&self) -> Option<Arc<parking_lot::Mutex<ProjectStore>>> {
        self.project_store.read().clone()
    }

    /// Set or update the project store.
    pub fn set_project_store(&self, store: Option<Arc<parking_lot::Mutex<ProjectStore>>>) {
        *self.project_store.write() = store;
    }

    /// Initialize project store, creating database if needed.
    pub fn init_project_store(&self) -> bool {
        // Return true if already initialized
        if self.project_store.read().is_some() {
            return true;
        }

        let config_dir = myme_core::Config::load_cached().config_dir.clone();

        let db_path = config_dir.join("projects.db");

        // Ensure directory exists
        if let Err(e) = std::fs::create_dir_all(&config_dir) {
            tracing::error!("Failed to create config directory: {}", e);
            return false;
        }

        match ProjectStore::open(&db_path) {
            Ok(store) => {
                self.set_project_store(Some(Arc::new(parking_lot::Mutex::new(store))));
                tracing::info!("Project store initialized at {:?}", db_path);
                true
            }
            Err(e) => {
                tracing::error!("Failed to open project store: {}", e);
                false
            }
        }
    }

    // =========== Weather Services ===========

    /// Get the weather provider if initialized.
    pub fn weather_provider(&self) -> Option<Arc<WeatherProvider>> {
        self.weather_provider.read().clone()
    }

    /// Set or update the weather provider.
    pub fn set_weather_provider(&self, provider: Option<Arc<WeatherProvider>>) {
        *self.weather_provider.write() = provider;
    }

    /// Get weather cache (from stored instance, or create and store on first use).
    pub fn weather_cache(&self) -> Option<WeatherCache> {
        {
            let guard = self.weather_cache.read();
            if let Some(mutex) = guard.as_ref() {
                return Some(mutex.lock().clone());
            }
        }
        // Lazy init: create, load, store, then return clone
        let config_dir = myme_core::Config::load_cached().config_dir.clone();
        let mut cache = WeatherCache::new(&config_dir);
        let _ = cache.load();
        *self.weather_cache.write() = Some(parking_lot::Mutex::new(cache.clone()));
        Some(cache)
    }

    /// Initialize weather services.
    pub fn init_weather_services(&self) -> bool {
        let config = myme_core::Config::load_cached();
        let temp_unit = config.weather.temperature_unit;

        // Convert to myme_weather::TemperatureUnit
        let weather_unit = match temp_unit {
            myme_core::TemperatureUnit::Celsius => myme_weather::TemperatureUnit::Celsius,
            myme_core::TemperatureUnit::Fahrenheit => myme_weather::TemperatureUnit::Fahrenheit,
            myme_core::TemperatureUnit::Auto => myme_weather::TemperatureUnit::Auto,
        };

        // Create and store weather cache
        let config_dir = config.config_dir.clone();
        let mut cache = WeatherCache::new(&config_dir);
        let _ = cache.load();
        *self.weather_cache.write() = Some(parking_lot::Mutex::new(cache));

        // Create weather provider
        match WeatherProvider::new(weather_unit) {
            Ok(provider) => {
                self.set_weather_provider(Some(Arc::new(provider)));
                tracing::info!("Weather provider initialized");
                true
            }
            Err(e) => {
                tracing::error!("Failed to create weather provider: {}", e);
                false
            }
        }
    }

    // Service channel methods (repo, note, weather, auth, project, workflow, kanban, gmail, calendar)
    service_channel_methods!(
        repo: RepoServiceMessage,
        note: NoteServiceMessage,
        weather: WeatherServiceMessage,
        auth: AuthServiceMessage,
        project: ProjectServiceMessage,
        workflow: WorkflowServiceMessage,
        kanban: KanbanServiceMessage,
        gmail: GmailServiceMessage,
        calendar: CalendarServiceMessage,
    );

    // =========== Repo Operation Cancellation ===========

    /// Create a new cancellation token for a repo operation.
    ///
    /// This replaces any existing token (cancelling its listeners).
    pub fn new_repo_cancel_token(&self) -> Arc<CancellationToken> {
        let token = Arc::new(CancellationToken::new());
        *self.repo_cancel_token.write() = Some(token.clone());
        token
    }

    /// Get the current repo cancellation token if one exists.
    pub fn repo_cancel_token(&self) -> Option<Arc<CancellationToken>> {
        self.repo_cancel_token.read().clone()
    }

    /// Cancel any active repo operation and clear the token.
    pub fn cancel_repo_operation(&self) {
        if let Some(token) = self.repo_cancel_token.write().take() {
            token.cancel();
            tracing::info!("Repo operation cancelled");
        }
    }

    /// Clear the repo cancellation token (call after operation completes).
    pub fn clear_repo_cancel_token(&self) {
        *self.repo_cancel_token.write() = None;
    }
}

// =========== Convenience Functions ===========
// These provide a simpler API for common operations

/// Get the global services instance.
pub fn services() -> Arc<AppServices> {
    AppServices::init()
}

/// Get the tokio runtime handle.
pub fn runtime() -> tokio::runtime::Handle {
    services().runtime()
}

/// Get note client and runtime handle.
pub fn note_client_and_runtime() -> Option<(Arc<NoteClient>, tokio::runtime::Handle)> {
    let svc = services();
    Some((svc.note_client()?, svc.runtime()))
}

/// Get unified note client, initializing if needed.
pub fn note_client_or_init() -> Option<Arc<NoteClient>> {
    let svc = services();
    svc.init_note_client();
    svc.note_client()
}

/// Get GitHub client and runtime handle.
pub fn github_client_and_runtime() -> Option<(Arc<GitHubClient>, tokio::runtime::Handle)> {
    let svc = services();
    Some((svc.github_client()?, svc.runtime()))
}

/// Get GitHub auth provider and runtime handle.
pub fn github_auth_and_runtime() -> Option<(Arc<GitHubAuth>, tokio::runtime::Handle)> {
    let svc = services();
    Some((svc.github_auth()?, svc.runtime()))
}

/// Get project store.
pub fn project_store() -> Option<Arc<parking_lot::Mutex<ProjectStore>>> {
    services().project_store()
}

/// Get project store, initializing if needed.
pub fn project_store_or_init() -> Option<Arc<parking_lot::Mutex<ProjectStore>>> {
    let svc = services();
    svc.init_project_store();
    svc.project_store()
}

/// Get weather services.
pub fn weather_services() -> Option<(Arc<WeatherProvider>, WeatherCache, tokio::runtime::Handle)> {
    let svc = services();
    Some((svc.weather_provider()?, svc.weather_cache()?, svc.runtime()))
}

/// Check if GitHub is authenticated.
pub fn is_github_authenticated() -> bool {
    services().is_github_authenticated()
}

/// Get repos local search path from config.
pub fn get_repos_local_search_path() -> Option<(std::path::PathBuf, bool)> {
    let config = myme_core::Config::load_cached();
    Some(config.repos.effective_local_search_path())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_services_init() {
        let svc1 = AppServices::init();
        let svc2 = AppServices::init();
        // Should be the same instance
        assert!(Arc::ptr_eq(&svc1, &svc2));
    }

    #[test]
    fn test_github_client_lifecycle() {
        let svc = AppServices::init();

        // Initially not authenticated
        assert!(!svc.is_github_authenticated());

        // Would need a mock GitHubClient to test setting
        // svc.set_github_client(Some(Arc::new(mock_client)));
        // assert!(svc.is_github_authenticated());

        // Clear should work
        svc.clear_github_client();
        assert!(!svc.is_github_authenticated());
    }
}
