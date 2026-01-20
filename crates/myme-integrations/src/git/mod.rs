use anyhow::{Context, Result};
use git2::{Repository as Git2Repository, Status, StatusOptions};
use std::path::{Path, PathBuf};

/// Local git repository information
#[derive(Debug, Clone)]
pub struct LocalRepo {
    /// Repository path
    pub path: PathBuf,

    /// Repository name (directory name)
    pub name: String,

    /// Current branch name
    pub current_branch: Option<String>,

    /// Is working directory clean
    pub is_clean: bool,

    /// Remote URL (origin)
    pub remote_url: Option<String>,

    /// Number of uncommitted changes
    pub uncommitted_changes: usize,

    /// Last commit message
    pub last_commit: Option<String>,

    /// Last commit time
    pub last_commit_time: Option<String>,
}

/// Git operations handler
pub struct GitOperations;

impl GitOperations {
    /// Discover git repositories in a directory
    ///
    /// # Arguments
    /// * `base_path` - Base directory to search for repositories
    /// * `max_depth` - Maximum directory depth to search (default: 3)
    pub fn discover_repositories(base_path: &Path, max_depth: Option<usize>) -> Result<Vec<LocalRepo>> {
        let max_depth = max_depth.unwrap_or(3);
        let mut repos = Vec::new();

        Self::walk_directory(base_path, &mut repos, 0, max_depth)?;

        tracing::info!("Discovered {} git repositories in {:?}", repos.len(), base_path);
        Ok(repos)
    }

    /// Walk directory recursively to find git repositories
    fn walk_directory(
        path: &Path,
        repos: &mut Vec<LocalRepo>,
        current_depth: usize,
        max_depth: usize,
    ) -> Result<()> {
        if current_depth > max_depth {
            return Ok(());
        }

        // Check if this directory is a git repository
        if let Ok(repo_info) = Self::get_repository_info(path) {
            repos.push(repo_info);
            return Ok(()); // Don't recurse into subdirectories of a git repo
        }

        // Walk subdirectories
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        let subpath = entry.path();
                        // Skip hidden directories except .git
                        if let Some(name) = subpath.file_name() {
                            if let Some(name_str) = name.to_str() {
                                if !name_str.starts_with('.') || name_str == ".git" {
                                    let _ = Self::walk_directory(&subpath, repos, current_depth + 1, max_depth);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get detailed information about a git repository
    ///
    /// # Arguments
    /// * `path` - Path to the repository
    pub fn get_repository_info(path: &Path) -> Result<LocalRepo> {
        let repo = Git2Repository::open(path)
            .context("Failed to open git repository")?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Get current branch
        let current_branch = repo.head()
            .ok()
            .and_then(|head| head.shorthand().map(|s| s.to_string()));

        // Get remote URL
        let remote_url = repo.find_remote("origin")
            .ok()
            .and_then(|remote| remote.url().map(|s| s.to_string()));

        // Check working directory status
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.recurse_untracked_dirs(true);

        let statuses = repo.statuses(Some(&mut status_opts))
            .context("Failed to get repository status")?;

        let uncommitted_changes = statuses.len();
        let is_clean = uncommitted_changes == 0;

        // Get last commit info
        let (last_commit, last_commit_time) = if let Ok(head) = repo.head() {
            if let Some(target) = head.target() {
                if let Ok(commit) = repo.find_commit(target) {
                    let message = commit.message().unwrap_or("").to_string();
                    let time = commit.time();
                    let datetime = chrono::DateTime::from_timestamp(
                        time.seconds(),
                        0,
                    ).map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string());

                    (Some(message), datetime)
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        Ok(LocalRepo {
            path: path.to_path_buf(),
            name,
            current_branch,
            is_clean,
            remote_url,
            uncommitted_changes,
            last_commit,
            last_commit_time,
        })
    }

    /// Clone a repository
    ///
    /// # Arguments
    /// * `url` - Repository URL to clone
    /// * `target_path` - Target directory for cloning
    pub fn clone_repository(url: &str, target_path: &Path) -> Result<LocalRepo> {
        tracing::info!("Cloning repository from {} to {:?}", url, target_path);

        let _repo = Git2Repository::clone(url, target_path)
            .context("Failed to clone repository")?;

        tracing::info!("Successfully cloned repository");

        Self::get_repository_info(target_path)
    }

    /// Pull latest changes from remote
    ///
    /// # Arguments
    /// * `path` - Repository path
    pub fn pull(path: &Path) -> Result<()> {
        let repo = Git2Repository::open(path)
            .context("Failed to open git repository")?;

        // Get the current branch
        let head = repo.head()
            .context("Failed to get HEAD reference")?;

        let branch_name = head.shorthand()
            .context("Failed to get branch name")?;

        // Fetch from origin
        let mut remote = repo.find_remote("origin")
            .context("Failed to find remote 'origin'")?;

        remote.fetch(&[branch_name], None, None)
            .context("Failed to fetch from remote")?;

        tracing::info!("Pulled latest changes for {:?}", path);
        Ok(())
    }

    /// Push changes to remote
    ///
    /// # Arguments
    /// * `path` - Repository path
    pub fn push(path: &Path) -> Result<()> {
        let repo = Git2Repository::open(path)
            .context("Failed to open git repository")?;

        // Get the current branch
        let head = repo.head()
            .context("Failed to get HEAD reference")?;

        let branch_name = head.shorthand()
            .context("Failed to get branch name")?;

        // Push to origin
        let mut remote = repo.find_remote("origin")
            .context("Failed to find remote 'origin'")?;

        remote.push(&[format!("refs/heads/{}", branch_name)], None)
            .context("Failed to push to remote")?;

        tracing::info!("Pushed changes for {:?}", path);
        Ok(())
    }

    /// Get list of uncommitted files
    ///
    /// # Arguments
    /// * `path` - Repository path
    pub fn get_uncommitted_files(path: &Path) -> Result<Vec<(String, String)>> {
        let repo = Git2Repository::open(path)
            .context("Failed to open git repository")?;

        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);
        status_opts.recurse_untracked_dirs(true);

        let statuses = repo.statuses(Some(&mut status_opts))
            .context("Failed to get repository status")?;

        let mut files = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = match entry.status() {
                s if s.contains(Status::WT_NEW) => "untracked",
                s if s.contains(Status::WT_MODIFIED) => "modified",
                s if s.contains(Status::WT_DELETED) => "deleted",
                s if s.contains(Status::INDEX_NEW) => "staged_new",
                s if s.contains(Status::INDEX_MODIFIED) => "staged_modified",
                s if s.contains(Status::INDEX_DELETED) => "staged_deleted",
                _ => "unknown",
            };

            files.push((path, status.to_string()));
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_operations_creation() {
        // Just verify the struct exists
        let _ops = GitOperations;
    }
}
