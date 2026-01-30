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

    /// Fetch from remote (update refs only, no merge).
    ///
    /// # Arguments
    /// * `path` - Repository path
    pub fn fetch(path: &Path) -> Result<()> {
        let repo = Git2Repository::open(path)
            .context("Failed to open git repository")?;

        let head = repo.head().context("Failed to get HEAD reference")?;
        let branch_name = head.shorthand().context("Failed to get branch name")?;

        let mut remote = repo.find_remote("origin")
            .context("Failed to find remote 'origin'")?;

        remote.fetch(&[branch_name], None, None)
            .context("Failed to fetch from remote")?;

        tracing::info!("Fetched latest for {:?}", path);
        Ok(())
    }

    /// Pull latest changes (fetch + merge).
    ///
    /// # Arguments
    /// * `path` - Repository path
    pub fn pull(path: &Path) -> Result<()> {
        Self::fetch(path)?;

        let repo = Git2Repository::open(path)
            .context("Failed to open git repository")?;

        let fetch_head = repo.find_reference("FETCH_HEAD")
            .context("Failed to find FETCH_HEAD")?;
        let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)
            .context("Failed to resolve FETCH_HEAD")?;

        let (analysis, _) = repo.merge_analysis(&[&fetch_commit])
            .context("Failed to analyze merge")?;

        if analysis.is_up_to_date() {
            tracing::info!("Already up to date: {:?}", path);
            return Ok(());
        }

        if analysis.is_fast_forward() {
            let refname = format!("refs/heads/{}", repo.head().context("HEAD")?.shorthand().context("branch")?);
            let mut reference = repo.find_reference(&refname)
                .context("Failed to find branch ref")?;
            reference.set_target(fetch_commit.id(), "fast-forward")
                .context("Failed to update ref")?;
            repo.set_head(&refname).context("Failed to set HEAD")?;
            repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                .context("Failed to checkout")?;
            tracing::info!("Fast-forwarded {:?}", path);
            return Ok(());
        }

        if analysis.is_normal() {
            repo.merge(&[&fetch_commit], None, None)
                .context("Failed to merge")?;
            let mut index = repo.index().context("Failed to get index")?;
            if index.has_conflicts() {
                anyhow::bail!("Merge conflicts; resolve manually");
            }
            index.write().context("Failed to write index")?;
            let tree_oid = index.write_tree().context("Failed to write tree")?;
            let tree = repo.find_tree(tree_oid).context("Failed to find tree")?;
            let head_ref = repo.head().context("HEAD")?;
            let head_commit = repo.find_commit(head_ref.target().context("HEAD target")?)
                .context("Failed to find HEAD commit")?;
            let their_commit = repo.find_commit(fetch_commit.id())
                .context("Failed to find fetch commit")?;
            let sig = repo.signature().context("Failed to get signature")?;
            repo.commit(
                Some("HEAD"),
                &sig,
                &sig,
                "Merge",
                &tree,
                &[&head_commit, &their_commit],
            )
            .context("Failed to create merge commit")?;
            tracing::info!("Merged for {:?}", path);
            return Ok(());
        }

        anyhow::bail!("Merge not possible (e.g. unrelated histories)");
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
    use std::fs;
    use std::io::Write;

    #[test]
    fn test_git_operations_creation() {
        // Just verify the struct exists
        let _ops = GitOperations;
    }

    #[test]
    fn test_discover_repositories() {
        let dir = tempfile::tempdir().expect("temp dir");
        let base = dir.path();

        // Create repo1
        let repo1_path = base.join("repo1");
        fs::create_dir_all(&repo1_path).unwrap();
        let repo1 = git2::Repository::init(&repo1_path).unwrap();
        let _sig = repo1.signature().unwrap();
        let mut index = repo1.index().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo1.find_tree(tree_id).unwrap();
        repo1
            .commit(Some("HEAD"), &_sig, &_sig, "init", &tree, &[])
            .unwrap();

        // Create repo2
        let repo2_path = base.join("repo2");
        fs::create_dir_all(&repo2_path).unwrap();
        git2::Repository::init(&repo2_path).unwrap();

        // Create non-repo dir
        fs::create_dir_all(base.join("not-a-repo")).unwrap();

        // Create nested: repo3 inside a folder
        let nested = base.join("nested").join("repo3");
        fs::create_dir_all(&nested).unwrap();
        git2::Repository::init(&nested).unwrap();

        let repos = GitOperations::discover_repositories(base, Some(5)).unwrap();
        assert!(repos.len() >= 3, "expected at least 3 repos, got {}", repos.len());
        let names: Vec<_> = repos.iter().map(|r| r.name.as_str()).collect();
        assert!(names.contains(&"repo1"));
        assert!(names.contains(&"repo2"));
        assert!(names.contains(&"repo3"));
    }

    #[test]
    fn test_clone_from_local() {
        // Create a "remote" repo
        let remote_dir = tempfile::tempdir().expect("remote temp dir");
        let remote_path = remote_dir.path();
        let repo = git2::Repository::init(remote_path).unwrap();
        let sig = repo.signature().unwrap();

        // Add and commit a file
        let readme = remote_path.join("README");
        fs::File::create(&readme)
            .unwrap()
            .write_all(b"hello")
            .unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("README")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        // Clone to target (file:// URL on Windows needs 3 slashes)
        let target_dir = tempfile::tempdir().expect("target temp dir");
        let target_path = target_dir.path().join("cloned");
        let url = format!("file:///{}", remote_path.display().to_string().replace('\\', "/"));
        let result = GitOperations::clone_repository(&url, &target_path);
        assert!(result.is_ok(), "clone failed: {:?}", result.err());
        let cloned = result.unwrap();
        assert_eq!(cloned.name, "cloned");
        assert!(target_path.join("README").exists());
    }

    #[test]
    fn test_fetch_and_pull() {
        // Create remote repo with a commit
        let remote_dir = tempfile::tempdir().expect("remote");
        let remote_path = remote_dir.path();
        let remote_repo = git2::Repository::init(remote_path).unwrap();
        let sig = remote_repo.signature().unwrap();
        let readme = remote_path.join("file.txt");
        fs::File::create(&readme).unwrap().write_all(b"v1").unwrap();
        let mut index = remote_repo.index().unwrap();
        index.add_path(std::path::Path::new("file.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = remote_repo.find_tree(tree_id).unwrap();
        remote_repo
            .commit(Some("HEAD"), &sig, &sig, "v1", &tree, &[])
            .unwrap();

        // Clone it
        let target_dir = tempfile::tempdir().expect("target");
        let target_path = target_dir.path().join("clone");
        let url = format!("file:///{}", remote_path.display().to_string().replace('\\', "/"));
        GitOperations::clone_repository(&url, &target_path).unwrap();

        // Add another commit to remote
        fs::File::create(&readme).unwrap().write_all(b"v2").unwrap();
        let mut index = remote_repo.index().unwrap();
        index.add_path(std::path::Path::new("file.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = remote_repo.find_tree(tree_id).unwrap();
        let head = remote_repo.head().unwrap().target().unwrap();
        let parent = remote_repo.find_commit(head).unwrap();
        remote_repo
            .commit(Some("HEAD"), &sig, &sig, "v2", &tree, &[&parent])
            .unwrap();

        // Fetch and pull in clone
        let pull_result = GitOperations::pull(&target_path);
        assert!(pull_result.is_ok(), "pull failed: {:?}", pull_result.err());
        let content = fs::read_to_string(target_path.join("file.txt")).unwrap();
        assert_eq!(content, "v2");
    }
}
