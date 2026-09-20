//! B1 — storage behind a trait. Phase 0 ships SQLite only; the trait is the seam Phase 1 keeps.
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::Mutex;

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
        conn.execute_batch("CREATE TABLE IF NOT EXISTS docs (id TEXT PRIMARY KEY, bytes BLOB NOT NULL);")?;
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
            "INSERT INTO docs (id, bytes) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET bytes = excluded.bytes",
            params![id, bytes],
        )?;
        Ok(())
    }
}
