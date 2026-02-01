// crates/myme-services/src/project_store.rs

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use crate::project::{Project, Task, TaskStatus};

/// Local SQLite storage for projects and tasks
pub struct ProjectStore {
    conn: Connection,
}

impl ProjectStore {
    /// Open or create the database
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open projects database")?;

        let store = Self { conn };
        store.init_schema()?;

        Ok(store)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                github_repo TEXT NOT NULL UNIQUE,
                description TEXT,
                created_at TEXT NOT NULL,
                last_synced TEXT
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                github_issue_number INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                status TEXT NOT NULL,
                labels TEXT NOT NULL,
                html_url TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id),
                UNIQUE (project_id, github_issue_number)
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);",
            )
            .context("Failed to initialize schema")?;

        Ok(())
    }

    /// Insert or update a project
    pub fn upsert_project(&self, project: &Project) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (id, github_repo, description, created_at, last_synced)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
                github_repo = excluded.github_repo,
                description = excluded.description,
                last_synced = excluded.last_synced",
            params![
                project.id,
                project.github_repo,
                project.description,
                project.created_at,
                project.last_synced,
            ],
        )?;
        Ok(())
    }

    /// Get all projects
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, github_repo, description, created_at, last_synced
             FROM projects ORDER BY last_synced DESC NULLS LAST",
        )?;

        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    github_repo: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    last_synced: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Get a project by ID
    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, github_repo, description, created_at, last_synced
             FROM projects WHERE id = ?1",
        )?;

        let project = stmt
            .query_row([id], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    github_repo: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    last_synced: row.get(4)?,
                })
            })
            .optional()?;

        Ok(project)
    }

    /// Delete a project and its tasks
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM tasks WHERE project_id = ?1", [id])?;
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Insert or update a task
    pub fn upsert_task(&self, task: &Task) -> Result<()> {
        let labels_json = serde_json::to_string(&task.labels)?;
        let status_str = serde_json::to_string(&task.status)?;

        self.conn.execute(
            "INSERT INTO tasks (id, project_id, github_issue_number, title, body, status, labels, html_url, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(project_id, github_issue_number) DO UPDATE SET
                title = excluded.title,
                body = excluded.body,
                status = excluded.status,
                labels = excluded.labels,
                html_url = excluded.html_url,
                updated_at = excluded.updated_at",
            params![
                task.id,
                task.project_id,
                task.github_issue_number,
                task.title,
                task.body,
                status_str,
                labels_json,
                task.html_url,
                task.created_at,
                task.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Get tasks for a project
    pub fn list_tasks(&self, project_id: &str) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, github_issue_number, title, body, status, labels, html_url, created_at, updated_at
             FROM tasks WHERE project_id = ?1 ORDER BY github_issue_number"
        )?;

        let tasks = stmt
            .query_map([project_id], |row| {
                let status_str: String = row.get(5)?;
                let labels_json: String = row.get(6)?;

                Ok(Task {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    github_issue_number: row.get(2)?,
                    title: row.get(3)?,
                    body: row.get(4)?,
                    status: serde_json::from_str(&status_str).unwrap_or(TaskStatus::Todo),
                    labels: serde_json::from_str(&labels_json).unwrap_or_default(),
                    html_url: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// Delete a task
    pub fn delete_task(&self, project_id: &str, issue_number: i32) -> Result<()> {
        self.conn.execute(
            "DELETE FROM tasks WHERE project_id = ?1 AND github_issue_number = ?2",
            params![project_id, issue_number],
        )?;
        Ok(())
    }

    /// Count tasks by status for a project
    pub fn count_tasks_by_status(&self, project_id: &str) -> Result<Vec<(TaskStatus, i32)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT status, COUNT(*) FROM tasks WHERE project_id = ?1 GROUP BY status")?;

        let counts = stmt
            .query_map([project_id], |row| {
                let status_str: String = row.get(0)?;
                let count: i32 = row.get(1)?;
                let status = serde_json::from_str(&status_str).unwrap_or(TaskStatus::Todo);
                Ok((status, count))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_list_project() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = ProjectStore::open(&db_path).unwrap();

        let project = Project {
            id: "test-123".to_string(),
            github_repo: "user/repo".to_string(),
            description: Some("Test project".to_string()),
            created_at: "2026-01-21T00:00:00Z".to_string(),
            last_synced: None,
        };

        store.upsert_project(&project).unwrap();

        let projects = store.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].github_repo, "user/repo");
    }

    #[test]
    fn test_create_and_list_tasks() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = ProjectStore::open(&db_path).unwrap();

        let project = Project {
            id: "proj-1".to_string(),
            github_repo: "user/repo".to_string(),
            description: None,
            created_at: "2026-01-21T00:00:00Z".to_string(),
            last_synced: None,
        };
        store.upsert_project(&project).unwrap();

        let task = Task {
            id: "task-1".to_string(),
            project_id: "proj-1".to_string(),
            github_issue_number: 42,
            title: "Test task".to_string(),
            body: Some("Description".to_string()),
            status: TaskStatus::InProgress,
            labels: vec!["in-progress".to_string()],
            html_url: "https://github.com/user/repo/issues/42".to_string(),
            created_at: "2026-01-21T00:00:00Z".to_string(),
            updated_at: "2026-01-21T00:00:00Z".to_string(),
        };
        store.upsert_task(&task).unwrap();

        let tasks = store.list_tasks("proj-1").unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::InProgress);
    }
}
