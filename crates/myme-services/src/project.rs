// crates/myme-services/src/project.rs

use serde::{Deserialize, Serialize};

/// Task status in kanban board
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Backlog,
    Todo,
    InProgress,
    Blocked,
    Review,
    Done,
}

impl TaskStatus {
    /// Get the GitHub label name for this status
    pub fn to_label(&self) -> Option<&'static str> {
        match self {
            TaskStatus::Backlog => Some("backlog"),
            TaskStatus::Todo => Some("todo"),
            TaskStatus::InProgress => Some("in-progress"),
            TaskStatus::Blocked => Some("blocked"),
            TaskStatus::Review => Some("review"),
            TaskStatus::Done => None, // Done = closed, no label needed
        }
    }

    /// Get the GitHub label color for this status
    pub fn label_color(&self) -> &'static str {
        match self {
            TaskStatus::Backlog => "e0e0e0",
            TaskStatus::Todo => "0366d6",
            TaskStatus::InProgress => "fbca04",
            TaskStatus::Blocked => "d93f0b",
            TaskStatus::Review => "6f42c1",
            TaskStatus::Done => "0e8a16",
        }
    }

    /// Parse status from GitHub issue state and labels
    pub fn from_github(state: &str, labels: &[String]) -> Self {
        if state == "closed" {
            return TaskStatus::Done;
        }

        // Check labels in priority order (blocked > review > in-progress > backlog > todo)
        // We check each priority level across all labels, not iterate labels in order
        let label_strs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

        if label_strs.contains(&"blocked") {
            return TaskStatus::Blocked;
        }
        if label_strs.contains(&"review") {
            return TaskStatus::Review;
        }
        if label_strs.contains(&"in-progress") {
            return TaskStatus::InProgress;
        }
        if label_strs.contains(&"backlog") {
            return TaskStatus::Backlog;
        }
        if label_strs.contains(&"todo") {
            return TaskStatus::Todo;
        }

        // Default for open issues with no status label
        TaskStatus::Todo
    }

    /// Get all status variants
    pub fn all() -> &'static [TaskStatus] {
        &[
            TaskStatus::Backlog,
            TaskStatus::Todo,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Review,
            TaskStatus::Done,
        ]
    }
}

/// Local project representation (first-class entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
}

/// Junction for many-to-many project <-> repo association
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRepo {
    pub project_id: String,
    pub repo_id: String,
}

/// Local task representation (first-class, belongs to project)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub body: Option<String>,
    pub status: TaskStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_status_from_closed_issue() {
        let status = TaskStatus::from_github("closed", &[]);
        assert_eq!(status, TaskStatus::Done);
    }

    #[test]
    fn test_status_from_labels() {
        let status = TaskStatus::from_github("open", &["in-progress".to_string()]);
        assert_eq!(status, TaskStatus::InProgress);
    }

    #[test]
    fn test_status_blocked_priority() {
        // Blocked should take priority over other labels
        let status = TaskStatus::from_github("open", &["todo".to_string(), "blocked".to_string()]);
        assert_eq!(status, TaskStatus::Blocked);
    }

    #[test]
    fn test_status_default_to_todo() {
        let status = TaskStatus::from_github("open", &["bug".to_string()]);
        assert_eq!(status, TaskStatus::Todo);
    }

    #[test]
    fn test_status_to_label() {
        assert_eq!(TaskStatus::InProgress.to_label(), Some("in-progress"));
        assert_eq!(TaskStatus::Done.to_label(), None);
    }
}
