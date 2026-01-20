use core::pin::Pin;
use std::sync::Arc;
use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;

use myme_integrations::{GitHubClient, Repository};
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
        #[qproperty(QString, username)]
        type RepoModel = super::RepoModelRust;

        #[qinvokable]
        fn authenticate(self: Pin<&mut RepoModel>);

        #[qinvokable]
        fn fetch_repositories(self: Pin<&mut RepoModel>);

        #[qinvokable]
        fn create_repository(
            self: Pin<&mut RepoModel>,
            name: &QString,
            description: &QString,
            is_private: bool,
        );

        #[qinvokable]
        fn sign_out(self: Pin<&mut RepoModel>);

        #[qinvokable]
        fn row_count(self: &RepoModel) -> i32;

        #[qinvokable]
        fn get_name(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_full_name(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_description(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_url(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_clone_url(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_stars(self: &RepoModel, index: i32) -> i32;

        #[qinvokable]
        fn get_is_private(self: &RepoModel, index: i32) -> bool;

        #[qsignal]
        fn repos_changed(self: Pin<&mut RepoModel>);

        #[qsignal]
        fn authentication_changed(self: Pin<&mut RepoModel>);
    }
}

#[derive(Default)]
pub struct RepoModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    username: QString,
    repos: Vec<Repository>,
    github_auth: Option<Arc<GitHubAuth>>,
    github_client: Option<Arc<GitHubClient>>,
    runtime: Option<tokio::runtime::Handle>,
}

impl RepoModelRust {
    pub fn initialize(
        &mut self,
        client_id: String,
        client_secret: String,
        runtime: tokio::runtime::Handle,
    ) {
        let auth = Arc::new(GitHubAuth::new(client_id, client_secret));

        // Check if already authenticated
        let authenticated = auth.is_authenticated();

        // Create client if authenticated
        let client = if authenticated {
            if let Some(token) = auth.get_token() {
                Some(Arc::new(GitHubClient::new(token.access_token)))
            } else {
                None
            }
        } else {
            None
        };

        self.github_auth = Some(auth);
        self.github_client = client;
        self.runtime = Some(runtime);
        self.authenticated = authenticated;
    }
}

impl qobject::RepoModel {
    pub fn authenticate(mut self: Pin<&mut Self>) {
        let auth = match &self.as_ref().rust().github_auth {
            Some(a) => a.clone(),
            None => {
                self.as_mut().set_error_message(QString::from("Not initialized"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        runtime.spawn(async move {
            match auth.authenticate().await {
                Ok(token_set) => {
                    tracing::info!("GitHub authentication successful");
                    // Create client with new token
                    let _client = GitHubClient::new(token_set.access_token);
                    // TODO: Signal authentication success to UI
                }
                Err(e) => {
                    tracing::error!("GitHub authentication failed: {}", e);
                    // TODO: Signal authentication failure to UI
                }
            }
        });
    }

    pub fn fetch_repositories(mut self: Pin<&mut Self>) {
        let client = match &self.as_ref().rust().github_client {
            Some(c) => c.clone(),
            None => {
                self.as_mut().set_error_message(QString::from("Not authenticated"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        self.as_mut().set_loading(true);
        self.as_mut().set_error_message(QString::from(""));

        runtime.spawn(async move {
            match client.list_repositories(Some("all"), Some("updated")).await {
                Ok(repos) => {
                    tracing::info!("Successfully fetched {} repositories", repos.len());
                    // TODO: Update repos list in model and signal UI
                }
                Err(e) => {
                    tracing::error!("Failed to fetch repositories: {}", e);
                    // TODO: Signal error to UI
                }
            }
        });
    }

    pub fn create_repository(
        mut self: Pin<&mut Self>,
        name: &QString,
        description: &QString,
        is_private: bool,
    ) {
        let client = match &self.as_ref().rust().github_client {
            Some(c) => c.clone(),
            None => {
                self.as_mut().set_error_message(QString::from("Not authenticated"));
                return;
            }
        };

        let runtime = match &self.as_ref().rust().runtime {
            Some(r) => r.clone(),
            None => return,
        };

        let name_str = name.to_string();
        let desc_str = description.to_string();
        let desc_opt = if desc_str.is_empty() {
            None
        } else {
            Some(desc_str)
        };

        self.as_mut().set_loading(true);

        runtime.spawn(async move {
            match client.create_repository(&name_str, desc_opt.as_deref(), is_private).await {
                Ok(repo) => {
                    tracing::info!("Created repository: {}", repo.full_name);
                    // TODO: Add repo to list and signal UI
                }
                Err(e) => {
                    tracing::error!("Failed to create repository: {}", e);
                    // TODO: Signal error to UI
                }
            }
        });
    }

    pub fn sign_out(mut self: Pin<&mut Self>) {
        if let Some(auth) = &self.as_ref().rust().github_auth {
            if let Err(e) = auth.sign_out() {
                tracing::error!("Failed to sign out: {}", e);
                self.as_mut().set_error_message(QString::from(&format!("Sign out failed: {}", e)));
                return;
            }
        }

        // Clear authentication state
        self.as_mut().set_authenticated(false);
        self.as_mut().set_username(QString::from(""));

        // Clear repos
        self.as_mut().rust_mut().repos.clear();
        self.as_mut().rust_mut().github_client = None;

        self.as_mut().repos_changed();
        self.as_mut().authentication_changed();

        tracing::info!("Signed out from GitHub");
    }

    pub fn row_count(&self) -> i32 {
        self.rust().repos.len() as i32
    }

    pub fn get_name(&self, index: i32) -> QString {
        let repos = &self.rust().repos;
        if index < 0 || index >= repos.len() as i32 {
            return QString::from("");
        }
        QString::from(&repos[index as usize].name)
    }

    pub fn get_full_name(&self, index: i32) -> QString {
        let repos = &self.rust().repos;
        if index < 0 || index >= repos.len() as i32 {
            return QString::from("");
        }
        QString::from(&repos[index as usize].full_name)
    }

    pub fn get_description(&self, index: i32) -> QString {
        let repos = &self.rust().repos;
        if index < 0 || index >= repos.len() as i32 {
            return QString::from("");
        }
        QString::from(repos[index as usize].description.as_deref().unwrap_or(""))
    }

    pub fn get_url(&self, index: i32) -> QString {
        let repos = &self.rust().repos;
        if index < 0 || index >= repos.len() as i32 {
            return QString::from("");
        }
        QString::from(&repos[index as usize].html_url)
    }

    pub fn get_clone_url(&self, index: i32) -> QString {
        let repos = &self.rust().repos;
        if index < 0 || index >= repos.len() as i32 {
            return QString::from("");
        }
        QString::from(&repos[index as usize].clone_url)
    }

    pub fn get_stars(&self, index: i32) -> i32 {
        let repos = &self.rust().repos;
        if index < 0 || index >= repos.len() as i32 {
            return 0;
        }
        repos[index as usize].stargazers_count as i32
    }

    pub fn get_is_private(&self, index: i32) -> bool {
        let repos = &self.rust().repos;
        if index < 0 || index >= repos.len() as i32 {
            return false;
        }
        repos[index as usize].private
    }
}
