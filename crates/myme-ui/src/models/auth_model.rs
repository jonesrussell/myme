// crates/myme-ui/src/models/auth_model.rs

use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_auth::{GitHubAuth, OAuth2Provider};

use crate::bridge;
use crate::services::{request_auth, AuthServiceMessage};

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
        #[qproperty(QString, provider_name)]
        type AuthModel = super::AuthModelRust;

        #[qinvokable]
        fn authenticate(self: Pin<&mut AuthModel>);

        #[qinvokable]
        fn check_auth(self: Pin<&mut AuthModel>);

        #[qinvokable]
        fn sign_out(self: Pin<&mut AuthModel>);

        /// Poll for async operation results. Call this from a QML Timer.
        #[qinvokable]
        fn poll_channel(self: Pin<&mut AuthModel>);

        #[qsignal]
        fn auth_changed(self: Pin<&mut AuthModel>);

        #[qsignal]
        fn auth_completed(self: Pin<&mut AuthModel>);
    }
}

/// Operation state tracking
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum OpState {
    #[default]
    Idle,
    Authenticating,
}

#[derive(Default)]
pub struct AuthModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    provider_name: QString,
    provider: Option<Arc<GitHubAuth>>,
    op_state: OpState,
}

impl AuthModelRust {
    /// Auto-initialize from global services
    fn ensure_initialized(&mut self) {
        if self.provider.is_some() {
            return;
        }

        // Default to GitHub provider
        if let Some((provider, _runtime)) = crate::bridge::get_github_auth_and_runtime() {
            self.provider = Some(provider);
            self.provider_name = QString::from("GitHub");
            tracing::info!("AuthModel initialized for GitHub");
        } else {
            tracing::warn!("AuthModel: GitHub auth provider not available");
        }
    }

    fn set_error(&mut self, msg: &str) {
        self.error_message = QString::from(msg);
    }

    fn clear_error(&mut self) {
        self.error_message = QString::from("");
    }
}

impl qobject::AuthModel {
    /// Start OAuth authentication flow (non-blocking)
    pub fn authenticate(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();

        // Prevent concurrent operations
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            tracing::warn!("authenticate: operation already in progress");
            return;
        }

        let provider = match &self.as_ref().rust().provider {
            Some(p) => p.clone(),
            None => {
                self.as_mut().set_error_message(QString::from(
                    "GitHub OAuth not configured. Please add client_id and client_secret to config.toml",
                ));
                return;
            }
        };

        // Initialize channel if needed
        bridge::init_auth_service_channel();
        let tx = match bridge::get_auth_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Service channel not ready"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().clear_error();
        self.as_mut().rust_mut().op_state = OpState::Authenticating;

        tracing::info!("Starting GitHub OAuth authentication flow...");

        // Spawn async operation (non-blocking)
        request_auth(&tx, provider);
    }

    /// Poll for async operation results. Call this from a QML Timer (e.g., every 100ms).
    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_auth_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            AuthServiceMessage::AuthenticateDone(result) => {
                self.as_mut().set_loading(false);
                self.as_mut().rust_mut().op_state = OpState::Idle;

                match result {
                    Ok(_token) => {
                        tracing::info!("GitHub authentication successful");

                        // Reinitialize GitHub client with new token
                        crate::bridge::reinitialize_github_client();

                        self.as_mut().rust_mut().clear_error();
                        self.as_mut().set_authenticated(true);
                        self.as_mut().auth_changed();
                        self.as_mut().auth_completed();
                    }
                    Err(e) => {
                        tracing::error!("GitHub authentication failed: {}", e);
                        self.as_mut()
                            .rust_mut()
                            .set_error(&format!("Authentication failed: {}", e));
                        self.as_mut().set_authenticated(false);
                    }
                }
            }
        }
    }

    /// Check if currently authenticated
    pub fn check_auth(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();

        let is_authenticated = self
            .as_ref()
            .rust()
            .provider
            .as_ref()
            .map(|p| p.is_authenticated())
            .unwrap_or(false);

        let was_authenticated = self.as_ref().rust().authenticated;
        self.as_mut().set_authenticated(is_authenticated);

        if was_authenticated != is_authenticated {
            self.as_mut().auth_changed();
        }
    }

    /// Sign out and remove stored token
    pub fn sign_out(mut self: Pin<&mut Self>) {
        let provider = match &self.as_ref().rust().provider {
            Some(p) => p.clone(),
            None => return,
        };

        match provider.sign_out() {
            Ok(_) => {
                tracing::info!("Signed out from GitHub successfully");
                self.as_mut().set_authenticated(false);
                self.as_mut().rust_mut().clear_error();
                self.as_mut().auth_changed();
            }
            Err(e) => {
                tracing::error!("Sign out failed: {}", e);
                self.as_mut()
                    .rust_mut()
                    .set_error(&format!("Sign out failed: {}", e));
            }
        }
    }
}
