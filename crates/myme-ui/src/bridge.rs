use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Arc, OnceLock};

use myme_auth::GitHubAuth;
use myme_services::{GitHubClient, ProjectStore, TodoClient};
use myme_weather::{WeatherCache, WeatherProvider};

// Static tokio runtime that lives for the duration of the application
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

// Static client and runtime for creating NoteModels
static TODO_CLIENT: OnceLock<Arc<TodoClient>> = OnceLock::new();

// Weather services
static WEATHER_PROVIDER: OnceLock<Arc<WeatherProvider>> = OnceLock::new();
static WEATHER_CACHE: OnceLock<std::sync::Mutex<WeatherCache>> = OnceLock::new();

// GitHub/Projects services
static GITHUB_CLIENT: OnceLock<Arc<GitHubClient>> = OnceLock::new();
static PROJECT_STORE: OnceLock<Arc<std::sync::Mutex<ProjectStore>>> = OnceLock::new();

// GitHub OAuth provider
static GITHUB_AUTH_PROVIDER: OnceLock<Arc<GitHubAuth>> = OnceLock::new();

// Repo service channel
static REPO_SERVICE_TX: OnceLock<std::sync::mpsc::Sender<crate::services::RepoServiceMessage>> =
    OnceLock::new();
static REPO_SERVICE_RX: OnceLock<
    std::sync::Mutex<std::sync::mpsc::Receiver<crate::services::RepoServiceMessage>>,
> = OnceLock::new();

// Note service channel
static NOTE_SERVICE_TX: OnceLock<std::sync::mpsc::Sender<crate::services::NoteServiceMessage>> =
    OnceLock::new();
static NOTE_SERVICE_RX: OnceLock<
    std::sync::Mutex<std::sync::mpsc::Receiver<crate::services::NoteServiceMessage>>,
> = OnceLock::new();

/// Initialize the tokio runtime (call once at application startup)
fn get_or_init_runtime() -> tokio::runtime::Handle {
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("myme-tokio")
                .build()
                .expect("Failed to create tokio runtime")
        })
        .handle()
        .clone()
}

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

    // Get JWT token from config file or environment variable
    let jwt_token = match myme_core::Config::load() {
        Ok(config) => {
            if let Some(token) = config.services.jwt_token {
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
            }
        }
        Err(e) => {
            tracing::warn!(
                "Failed to load config: {}. Checking environment variable.",
                e
            );
            std::env::var("GODO_JWT_TOKEN").ok()
        }
    };

    // Create the TodoClient
    let client = match TodoClient::new(base_url_str, jwt_token) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to create TodoClient: {}", e);
            return false;
        }
    };

    // Get or initialize tokio runtime
    let _runtime = get_or_init_runtime();

    // Store client globally so NoteModels can use it
    if TODO_CLIENT.set(client).is_err() {
        tracing::warn!("TodoClient already initialized");
    }

    tracing::info!("NoteModel services initialized successfully");
    true
}

/// Get the initialized TodoClient and runtime for use by NoteModels
pub fn get_todo_client_and_runtime() -> Option<(Arc<TodoClient>, tokio::runtime::Handle)> {
    let client = TODO_CLIENT.get()?.clone();
    let runtime = RUNTIME.get()?.handle().clone();
    Some((client, runtime))
}

/// Initialize weather services
/// Must be called before QML tries to access WeatherModel
#[no_mangle]
pub extern "C" fn initialize_weather_services() -> bool {
    // Ensure runtime is initialized
    let _runtime = get_or_init_runtime();

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
    let provider = match WeatherProvider::new(weather_unit) {
        Ok(p) => Arc::new(p),
        Err(e) => {
            tracing::error!("Failed to create WeatherProvider: {}", e);
            return false;
        }
    };

    // Create weather cache
    let config_dir = myme_core::Config::load()
        .map(|c| c.config_dir)
        .unwrap_or_else(|_| {
            dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("myme")
        });

    let mut cache = WeatherCache::new(&config_dir);
    if let Err(e) = cache.load() {
        tracing::warn!("Failed to load weather cache: {}", e);
    }

    // Store globally
    if WEATHER_PROVIDER.set(provider).is_err() {
        tracing::warn!("WeatherProvider already initialized");
    }

    if WEATHER_CACHE.set(std::sync::Mutex::new(cache)).is_err() {
        tracing::warn!("WeatherCache already initialized");
    }

    tracing::info!("Weather services initialized successfully");
    true
}

