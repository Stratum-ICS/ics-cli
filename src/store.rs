use crate::error::{IcsError, Result};
use crate::paths;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
PRAGMA user_version = 1;

CREATE TABLE IF NOT EXISTS commits (
  commit_id TEXT PRIMARY KEY NOT NULL,
  tree_id TEXT NOT NULL,
  parent_commit_ids TEXT NOT NULL,
  message TEXT NOT NULL,
  author TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS refs (
  name TEXT PRIMARY KEY NOT NULL,
  commit_id TEXT NOT NULL
);
"#;

pub struct Store {
    conn: Connection,
    pub ics_dir: std::path::PathBuf,
}

#[derive(Debug, Clone)]
pub struct CommitRow {
    pub commit_id: String,
    pub tree_id: String,
    pub parent_commit_ids: String,
    pub message: String,
    pub author: String,
    pub created_at: i64,
}

impl Store {
    pub fn open(ics_dir: &Path) -> Result<Self> {
        let db_path = paths::store_db_path(ics_dir);
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn,
            ics_dir: ics_dir.to_path_buf(),
        })
    }

    /// Creates `.ics/` under `repo_root` and opens the database.
    pub fn init(repo_root: &Path) -> Result<Self> {
        let ics = paths::ics_dir(repo_root);
        if ics.exists() {
            return Err(IcsError::AlreadyInitialized);
        }
        fs::create_dir_all(ics.join("objects/blobs"))?;
        fs::create_dir_all(ics.join("objects/trees"))?;
        Self::open(&ics)
    }

    pub fn insert_commit(
        &self,
        commit_id: &str,
        tree_id: &str,
        parent_commit_ids_json: &str,
        message: &str,
        author: &str,
        created_at: i64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO commits (commit_id, tree_id, parent_commit_ids, message, author, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                commit_id,
                tree_id,
                parent_commit_ids_json,
                message,
                author,
                created_at
            ],
        )?;
        Ok(())
    }

    pub fn set_ref(&self, name: &str, commit_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO refs (name, commit_id) VALUES (?1, ?2) ON CONFLICT(name) DO UPDATE SET commit_id = excluded.commit_id",
            params![name, commit_id],
        )?;
        Ok(())
    }

    pub fn get_ref(&self, name: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT commit_id FROM refs WHERE name = ?1")?;
        let mut rows = stmt.query(params![name])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn head_commit(&self) -> Result<Option<String>> {
        self.get_ref("HEAD")
    }

    pub fn get_commit_row(&self, commit_id: &str) -> Result<Option<CommitRow>> {
        let mut stmt = self.conn.prepare(
            "SELECT commit_id, tree_id, parent_commit_ids, message, author, created_at FROM commits WHERE commit_id = ?1",
        )?;
        let mut rows = stmt.query(params![commit_id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(CommitRow {
                commit_id: row.get(0)?,
                tree_id: row.get(1)?,
                parent_commit_ids: row.get(2)?,
                message: row.get(3)?,
                author: row.get(4)?,
                created_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn insert_commit_and_refs_round_trip() {
        let dir = tempdir().unwrap();
        let ics = dir.path().join(".ics");
        fs::create_dir_all(ics.join("objects/blobs")).unwrap();
        fs::create_dir_all(ics.join("objects/trees")).unwrap();
        let store = Store::open(&ics).unwrap();
        store
            .insert_commit(
                "abc123",
                "treehex",
                "[]",
                "msg",
                "author",
                42,
            )
            .unwrap();
        store.set_ref("HEAD", "abc123").unwrap();
        store.set_ref("main", "abc123").unwrap();
        assert_eq!(store.head_commit().unwrap(), Some("abc123".into()));
        let row = store.get_commit_row("abc123").unwrap().unwrap();
        assert_eq!(row.parent_commit_ids, "[]");
        assert_eq!(row.tree_id, "treehex");
    }
}
