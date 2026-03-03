pub mod git;
pub mod github;
pub mod northcloud;
pub mod repo;
pub mod repo_url;

pub use git::{GitOperations, LocalRepo};
pub use github::{GitHubClient, Issue, Repository};
pub use northcloud::{NorthCloudClient, RfpData, RfpHit, RfpSearchParams, RfpSearchResponse};
pub use repo::{match_repos, RepoEntry, RepoId, RepoState};
pub use repo_url::normalize_github_url;
