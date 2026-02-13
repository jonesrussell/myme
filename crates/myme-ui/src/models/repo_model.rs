use core::pin::Pin;

use cxx_qt::CxxQtType;
use cxx_qt_lib::QString;
use myme_integrations::{RepoEntry, RepoState};

use crate::bridge;
use crate::services::{request_clone, request_pull, request_refresh, RepoServiceMessage};

#[derive(Clone, Copy, PartialEq, Eq)]
enum OpState {
    Idle,
    BusyRefresh,
    BusyClone(usize),
    BusyPull(usize),
}

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
        #[qproperty(bool, config_path_invalid)]
        #[qproperty(QString, effective_path)]
        type RepoModel = super::RepoModelRust;

        #[qinvokable]
        fn check_auth(self: Pin<&mut RepoModel>);

        #[qinvokable]
        fn fetch_repos(self: Pin<&mut RepoModel>);

        #[qinvokable]
        fn clone_repo(self: Pin<&mut RepoModel>, index: i32);

        #[qinvokable]
        fn pull_repo(self: Pin<&mut RepoModel>, index: i32);

        #[qinvokable]
        fn cancel_operation(self: Pin<&mut RepoModel>);

        #[qinvokable]
        fn poll_channel(self: Pin<&mut RepoModel>);

        #[qinvokable]
        fn clear_error(self: Pin<&mut RepoModel>);

        #[qinvokable]
        fn row_count(self: &RepoModel) -> i32;

        #[qinvokable]
        fn get_full_name(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_local_path(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_branch(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_is_clean(self: &RepoModel, index: i32) -> bool;

        #[qinvokable]
        fn get_has_local(self: &RepoModel, index: i32) -> bool;

        #[qinvokable]
        fn get_has_github(self: &RepoModel, index: i32) -> bool;

        #[qinvokable]
        fn get_clone_url(self: &RepoModel, index: i32) -> QString;

        #[qinvokable]
        fn get_state(self: &RepoModel, index: i32) -> i32;

        #[qinvokable]
        fn get_busy(self: &RepoModel, index: i32) -> bool;

        #[qinvokable]
        fn get_html_url(self: &RepoModel, index: i32) -> QString;

        #[qsignal]
        fn repos_changed(self: Pin<&mut RepoModel>);

        #[qsignal]
        fn auth_changed(self: Pin<&mut RepoModel>);

        #[qsignal]
        fn error_occurred(self: Pin<&mut RepoModel>);
    }
}

#[derive(Default)]
pub struct RepoModelRust {
    loading: bool,
    authenticated: bool,
    error_message: QString,
    config_path_invalid: bool,
    effective_path: QString,
    entries: Vec<RepoEntry>,
    op_state: OpState,
}

impl RepoModelRust {
    fn get_entry(&self, index: i32) -> Option<&RepoEntry> {
        if index < 0 {
            return None;
        }
        self.entries.get(index as usize)
    }

    fn set_error(&mut self, s: String) {
        self.error_message = QString::from(&s);
    }

    fn clear_error_msg(&mut self) {
        self.error_message = QString::from("");
    }
}

impl Default for OpState {
    fn default() -> Self {
        OpState::Idle
    }
}

impl qobject::RepoModel {
    pub fn check_auth(mut self: Pin<&mut Self>) {
        bridge::init_repo_service_channel();
        let auth = bridge::is_github_authenticated();
        self.as_mut().set_authenticated(auth);
        if let Some((path, invalid)) = bridge::get_repos_local_search_path() {
            self.as_mut().set_config_path_invalid(invalid);
            self.as_mut().set_effective_path(QString::from(path.to_string_lossy().as_ref()));
        }
        self.as_mut().auth_changed();
    }

    pub fn fetch_repos(mut self: Pin<&mut Self>) {
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            self.as_mut().rust_mut().set_error("Operation in progress".into());
            self.as_mut().error_occurred();
            return;
        }

        bridge::init_repo_service_channel();
        let tx = match bridge::get_repo_service_tx() {
            Some(t) => t,
            None => {
                self.as_mut().rust_mut().set_error("Repo service not initialized".into());
                self.as_mut().error_occurred();
                return;
            }
        };

        if let Some((path, invalid)) = bridge::get_repos_local_search_path() {
            self.as_mut().set_config_path_invalid(invalid);
            self.as_mut().set_effective_path(QString::from(path.to_string_lossy().as_ref()));
        }
        self.as_mut().set_loading(true);
        self.as_mut().rust_mut().op_state = OpState::BusyRefresh;
        self.as_mut().rust_mut().clear_error_msg();

        request_refresh(&tx);
    }

    pub fn clone_repo(mut self: Pin<&mut Self>, index: i32) {
        if index < 0 {
            return;
        }
        let i = index as usize;
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            return;
        }
        let ent = match self.as_ref().rust().get_entry(index) {
            Some(e) => e.clone(),
            None => return,
        };
        if ent.state != RepoState::GitHubOnly {
            return;
        }
        let clone_url = ent.github.as_ref().and_then(|g| {
            g.clone_url
                .clone()
                .filter(|s| !s.is_empty())
                .or_else(|| Some(format!("https://github.com/{}.git", g.full_name)))
        });
        let clone_url = match clone_url {
            Some(u) => u,
            None => {
                self.as_mut().rust_mut().set_error("No clone URL for this repo".into());
                self.as_mut().error_occurred();
                return;
            }
        };

        let (base_path, _) =
            bridge::get_repos_local_search_path().unwrap_or((std::path::PathBuf::from("."), true));
        let full_name = ent.full_name.clone();
        let sep = std::path::MAIN_SEPARATOR;
        let target_path = base_path.join(full_name.replace('/', &sep.to_string()));

        bridge::init_repo_service_channel();
        let tx = match bridge::get_repo_service_tx() {
            Some(t) => t,
            None => return,
        };

        // Create a cancellation token for this operation
        let cancel_token = bridge::new_repo_cancel_token();

        self.as_mut().rust_mut().op_state = OpState::BusyClone(i);
        if let Some(e) = self.as_mut().rust_mut().entries.get_mut(i) {
            e.busy = true;
        }
        self.as_mut().repos_changed();

        request_clone(&tx, i, clone_url, target_path, Some(cancel_token));
    }

    pub fn pull_repo(mut self: Pin<&mut Self>, index: i32) {
        if index < 0 {
            return;
        }
        let i = index as usize;
        if !matches!(self.as_ref().rust().op_state, OpState::Idle) {
            return;
        }
        let path = match self.as_ref().rust().get_entry(index) {
            Some(e) => e.local.as_ref().map(|l| l.path.clone()),
            None => None,
        };
        let path = match path {
            Some(p) => p,
            None => return,
        };

        bridge::init_repo_service_channel();
        let tx = match bridge::get_repo_service_tx() {
            Some(t) => t,
            None => return,
        };

        // Create a cancellation token for this operation
        let cancel_token = bridge::new_repo_cancel_token();

        self.as_mut().rust_mut().op_state = OpState::BusyPull(i);
        if let Some(e) = self.as_mut().rust_mut().entries.get_mut(i) {
            e.busy = true;
        }
        self.as_mut().repos_changed();

        request_pull(&tx, i, path, Some(cancel_token));
    }

    pub fn cancel_operation(mut self: Pin<&mut Self>) {
        // Cancel any active operation
        bridge::cancel_repo_operation();

        // Reset state based on current operation
        match self.as_ref().rust().op_state {
            OpState::BusyClone(idx) | OpState::BusyPull(idx) => {
                if let Some(e) = self.as_mut().rust_mut().entries.get_mut(idx) {
                    e.busy = false;
                }
            }
            OpState::BusyRefresh => {
                self.as_mut().set_loading(false);
            }
            OpState::Idle => {}
        }

        self.as_mut().rust_mut().op_state = OpState::Idle;
        self.as_mut().repos_changed();
        tracing::info!("Repo operation cancelled by user");
    }

    pub fn poll_channel(mut self: Pin<&mut Self>) {
        let msg = match bridge::try_recv_repo_message() {
            Some(m) => m,
            None => return,
        };

        match msg {
            RepoServiceMessage::RefreshDone(res) => {
                self.as_mut().set_loading(false);
                self.as_mut().rust_mut().op_state = OpState::Idle;
                match res {
                    Ok(entries) => {
                        self.as_mut().rust_mut().clear_error_msg();
                        self.as_mut().rust_mut().entries = entries;
                        self.as_mut().repos_changed();
                    }
                    Err(e) => {
                        self.as_mut()
                            .rust_mut()
                            .set_error(myme_core::AppError::from(e).user_message().to_string());
                        self.as_mut().error_occurred();
                    }
                }
            }
            RepoServiceMessage::CloneDone { index, result } => {
                // Clear cancellation token
                bridge::clear_repo_cancel_token();

                if let Some(e) = self.as_mut().rust_mut().entries.get_mut(index) {
                    e.busy = false;
                }
                self.as_mut().rust_mut().op_state = OpState::Idle;
                self.as_mut().repos_changed();

                match &result {
                    Ok(()) => {
                        // Trigger refresh after successful clone
                        if let Some(tx) = bridge::get_repo_service_tx() {
                            self.as_mut().set_loading(true);
                            self.as_mut().rust_mut().op_state = OpState::BusyRefresh;
                            request_refresh(&tx);
                        }
                    }
                    Err(crate::services::RepoError::Cancelled) => {
                        // Silently handle cancellation - no error, no refresh
                        tracing::info!("Clone operation was cancelled");
                    }
                    Err(e) => {
                        self.as_mut().rust_mut().set_error(
                            myme_core::AppError::from(e.clone()).user_message().to_string(),
                        );
                        self.as_mut().error_occurred();
                    }
                }
            }
            RepoServiceMessage::PullDone { index, result } => {
                // Clear cancellation token
                bridge::clear_repo_cancel_token();

                if let Some(e) = self.as_mut().rust_mut().entries.get_mut(index) {
                    e.busy = false;
                }
                self.as_mut().rust_mut().op_state = OpState::Idle;
                self.as_mut().repos_changed();

                match &result {
                    Ok(()) => {
                        // Trigger refresh after successful pull
                        if let Some(tx) = bridge::get_repo_service_tx() {
                            self.as_mut().set_loading(true);
                            self.as_mut().rust_mut().op_state = OpState::BusyRefresh;
                            request_refresh(&tx);
                        }
                    }
                    Err(crate::services::RepoError::Cancelled) => {
                        // Silently handle cancellation - no error, no refresh
                        tracing::info!("Pull operation was cancelled");
                    }
                    Err(e) => {
                        self.as_mut().rust_mut().set_error(
                            myme_core::AppError::from(e.clone()).user_message().to_string(),
                        );
                        self.as_mut().error_occurred();
                    }
                }
            }
        }
    }

    pub fn clear_error(mut self: Pin<&mut Self>) {
        self.as_mut().rust_mut().clear_error_msg();
    }

    pub fn row_count(&self) -> i32 {
        self.rust().entries.len() as i32
    }

    pub fn get_full_name(&self, index: i32) -> QString {
        self.rust()
            .get_entry(index)
            .map(|e| QString::from(&e.full_name))
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_local_path(&self, index: i32) -> QString {
        self.rust()
            .get_entry(index)
            .and_then(|e| e.local.as_ref())
            .map(|l| l.path.to_string_lossy().into_owned())
            .map(QString::from)
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_branch(&self, index: i32) -> QString {
        self.rust()
            .get_entry(index)
            .and_then(|e| e.local.as_ref())
            .and_then(|l| l.current_branch.as_deref())
            .map(QString::from)
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_is_clean(&self, index: i32) -> bool {
        self.rust()
            .get_entry(index)
            .and_then(|e| e.local.as_ref())
            .map(|l| l.is_clean)
            .unwrap_or(true)
    }

    pub fn get_has_local(&self, index: i32) -> bool {
        self.rust().get_entry(index).map(|e| e.local.is_some()).unwrap_or(false)
    }

    pub fn get_has_github(&self, index: i32) -> bool {
        self.rust().get_entry(index).map(|e| e.github.is_some()).unwrap_or(false)
    }

    pub fn get_clone_url(&self, index: i32) -> QString {
        self.rust()
            .get_entry(index)
            .and_then(|e| e.github.as_ref())
            .and_then(|g| g.clone_url.as_deref())
            .map(QString::from)
            .unwrap_or_else(|| QString::from(""))
    }

    pub fn get_state(&self, index: i32) -> i32 {
        self.rust().get_entry(index).map(|e| e.state as i32).unwrap_or(0)
    }

    pub fn get_busy(&self, index: i32) -> bool {
        self.rust().get_entry(index).map(|e| e.busy).unwrap_or(false)
    }

    pub fn get_html_url(&self, index: i32) -> QString {
        self.rust()
            .get_entry(index)
            .and_then(|e| e.github.as_ref())
            .map(|g| &g.html_url)
            .map(QString::from)
            .unwrap_or_else(|| QString::from(""))
    }
}
