use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::{Arc, OnceLock};

use myme_services::TodoClient;
use myme_weather::{WeatherCache, WeatherProvider};

// Static tokio runtime that lives for the duration of the application
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

// Static client and runtime for creating NoteModels
static TODO_CLIENT: OnceLock<Arc<TodoClient>> = OnceLock::new();

// Weather services
static WEATHER_PROVIDER: OnceLock<Arc<WeatherProvider>> = OnceLock::new();
static WEATHER_CACHE: OnceLock<std::sync::Mutex<WeatherCache>> = OnceLock::new();

/// Initialize the tokio runtime (call once at application startup)
fn get_or_init_runtime() -> tokio::runtime::Handle {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("myme-tokio")
            .build()
            .expect("Failed to create tokio runtime")
    }).handle().clone()
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
                tracing::warn!("No JWT token configured - API calls may fail if authentication is required");
                None
            }
        }
        Err(e) => {
            tracing::warn!("Failed to load config: {}. Checking environment variable.", e);
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
pub fn get_weather_services() -> Option<(Arc<WeatherProvider>, WeatherCache, tokio::runtime::Handle)> {
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
