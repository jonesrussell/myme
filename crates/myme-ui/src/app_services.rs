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
use myme_services::{GitHubClient, NoteClient, ProjectStore, SqliteNoteStore, TodoClient};
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

/// Global application services container.
///
/// This is initialized once at application startup and provides mutable
/// access to all shared services through RwLock.
pub struct AppServices {
    /// Tokio runtime for async operations
    runtime: tokio::runtime::Runtime,

    /// Shutdown signal broadcaster
    shutdown_tx: broadcast::Sender<()>,

    /// Todo/Notes API client (for Godo integration) - LEGACY
    /// Use `note_client` instead for new code.
    todo_client: RwLock<Option<Arc<TodoClient>>>,

    /// Unified note client (SQLite or HTTP backend)
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
    repo_service_rx: RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<RepoServiceMessage>>>>,

    /// Note service channel sender
    note_service_tx: RwLock<Option<std::sync::mpsc::Sender<NoteServiceMessage>>>,

    /// Note service channel receiver
    note_service_rx: RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<NoteServiceMessage>>>>,

    /// Weather service channel sender
    weather_service_tx: RwLock<Option<std::sync::mpsc::Sender<WeatherServiceMessage>>>,

    /// Weather service channel receiver
    weather_service_rx: RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<WeatherServiceMessage>>>>,

    /// Auth service channel sender
    auth_service_tx: RwLock<Option<std::sync::mpsc::Sender<AuthServiceMessage>>>,

    /// Auth service channel receiver
    auth_service_rx: RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<AuthServiceMessage>>>>,

    /// Project service channel sender
    project_service_tx: RwLock<Option<std::sync::mpsc::Sender<ProjectServiceMessage>>>,

    /// Project service channel receiver
    project_service_rx: RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<ProjectServiceMessage>>>>,

    /// Workflow service channel sender
    workflow_service_tx: RwLock<Option<std::sync::mpsc::Sender<WorkflowServiceMessage>>>,

    /// Workflow service channel receiver
    workflow_service_rx: RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<WorkflowServiceMessage>>>>,

    /// Kanban service channel sender
    kanban_service_tx: RwLock<Option<std::sync::mpsc::Sender<KanbanServiceMessage>>>,

    /// Kanban service channel receiver
    kanban_service_rx: RwLock<Option<parking_lot::Mutex<std::sync::mpsc::Receiver<KanbanServiceMessage>>>>,

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
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .thread_name("myme-tokio")
                    .build()
                    .expect("Failed to create tokio runtime");

                let (shutdown_tx, _) = broadcast::channel(16);

                Arc::new(Self {
                    runtime,
                    shutdown_tx,
                    todo_client: RwLock::new(None),
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
        *self.todo_client.write() = None;
        *self.note_client.write() = None;
        *self.github_client.write() = None;
        *self.github_auth.write() = None;
        *self.project_store.write() = None;
        *self.weather_provider.write() = None;
        *self.weather_cache.write() = None;
        *self.repo_service_tx.write() = None;
        *self.repo_service_rx.write() = None;
        *self.note_service_tx.write() = None;
        *self.note_service_rx.write() = None;
        *self.weather_service_tx.write() = None;
        *self.weather_service_rx.write() = None;
        *self.auth_service_tx.write() = None;
        *self.auth_service_rx.write() = None;
        *self.project_service_tx.write() = None;
        *self.project_service_rx.write() = None;
        *self.workflow_service_tx.write() = None;
        *self.workflow_service_rx.write() = None;
        *self.kanban_service_tx.write() = None;
        *self.kanban_service_rx.write() = None;

        // Cancel any active repo operations
        if let Some(token) = self.repo_cancel_token.write().take() {
            token.cancel();
        }

        tracing::info!("AppServices shutdown complete");
    }

    // =========== Todo Client ===========

    /// Get the Todo client if initialized.
    pub fn todo_client(&self) -> Option<Arc<TodoClient>> {
        self.todo_client.read().clone()
    }

    /// Set or update the Todo client.
    pub fn set_todo_client(&self, client: Option<Arc<TodoClient>>) {
        *self.todo_client.write() = client;
    }

    /// Initialize Todo client from configuration.
    ///
    /// The `allow_invalid_certs` parameter only takes effect in debug builds.
    pub fn init_todo_client(
        &self,
        base_url: &str,
        jwt_token: Option<String>,
        allow_invalid_certs: bool,
    ) -> bool {
        let config = myme_services::TodoClientConfig {
            base_url: base_url.to_string(),
            jwt_token,
            allow_invalid_certs,
        };

        match TodoClient::new_with_config(config) {
            Ok(client) => {
                self.set_todo_client(Some(Arc::new(client)));
                tracing::info!("Todo client initialized with base_url: {}", base_url);
                true
            }
            Err(e) => {
                tracing::error!("Failed to create Todo client: {}", e);
                false
            }
        }
    }

    // =========== Note Client (unified) ===========

    /// Get the unified note client if initialized.
    pub fn note_client(&self) -> Option<Arc<NoteClient>> {
        self.note_client.read().clone()
    }

    /// Set or update the unified note client.
    pub fn set_note_client(&self, client: Option<Arc<NoteClient>>) {
        *self.note_client.write() = client;
    }

    /// Initialize unified note client from configuration.
    ///
    /// Reads the notes config and creates either a SQLite or HTTP backend.
    /// Also runs migration from Godo if configured.
    pub fn init_note_client(&self) -> bool {
        // Return true if already initialized
        if self.note_client.read().is_some() {
            return true;
        }

        let config = match myme_core::Config::load() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to load config for notes: {}. Using defaults.", e);
                myme_core::Config::default()
            }
        };

