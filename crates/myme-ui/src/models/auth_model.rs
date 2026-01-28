// crates/myme-ui/src/models/auth_model.rs

use core::pin::Pin;
use std::sync::Arc;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_auth::{GitHubAuth, OAuth2Provider};

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

        #[qsignal]
        fn auth_changed(self: Pin<&mut AuthModel>);

        #[qsignal]
        fn auth_completed(self: Pin<&mut AuthModel>);
    }
}

#[derive(Default)]
pub struct AuthModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    provider_name: QString,
    provider: Option<Arc<GitHubAuth>>,
    runtime: Option<tokio::runtime::Handle>,
}

impl AuthModelRust {
    /// Auto-initialize from global services
    fn ensure_initialized(&mut self) {
        if self.provider.is_some() && self.runtime.is_some() {
            return;
        }

        // Default to GitHub provider
        if let Some((provider, runtime)) = crate::bridge::get_github_auth_and_runtime() {
            self.provider = Some(provider);
            self.runtime = Some(runtime);
            self.provider_name = QString::from("GitHub");
            tracing::info!("AuthModel initialized for GitHub");
        } else {
            tracing::warn!("AuthModel: GitHub auth provider not available");
        }
    }
}

impl qobject::AuthModel {
    /// Start OAuth authentication flow
    pub fn authenticate(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().ensure_initialized();

        let provider = match &self.as_ref().rust().provider {
            Some(p) => p.clone(),
            None => {
                self.as_mut().set_error_message(QString::from(
                    "GitHub OAuth not configured. Please add client_id and client_secret to config.toml",
                ));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => {
                self.as_mut()
                    .set_error_message(QString::from("Runtime not initialized"));
                return;
            }
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        tracing::info!("Starting GitHub OAuth authentication flow...");

        // Run OAuth flow (blocks UI thread - acceptable for auth)
        match runtime.block_on(async { provider.authenticate().await }) {
            Ok(_token) => {
                tracing::info!("GitHub authentication successful");

                // Reinitialize GitHub client with new token
                crate::bridge::reinitialize_github_client();

                self.as_mut().set_authenticated(true);
                self.as_mut().set_loading(false);
                self.as_mut().auth_changed();
                self.as_mut().auth_completed();
            }
            Err(e) => {
                tracing::error!("GitHub authentication failed: {}", e);
                self.as_mut()
                    .set_error_message(QString::from(format!("Authentication failed: {}", e)));
                self.as_mut().set_loading(false);
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
                self.as_mut().set_error_message(QString::from(""));
                self.as_mut().auth_changed();
            }
            Err(e) => {
                tracing::error!("Sign out failed: {}", e);
                self.as_mut()
                    .set_error_message(QString::from(format!("Sign out failed: {}", e)));
            }
        }
    }
}
