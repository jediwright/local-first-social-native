//! Phase 1 storage (Run 30). One storage adapter owns all persisted bytes —
//! the H1 seam. The Phase 0 stub schema (bare `docs(id, bytes)`) is removed;
//! the Phase 1 schema carries an app-owned `format` tag on every row so H1's
//! grep test has its target when 1b opens and Keyhive bytes start landing
//! here. The `DocStore` trait is unchanged from Phase 0 (the seam the plan
//! keeps); the tag is stamped inside the adapter, not threaded through
//! callers. rusqlite 0.40.2 bundled, pinned (plan §2).
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Mutex;

/// App-owned format tag stamped on every persisted row (H1 discipline).
/// Phase 1 1a persists exactly one format: Automerge `save()` bytes.
/// 1b introduces Keyhive-enveloped rows under their own tag values; existing
/// rows are never rewritten to a new tag — a tag names what the bytes ARE.
pub const FORMAT_AUTOMERGE_SAVE_V1: &str = "automerge.save.v1";

pub trait DocStore: Send + Sync {
    fn read(&self, id: &str) -> rusqlite::Result<Option<Vec<u8>>>;
    fn write(&self, id: &str, bytes: &[u8]) -> rusqlite::Result<()>;
}

pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open(path: &str) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS docs (
                id     TEXT PRIMARY KEY,
                format TEXT NOT NULL,
                bytes  BLOB NOT NULL
             );",
        )?;
        // Migration: a db created by the Phase 0 stub has `docs(id, bytes)`
        // without the format column. Tag existing rows with the one format
        // Phase 0 ever wrote. Additive; no bytes change.
        let has_format: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('docs') WHERE name = 'format'")?
            .query_row([], |_| Ok(true))
            .optional()?
            .unwrap_or(false);
        if !has_format {
            conn.execute_batch(&format!(
                "ALTER TABLE docs ADD COLUMN format TEXT NOT NULL DEFAULT '{FORMAT_AUTOMERGE_SAVE_V1}';"
            ))?;
        }
        Ok(Self { conn: Mutex::new(conn) })
    }
}

impl DocStore for SqliteStore {
    fn read(&self, id: &str) -> rusqlite::Result<Option<Vec<u8>>> {
        self.conn
            .lock()
            .unwrap()
            .query_row("SELECT bytes FROM docs WHERE id = ?1", params![id], |r| r.get(0))
            .optional()
    }
    fn write(&self, id: &str, bytes: &[u8]) -> rusqlite::Result<()> {
        self.conn.lock().unwrap().execute(
            "INSERT INTO docs (id, format, bytes) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET format = excluded.format, bytes = excluded.bytes",
            params![id, FORMAT_AUTOMERGE_SAVE_V1, bytes],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run 30 done-when (plan §1): INSERT/SELECT roundtrip under the trait.
    #[test]
    fn roundtrip_under_trait_with_format_tag() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p1.sqlite").to_string_lossy().to_string();
        let store: Box<dyn DocStore> = Box::new(SqliteStore::open(&path).unwrap());
        store.write("d1", b"bytes-1").unwrap();
        assert_eq!(store.read("d1").unwrap().as_deref(), Some(&b"bytes-1"[..]));
        store.write("d1", b"bytes-2").unwrap(); // upsert
        assert_eq!(store.read("d1").unwrap().as_deref(), Some(&b"bytes-2"[..]));
        // H1 target: every row carries the app-owned format tag.
        let conn = Connection::open(&path).unwrap();
        let fmt: String = conn
            .query_row("SELECT format FROM docs WHERE id = 'd1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fmt, FORMAT_AUTOMERGE_SAVE_V1);
    }

    /// A Phase 0 stub db (no format column) opens and migrates additively.
    #[test]
    fn migrates_phase0_stub_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p0.sqlite").to_string_lossy().to_string();
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                "CREATE TABLE docs (id TEXT PRIMARY KEY, bytes BLOB NOT NULL);
                 INSERT INTO docs (id, bytes) VALUES ('old', x'AB');",
            )
            .unwrap();
        }
        let store = SqliteStore::open(&path).unwrap();
        assert_eq!(store.read("old").unwrap().as_deref(), Some(&[0xABu8][..]));
        let conn = Connection::open(&path).unwrap();
        let fmt: String = conn
            .query_row("SELECT format FROM docs WHERE id = 'old'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(fmt, FORMAT_AUTOMERGE_SAVE_V1);
    }
}
