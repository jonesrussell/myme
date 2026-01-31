//! Integration tests for GitHubClient.
//!
//! Note: The GitHubClient currently hardcodes the GitHub API URL,
//! so we can't easily use wiremock without modifying the implementation.
//! These tests verify the client creation and response parsing work correctly.

use myme_services::{GitHubClient, CreateIssueRequest, UpdateIssueRequest};

/// Helper to create a test repo JSON
fn test_repo(id: i64, name: &str, full_name: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "full_name": full_name,
        "description": "Test repo description",
        "html_url": format!("https://github.com/{}", full_name),
        "clone_url": format!("https://github.com/{}.git", full_name),
        "ssh_url": format!("git@github.com:{}.git", full_name),
        "private": false,
        "default_branch": "main",
        "open_issues_count": 5,
        "updated_at": "2026-01-30T12:00:00Z"
    })
}

/// Helper to create a test issue JSON
fn test_issue(id: i64, number: i32, title: &str, state: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "number": number,
        "title": title,
        "body": "Test issue body",
        "state": state,
        "html_url": format!("https://github.com/test/repo/issues/{}", number),
        "labels": [],
        "created_at": "2026-01-30T12:00:00Z",
        "updated_at": "2026-01-30T12:00:00Z"
    })
}

/// Helper to create a test label JSON
fn test_label(id: i64, name: &str, color: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "name": name,
        "color": color
    })
}

#[tokio::test]
async fn test_github_client_creation() {
    let client = GitHubClient::new("test-token".to_string());
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_github_repo_deserialization() {
    let repo_json = test_repo(123, "my-repo", "user/my-repo");
    let repo: myme_services::GitHubRepo = serde_json::from_value(repo_json).unwrap();

    assert_eq!(repo.id, 123);
    assert_eq!(repo.name, "my-repo");
    assert_eq!(repo.full_name, "user/my-repo");
    assert!(!repo.private);
}

#[tokio::test]
async fn test_github_issue_deserialization() {
    let issue_json = test_issue(456, 42, "Test Issue", "open");
    let issue: myme_services::GitHubIssue = serde_json::from_value(issue_json).unwrap();

    assert_eq!(issue.id, 456);
    assert_eq!(issue.number, 42);
    assert_eq!(issue.title, "Test Issue");
    assert_eq!(issue.state, "open");
}

#[tokio::test]
async fn test_github_label_deserialization() {
    let label_json = test_label(789, "bug", "ff0000");
    let label: myme_services::GitHubLabel = serde_json::from_value(label_json).unwrap();

    assert_eq!(label.id, 789);
    assert_eq!(label.name, "bug");
    assert_eq!(label.color, "ff0000");
}

#[tokio::test]
async fn test_create_issue_request_serialization() {
    let req = CreateIssueRequest {
        title: "New Bug".to_string(),
        body: Some("Description here".to_string()),
        labels: Some(vec!["bug".to_string(), "high-priority".to_string()]),
    };

    let json = serde_json::to_value(&req).unwrap();

    assert_eq!(json["title"], "New Bug");
    assert_eq!(json["body"], "Description here");
    assert_eq!(json["labels"][0], "bug");
}

#[tokio::test]
async fn test_update_issue_request_partial() {
    // Only state field set
    let req = UpdateIssueRequest {
        title: None,
        body: None,
        state: Some("closed".to_string()),
        labels: None,
    };

    let json = serde_json::to_string(&req).unwrap();

    // Should only contain state, not null fields
    assert!(json.contains("closed"));
    assert!(!json.contains("title"));
    assert!(!json.contains("body"));
}

#[tokio::test]
async fn test_issue_with_labels_deserialization() {
    let issue_json = serde_json::json!({
        "id": 100,
        "number": 10,
        "title": "Issue with labels",
        "body": null,
        "state": "open",
        "html_url": "https://github.com/test/repo/issues/10",
        "labels": [
            {"id": 1, "name": "bug", "color": "ff0000"},
            {"id": 2, "name": "urgent", "color": "ffff00"}
        ],
        "created_at": "2026-01-30T12:00:00Z",
        "updated_at": "2026-01-30T12:00:00Z"
    });

    let issue: myme_services::GitHubIssue = serde_json::from_value(issue_json).unwrap();

    assert_eq!(issue.labels.len(), 2);
    assert_eq!(issue.labels[0].name, "bug");
    assert_eq!(issue.labels[1].name, "urgent");
}

#[tokio::test]
async fn test_repo_optional_fields() {
    // Repo without clone_url and ssh_url (defaults should work)
    let repo_json = serde_json::json!({
        "id": 999,
        "name": "minimal-repo",
        "full_name": "user/minimal-repo",
        "description": null,
        "html_url": "https://github.com/user/minimal-repo",
        "private": true,
        "default_branch": "main",
        "updated_at": "2026-01-30T12:00:00Z"
    });

    let repo: myme_services::GitHubRepo = serde_json::from_value(repo_json).unwrap();

    assert_eq!(repo.id, 999);
    assert!(repo.clone_url.is_none());
    assert!(repo.ssh_url.is_none());
    assert!(repo.private);
    assert_eq!(repo.open_issues_count, 0); // default
}
