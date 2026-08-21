use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRecord {
    pub id: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: String,
    pub secret_ref: Option<String>,
    pub is_default: bool,
    pub compat_mode: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRecord {
    pub id: String,
    pub provider_id: String,
    pub model_id: String,
    pub display_name: String,
    pub context_window: Option<i64>,
    pub max_tokens: Option<i64>,
    pub supports_reasoning: bool,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub discovered_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotRecord {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub snapshot_path: String,
    pub git_commit: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHistoryRecord {
    pub id: i64,
    pub from_version: Option<String>,
    pub to_version: String,
    pub status: String,
    pub error_message: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxRunRecord {
    pub id: String,
    pub project_id: String,
    pub workspace_path: String,
    pub status: String,
    pub test_command: Option<String>,
    pub test_output: Option<String>,
    pub diff_content: Option<String>,
    pub created_at: String,
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    #[allow(dead_code)]
    db_path: PathBuf,
}

impl Database {
    pub fn init<P: AsRef<Path>>(base_data_dir: P) -> Result<Self> {
        let db_dir = base_data_dir.as_ref().join("database");
        fs::create_dir_all(&db_dir)
            .with_context(|| format!("Failed to create database directory {:?}", db_dir))?;

        let db_path = db_dir.join("dsh_desktop.sqlite");
        let conn = Connection::open(&db_path)
            .with_context(|| format!("Failed to open sqlite database at {:?}", db_path))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };

        db.run_migrations()?;
        Ok(db)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS application_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider_type TEXT NOT NULL,
                base_url TEXT NOT NULL,
                secret_ref TEXT,
                is_default BOOLEAN DEFAULT 0,
                compat_mode TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS provider_models (
                id TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
                model_id TEXT NOT NULL,
                display_name TEXT NOT NULL,
                context_window INTEGER,
                max_tokens INTEGER,
                supports_reasoning BOOLEAN DEFAULT 0,
                supports_vision BOOLEAN DEFAULT 0,
                supports_tools BOOLEAN DEFAULT 1,
                discovered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS runtime_versions (
                version TEXT PRIMARY KEY,
                channel TEXT NOT NULL,
                installed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                status TEXT NOT NULL,
                integrity_hash TEXT,
                install_path TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS update_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_version TEXT,
                to_version TEXT NOT NULL,
                status TEXT NOT NULL,
                error_message TEXT,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                name TEXT NOT NULL,
                project_type TEXT NOT NULL,
                last_opened_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS project_snapshots (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                title TEXT NOT NULL,
                description TEXT,
                snapshot_path TEXT NOT NULL,
                git_commit TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS sandbox_runs (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                workspace_path TEXT NOT NULL,
                status TEXT NOT NULL,
                test_command TEXT,
                test_output TEXT,
                diff_content TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )?;

        Ok(())
    }

    // --- Settings helpers ---
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM application_settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO application_settings (key, value, updated_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = CURRENT_TIMESTAMP",
            params![key, value],
        )?;
        Ok(())
    }

    // --- Providers helpers ---
    pub fn list_providers(&self) -> Result<Vec<ProviderRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, provider_type, base_url, secret_ref, is_default, compat_mode, created_at, updated_at 
             FROM providers ORDER BY is_default DESC, name ASC"
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ProviderRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                provider_type: row.get(2)?,
                base_url: row.get(3)?,
                secret_ref: row.get(4)?,
                is_default: row.get(5)?,
                compat_mode: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn save_provider(&self, provider: &ProviderRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        if provider.is_default {
            conn.execute("UPDATE providers SET is_default = 0", [])?;
        }
        conn.execute(
            "INSERT INTO providers (id, name, provider_type, base_url, secret_ref, is_default, compat_mode, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                provider_type = excluded.provider_type,
                base_url = excluded.base_url,
                secret_ref = excluded.secret_ref,
                is_default = excluded.is_default,
                compat_mode = excluded.compat_mode,
                updated_at = CURRENT_TIMESTAMP",
            params![
                provider.id,
                provider.name,
                provider.provider_type,
                provider.base_url,
                provider.secret_ref,
                provider.is_default,
                provider.compat_mode,
                provider.created_at,
                provider.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete_provider(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM providers WHERE id = ?1", params![id])?;
        Ok(())
    }

    // --- Provider Models ---
    pub fn list_models(&self, provider_id: &str) -> Result<Vec<ModelRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, provider_id, model_id, display_name, context_window, max_tokens, supports_reasoning, supports_vision, supports_tools, discovered_at
             FROM provider_models WHERE provider_id = ?1 ORDER BY display_name ASC"
        )?;

        let rows = stmt.query_map(params![provider_id], |row| {
            Ok(ModelRecord {
                id: row.get(0)?,
                provider_id: row.get(1)?,
                model_id: row.get(2)?,
                display_name: row.get(3)?,
                context_window: row.get(4)?,
                max_tokens: row.get(5)?,
                supports_reasoning: row.get(6)?,
                supports_vision: row.get(7)?,
                supports_tools: row.get(8)?,
                discovered_at: row.get(9)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn replace_models(&self, provider_id: &str, models: &[ModelRecord]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute("DELETE FROM provider_models WHERE provider_id = ?1", params![provider_id])?;
        for m in models {
            tx.execute(
                "INSERT INTO provider_models (id, provider_id, model_id, display_name, context_window, max_tokens, supports_reasoning, supports_vision, supports_tools, discovered_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    m.id,
                    m.provider_id,
                    m.model_id,
                    m.display_name,
                    m.context_window,
                    m.max_tokens,
                    m.supports_reasoning,
                    m.supports_vision,
                    m.supports_tools,
                    m.discovered_at,
                ],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    // --- Update History ---
    pub fn record_update(&self, from_ver: Option<&str>, to_ver: &str, status: &str, error_msg: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO update_history (from_version, to_version, status, error_message) VALUES (?1, ?2, ?3, ?4)",
            params![from_ver, to_ver, status, error_msg],
        )?;
        Ok(())
    }

    pub fn list_update_history(&self) -> Result<Vec<UpdateHistoryRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, from_version, to_version, status, error_message, timestamp FROM update_history ORDER BY id DESC LIMIT 50"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(UpdateHistoryRecord {
                id: row.get(0)?,
                from_version: row.get(1)?,
                to_version: row.get(2)?,
                status: row.get(3)?,
                error_message: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // --- Snapshots ---
    pub fn save_snapshot(&self, snap: &SnapshotRecord) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO project_snapshots (id, project_id, title, description, snapshot_path, git_commit, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                snap.id,
                snap.project_id,
                snap.title,
                snap.description,
                snap.snapshot_path,
                snap.git_commit,
                snap.created_at
            ],
        )?;
        Ok(())
    }

    pub fn list_snapshots(&self, project_id: &str) -> Result<Vec<SnapshotRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, project_id, title, description, snapshot_path, git_commit, created_at
             FROM project_snapshots WHERE project_id = ?1 ORDER BY created_at DESC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            Ok(SnapshotRecord {
                id: row.get(0)?,
                project_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                snapshot_path: row.get(4)?,
                git_commit: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_snapshot(&self, snapshot_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM project_snapshots WHERE id = ?1", params![snapshot_id])?;
        Ok(())
    }
}
