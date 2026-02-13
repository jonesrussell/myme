// crates/myme-services/src/project_store.rs

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use crate::project::{Project, Task, TaskStatus};

const SCHEMA_VERSION: i32 = 3;

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

    /// Initialize database schema and run migrations if needed
    fn init_schema(&self) -> Result<()> {
        // Create schema version table
        self.conn
            .execute("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL)", [])?;

        let version: i32 = self
            .conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| row.get(0))
            .optional()?
            .unwrap_or(0);

        if version < 2 {
            self.migrate_to_v2(version)?;
        }
        if version < SCHEMA_VERSION {
            self.migrate_to_v3()?;
        }

        // Ensure schema exists
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS project_repos (
                project_id TEXT NOT NULL,
                repo_id TEXT NOT NULL,
                PRIMARY KEY (project_id, repo_id),
                FOREIGN KEY (project_id) REFERENCES projects(id)
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id)
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
            CREATE INDEX IF NOT EXISTS idx_project_repos_project ON project_repos(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_repos_repo ON project_repos(repo_id);",
            )
            .context("Failed to initialize schema")?;

        Ok(())
    }

    /// Migrate from v1 (github_repo per project) to v2 (many-to-many)
    fn migrate_to_v2(&self, _from_version: i32) -> Result<()> {
        // Check if old schema exists
        let old_exists: bool = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='projects'",
            [],
            |row| row.get::<_, i32>(0),
        )? > 0;

        if !old_exists {
            self.conn.execute("DELETE FROM schema_version", [])?;
            self.conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
            return Ok(());
        }

        // Check if old schema has github_repo column
        let table_info: Vec<String> = self
            .conn
            .prepare("PRAGMA table_info(projects)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;

        let has_github_repo = table_info.iter().any(|name| name == "github_repo");

        if !has_github_repo {
            // Already v2
            self.conn.execute("DELETE FROM schema_version", [])?;
            self.conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
            return Ok(());
        }

        // Migrate: copy old projects -> new projects (name = github_repo), insert project_repos
        // Copy old tasks -> new tasks (repo_id from project.github_repo)
        self.conn.execute_batch("BEGIN TRANSACTION;

            -- Create new tables if not exist (idempotent)
            CREATE TABLE IF NOT EXISTS projects_new (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS project_repos_new (
                project_id TEXT NOT NULL,
                repo_id TEXT NOT NULL,
                PRIMARY KEY (project_id, repo_id)
            );
            CREATE TABLE IF NOT EXISTS tasks_new (
                id TEXT PRIMARY KEY,
                repo_id TEXT NOT NULL,
                github_issue_number INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                status TEXT NOT NULL,
                labels TEXT NOT NULL,
                html_url TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                UNIQUE (repo_id, github_issue_number)
            );

            -- Migrate projects: name = github_repo
            INSERT OR REPLACE INTO projects_new (id, name, description, created_at)
            SELECT id, github_repo, description, created_at FROM projects;

            -- Migrate project_repos
            INSERT OR IGNORE INTO project_repos_new (project_id, repo_id)
            SELECT id, github_repo FROM projects;

            -- Migrate tasks: repo_id from project
            INSERT OR REPLACE INTO tasks_new (id, repo_id, github_issue_number, title, body, status, labels, html_url, created_at, updated_at)
            SELECT t.id, p.github_repo, t.github_issue_number, t.title, t.body, t.status, t.labels, t.html_url, t.created_at, t.updated_at
            FROM tasks t JOIN projects p ON t.project_id = p.id;

            -- Drop old tables
            DROP TABLE IF EXISTS tasks;
            DROP TABLE IF EXISTS projects;

            -- Rename new tables
            ALTER TABLE projects_new RENAME TO projects;
            ALTER TABLE project_repos_new RENAME TO project_repos;
            ALTER TABLE tasks_new RENAME TO tasks;

            -- Create indexes
            CREATE INDEX IF NOT EXISTS idx_tasks_repo ON tasks(repo_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
            CREATE INDEX IF NOT EXISTS idx_project_repos_project ON project_repos(project_id);
            CREATE INDEX IF NOT EXISTS idx_project_repos_repo ON project_repos(repo_id);

            COMMIT;")?;

        self.conn.execute("DELETE FROM schema_version", [])?;
        self.conn
            .execute("INSERT INTO schema_version (version) VALUES (?1)", params![SCHEMA_VERSION])?;

        Ok(())
    }

    /// Migrate from v2 (repo-based tasks) to v3 (project-based tasks)
    fn migrate_to_v3(&self) -> Result<()> {
        let version: i32 = self
            .conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| row.get(0))
            .optional()?
            .unwrap_or(2);

        if version >= SCHEMA_VERSION {
            return Ok(());
        }

        let has_tasks: bool = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tasks'",
            [],
            |row| row.get::<_, i32>(0),
        )? > 0;

        if !has_tasks {
            self.conn.execute("DELETE FROM schema_version", [])?;
            self.conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
            return Ok(());
        }

        let table_info: Vec<String> = self
            .conn
            .prepare("PRAGMA table_info(tasks)")?
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<Result<Vec<_>, _>>()?;

        let has_project_id = table_info.iter().any(|c| c == "project_id");
        if has_project_id {
            self.conn.execute("DELETE FROM schema_version", [])?;
            self.conn.execute(
                "INSERT INTO schema_version (version) VALUES (?1)",
                params![SCHEMA_VERSION],
            )?;
            return Ok(());
        }

        self.conn.execute_batch(
            "BEGIN TRANSACTION;

            CREATE TABLE IF NOT EXISTS tasks_new (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES projects(id)
            );

            -- Migrate: for each task with repo_id, find first project that has that repo
            INSERT INTO tasks_new (id, project_id, title, body, status, created_at, updated_at)
            SELECT t.id,
                   (SELECT pr.project_id FROM project_repos pr WHERE pr.repo_id = t.repo_id LIMIT 1),
                   t.title,
                   t.body,
                   t.status,
                   t.created_at,
                   t.updated_at
            FROM tasks t
            WHERE EXISTS (SELECT 1 FROM project_repos pr WHERE pr.repo_id = t.repo_id);

            DROP TABLE tasks;
            ALTER TABLE tasks_new RENAME TO tasks;

            CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
            CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);

            COMMIT;"
        )?;

        self.conn.execute("DELETE FROM schema_version", [])?;
        self.conn
            .execute("INSERT INTO schema_version (version) VALUES (?1)", params![SCHEMA_VERSION])?;

        Ok(())
    }

    /// Insert or update a project
    pub fn upsert_project(&self, project: &Project) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (id, name, description, created_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description",
            params![project.id, project.name, project.description, project.created_at,],
        )?;
        Ok(())
    }

    /// Get all projects
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at
             FROM projects ORDER BY created_at DESC",
        )?;

        let projects = stmt
            .query_map([], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Get a project by ID
    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, created_at
             FROM projects WHERE id = ?1",
        )?;

        let project = stmt
            .query_row([id], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })
            .optional()?;

        Ok(project)
    }

    /// Delete a project, its project_repos links, and its tasks
    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE project_id = ?1", [id])?;
        self.conn.execute("DELETE FROM project_repos WHERE project_id = ?1", [id])?;
        self.conn.execute("DELETE FROM projects WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Add a repo to a project
    pub fn add_repo_to_project(&self, project_id: &str, repo_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO project_repos (project_id, repo_id) VALUES (?1, ?2)",
            params![project_id, repo_id],
        )?;
        Ok(())
    }

    /// Remove a repo from a project
    pub fn remove_repo_from_project(&self, project_id: &str, repo_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM project_repos WHERE project_id = ?1 AND repo_id = ?2",
            params![project_id, repo_id],
        )?;
        Ok(())
    }

    /// List repos associated with a project
    pub fn list_repos_for_project(&self, project_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT repo_id FROM project_repos WHERE project_id = ?1 ORDER BY repo_id")?;

        let repos =
            stmt.query_map([project_id], |row| row.get(0))?.collect::<Result<Vec<_>, _>>()?;
        Ok(repos)
    }

    /// List all distinct repo_ids linked to any project (owner/repo format)
    pub fn list_all_linked_repo_ids(&self) -> Result<Vec<String>> {
        let mut stmt =
            self.conn.prepare("SELECT DISTINCT repo_id FROM project_repos ORDER BY repo_id")?;

        let repos = stmt.query_map([], |row| row.get(0))?.collect::<Result<Vec<_>, _>>()?;
        Ok(repos)
    }

    /// List projects that contain a repo
    pub fn list_projects_for_repo(&self, repo_id: &str) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.name, p.description, p.created_at
             FROM projects p
             JOIN project_repos pr ON p.id = pr.project_id
             WHERE pr.repo_id = ?1
             ORDER BY p.created_at DESC",
        )?;

        let projects = stmt
            .query_map([repo_id], |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Insert or update a task
    pub fn upsert_task(&self, task: &Task) -> Result<()> {
        let status_str = serde_json::to_string(&task.status)?;

        self.conn.execute(
            "INSERT INTO tasks (id, project_id, title, body, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                body = excluded.body,
                status = excluded.status,
                updated_at = excluded.updated_at",
            params![
                task.id,
                task.project_id,
                task.title,
                task.body,
                status_str,
                task.created_at,
                task.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Get tasks for a project
    pub fn list_tasks_for_project(&self, project_id: &str) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, title, body, status, created_at, updated_at
             FROM tasks WHERE project_id = ?1 ORDER BY created_at",
        )?;

        let tasks = stmt
            .query_map([project_id], |row| {
                let status_str: String = row.get(4)?;
                Ok(Task {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    body: row.get(3)?,
                    status: serde_json::from_str(&status_str).unwrap_or(TaskStatus::Todo),
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// Delete a task by id
    pub fn delete_task(&self, task_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", [task_id])?;
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
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_and_list_project() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = ProjectStore::open(&db_path).unwrap();

        let project = Project {
            id: "test-123".to_string(),
            name: "My Project".to_string(),
            description: Some("Test project".to_string()),
            created_at: "2026-01-21T00:00:00Z".to_string(),
        };

        store.upsert_project(&project).unwrap();

        let projects = store.list_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "My Project");
    }

    #[test]
    fn test_project_repos_many_to_many() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = ProjectStore::open(&db_path).unwrap();

        let p1 = Project {
            id: "proj-1".to_string(),
            name: "Project 1".to_string(),
            description: None,
            created_at: "2026-01-21T00:00:00Z".to_string(),
        };
        let p2 = Project {
            id: "proj-2".to_string(),
            name: "Project 2".to_string(),
            description: None,
            created_at: "2026-01-21T00:00:00Z".to_string(),
        };
        store.upsert_project(&p1).unwrap();
        store.upsert_project(&p2).unwrap();

        store.add_repo_to_project("proj-1", "owner/repo-a").unwrap();
        store.add_repo_to_project("proj-1", "owner/repo-b").unwrap();
        store.add_repo_to_project("proj-2", "owner/repo-b").unwrap(); // repo-b in both

        let repos_p1 = store.list_repos_for_project("proj-1").unwrap();
        assert_eq!(repos_p1.len(), 2);
        assert!(repos_p1.contains(&"owner/repo-a".to_string()));
        assert!(repos_p1.contains(&"owner/repo-b".to_string()));

        let projects_for_b = store.list_projects_for_repo("owner/repo-b").unwrap();
        assert_eq!(projects_for_b.len(), 2);

        let all_repos = store.list_all_linked_repo_ids().unwrap();
        assert_eq!(all_repos.len(), 2); // distinct: repo-a, repo-b
        assert_eq!(all_repos[0], "owner/repo-a");
        assert_eq!(all_repos[1], "owner/repo-b");
    }

    #[test]
    fn test_list_all_linked_repo_ids_empty() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = ProjectStore::open(&db_path).unwrap();
        let all = store.list_all_linked_repo_ids().unwrap();
        assert!(all.is_empty());
    }

    #[test]
    fn test_create_and_list_tasks() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = ProjectStore::open(&db_path).unwrap();

        let project = Project {
            id: "proj-1".to_string(),
            name: "Test Project".to_string(),
            description: None,
            created_at: "2026-01-21T00:00:00Z".to_string(),
        };
        store.upsert_project(&project).unwrap();

        let task = Task {
            id: "task-1".to_string(),
            project_id: "proj-1".to_string(),
            title: "Test task".to_string(),
            body: Some("Description".to_string()),
            status: TaskStatus::InProgress,
            created_at: "2026-01-21T00:00:00Z".to_string(),
            updated_at: "2026-01-21T00:00:00Z".to_string(),
        };
        store.upsert_task(&task).unwrap();

        let tasks = store.list_tasks_for_project("proj-1").unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::InProgress);
        assert_eq!(tasks[0].project_id, "proj-1");
    }
}
