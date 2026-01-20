pub mod github;
pub mod git;

pub use github::{GitHubClient, Repository, Issue};
pub use git::{LocalRepo, GitOperations};
