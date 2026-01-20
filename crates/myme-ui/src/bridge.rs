use std::sync::{Arc, OnceLock, Mutex};
use std::ffi::CStr;
use std::os::raw::c_char;
use core::pin::Pin;

use crate::models::note_model::qobject::NoteModel;
use crate::models::note_model::NoteModelRust;
use myme_services::TodoClient;

// Static tokio runtime that lives for the duration of the application
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

// Static client and runtime for creating NoteModels
static TODO_CLIENT: OnceLock<Arc<TodoClient>> = OnceLock::new();

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

    // Create the TodoClient
    let client = match TodoClient::new(base_url_str, None) {
        Ok(c) => Arc::new(c),
        Err(e) => {
            tracing::error!("Failed to create TodoClient: {}", e);
            return false;
        }
    };

    // Get or initialize tokio runtime
    let runtime = get_or_init_runtime();

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