        let client = match config.notes.backend {
            myme_core::NotesBackend::Sqlite => {
                self.init_sqlite_note_client(&config)
            }
            myme_core::NotesBackend::Api => {
                self.init_api_note_client(&config)
            }
        };

        match client {
            Some(c) => {
                self.set_note_client(Some(Arc::new(c)));
                tracing::info!("Note client initialized ({:?} backend)", config.notes.backend);
                true
            }
            None => {
                tracing::error!("Failed to initialize note client");
                false
            }
        }
    }

    /// Initialize SQLite-backed note client.
    fn init_sqlite_note_client(&self, config: &myme_core::Config) -> Option<NoteClient> {
        let db_path = config.notes.sqlite_path();

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                tracing::error!("Failed to create notes database directory: {}", e);
                return None;
            }
        }

        // Create store
        let store = match SqliteNoteStore::new(&db_path) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to create SQLite note store at {:?}: {}", db_path, e);
                return None;
            }
        };

        tracing::info!("SQLite note store opened at {:?}", db_path);

        // Run migration from Godo if configured
        if config.notes.migrate_from_godo {
            let godo_path = config.notes.godo_path();
            if godo_path.exists() {
                tracing::info!("Running migration from Godo database: {:?}", godo_path);
                match myme_services::migrate_from_godo(&godo_path, &store) {
                    Ok(result) => {
                        tracing::info!("{}", result);
                    }
                    Err(e) => {
                        tracing::warn!("Migration from Godo failed: {}", e);
                        // Continue anyway - store is still usable
                    }
                }
            } else {
                tracing::debug!("Godo database not found at {:?}, skipping migration", godo_path);
            }
        }

        Some(NoteClient::sqlite(store))
    }

    /// Initialize HTTP API-backed note client (legacy Godo).
    fn init_api_note_client(&self, config: &myme_core::Config) -> Option<NoteClient> {
        let todo_config = myme_services::TodoClientConfig {
            base_url: config.services.todo_api_url.clone(),
            jwt_token: config.services.jwt_token.clone(),
            allow_invalid_certs: config.services.allow_invalid_certs,
        };

        match TodoClient::new_with_config(todo_config) {
            Ok(client) => {
                tracing::info!(
                    "HTTP note client initialized with base_url: {}",
                    config.services.todo_api_url
                );
                Some(NoteClient::http(client))
            }
            Err(e) => {
                tracing::error!("Failed to create HTTP note client: {}", e);
                None
            }
        }
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
        let config = match myme_core::Config::load() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to load config for GitHub auth: {}", e);
                return false;
            }
        };

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

        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("myme");

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

    /// Get weather cache, cloning the current state.
    pub fn weather_cache(&self) -> Option<WeatherCache> {
        let config_dir = myme_core::Config::load()
            .map(|c| c.config_dir)
            .unwrap_or_else(|_| {
                dirs::config_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("myme")
            });

        let mut cache = WeatherCache::new(&config_dir);
        let _ = cache.load();
        Some(cache)
    }

    /// Initialize weather services.
    pub fn init_weather_services(&self) -> bool {
        // Load config for temperature unit preference
        let temp_unit = match myme_core::Config::load() {
            Ok(config) => config.weather.temperature_unit,
            Err(e) => {
                tracing::warn!("Failed to load config for weather: {}. Using auto.", e);
                myme_core::TemperatureUnit::Auto
            }
        };

        // Convert to myme_weather::TemperatureUnit
        let weather_unit = match temp_unit {
            myme_core::TemperatureUnit::Celsius => myme_weather::TemperatureUnit::Celsius,
            myme_core::TemperatureUnit::Fahrenheit => myme_weather::TemperatureUnit::Fahrenheit,
            myme_core::TemperatureUnit::Auto => myme_weather::TemperatureUnit::Auto,
        };

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

    // =========== Repo Service Channel ===========

    /// Get repo service sender.
    pub fn repo_service_tx(&self) -> Option<std::sync::mpsc::Sender<RepoServiceMessage>> {
        self.repo_service_tx.read().clone()
    }

    /// Initialize repo service channel.
    pub fn init_repo_service_channel(&self) -> bool {
        if self.repo_service_tx.read().is_some() {
            return true;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        *self.repo_service_tx.write() = Some(tx);
        *self.repo_service_rx.write() = Some(parking_lot::Mutex::new(rx));
        tracing::info!("Repo service channel initialized");
        true
    }

    /// Try to receive a message from the repo service channel (non-blocking).
    pub fn try_recv_repo_message(&self) -> Option<RepoServiceMessage> {
        let guard = self.repo_service_rx.read();
        let rx_mutex = guard.as_ref()?;
        let result = rx_mutex.lock().try_recv().ok();
        result
    }

    // =========== Note Service Channel ===========

    /// Get note service sender.
    pub fn note_service_tx(&self) -> Option<std::sync::mpsc::Sender<NoteServiceMessage>> {
        self.note_service_tx.read().clone()
    }

    /// Initialize note service channel.
    pub fn init_note_service_channel(&self) -> bool {
        if self.note_service_tx.read().is_some() {
            return true;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        *self.note_service_tx.write() = Some(tx);
        *self.note_service_rx.write() = Some(parking_lot::Mutex::new(rx));
        tracing::info!("Note service channel initialized");
        true
    }

    /// Try to receive a message from the note service channel (non-blocking).
    pub fn try_recv_note_message(&self) -> Option<NoteServiceMessage> {
        let guard = self.note_service_rx.read();
        let rx_mutex = guard.as_ref()?;
        let result = rx_mutex.lock().try_recv().ok();
        result
    }

    // =========== Weather Service Channel ===========

    /// Get weather service sender.
    pub fn weather_service_tx(&self) -> Option<std::sync::mpsc::Sender<WeatherServiceMessage>> {
        self.weather_service_tx.read().clone()
    }

    /// Initialize weather service channel.
    pub fn init_weather_service_channel(&self) -> bool {
        if self.weather_service_tx.read().is_some() {
            return true;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        *self.weather_service_tx.write() = Some(tx);
        *self.weather_service_rx.write() = Some(parking_lot::Mutex::new(rx));
        tracing::info!("Weather service channel initialized");
        true
    }

    /// Try to receive a message from the weather service channel (non-blocking).
    pub fn try_recv_weather_message(&self) -> Option<WeatherServiceMessage> {
        let guard = self.weather_service_rx.read();
        let rx_mutex = guard.as_ref()?;
        let result = rx_mutex.lock().try_recv().ok();
        result
    }

    // =========== Auth Service Channel ===========

    /// Get auth service sender.
    pub fn auth_service_tx(&self) -> Option<std::sync::mpsc::Sender<AuthServiceMessage>> {
        self.auth_service_tx.read().clone()
    }

    /// Initialize auth service channel.
    pub fn init_auth_service_channel(&self) -> bool {
        if self.auth_service_tx.read().is_some() {
            return true;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        *self.auth_service_tx.write() = Some(tx);
        *self.auth_service_rx.write() = Some(parking_lot::Mutex::new(rx));
        tracing::info!("Auth service channel initialized");
        true
    }

    /// Try to receive a message from the auth service channel (non-blocking).
    pub fn try_recv_auth_message(&self) -> Option<AuthServiceMessage> {
        let guard = self.auth_service_rx.read();
        let rx_mutex = guard.as_ref()?;
        let result = rx_mutex.lock().try_recv().ok();
        result
    }

    // =========== Project Service Channel ===========

    /// Get project service sender.
    pub fn project_service_tx(&self) -> Option<std::sync::mpsc::Sender<ProjectServiceMessage>> {
        self.project_service_tx.read().clone()
    }

    /// Initialize project service channel.
    pub fn init_project_service_channel(&self) -> bool {
        if self.project_service_tx.read().is_some() {
            return true;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        *self.project_service_tx.write() = Some(tx);
        *self.project_service_rx.write() = Some(parking_lot::Mutex::new(rx));
        tracing::info!("Project service channel initialized");
        true
    }

    /// Try to receive a message from the project service channel (non-blocking).
    pub fn try_recv_project_message(&self) -> Option<ProjectServiceMessage> {
        let guard = self.project_service_rx.read();
        let rx_mutex = guard.as_ref()?;
        let result = rx_mutex.lock().try_recv().ok();
        result
    }

    // =========== Workflow Service Channel ===========

    /// Get workflow service sender.
    pub fn workflow_service_tx(&self) -> Option<std::sync::mpsc::Sender<WorkflowServiceMessage>> {
        self.workflow_service_tx.read().clone()
    }

    /// Initialize workflow service channel.
    pub fn init_workflow_service_channel(&self) -> bool {
        if self.workflow_service_tx.read().is_some() {
            return true;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        *self.workflow_service_tx.write() = Some(tx);
        *self.workflow_service_rx.write() = Some(parking_lot::Mutex::new(rx));
        tracing::info!("Workflow service channel initialized");
        true
    }

    /// Try to receive a message from the workflow service channel (non-blocking).
    pub fn try_recv_workflow_message(&self) -> Option<WorkflowServiceMessage> {
        let guard = self.workflow_service_rx.read();
        let rx_mutex = guard.as_ref()?;
        let result = rx_mutex.lock().try_recv().ok();
        result
    }

    // =========== Kanban Service Channel ===========

    /// Get kanban service sender.
    pub fn kanban_service_tx(&self) -> Option<std::sync::mpsc::Sender<KanbanServiceMessage>> {
        self.kanban_service_tx.read().clone()
    }

    /// Initialize kanban service channel.
    pub fn init_kanban_service_channel(&self) -> bool {
        if self.kanban_service_tx.read().is_some() {
            return true;
        }

        let (tx, rx) = std::sync::mpsc::channel();
        *self.kanban_service_tx.write() = Some(tx);
        *self.kanban_service_rx.write() = Some(parking_lot::Mutex::new(rx));
        tracing::info!("Kanban service channel initialized");
        true
    }

    /// Try to receive a message from the kanban service channel (non-blocking).
    pub fn try_recv_kanban_message(&self) -> Option<KanbanServiceMessage> {
        let guard = self.kanban_service_rx.read();
        let rx_mutex = guard.as_ref()?;
        let result = rx_mutex.lock().try_recv().ok();
        result
    }

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

/// Get Todo client and runtime handle (legacy).
/// Prefer using `note_client_and_runtime()` for new code.
pub fn todo_client_and_runtime() -> Option<(Arc<TodoClient>, tokio::runtime::Handle)> {
    let svc = services();
    Some((svc.todo_client()?, svc.runtime()))
}

/// Get unified note client and runtime handle.
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
    let config = myme_core::Config::load().ok()?;
    Some(config.repos.effective_local_search_path())
}

#[cfg(test)]
mod tests {
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
