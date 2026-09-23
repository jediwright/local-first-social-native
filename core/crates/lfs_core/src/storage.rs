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

/// Run 37 (identity ceremony) — the FIRST Keyhive format, ruled in-run (plan
/// §9 row 3): the identity document's delegation heads as
/// `Vec<Signed<StaticDelegation<[u8; 32]>>>` encoded with `bincode` over
/// `keyhive_core`'s serde shape at the H6 pin `90fe4a51`. Public proof bytes
/// only — no secrets (the cold keys never enter this adapter; the full
/// `Archive` was rejected because it carries prekey secrets). `.v1` names
/// THAT shape: a T4 re-encode (`kh0` → `kh1`, spec L-16) or any upstream
/// serde change lands under a new value; existing rows are never re-tagged
/// (a tag names what the bytes ARE — Run 30 rule). This tag is the seam the
/// Consumer Evidence Note v1 measures against.
pub const FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1: &str = "keyhive.static-delegations.bincode.v1";

/// Run 38 (D-38-4; tag value RULED in-run under Task 2 (iv)) — the second
/// Keyhive row: `bincode` over `Vec<keyhive_core::event::static_event::
/// StaticEvent<[u8; 32]>>` at `90fe4a51`, holding the identity ceremony's
/// admin and device `KeyOp`s as `PrekeysExpanded` events — public prekey
/// material only, the exact type `ingest_unsorted_static_events` consumes.
/// NOT the v1 delegation type (a different serde enum), so a NEW value;
/// `.v1` names THAT shape; same never-re-tag rule as the delegation tag.
pub const FORMAT_KEYHIVE_STATIC_EVENTS_V1: &str = "keyhive.static-events.bincode.v1";

/// Run 36 H1 housekeeping (flagged at Run 30): the Keyhive pin label shown in
/// `Core::pins()` lives with the adapter that owns the Keyhive bytes.
/// Run 37 RE-RULED the display string alongside the first format tag: the
/// crates.io version string alone does not identify the pinned code (SL-0242:
/// `=0.5.0` and `main@90fe4a51` carry the same version string, 12 commits
/// apart), so the label now carries both — version + git rev. Shells' pins
/// footer changes accordingly (predicted in the Run 37 apply guide).
pub const PIN_LABEL_KEYHIVE_CORE: &str = "keyhive_core=0.5.0+90fe4a51";

// B3 (Phase 0) — force the pin to link, without creating any Keyhive object.
// Relocated from lib.rs in Run 36 (H1 housekeeping); unchanged otherwise.
#[allow(dead_code)]
pub(crate) fn keyhive_core_linked() -> &'static str {
    std::any::type_name::<keyhive_core::access::Access>()
}

pub trait DocStore: Send + Sync {
    fn read(&self, id: &str) -> rusqlite::Result<Option<Vec<u8>>>;
    fn write(&self, id: &str, bytes: &[u8]) -> rusqlite::Result<()>;
    /// Run 37 — read a row with its format tag, so a caller can tell an
    /// Automerge row from a Keyhive row before interpreting the bytes.
    /// Additive; `read` is unchanged (Phase 0/1a callers never see a Keyhive
    /// row under the ids they use).
    fn read_tagged(&self, id: &str) -> rusqlite::Result<Option<(String, Vec<u8>)>>;
    /// Run 37 — write Keyhive static-delegation bytes. The tag is stamped
    /// HERE, never threaded through callers (H1: one adapter owns every
    /// persisted Keyhive byte and its tag). Upsert like `write`.
    fn write_keyhive_static_delegations(&self, id: &str, bytes: &[u8]) -> rusqlite::Result<()>;
    /// Run 38 — write Keyhive static-event bytes (the `KeyOp` row). Tag
    /// stamped HERE (H1), same upsert as the other writes. Additive.
    fn write_keyhive_static_events(&self, id: &str, bytes: &[u8]) -> rusqlite::Result<()>;
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
        self.upsert(id, FORMAT_AUTOMERGE_SAVE_V1, bytes)
    }
    fn read_tagged(&self, id: &str) -> rusqlite::Result<Option<(String, Vec<u8>)>> {
        self.conn
            .lock()
            .unwrap()
            .query_row("SELECT format, bytes FROM docs WHERE id = ?1", params![id], |r| Ok((r.get(0)?, r.get(1)?)))
            .optional()
    }
    fn write_keyhive_static_delegations(&self, id: &str, bytes: &[u8]) -> rusqlite::Result<()> {
        self.upsert(id, FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1, bytes)
    }
    fn write_keyhive_static_events(&self, id: &str, bytes: &[u8]) -> rusqlite::Result<()> {
        self.upsert(id, FORMAT_KEYHIVE_STATIC_EVENTS_V1, bytes)
    }
}

impl SqliteStore {
    /// The one write path (Run 37: `write` and the Keyhive write share it; the
    /// format is a parameter here and a constant at each trait method).
    fn upsert(&self, id: &str, format: &str, bytes: &[u8]) -> rusqlite::Result<()> {
        self.conn.lock().unwrap().execute(
            "INSERT INTO docs (id, format, bytes) VALUES (?1, ?2, ?3)
             ON CONFLICT(id) DO UPDATE SET format = excluded.format, bytes = excluded.bytes",
            params![id, format, bytes],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// B3 link smoke, relocated from lib.rs in Run 36 (H1 housekeeping).
    #[test]
    fn keyhive_pin_links() {
        assert!(keyhive_core_linked().contains("keyhive_core"));
    }

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

    /// Run 37 — a Keyhive row and an Automerge row coexist in one table, each
    /// under its own tag; `read` still returns raw bytes; `read_tagged` tells
    /// them apart; a Keyhive write never re-tags an Automerge row.
    #[test]
    fn keyhive_and_automerge_rows_coexist_under_their_own_tags() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p1-tags.sqlite").to_string_lossy().to_string();
        let store: Box<dyn DocStore> = Box::new(SqliteStore::open(&path).unwrap());
        store.write("profile", b"am-bytes").unwrap();
        store.write_keyhive_static_delegations("identity", b"kh-bytes").unwrap();
        assert_eq!(store.read("identity").unwrap().as_deref(), Some(&b"kh-bytes"[..]));
        assert_eq!(
            store.read_tagged("identity").unwrap(),
            Some((FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1.to_string(), b"kh-bytes".to_vec()))
        );
        assert_eq!(
            store.read_tagged("profile").unwrap(),
            Some((FORMAT_AUTOMERGE_SAVE_V1.to_string(), b"am-bytes".to_vec()))
        );
        assert_eq!(store.read_tagged("absent").unwrap(), None);
        assert_ne!(FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1, FORMAT_AUTOMERGE_SAVE_V1);
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
