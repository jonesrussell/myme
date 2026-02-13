//! Google OAuth authentication model for QML.
//!
//! Provides Google Sign-In for Gmail and Calendar access.

use core::pin::Pin;
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_auth::{GoogleOAuth2Provider, SecureStorage, TokenSet};

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(bool, loading)]
        #[qproperty(bool, authenticated)]
        #[qproperty(QString, error_message)]
        #[qproperty(QString, user_email)]
        type GoogleAuthModel = super::GoogleAuthModelRust;

        #[qinvokable]
        fn authenticate(self: Pin<&mut GoogleAuthModel>);

        #[qinvokable]
        fn check_auth(self: Pin<&mut GoogleAuthModel>);

        #[qinvokable]
        fn sign_out(self: Pin<&mut GoogleAuthModel>);

        /// Poll for async operation results. Call this from a QML Timer.
        #[qinvokable]
        fn poll_channel(self: Pin<&mut GoogleAuthModel>);

        /// Open the application config directory in the system file manager.
        #[qinvokable]
        fn open_config_folder(self: Pin<&mut GoogleAuthModel>) -> bool;

        #[qsignal]
        fn auth_changed(self: Pin<&mut GoogleAuthModel>);

        #[qsignal]
        fn auth_completed(self: Pin<&mut GoogleAuthModel>);
    }
}

/// Messages for async operations
enum AuthMessage {
    AuthenticateDone(Result<String, String>), // Result<access_token, error>
}

/// Operation state tracking
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum OpState {
    #[default]
    Idle,
    Authenticating,
}

#[derive(Default)]
pub struct GoogleAuthModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    user_email: QString,
    op_state: OpState,
    rx: Option<mpsc::Receiver<AuthMessage>>,
}

impl GoogleAuthModelRust {
    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }

    fn get_google_config() -> Option<(String, String)> {
        match myme_core::Config::load() {
            Ok(config) => {
                let client_id = config.google.as_ref()?.client_id.clone()?;
                let client_secret = config.google.as_ref()?.client_secret.clone()?;
                Some((client_id, client_secret))
            }
            Err(e) => {
                tracing::warn!("Failed to load Google config: {}", e);
                None
            }
        }
    }

    /// Path to config.toml for display in error messages (platform-specific).
    fn config_display_path() -> String {
        dirs::config_dir()
            .map(|d| d.join("myme").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
            .display()
            .to_string()
    }
}

impl qobject::GoogleAuthModel {
    /// Start Google OAuth authentication flow (non-blocking)
    pub fn authenticate(mut self: Pin<&mut Self>) {
        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("authenticate: operation already in progress");
            return;
        }

        let (client_id, client_secret) = match GoogleAuthModelRust::get_google_config() {
            Some(config) => config,
            None => {
                let path = GoogleAuthModelRust::config_display_path();
                self.as_mut().set_error_message(QString::from(&format!(
                    "Google OAuth not configured. Add a [google] section with client_id and client_secret to {}",
                    path
                )));
                return;
            }
        };

        // Create channel for async result
        let (tx, rx) = mpsc::channel();
        self.as_mut().rust_mut().rx = Some(rx);

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::Authenticating;

        tracing::info!("Starting Google OAuth authentication flow...");

        // Spawn async operation
        std::thread::spawn(move || {
            // Runtime creation in worker thread; failure is fatal.
            #[allow(clippy::unwrap_used)]
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                let provider = GoogleOAuth2Provider::new(client_id, client_secret);

                // Find available port
                let port = find_available_port(8080, 8089).ok_or("No available port")?;

                // Generate auth URL
                let (auth_url, expected_state) = provider.authorization_url(port);

                // Open browser
                if let Err(e) = webbrowser::open(&auth_url) {
                    return Err(format!("Failed to open browser: {}", e));
                }

                // Start callback server
                let code = wait_for_callback(port, &expected_state).await?;

                // Exchange code for tokens
                let token_response = provider
                    .exchange_code(&code, port)
                    .await
                    .map_err(|e| format!("Token exchange failed: {}", e))?;

                // Get user info
                let user_info = provider
                    .get_user_info(&token_response.access_token)
                    .await
                    .map_err(|e| format!("Failed to get user info: {}", e))?;

                // Calculate expiration
                let expires_at = chrono::Utc::now().timestamp() + token_response.expires_in as i64;

                // Store tokens securely
                let token_set = TokenSet {
                    access_token: token_response.access_token.clone(),
                    refresh_token: token_response.refresh_token,
                    expires_at,
                    scopes: token_response.scope.split(' ').map(|s| s.to_string()).collect(),
                };

                SecureStorage::store_token("google", &token_set)
                    .map_err(|e| format!("Failed to store token: {}", e))?;

                tracing::info!("Google authentication successful for {}", user_info.email);
                Ok(user_info.email)
            });