/// Get the initialized weather services for use by WeatherModels
pub fn get_weather_services() -> Option<(Arc<WeatherProvider>, WeatherCache, tokio::runtime::Handle)>
{
    let provider = WEATHER_PROVIDER.get()?.clone();
    let runtime = RUNTIME.get()?.handle().clone();

    // Clone the cache data (WeatherCache doesn't impl Clone, so we create a new one and load)
    let config_dir = myme_core::Config::load()
        .map(|c| c.config_dir)
        .unwrap_or_else(|_| {
            dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("myme")
        });

    let mut cache = WeatherCache::new(&config_dir);
    let _ = cache.load();

    Some((provider, cache, runtime))
}

/// Initialize GitHub client and project store
/// Must be called before QML tries to access ProjectModel
#[no_mangle]
pub extern "C" fn initialize_github_client() -> bool {
    // Ensure runtime is initialized
    let _runtime = get_or_init_runtime();

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
    let client = match GitHubClient::new(token) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to create GitHub client: {}", e);
            return false;
        }
    };

    if GITHUB_CLIENT.set(client).is_err() {
        tracing::warn!("GitHub client already initialized");
    }

    // Initialize project store
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("myme");

    let db_path = config_dir.join("projects.db");

    // Ensure directory exists
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        tracing::error!("Failed to create config directory: {}", e);
        return false;
    }

    let store = match ProjectStore::open(&db_path) {
        Ok(s) => Arc::new(std::sync::Mutex::new(s)),
        Err(e) => {
            tracing::error!("Failed to open project store: {}", e);
            return false;
        }
    };

    if PROJECT_STORE.set(store).is_err() {
        tracing::warn!("Project store already initialized");
    }

    tracing::info!("GitHub client and project store initialized");
    true
}

/// Get GitHub client and runtime
pub fn get_github_client_and_runtime() -> Option<(Arc<GitHubClient>, tokio::runtime::Handle)> {
    let client = GITHUB_CLIENT.get()?.clone();
    let runtime = RUNTIME.get()?.handle().clone();
    Some((client, runtime))
}

/// Get project store
pub fn get_project_store() -> Option<Arc<std::sync::Mutex<ProjectStore>>> {
    PROJECT_STORE.get().cloned()
}

/// Check if GitHub is authenticated
pub fn is_github_authenticated() -> bool {
    GITHUB_CLIENT.get().is_some()
}

/// Get the runtime handle (always available after any initialization)
pub fn get_runtime() -> Option<tokio::runtime::Handle> {
    RUNTIME.get().map(|r| r.handle().clone())
}

/// Initialize project store only (without GitHub client)
/// Call this to enable local project storage even without GitHub auth
fn ensure_project_store() -> Option<Arc<std::sync::Mutex<ProjectStore>>> {
    // Return existing store if already set
    if let Some(store) = PROJECT_STORE.get() {
        return Some(store.clone());
    }

    // Initialize project store
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("myme");

    let db_path = config_dir.join("projects.db");

    // Ensure directory exists
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        tracing::error!("Failed to create config directory: {}", e);
        return None;
    }

    match ProjectStore::open(&db_path) {
        Ok(s) => {
            let store = Arc::new(std::sync::Mutex::new(s));
            let _ = PROJECT_STORE.set(store.clone());
            tracing::info!("Project store initialized at {:?}", db_path);
            Some(store)
        }
        Err(e) => {
            tracing::error!("Failed to open project store: {}", e);
            None
        }
    }
}

/// Get project store, initializing if needed
pub fn get_project_store_or_init() -> Option<Arc<std::sync::Mutex<ProjectStore>>> {
    // Ensure runtime is initialized first
    let _ = get_or_init_runtime();
    ensure_project_store()
}

