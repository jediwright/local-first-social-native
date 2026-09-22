//! lfs_core — Phase 0 skeleton (native-track-phase0-build-plan-v0-1 §2 B1–B5).
//! API surface is exactly the plan's B1 list. No delegation-level concept exists here (L-12 interim).
uniffi::setup_scaffolding!();

pub mod docs;
pub mod resolve;
pub(crate) mod runtime;
pub mod storage;

use autosurgeon::{hydrate, reconcile, Hydrate, Reconcile};
use docs::{PingsDoc, ProfileDoc, ThreadsDoc};
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

/// The Phase 0 round-trip shape. Run 32 decision: KEPT, not retired — it
/// backs the generic KV FFI surface (`open_doc`/`put`/`get`) both shells
/// wired in Run 31; retiring it would churn the shell contract inside an
/// additive doc-model run. The Phase 1 typed shapes live in `docs`
/// (profile/pings/threads); HelloDoc carries no schema ruling.
#[derive(Default, Debug, Clone, Hydrate, Reconcile, PartialEq)]
pub struct HelloDoc {
    pub entries: BTreeMap<String, String>,
}

/// Run 33 — which Phase 1 typed shape a document carries. Selects the
/// default reconciled into a NEW document by `open_typed_doc`; an existing
/// document is loaded as stored and the kind is not re-checked (the doc id
/// is the shell's contract, as for `open_doc`). Not a persisted tag: the
/// storage row's format tag stays `automerge.save.v1` (Run 30 rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum DocKind {
    Profile,
    Pings,
    Threads,
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
    /// Run 32 — membership-version counter state (plan §1 1a: interface
    /// only, no Keyhive backend). In-memory by design: the counter is the
    /// seam 1b's real membership events will drive; nothing here is a
    /// persistence ruling.
    membership: Mutex<HashMap<String, u64>>,
}

/// Run 30 — the explicit FFI init entry point (Run 27 lifecycle contract).
/// Opens (creating if absent) the SQLite db at the shell-provided path AND
/// touches the shared runtime, so both the storage layer and the core's
/// long-lived tokio runtime are initialised explicitly at startup rather
/// than lazily on first use (lazy construction remains the backstop).
/// Blocking; shells dispatch it off-main (wired in Run 31, A-O22).
/// The sole FFI init path: `Core::new` is crate-internal as of Run 31.
#[uniffi::export]
pub fn init_core(db_path: String) -> Result<std::sync::Arc<Core>, CoreError> {
    let _ = runtime::rt(); // explicit-at-startup runtime init
    Ok(std::sync::Arc::new(Core::new(db_path)?))
}

#[uniffi::export]
impl Core {
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

    // ------------------------------------------------------------------
    // Run 33 — typed FFI surface for the Phase 1 doc types (plan §1 1a:
    // "Local UI on both shells ... displays doc content from SQLite").
    // Build rule (record §3): typed uniffi records mirroring `docs.rs`
    // 1:1 — the `docs` structs themselves derive `uniffi::Record`, so the
    // spec-derived shape is defined once and crosses the boundary typed.
    // Rejected: JSON-over-FFI (a schema copy + parser dependency per shell);
    // generic KV with typed accessors (the shapes are nested maps of lists,
    // which the flat `put`/`get` string surface cannot carry). The generic
    // KV surface and HelloDoc are KEPT (Run 32 rule) — additions only.
    // ------------------------------------------------------------------

    /// open_typed_doc(id, kind) -> handle. Loads from the store if a row
    /// exists under `id`; otherwise starts a NEW document holding the
    /// default of `kind` (so a first `get_*` on a fresh install returns an
    /// empty, well-formed doc rather than a hydrate error). Handles are the
    /// same space as `open_doc`; `save`/`load` apply unchanged.
    pub fn open_typed_doc(&self, id: String, kind: DocKind) -> Result<u64, CoreError> {
        let doc = match self.store.read(&id).map_err(|e| CoreError::Storage(e.to_string()))? {
            Some(bytes) => automerge::AutoCommit::load(&bytes).map_err(|e| CoreError::Automerge(e.to_string()))?,
            None => {
                let mut d = automerge::AutoCommit::new();
                let r = match kind {
                    DocKind::Profile => reconcile(&mut d, &ProfileDoc::default()),
                    DocKind::Pings => reconcile(&mut d, &PingsDoc::default()),
                    DocKind::Threads => reconcile(&mut d, &ThreadsDoc::default()),
                };
                r.map_err(|e| CoreError::Automerge(e.to_string()))?;
                d
            }
        };
        Ok(self.insert(Open { id, doc }))
    }