            let _ = tx.send(AuthMessage::AuthenticateDone(result));
        });
    }

    /// Poll for async operation results
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match self.as_ref().rust().rx.as_ref().and_then(|rx| rx.try_recv().ok()) {
            Some(m) => m,
            None => return,
        };

        match msg {
            AuthMessage::AuthenticateDone(result) => {
                self.as_mut().set_loading(false);
                self.as_mut().rust_mut().op_state = OpState::Idle;

                match result {
                    Ok(email) => {
                        tracing::info!("Google authentication completed");
                        self.as_mut().rust_mut().clear_error();
                        self.as_mut().set_authenticated(true);
                        self.as_mut().set_user_email(QString::from(&email));
                        self.as_mut().auth_changed();
                        self.as_mut().auth_completed();
                    }
                    Err(e) => {
                        tracing::error!("Google authentication failed: {}", e);
                        self.as_mut().rust_mut().set_error(&e);
                        self.as_mut().set_authenticated(false);
                    }
                }
            }
        }
    }

    /// Check if currently authenticated
    pub fn check_auth(mut self: Pin<&mut Self>) {
        let is_authenticated = SecureStorage::has_token("google");

        if is_authenticated {
            // Try to get user email from stored token
            if let Ok(token_set) = SecureStorage::retrieve_token("google") {
                if !token_set.is_expired() {
                    self.as_mut().set_authenticated(true);
                    // We don't have email stored, would need to fetch it
                    return;
                }
            }
        }

        self.as_mut().set_authenticated(false);
    }

    /// Open the application config directory in the system file manager.
    pub fn open_config_folder(self: Pin<&mut Self>) -> bool {
        let config_dir =
            dirs::config_dir().map(|d| d.join("myme")).unwrap_or_else(|| PathBuf::from("."));
        if !config_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&config_dir) {
                tracing::warn!("Failed to create config dir: {}", e);
                return false;
            }
        }
        let result = if cfg!(target_os = "windows") {
            Command::new("explorer").arg(&config_dir).spawn()
        } else if cfg!(target_os = "macos") {
            Command::new("open").arg(&config_dir).spawn()
        } else {
            Command::new("xdg-open").arg(&config_dir).spawn()
        };
        match result {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Failed to open config folder: {}", e);
                false
            }
        }
    }

    /// Sign out and remove stored token
    pub fn sign_out(mut self: Pin<&mut Self>) {
        match SecureStorage::delete_token("google") {
            Ok(_) => {
                tracing::info!("Signed out from Google successfully");
                self.as_mut().set_authenticated(false);
                self.as_mut().set_user_email(QString::from(""));
                self.as_mut().rust_mut().clear_error();
                self.as_mut().auth_changed();
            }
            Err(e) => {
                tracing::error!("Sign out failed: {}", e);
                self.as_mut().rust_mut().set_error(myme_core::AppError::from(e).user_message());
            }
        }
    }
}

/// Find an available port in the given range
fn find_available_port(start: u16, end: u16) -> Option<u16> {
    for port in start..=end {
        if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() {
            return Some(port);
        }
    }
    None
}

/// Wait for OAuth callback
async fn wait_for_callback(port: u16, expected_state: &str) -> Result<String, String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::TcpListener;

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .map_err(|e| format!("Failed to bind: {}", e))?;

    tracing::info!("Waiting for OAuth callback on port {}", port);

    let (mut stream, _) = listener.accept().await.map_err(|e| format!("Accept failed: {}", e))?;

    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await.map_err(|e| format!("Read failed: {}", e))?;

    // Parse the callback URL
    // GET /callback?code=xxx&state=yyy HTTP/1.1
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid request".to_string());
    }

    let url = url::Url::parse(&format!("http://localhost{}", parts[1]))
        .map_err(|e| format!("URL parse failed: {}", e))?;

    let code = url
        .query_pairs()
        .find(|(k, _): &(std::borrow::Cow<str>, std::borrow::Cow<str>)| k == "code")
        .map(|(_, v)| v.to_string())
        .ok_or("No code in callback")?;

    let state = url
        .query_pairs()
        .find(|(k, _): &(std::borrow::Cow<str>, std::borrow::Cow<str>)| k == "state")
        .map(|(_, v)| v.to_string())
        .ok_or("No state in callback")?;

    if state != expected_state {
        return Err("State mismatch - possible CSRF attack".to_string());
    }

    // Send success response
    let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<html><body><h1>Authentication successful!</h1><p>You can close this window.</p></body></html>";
    let (_read_half, mut write_half) = stream.into_split();
    write_half.write_all(response.as_bytes()).await.ok();

    Ok(code)
}
