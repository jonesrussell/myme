//! Repo backend: discover local, fetch GitHub, match, clone, pull.
//! All heavy work runs off the UI thread; results sent via mpsc.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use myme_integrations::{match_repos, GitOperations, RepoEntry};

use crate::bridge;

#[derive(Debug, Clone)]
pub enum RepoError {
    Git(String),
    GitHub(String),
    Io(String),
    Config(String),
}

impl std::fmt::Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoError::Git(s) => write!(f, "Git: {}", s),
            RepoError::GitHub(s) => write!(f, "GitHub: {}", s),
            RepoError::Io(s) => write!(f, "IO: {}", s),
            RepoError::Config(s) => write!(f, "Config: {}", s),
        }
    }
}

#[derive(Debug)]
pub enum RepoServiceMessage {
    RefreshDone(Result<Vec<RepoEntry>, RepoError>),
    CloneDone {
        index: usize,
        result: Result<(), RepoError>,
    },
    PullDone {
        index: usize,
        result: Result<(), RepoError>,
    },
}

const GITHUB_CACHE_TTL_SECS: u64 = 60;
static GH_CACHE_TIME: AtomicU64 = AtomicU64::new(0);
static GH_CACHE: std::sync::OnceLock<std::sync::Mutex<Option<Vec<myme_services::GitHubRepo>>>> =
    std::sync::OnceLock::new();

fn github_cache() -> &'static std::sync::Mutex<Option<Vec<myme_services::GitHubRepo>>> {
    GH_CACHE.get_or_init(|| std::sync::Mutex::new(None))
}

fn use_cached_github() -> Option<Vec<myme_services::GitHubRepo>> {
    let cached_at = GH_CACHE_TIME.load(Ordering::Relaxed);
    if cached_at == 0 {
        return None;
    }
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now_secs.saturating_sub(cached_at) > GITHUB_CACHE_TTL_SECS {
        return None;
    }
    github_cache().lock().ok()?.clone()
}

fn set_github_cache(repos: Vec<myme_services::GitHubRepo>) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    GH_CACHE_TIME.store(now, Ordering::Relaxed);
    if let Ok(mut g) = github_cache().lock() {
        *g = Some(repos);
    }
}

/// Request a full refresh (discover local + fetch GitHub + match).
/// Sends `RefreshDone` on the channel when done.
pub fn request_refresh(tx: &std::sync::mpsc::Sender<RepoServiceMessage>) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(RepoServiceMessage::RefreshDone(Err(RepoError::Config(
                "Runtime not initialized".into(),
            ))));
            return;
        }
    };

    let (effective_path, _invalid) =
        bridge::get_repos_local_search_path().unwrap_or_else(|| (PathBuf::from("."), true));

    let github_client = bridge::get_github_client_and_runtime().map(|(c, _)| c);
    let authenticated = bridge::is_github_authenticated();

    runtime.spawn(async move {
        let local = tokio::task::spawn_blocking({
            let path = effective_path.clone();
            move || GitOperations::discover_repositories(&path, Some(5))
        })
        .await;

        let local = match local {
            Ok(Ok(repos)) => repos,
            Ok(Err(e)) => {
                let _ = tx.send(RepoServiceMessage::RefreshDone(Err(RepoError::Git(
                    e.to_string(),
                ))));
                return;
            }
            Err(e) => {
                let _ = tx.send(RepoServiceMessage::RefreshDone(Err(RepoError::Io(
                    e.to_string(),
                ))));
                return;
            }
        };

        let remote = if authenticated {
            if let Some(cached) = use_cached_github() {
                cached
            } else if let Some(client) = github_client {
                match client.list_repos().await {
                    Ok(repos) => {
                        set_github_cache(repos.clone());
                        repos
                    }
                    Err(e) => {
                        let _ = tx.send(RepoServiceMessage::RefreshDone(Err(RepoError::GitHub(
                            e.to_string(),
                        ))));
                        return;
                    }
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        let entries = match_repos(&local, &remote);
        let _ = tx.send(RepoServiceMessage::RefreshDone(Ok(entries)));
    });
}

/// Request clone for a GitHub-only repo. Sends `CloneDone { index, result }`, then
/// the pump should trigger a refresh.
pub fn request_clone(
    tx: &std::sync::mpsc::Sender<RepoServiceMessage>,
    index: usize,
    clone_url: String,
    target_path: PathBuf,
) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(RepoServiceMessage::CloneDone {
                index,
                result: Err(RepoError::Config("Runtime not initialized".into())),
            });
            return;
        }
    };

    runtime.spawn_blocking(move || {
        let result = GitOperations::clone_repository(&clone_url, &target_path)
            .map(|_| ())
            .map_err(|e| RepoError::Git(e.to_string()));
        let _ = tx.send(RepoServiceMessage::CloneDone { index, result });
    });
}

/// Request pull for a local repo. Sends `PullDone { index, result }`, then
/// the pump should trigger a refresh.
pub fn request_pull(tx: &std::sync::mpsc::Sender<RepoServiceMessage>, index: usize, path: PathBuf) {
    let tx = tx.clone();
    let runtime = match bridge::get_runtime() {
        Some(r) => r,
        None => {
            let _ = tx.send(RepoServiceMessage::PullDone {
                index,
                result: Err(RepoError::Config("Runtime not initialized".into())),
            });
            return;
        }
    };

    runtime.spawn_blocking(move || {
        let result = GitOperations::pull(&path).map_err(|e| RepoError::Git(e.to_string()));
        let _ = tx.send(RepoServiceMessage::PullDone { index, result });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_error_display() {
        assert!(format!("{}", RepoError::Git("x".into())).contains("Git"));
        assert!(format!("{}", RepoError::GitHub("y".into())).contains("GitHub"));
        assert!(format!("{}", RepoError::Io("i".into())).contains("IO"));
        assert!(format!("{}", RepoError::Config("z".into())).contains("Config"));
    }

    #[test]
    fn repo_service_message_variants() {
        // Verify we can construct and match all message variants
        let _refresh_ok: RepoServiceMessage = RepoServiceMessage::RefreshDone(Ok(vec![]));
        let _refresh_err: RepoServiceMessage =
            RepoServiceMessage::RefreshDone(Err(RepoError::Config("x".into())));
        let _clone: RepoServiceMessage = RepoServiceMessage::CloneDone {
            index: 0,
            result: Ok(()),
        };
        let _pull: RepoServiceMessage = RepoServiceMessage::PullDone {
            index: 1,
            result: Err(RepoError::Git("e".into())),
        };
    }
}