    /// Typed read: hydrate the whole profile document across the boundary.
    pub fn get_profile(&self, handle: u64) -> Result<ProfileDoc, CoreError> {
        self.with(handle, |o| hydrate(&o.doc).map_err(|e| CoreError::Automerge(e.to_string())))
    }

    /// Typed write: reconcile the whole profile document. Not persisted
    /// until `save` (same contract as `put`).
    pub fn put_profile(&self, handle: u64, profile: ProfileDoc) -> Result<(), CoreError> {
        self.with(handle, |o| reconcile(&mut o.doc, &profile).map_err(|e| CoreError::Automerge(e.to_string())))
    }

    /// Typed read: the pings document (channel → pings). Expired pings are
    /// returned as stored; ephemerality is display-side this run (docs.rs).
    pub fn get_pings(&self, handle: u64) -> Result<PingsDoc, CoreError> {
        self.with(handle, |o| hydrate(&o.doc).map_err(|e| CoreError::Automerge(e.to_string())))
    }

    /// Typed write: the pings document. Not persisted until `save`.
    pub fn put_pings(&self, handle: u64, pings: PingsDoc) -> Result<(), CoreError> {
        self.with(handle, |o| reconcile(&mut o.doc, &pings).map_err(|e| CoreError::Automerge(e.to_string())))
    }

    /// Typed read: the threads document (contact → messages).
    pub fn get_threads(&self, handle: u64) -> Result<ThreadsDoc, CoreError> {
        self.with(handle, |o| hydrate(&o.doc).map_err(|e| CoreError::Automerge(e.to_string())))
    }

    /// Typed write: the threads document. Not persisted until `save`.
    pub fn put_threads(&self, handle: u64, threads: ThreadsDoc) -> Result<(), CoreError> {
        self.with(handle, |o| reconcile(&mut o.doc, &threads).map_err(|e| CoreError::Automerge(e.to_string())))
    }

    /// B4 — resolve a DID to its PDS endpoint. One live read; no writes. Blocks on an internal runtime.
    pub fn resolve_pds(&self, did: String) -> Result<String, CoreError> {
        resolve::resolve_pds_blocking(&did)
    }

    /// Run 32 — membership-version counter, read side (plan §1 1a:
    /// "Counter increments on membership event; interface in place").
    /// Interface only: no Keyhive backend; 1b backs this with Keyhive
    /// group membership. Version 0 = no membership event seen for the group.
    pub fn membership_version(&self, group_id: String) -> u64 {
        *self.membership.lock().unwrap().get(&group_id).unwrap_or(&0)
    }

    /// Run 32 — membership-version counter, event side. Records one
    /// (simulated) membership event for `group_id` and returns the new
    /// version. 1b replaces the simulation with real Keyhive membership
    /// events driving the same interface; the signature is the seam.
    pub fn record_membership_event(&self, group_id: String) -> u64 {
        let mut m = self.membership.lock().unwrap();
        let v = m.entry(group_id).or_insert(0);
        *v += 1;
        *v
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
    /// Crate-internal constructor. Retired from the FFI surface in the
    /// A-O22 shell-integration run (Run 31): both shells now call
    /// `init_core`, the one explicit init path across FFI (finding (a)
    /// settled — retirement lands with the shell migration, same run).
    /// `db_path` — SQLite file; `":memory:"` for tests.
    pub(crate) fn new(db_path: String) -> Result<Self, CoreError> {
        let store = SqliteStore::open(&db_path).map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self {
            store: Box::new(store),
            open: Mutex::new(HashMap::new()),
            next: Mutex::new(1),
            membership: Mutex::new(HashMap::new()),
        })
    }

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

