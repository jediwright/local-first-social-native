//! lfs_core — Phase 0 skeleton (native-track-phase0-build-plan-v0-1 §2 B1–B5).
//! API surface is exactly the plan's B1 list. No delegation-level concept exists here (L-12 interim).
uniffi::setup_scaffolding!();

pub mod resolve;
pub mod storage;

use autosurgeon::{hydrate, reconcile, Hydrate, Reconcile};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;
use storage::{DocStore, SqliteStore};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("no such handle: {0}")]
    NoSuchHandle(u64),
    #[error("storage: {0}")]
    Storage(String),
    #[error("automerge: {0}")]
    Automerge(String),
    #[error("network: {0}")]
    Network(String),
}

/// The one document shape Phase 0 round-trips. Phase 1+ replaces it; nothing here is a schema ruling.
#[derive(Default, Debug, Clone, Hydrate, Reconcile, PartialEq)]
pub struct HelloDoc {
    pub entries: BTreeMap<String, String>,
}

struct Open {
    id: String,
    doc: automerge::AutoCommit,
}

#[derive(uniffi::Object)]
pub struct Core {
    store: Box<dyn DocStore>,
    open: Mutex<HashMap<u64, Open>>,
    next: Mutex<u64>,
}

#[uniffi::export]
impl Core {
    /// `db_path` — SQLite file; `":memory:"` for tests.
    #[uniffi::constructor]
    pub fn new(db_path: String) -> Result<Self, CoreError> {
        let store = SqliteStore::open(&db_path).map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self { store: Box::new(store), open: Mutex::new(HashMap::new()), next: Mutex::new(1) })
    }

    /// open_doc(id) -> handle. Loads from the store if present, else starts an empty doc.
    pub fn open_doc(&self, id: String) -> Result<u64, CoreError> {
        let doc = match self.store.read(&id).map_err(|e| CoreError::Storage(e.to_string()))? {
            Some(bytes) => automerge::AutoCommit::load(&bytes).map_err(|e| CoreError::Automerge(e.to_string()))?,
            None => {
                let mut d = automerge::AutoCommit::new();
                reconcile(&mut d, &HelloDoc::default()).map_err(|e| CoreError::Automerge(e.to_string()))?;
                d
            }
        };
        Ok(self.insert(Open { id, doc }))
    }

    pub fn put(&self, handle: u64, key: String, value: String) -> Result<(), CoreError> {
        self.with(handle, |o| {
            let mut h: HelloDoc = hydrate(&o.doc).map_err(|e| CoreError::Automerge(e.to_string()))?;
            h.entries.insert(key, value);
            reconcile(&mut o.doc, &h).map_err(|e| CoreError::Automerge(e.to_string()))
        })
    }

    pub fn get(&self, handle: u64, key: String) -> Result<Option<String>, CoreError> {
        self.with(handle, |o| {
            let h: HelloDoc = hydrate(&o.doc).map_err(|e| CoreError::Automerge(e.to_string()))?;
            Ok(h.entries.get(&key).cloned())
        })
    }

    /// save(handle) -> bytes. Also persists to the store under the doc id.
    pub fn save(&self, handle: u64) -> Result<Vec<u8>, CoreError> {
        let (id, bytes) = self.with(handle, |o| Ok((o.id.clone(), o.doc.save())))?;
        self.store.write(&id, &bytes).map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(bytes)
    }

    /// load(bytes) -> handle. Not persisted until `save`.
    pub fn load(&self, id: String, bytes: Vec<u8>) -> Result<u64, CoreError> {
        let doc = automerge::AutoCommit::load(&bytes).map_err(|e| CoreError::Automerge(e.to_string()))?;
        Ok(self.insert(Open { id, doc }))
    }

    /// B4 — resolve a DID to its PDS endpoint. One live read; no writes. Blocks on an internal runtime.
    pub fn resolve_pds(&self, did: String) -> Result<String, CoreError> {
        resolve::resolve_pds_blocking(&did)
    }

    /// Pin attestation for the observation log.
    pub fn pins(&self) -> String {
        format!(
            "keyhive_core=0.5.0 automerge=0.12 samod=0.14 autosurgeon=0.14 atrium-api=0.25 subduction={}",
            if cfg!(feature = "subduction") { "21b2e6b8(declared)" } else { "off" }
        )
    }
}

impl Core {
    fn insert(&self, o: Open) -> u64 {
        let mut n = self.next.lock().unwrap();
        let h = *n;
        *n += 1;
        self.open.lock().unwrap().insert(h, o);
        h
    }
    fn with<T>(&self, handle: u64, f: impl FnOnce(&mut Open) -> Result<T, CoreError>) -> Result<T, CoreError> {
        let mut m = self.open.lock().unwrap();
        let o = m.get_mut(&handle).ok_or(CoreError::NoSuchHandle(handle))?;
        f(o)
    }
}

// B3 — force the pin to link, without creating any Keyhive object.
#[allow(dead_code)]
fn keyhive_core_linked() -> &'static str {
    std::any::type_name::<keyhive_core::access::Access>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_in_memory() {
        let core = Core::new(":memory:".into()).unwrap();
        let h = core.open_doc("hello".into()).unwrap();
        core.put(h, "greeting".into(), "hello, world".into()).unwrap();
        let bytes = core.save(h).unwrap();
        let h2 = core.load("hello".into(), bytes).unwrap();
        assert_eq!(core.get(h2, "greeting".into()).unwrap().as_deref(), Some("hello, world"));
    }

    #[test]
    fn survives_save_load_through_sqlite_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("phase0.sqlite").to_string_lossy().to_string();
        {
            let core = Core::new(path.clone()).unwrap();
            let h = core.open_doc("persisted".into()).unwrap();
            core.put(h, "k".into(), "v".into()).unwrap();
            core.save(h).unwrap();
        }
        let core = Core::new(path).unwrap();
        let h = core.open_doc("persisted".into()).unwrap();
        assert_eq!(core.get(h, "k".into()).unwrap().as_deref(), Some("v"));
    }

    #[test]
    fn keyhive_pin_links() {
        assert!(keyhive_core_linked().contains("keyhive_core"));
    }

    /// B2 — samod is constructed (repo layer), not just declared. In-memory storage; no peers.
    #[tokio::test]
    async fn samod_repo_constructs_and_creates_doc() {
        let repo = samod::Repo::build_tokio().load().await;
        let mut doc = automerge::Automerge::new();
        let mut tx = doc.transaction();
        automerge::transaction::Transactable::put(&mut tx, automerge::ROOT, "k", "v").unwrap();
        tx.commit();
        let handle = repo.create(doc).await.expect("repo running");
        assert!(!handle.document_id().to_string().is_empty());
        repo.stop().await;
    }

    /// B4 on host — live network. Ignored by default; run with `--ignored` on a machine with egress.
    /// In the Phase 0 build container this is logged `null` (egress denied), not `fail`.
    #[test]
    #[ignore]
    fn resolve_known_did_to_pds() {
        let core = Core::new(":memory:".into()).unwrap();
        let pds = core.resolve_pds("did:plc:z72i7hdynmk6r22z27h6tvur".into()).unwrap(); // bsky.app
        assert!(pds.starts_with("https://"));
    }
}
