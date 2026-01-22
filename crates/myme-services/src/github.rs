// crates/myme-services/src/github.rs

use serde::{Deserialize, Serialize};

/// GitHub repository representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
    pub private: bool,
    pub default_branch: String,
    #[serde(default)]
    pub open_issues_count: i32,
    pub updated_at: String,
}

/// GitHub issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: i64,
    pub number: i32,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub html_url: String,
    pub labels: Vec<GitHubLabel>,
    pub created_at: String,
    pub updated_at: String,
}

/// GitHub label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubLabel {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// Request to create a new repo
#[derive(Debug, Serialize)]
pub struct CreateRepoRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub private: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_init: Option<bool>,
}

/// Request to create a new issue
#[derive(Debug, Serialize)]
pub struct CreateIssueRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

/// Request to update an issue
#[derive(Debug, Serialize)]
pub struct UpdateIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

/// Request to create a label
#[derive(Debug, Serialize)]
pub struct CreateLabelRequest {
    pub name: String,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_deserialization() {
        let json = r#"{
            "id": 123,
            "name": "test-repo",
            "full_name": "user/test-repo",
            "description": "A test repo",
            "html_url": "https://github.com/user/test-repo",
            "private": false,
            "default_branch": "main",
            "open_issues_count": 5,
            "updated_at": "2026-01-21T00:00:00Z"
        }"#;
        let repo: GitHubRepo = serde_json::from_str(json).unwrap();
        assert_eq!(repo.name, "test-repo");
        assert_eq!(repo.open_issues_count, 5);
    }

    #[test]
    fn test_issue_deserialization() {
        let json = r#"{
            "id": 456,
            "number": 42,
            "title": "Test issue",
            "body": "Issue body",
            "state": "open",
            "html_url": "https://github.com/user/repo/issues/42",
            "labels": [{"id": 1, "name": "bug", "color": "ff0000"}],
            "created_at": "2026-01-21T00:00:00Z",
            "updated_at": "2026-01-21T00:00:00Z"
        }"#;
        let issue: GitHubIssue = serde_json::from_str(json).unwrap();
        assert_eq!(issue.number, 42);
        assert_eq!(issue.labels.len(), 1);
    }

    #[test]
    fn test_create_issue_serialization() {
        let req = CreateIssueRequest {
            title: "New issue".to_string(),
            body: Some("Description".to_string()),
            labels: Some(vec!["todo".to_string()]),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("New issue"));
        assert!(json.contains("todo"));
    }

    #[test]
    fn test_update_issue_skips_none() {
        let req = UpdateIssueRequest {
            title: None,
            body: None,
            state: Some("closed".to_string()),
            labels: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("title"));
        assert!(json.contains("closed"));
    }
}