    /// Run 30 done-when: db file created at the path the init entry point is
    /// given; runtime touched; core usable end to end through it.
    #[test]
    fn init_core_creates_db_at_path_and_inits_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("init.sqlite").to_string_lossy().to_string();
        let core = init_core(path.clone()).unwrap();
        assert!(std::path::Path::new(&path).exists(), "db file created at shell-provided path");
        let h = core.open_doc("via-init".into()).unwrap();
        core.put(h, "k".into(), "v".into()).unwrap();
        core.save(h).unwrap();
        assert_eq!(core.get(h, "k".into()).unwrap().as_deref(), Some("v"));
        // runtime is initialised and shared
        assert_eq!(crate::runtime::rt().block_on(async { 1 + 1 }), 2);
    }

    /// Run 32 helper — round-trip one typed doc through the Storage trait:
    /// reconcile → automerge save → trait write (automerge.save.v1 tag,
    /// Run 30 schema) → trait read → load → hydrate → equality. Also
    /// asserts the row carries the format tag (rows never re-tagged).
    fn round_trip_typed<T: autosurgeon::Hydrate + autosurgeon::Reconcile + PartialEq + std::fmt::Debug>(
        path: &str,
        id: &str,
        value: &T,
    ) {
        let store: Box<dyn DocStore> = Box::new(SqliteStore::open(path).unwrap());
        let mut doc = automerge::AutoCommit::new();
        reconcile(&mut doc, value).unwrap();
        store.write(id, &doc.save()).unwrap();
        let bytes = store.read(id).unwrap().expect("row present");
        let loaded = automerge::AutoCommit::load(&bytes).unwrap();
        let back: T = hydrate(&loaded).unwrap();
        assert_eq!(&back, value);
        let conn = rusqlite::Connection::open(path).unwrap();
        let fmt: String = conn
            .query_row("SELECT format FROM docs WHERE id = ?1", rusqlite::params![id], |r| r.get(0))
            .unwrap();
        assert_eq!(fmt, storage::FORMAT_AUTOMERGE_SAVE_V1);
    }

    /// Run 32 done-when (1 of 3): profile doc round-trips under the trait.
    #[test]
    fn profile_doc_round_trips_under_trait() {
        use crate::docs::*;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p1-profile.sqlite").to_string_lossy().to_string();
        let mut trust = HashMap::new();
        trust.insert(
            "contact-1".into(),
            TrustEntry { tier: "close".into(), connected_at: "2026-09-21T00:00:00Z".into(), sync_status: "synced".into() },
        );
        let profile = ProfileDoc {
            identity: Identity {
                display_name: "Jedi".into(),
                handle: "@jedi".into(),
                handle_registered_at: "2026-09-21T00:00:00Z".into(),
                avatar_color: "#3a7".into(),
                created_at: "2026-09-21T00:00:00Z".into(),
            },
            preferences: Preferences { default_ping_type: "here".into(), notifications_enabled: true, discoverable: false },
            trust_graph: trust,
            ping_history: vec![Ping {
                ping_id: "p1".into(),
                ping_type: "thinking-of-you".into(),
                sender_id: "self".into(),
                sent_at: "2026-09-21T01:00:00Z".into(),
                expires_at: "2026-09-21T02:00:00Z".into(),
                content: None,
            }],
            channel_memberships: vec![ChannelMembership {
                channel_id: "ch-local-first".into(),
                joined_at: "2026-09-21T00:30:00Z".into(),
                last_ping_at: "2026-09-21T01:00:00Z".into(),
            }],
        };
        round_trip_typed(&path, "profile", &profile);
    }

    /// Run 32 done-when (2 of 3): pings doc round-trips under the trait.
    #[test]
    fn pings_doc_round_trips_under_trait() {
        use crate::docs::*;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p1-pings.sqlite").to_string_lossy().to_string();
        let mut channels = HashMap::new();
        channels.insert(
            "ch-local-first".into(),
            vec![
                Ping {
                    ping_id: "p2".into(),
                    ping_type: "check-this".into(),
                    sender_id: "contact-1".into(),
                    sent_at: "2026-09-21T01:10:00Z".into(),
                    expires_at: "2026-09-21T03:10:00Z".into(),
                    content: Some("subduction thread".into()),
                },
                Ping {
                    ping_id: "p3".into(),
                    ping_type: "here".into(),
                    sender_id: "self".into(),
                    sent_at: "2026-09-21T01:12:00Z".into(),
                    expires_at: "2026-09-21T02:12:00Z".into(),
                    content: None,
                },
            ],
        );
        round_trip_typed(&path, "pings", &PingsDoc { channels });
    }

    /// Run 32 done-when (3 of 3): threads doc round-trips under the trait.
    #[test]
    fn threads_doc_round_trips_under_trait() {
        use crate::docs::*;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p1-threads.sqlite").to_string_lossy().to_string();
        let mut threads = HashMap::new();
        threads.insert(
            "contact-1".into(),
            vec![Message {
                message_id: "m1".into(),
                sender_id: "contact-1".into(),
                sent_at: "2026-09-21T01:20:00Z".into(),
                content: "elevating to a thread".into(),
                asset_ref: Some("asset-9".into()),
                read_at: None,
            }],
        );
        round_trip_typed(&path, "threads", &ThreadsDoc { threads });
    }

    /// Run 32 done-when (Task 3): counter increments on a simulated
    /// membership event through the FFI surface (`init_core` + exported
    /// methods — the same path the shells call).
    #[test]
    fn membership_counter_increments_via_ffi_surface() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("p1-membership.sqlite").to_string_lossy().to_string();
        let core = init_core(path).unwrap();
        assert_eq!(core.membership_version("group-a".into()), 0, "no events yet");
        assert_eq!(core.record_membership_event("group-a".into()), 1);
        assert_eq!(core.record_membership_event("group-a".into()), 2);
        assert_eq!(core.membership_version("group-a".into()), 2);
        assert_eq!(core.membership_version("group-b".into()), 0, "per-group isolation");
    }

    // ---- Run 33 done-when: each doc type crosses the FFI boundary and
    // round-trips from a shell's perspective — through `init_core` and the
    // exported typed methods only, across a process-style core restart on
    // the same SQLite file (what kill → relaunch does on device).

    fn sample_profile() -> docs::ProfileDoc {
        use crate::docs::*;
        let mut trust = HashMap::new();
        trust.insert(
            "contact-1".into(),
            TrustEntry { tier: "close".into(), connected_at: "2026-09-21T00:00:00Z".into(), sync_status: "synced".into() },
        );
        trust.insert(
            "contact-2".into(),
            TrustEntry { tier: "contact".into(), connected_at: "2026-09-21T00:05:00Z".into(), sync_status: "pending".into() },
        );
        ProfileDoc {
            identity: Identity {
                display_name: "Jedi".into(),
                handle: "@jedi".into(),
                handle_registered_at: "2026-09-21T00:00:00Z".into(),
                avatar_color: "#3a7".into(),
                created_at: "2026-09-21T00:00:00Z".into(),
            },
            preferences: Preferences { default_ping_type: "here".into(), notifications_enabled: true, discoverable: false },
            trust_graph: trust,
            ping_history: vec![Ping {
                ping_id: "p1".into(),
                ping_type: "thinking-of-you".into(),
                sender_id: "self".into(),
                sent_at: "2026-09-21T01:00:00Z".into(),
                expires_at: "2026-09-22T01:00:00Z".into(),
                content: None,
            }],
            channel_memberships: vec![ChannelMembership {
                channel_id: "ch-local-first".into(),
                joined_at: "2026-09-21T00:30:00Z".into(),
                last_ping_at: "2026-09-21T01:00:00Z".into(),
            }],
        }
    }

    /// Run 33 done-when (1 of 3): profile crosses the exported surface and
    /// survives a core restart on the same file.
    #[test]
    fn profile_round_trips_via_typed_ffi_surface() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run33-profile.sqlite").to_string_lossy().to_string();
        let want = sample_profile();
        {
            let core = init_core(path.clone()).unwrap();
            let h = core.open_typed_doc("profile".into(), DocKind::Profile).unwrap();
            assert_eq!(core.get_profile(h).unwrap(), docs::ProfileDoc::default(), "fresh doc is the typed default");
            core.put_profile(h, want.clone()).unwrap();
            core.save(h).unwrap();
        }
        let core = init_core(path).unwrap();
        let h = core.open_typed_doc("profile".into(), DocKind::Profile).unwrap();
        assert_eq!(core.get_profile(h).unwrap(), want);
    }

    /// Run 33 done-when (2 of 3): pings cross the exported surface, incl. an
    /// already-expired ping — the core returns it as stored (display-side
    /// ephemerality is the recorded scope; no expiry engine).
    #[test]
    fn pings_round_trip_via_typed_ffi_surface() {
        use crate::docs::*;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run33-pings.sqlite").to_string_lossy().to_string();
        let mut channels = HashMap::new();
        channels.insert(
            "ch-local-first".into(),
            vec![
                Ping {
                    ping_id: "p2".into(),
                    ping_type: "check-this".into(),
                    sender_id: "contact-1".into(),
                    sent_at: "2026-09-21T01:10:00Z".into(),
                    expires_at: "2026-09-28T01:10:00Z".into(),
                    content: Some("subduction thread".into()),
                },
                Ping {
                    ping_id: "p0".into(),
                    ping_type: "here".into(),
                    sender_id: "contact-2".into(),
                    sent_at: "2020-01-01T00:00:00Z".into(),
                    expires_at: "2020-01-02T00:00:00Z".into(),
                    content: None,
                },
            ],
        );
        let want = PingsDoc { channels };
        {
            let core = init_core(path.clone()).unwrap();
            let h = core.open_typed_doc("pings".into(), DocKind::Pings).unwrap();
            core.put_pings(h, want.clone()).unwrap();
            core.save(h).unwrap();
        }
        let core = init_core(path).unwrap();
        let h = core.open_typed_doc("pings".into(), DocKind::Pings).unwrap();
        let got = core.get_pings(h).unwrap();
        assert_eq!(got, want);
        assert_eq!(got.channels["ch-local-first"].len(), 2, "expired ping returned as stored — filtering is the shell's");
    }

    /// Run 33 done-when (3 of 3): threads cross the exported surface.
    #[test]
    fn threads_round_trip_via_typed_ffi_surface() {
        use crate::docs::*;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run33-threads.sqlite").to_string_lossy().to_string();
        let mut threads = HashMap::new();
        threads.insert(
            "contact-1".into(),
            vec![
                Message {
                    message_id: "m1".into(),
                    sender_id: "contact-1".into(),
                    sent_at: "2026-09-21T01:20:00Z".into(),
                    content: "elevating to a thread".into(),
                    asset_ref: Some("asset-9".into()),
                    read_at: None,
                },
                Message {
                    message_id: "m2".into(),
                    sender_id: "self".into(),
                    sent_at: "2026-09-21T01:21:00Z".into(),
                    content: "yes — let's".into(),
                    asset_ref: None,
                    read_at: Some("2026-09-21T01:22:00Z".into()),
                },
            ],
        );
        let want = ThreadsDoc { threads };
        {
            let core = init_core(path.clone()).unwrap();
            let h = core.open_typed_doc("threads".into(), DocKind::Threads).unwrap();
            core.put_threads(h, want.clone()).unwrap();
            core.save(h).unwrap();
        }
        let core = init_core(path).unwrap();
        let h = core.open_typed_doc("threads".into(), DocKind::Threads).unwrap();
        assert_eq!(core.get_threads(h).unwrap(), want);
    }

    /// Run 33 — the KV surface and the typed surface share one handle space
    /// and one store; a HelloDoc row and a typed row coexist under distinct
    /// ids (HelloDoc KEPT, Run 32 rule; additions-only this run).
    #[test]
    fn kv_and_typed_surfaces_coexist_in_one_store() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run33-coexist.sqlite").to_string_lossy().to_string();
        let core = init_core(path).unwrap();
        let hk = core.open_doc("note".into()).unwrap();
        core.put(hk, "text".into(), "hello phase 0".into()).unwrap();
        core.save(hk).unwrap();
        let hp = core.open_typed_doc("profile".into(), DocKind::Profile).unwrap();
        core.put_profile(hp, sample_profile()).unwrap();
        core.save(hp).unwrap();
        assert_ne!(hk, hp);
        assert_eq!(core.get(hk, "text".into()).unwrap().as_deref(), Some("hello phase 0"));
        assert_eq!(core.get_profile(hp).unwrap().identity.handle, "@jedi");
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

    /// Run 29 done-when (Run 27 wording): two consecutive resolvePds calls in one
    /// process — reuse of the shared runtime proven, not construct-per-call.
    /// Live network like the test above; `--ignored` on a machine with egress.
    #[test]
    #[ignore]
    fn resolve_pds_twice_in_one_process() {
        let core = Core::new(":memory:".into()).unwrap();
        let a = core.resolve_pds("did:plc:z72i7hdynmk6r22z27h6tvur".into()).unwrap();
        let b = core.resolve_pds("did:plc:z72i7hdynmk6r22z27h6tvur".into()).unwrap();
        assert!(a.starts_with("https://") && a == b);
    }
}