/// Initialize GitHub OAuth provider
/// Must be called before QML tries to use AuthModel
#[no_mangle]
pub extern "C" fn initialize_github_auth() -> bool {
    // Ensure runtime is initialized
    let _runtime = get_or_init_runtime();

    // Load GitHub OAuth config
    let config = match myme_core::Config::load() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to load config for GitHub auth: {}", e);
            return false;
        }
    };

    // Check if GitHub is configured
    if !config.github.is_configured() {
        tracing::info!("GitHub OAuth not configured (using placeholder values)");
        return false;
    }

    // Create GitHub auth provider
    let provider = Arc::new(GitHubAuth::new(
        config.github.client_id.clone(),
        config.github.client_secret.clone(),
    ));

    if GITHUB_AUTH_PROVIDER.set(provider).is_err() {
        tracing::warn!("GitHub auth provider already initialized");
    }

    tracing::info!("GitHub OAuth provider initialized");
    true
}

/// Get GitHub auth provider and runtime for use by AuthModel
pub fn get_github_auth_and_runtime() -> Option<(Arc<GitHubAuth>, tokio::runtime::Handle)> {
    let provider = GITHUB_AUTH_PROVIDER.get()?.clone();
    let runtime = RUNTIME.get()?.handle().clone();
    Some((provider, runtime))
}

/// Get effective repos local search path and whether config path was invalid.
/// Returns (effective_path, config_path_invalid).
pub fn get_repos_local_search_path() -> Option<(std::path::PathBuf, bool)> {
    let config = myme_core::Config::load().ok()?;
    Some(config.repos.effective_local_search_path())
}

/// Initialize repo service channel. Call once when RepoModel is first created.
/// Returns true if initialized (or already initialized).
pub fn init_repo_service_channel() -> bool {
    if REPO_SERVICE_TX.get().is_some() {
        return true;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    REPO_SERVICE_TX.set(tx).ok();
    REPO_SERVICE_RX.set(std::sync::Mutex::new(rx)).ok();
    true
}

/// Get repo service sender for request_* calls. None if init_repo_service_channel not called yet.
pub fn get_repo_service_tx() -> Option<std::sync::mpsc::Sender<crate::services::RepoServiceMessage>>
{
    REPO_SERVICE_TX.get().cloned()
}

/// Non-blocking recv from repo service channel. Called by RepoModel::poll_channel.
pub fn try_recv_repo_message() -> Option<crate::services::RepoServiceMessage> {
    let rx = REPO_SERVICE_RX.get()?;
    rx.lock().ok()?.try_recv().ok()
}

/// Initialize note service channel. Call once when NoteModel is first created.
/// Returns true if initialized (or already initialized).
pub fn init_note_service_channel() -> bool {
    if NOTE_SERVICE_TX.get().is_some() {
        return true;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    NOTE_SERVICE_TX.set(tx).ok();
    NOTE_SERVICE_RX.set(std::sync::Mutex::new(rx)).ok();
    true
}

/// Get note service sender for request_* calls. None if init_note_service_channel not called yet.
pub fn get_note_service_tx() -> Option<std::sync::mpsc::Sender<crate::services::NoteServiceMessage>>
{
    NOTE_SERVICE_TX.get().cloned()
}

/// Non-blocking recv from note service channel. Called by NoteModel::poll_channel.
pub fn try_recv_note_message() -> Option<crate::services::NoteServiceMessage> {
    let rx = NOTE_SERVICE_RX.get()?;
    rx.lock().ok()?.try_recv().ok()
}

/// Reinitialize GitHub client after successful OAuth
/// Call this after authentication completes to refresh the client with new token
pub fn reinitialize_github_client() {
    tracing::info!("Attempting to reinitialize GitHub client after OAuth...");

    // Note: OnceLock doesn't support clearing/replacing, so if the client
    // was already initialized with no token, it can't be updated without
    // restarting the app. This is a known limitation.
    //
    // If the client wasn't initialized yet (user hadn't authenticated before),
    // this will set it up with the new token from secure storage.
    let success = initialize_github_client();

    if success {
        tracing::info!("GitHub client initialized with new token");
    } else {
        tracing::warn!(
            "GitHub client initialization returned false - may need app restart for new token"
        );
    }
}
