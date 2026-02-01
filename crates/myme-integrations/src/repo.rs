//! Repo matching: combine local and GitHub repos into unified RepoEntry list.

use std::collections::HashMap;

use myme_services::GitHubRepo;

use crate::git::LocalRepo;
use crate::repo_url;

/// Stable identifier for a repo (owner/repo or path for local-only).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum RepoState {
    LocalOnly = 0,
    GitHubOnly = 1,
    Both = 2,
}

#[derive(Debug, Clone)]
pub struct RepoEntry {
    pub id: RepoId,
    pub full_name: String,
    pub local: Option<LocalRepo>,
    pub github: Option<GitHubRepo>,
    pub state: RepoState,
    pub busy: bool,
}

/// Build owner/repo from LocalRepo's remote_url if it's a GitHub URL.
fn local_owner_repo(local: &LocalRepo) -> Option<String> {
    local
        .remote_url
        .as_deref()
        .and_then(repo_url::normalize_github_url)
}

/// Build owner/repo from GitHubRepo (clone_url or full_name).
fn github_owner_repo(gh: &GitHubRepo) -> String {
    gh.clone_url
        .as_deref()
        .and_then(repo_url::normalize_github_url)
        .unwrap_or_else(|| gh.full_name.clone())
}

/// Pure function: match local and remote repos into a unified `Vec<RepoEntry>`.
pub fn match_repos(local: &[LocalRepo], remote: &[GitHubRepo]) -> Vec<RepoEntry> {
    let mut by_owner_repo: HashMap<String, (Option<LocalRepo>, Option<GitHubRepo>)> =
        HashMap::new();

    for loc in local {
        let key = local_owner_repo(loc).unwrap_or_else(|| {
            // Local-only: use path as unique id
            loc.path.to_string_lossy().into_owned()
        });
        let entry = by_owner_repo.entry(key.clone()).or_default();
        if entry.0.is_none() {
            entry.0 = Some(loc.clone());
        }
    }

    for gh in remote {
        let key = github_owner_repo(gh);
        let entry = by_owner_repo.entry(key.clone()).or_default();
        if entry.1.is_none() {
            entry.1 = Some(gh.clone());
        }
    }

    let mut out = Vec::new();
    for (id_str, (loc, gh)) in by_owner_repo {
        let (state, full_name) = match (&loc, &gh) {
            (Some(_l), Some(g)) => (RepoState::Both, g.full_name.clone()),
            (Some(l), None) => (
                RepoState::LocalOnly,
                l.name.clone(), // local-only: use dir name
            ),
            (None, Some(g)) => (RepoState::GitHubOnly, g.full_name.clone()),
            (None, None) => continue,
        };
        let id = RepoId(id_str);
        out.push(RepoEntry {
            id,
            full_name,
            local: loc,
            github: gh,
            state,
            busy: false,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn local(remote_url: Option<&str>, name: &str, path: &str) -> LocalRepo {
        LocalRepo {
            path: PathBuf::from(path),
            name: name.to_string(),
            current_branch: Some("main".to_string()),
            is_clean: true,
            remote_url: remote_url.map(String::from),
            uncommitted_changes: 0,
            last_commit: None,
            last_commit_time: None,
        }
    }

    fn github(full_name: &str, clone_url: Option<&str>) -> GitHubRepo {
        GitHubRepo {
            id: 1,
            name: full_name.split('/').last().unwrap_or("").to_string(),
            full_name: full_name.to_string(),
            description: None,
            html_url: format!("https://github.com/{}", full_name),
            clone_url: clone_url.map(String::from),
            ssh_url: None,
            private: false,
            default_branch: "main".to_string(),
            open_issues_count: 0,
            updated_at: String::new(),
        }
    }

    #[test]
    fn test_local_only() {
        let local_repos = vec![local(None, "my-local", "/home/user/dev/my-local")];
        let remote: Vec<GitHubRepo> = vec![];
        let out = match_repos(&local_repos, &remote);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].state, RepoState::LocalOnly);
        assert_eq!(out[0].full_name, "my-local");
        assert!(out[0].local.is_some());
        assert!(out[0].github.is_none());
    }

    #[test]
    fn test_github_only() {
        let local: Vec<LocalRepo> = vec![];
        let remote = vec![github(
            "owner/repo",
            Some("https://github.com/owner/repo.git"),
        )];
        let out = match_repos(&local, &remote);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].state, RepoState::GitHubOnly);
        assert_eq!(out[0].full_name, "owner/repo");
        assert!(out[0].local.is_none());
        assert!(out[0].github.is_some());
    }

    #[test]
    fn test_both() {
        let local_repos = vec![local(
            Some("https://github.com/owner/repo.git"),
            "repo",
            "/home/user/dev/repo",
        )];
        let remote = vec![github(
            "owner/repo",
            Some("https://github.com/owner/repo.git"),
        )];
        let out = match_repos(&local_repos, &remote);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].state, RepoState::Both);
        assert_eq!(out[0].full_name, "owner/repo");
        assert!(out[0].local.is_some());
        assert!(out[0].github.is_some());
    }

    #[test]
    fn test_empty() {
        let out = match_repos(&[], &[]);
        assert!(out.is_empty());
    }
}
