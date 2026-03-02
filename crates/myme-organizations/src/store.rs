use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

use crate::models::{Organization, Prospect, ProspectStage};

const SCHEMA_VERSION: i32 = 1;

/// Local SQLite storage for organizations and prospects
pub struct OrganizationStore {
    conn: Connection,
}

impl OrganizationStore {
    /// Open or create the database
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).context("Failed to open organizations database")?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute("CREATE TABLE IF NOT EXISTS schema_version (version INTEGER NOT NULL)", [])?;

        let version: i32 = self
            .conn
            .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| row.get(0))
            .optional()?
            .unwrap_or(0);

        if version < SCHEMA_VERSION {
            self.conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS organizations (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    website TEXT,
                    contact_name TEXT,
                    contact_email TEXT,
                    contact_phone TEXT,
                    contact_role TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS prospects (
                    id TEXT PRIMARY KEY,
                    organization_id TEXT NOT NULL,
                    name TEXT NOT NULL,
                    description TEXT,
                    stage TEXT NOT NULL,
                    value TEXT,
                    contact_name TEXT,
                    contact_email TEXT,
                    contact_role TEXT,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (organization_id) REFERENCES organizations(id)
                );

                CREATE TABLE IF NOT EXISTS organization_projects (
                    organization_id TEXT NOT NULL,
                    project_id TEXT NOT NULL,
                    PRIMARY KEY (organization_id, project_id),
                    FOREIGN KEY (organization_id) REFERENCES organizations(id)
                );

                CREATE INDEX IF NOT EXISTS idx_prospects_org ON prospects(organization_id);
                CREATE INDEX IF NOT EXISTS idx_prospects_stage ON prospects(stage);
                CREATE INDEX IF NOT EXISTS idx_org_projects_org ON organization_projects(organization_id);
                CREATE INDEX IF NOT EXISTS idx_org_projects_proj ON organization_projects(project_id);",
            )
            .context("Failed to initialize schema")?;

            self.conn.execute("DELETE FROM schema_version", [])?;
            self.conn
                .execute("INSERT INTO schema_version (version) VALUES (?1)", params![SCHEMA_VERSION])?;
        }

        Ok(())
    }

    // =========== Organizations ===========

    /// Insert or update an organization
    pub fn upsert_organization(&self, org: &Organization) -> Result<()> {
        self.conn.execute(
            "INSERT INTO organizations (id, name, description, website, contact_name, contact_email, contact_phone, contact_role, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                website = excluded.website,
                contact_name = excluded.contact_name,
                contact_email = excluded.contact_email,
                contact_phone = excluded.contact_phone,
                contact_role = excluded.contact_role,
                updated_at = excluded.updated_at",
            params![
                org.id,
                org.name,
                org.description,
                org.website,
                org.contact_name,
                org.contact_email,
                org.contact_phone,
                org.contact_role,
                org.created_at,
                org.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Get all organizations
    pub fn list_organizations(&self) -> Result<Vec<Organization>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, website, contact_name, contact_email, contact_phone, contact_role, created_at, updated_at
             FROM organizations ORDER BY created_at DESC",
        )?;

        let orgs = stmt
            .query_map([], |row| {
                Ok(Organization {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    website: row.get(3)?,
                    contact_name: row.get(4)?,
                    contact_email: row.get(5)?,
                    contact_phone: row.get(6)?,
                    contact_role: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(orgs)
    }

    /// Get an organization by ID
    pub fn get_organization(&self, id: &str) -> Result<Option<Organization>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, description, website, contact_name, contact_email, contact_phone, contact_role, created_at, updated_at
             FROM organizations WHERE id = ?1",
        )?;

        let org = stmt
            .query_row([id], |row| {
                Ok(Organization {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    website: row.get(3)?,
                    contact_name: row.get(4)?,
                    contact_email: row.get(5)?,
                    contact_phone: row.get(6)?,
                    contact_role: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .optional()?;

        Ok(org)
    }

    /// Delete an organization and cascade-delete its prospects and project links
    pub fn delete_organization(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM prospects WHERE organization_id = ?1", [id])?;
        self.conn
            .execute("DELETE FROM organization_projects WHERE organization_id = ?1", [id])?;
        self.conn.execute("DELETE FROM organizations WHERE id = ?1", [id])?;
        Ok(())
    }

    // =========== Prospects ===========

    /// Insert or update a prospect
    pub fn upsert_prospect(&self, prospect: &Prospect) -> Result<()> {
        let stage_str = serde_json::to_string(&prospect.stage)?;

        self.conn.execute(
            "INSERT INTO prospects (id, organization_id, name, description, stage, value, contact_name, contact_email, contact_role, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                stage = excluded.stage,
                value = excluded.value,
                contact_name = excluded.contact_name,
                contact_email = excluded.contact_email,
                contact_role = excluded.contact_role,
                updated_at = excluded.updated_at",
            params![
                prospect.id,
                prospect.organization_id,
                prospect.name,
                prospect.description,
                stage_str,
                prospect.value,
                prospect.contact_name,
                prospect.contact_email,
                prospect.contact_role,
                prospect.created_at,
                prospect.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Get prospects for an organization
    pub fn list_prospects(&self, organization_id: &str) -> Result<Vec<Prospect>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, organization_id, name, description, stage, value, contact_name, contact_email, contact_role, created_at, updated_at
             FROM prospects WHERE organization_id = ?1 ORDER BY created_at",
        )?;

        let prospects = stmt
            .query_map([organization_id], |row| {
                let stage_str: String = row.get(4)?;
                Ok(Prospect {
                    id: row.get(0)?,
                    organization_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    stage: serde_json::from_str(&stage_str).unwrap_or(ProspectStage::Lead),
                    value: row.get(5)?,
                    contact_name: row.get(6)?,
                    contact_email: row.get(7)?,
                    contact_role: row.get(8)?,
                    created_at: row.get(9)?,
                    updated_at: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(prospects)
    }

    /// Delete a prospect by ID
    pub fn delete_prospect(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM prospects WHERE id = ?1", [id])?;
        Ok(())
    }

    /// Update a prospect's stage
    pub fn update_prospect_stage(&self, id: &str, stage: ProspectStage) -> Result<()> {
        let stage_str = serde_json::to_string(&stage)?;
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE prospects SET stage = ?1, updated_at = ?2 WHERE id = ?3",
            params![stage_str, now, id],
        )?;
        Ok(())
    }

    /// Count prospects by stage for an organization
    pub fn count_prospects_by_stage(
        &self,
        organization_id: &str,
    ) -> Result<Vec<(ProspectStage, i32)>> {
        let mut stmt = self.conn.prepare(
            "SELECT stage, COUNT(*) FROM prospects WHERE organization_id = ?1 GROUP BY stage",
        )?;

        let counts = stmt
            .query_map([organization_id], |row| {
                let stage_str: String = row.get(0)?;
                let count: i32 = row.get(1)?;
                let stage = serde_json::from_str(&stage_str).unwrap_or(ProspectStage::Lead);
                Ok((stage, count))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(counts)
    }

    // =========== Project Linking ===========

    /// Link a project to an organization
    pub fn link_project(&self, organization_id: &str, project_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO organization_projects (organization_id, project_id) VALUES (?1, ?2)",
            params![organization_id, project_id],
        )?;
        Ok(())
    }

    /// Unlink a project from an organization
    pub fn unlink_project(&self, organization_id: &str, project_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM organization_projects WHERE organization_id = ?1 AND project_id = ?2",
            params![organization_id, project_id],
        )?;
        Ok(())
    }

    /// List project IDs linked to an organization
    pub fn list_linked_projects(&self, organization_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT project_id FROM organization_projects WHERE organization_id = ?1 ORDER BY project_id",
        )?;

        let projects = stmt
            .query_map([organization_id], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(projects)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;
    use tempfile::tempdir;

    fn test_store() -> (OrganizationStore, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let store = OrganizationStore::open(&db_path).unwrap();
        (store, dir)
    }

    fn test_org() -> Organization {
        Organization {
            id: "org-1".to_string(),
            name: "Web Networks".to_string(),
            description: Some("Canadian nonprofit ISP".to_string()),
            website: Some("https://web.net".to_string()),
            contact_name: Some("Greg MacKenzie".to_string()),
            contact_email: Some("greg@web.net".to_string()),
            contact_phone: None,
            contact_role: Some("Director".to_string()),
            created_at: "2026-03-02T00:00:00Z".to_string(),
            updated_at: "2026-03-02T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_create_and_list_organizations() {
        let (store, _dir) = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let orgs = store.list_organizations().unwrap();
        assert_eq!(orgs.len(), 1);
        assert_eq!(orgs[0].name, "Web Networks");
    }

    #[test]
    fn test_get_organization() {
        let (store, _dir) = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let org = store.get_organization("org-1").unwrap();
        assert!(org.is_some());
        assert_eq!(org.unwrap().website.unwrap(), "https://web.net");
    }

    #[test]
    fn test_delete_organization_cascades() {
        let (store, _dir) = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let prospect = Prospect {
            id: "p-1".to_string(),
            organization_id: "org-1".to_string(),
            name: "Ontario Nature".to_string(),
            description: None,
            stage: ProspectStage::Lead,
            value: None,
            contact_name: None,
            contact_email: None,
            contact_role: None,
            created_at: "2026-03-02T00:00:00Z".to_string(),
            updated_at: "2026-03-02T00:00:00Z".to_string(),
        };
        store.upsert_prospect(&prospect).unwrap();
        store.delete_organization("org-1").unwrap();
        assert!(store.list_prospects("org-1").unwrap().is_empty());
        assert!(store.get_organization("org-1").unwrap().is_none());
    }

    #[test]
    fn test_prospect_crud() {
        let (store, _dir) = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let prospect = Prospect {
            id: "p-1".to_string(),
            organization_id: "org-1".to_string(),
            name: "CUPE Local 79".to_string(),
            description: Some("Website rebuild".to_string()),
            stage: ProspectStage::Lead,
            value: Some("$15,000".to_string()),
            contact_name: Some("Jane Doe".to_string()),
            contact_email: None,
            contact_role: None,
            created_at: "2026-03-02T00:00:00Z".to_string(),
            updated_at: "2026-03-02T00:00:00Z".to_string(),
        };
        store.upsert_prospect(&prospect).unwrap();
        let prospects = store.list_prospects("org-1").unwrap();
        assert_eq!(prospects.len(), 1);
        assert_eq!(prospects[0].name, "CUPE Local 79");
    }

    #[test]
    fn test_update_prospect_stage() {
        let (store, _dir) = test_store();
        store.upsert_organization(&test_org()).unwrap();
        let prospect = Prospect {
            id: "p-1".to_string(),
            organization_id: "org-1".to_string(),
            name: "Test".to_string(),
            description: None,
            stage: ProspectStage::Lead,
            value: None,
            contact_name: None,
            contact_email: None,
            contact_role: None,
            created_at: "2026-03-02T00:00:00Z".to_string(),
            updated_at: "2026-03-02T00:00:00Z".to_string(),
        };
        store.upsert_prospect(&prospect).unwrap();
        store
            .update_prospect_stage("p-1", ProspectStage::Contacted)
            .unwrap();
        let prospects = store.list_prospects("org-1").unwrap();
        assert_eq!(prospects[0].stage, ProspectStage::Contacted);
    }

    #[test]
    fn test_count_prospects_by_stage() {
        let (store, _dir) = test_store();
        store.upsert_organization(&test_org()).unwrap();
        for (i, stage) in [ProspectStage::Lead, ProspectStage::Lead, ProspectStage::Contacted]
            .iter()
            .enumerate()
        {
            let p = Prospect {
                id: format!("p-{}", i),
                organization_id: "org-1".to_string(),
                name: format!("Prospect {}", i),
                description: None,
                stage: *stage,
                value: None,
                contact_name: None,
                contact_email: None,
                contact_role: None,
                created_at: "2026-03-02T00:00:00Z".to_string(),
                updated_at: "2026-03-02T00:00:00Z".to_string(),
            };
            store.upsert_prospect(&p).unwrap();
        }
        let counts = store.count_prospects_by_stage("org-1").unwrap();
        let lead_count = counts
            .iter()
            .find(|(s, _)| *s == ProspectStage::Lead)
            .map(|(_, c)| *c)
            .unwrap_or(0);
        assert_eq!(lead_count, 2);
    }

    #[test]
    fn test_project_linking() {
        let (store, _dir) = test_store();
        store.upsert_organization(&test_org()).unwrap();
        store.link_project("org-1", "proj-abc").unwrap();
        store.link_project("org-1", "proj-def").unwrap();
        let projects = store.list_linked_projects("org-1").unwrap();
        assert_eq!(projects.len(), 2);
        store.unlink_project("org-1", "proj-abc").unwrap();
        let projects = store.list_linked_projects("org-1").unwrap();
        assert_eq!(projects.len(), 1);
    }
}
