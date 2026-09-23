//! lfs_core — Phase 0 skeleton (native-track-phase0-build-plan-v0-1 §2 B1–B5).
//! API surface is exactly the plan's B1 list. No delegation-level concept exists here (L-12 interim).
uniffi::setup_scaffolding!();

pub mod ceremony;
pub mod docs;
pub mod groups;
pub mod policy;
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
    /// Run 35 — the injected clock did not parse as RFC 3339. Raised by
    /// `open_typed_doc_at`; a bad clock is an error, never a silent no-op
    /// (an unparseable `expires_at` on a ping is the retained case, not
    /// this one).
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
    /// Run 37 — a consumer-policy refusal (`policy::PolicyViolation` text).
    /// Ruled for the delegation floor; the grant bar reuses it in Run 38.
    #[error("policy: {0}")]
    Policy(String),
    /// Run 37 — the identity ceremony could not complete (Keyhive-side
    /// failure, signature, or encoding). Never raised for a policy refusal.
    #[error("ceremony: {0}")]
    Ceremony(String),
    /// Run 37 — an identity row already exists; enrollment happens once.
    #[error("identity already enrolled")]
    IdentityAlreadyEnrolled,
    /// Run 38 — a Keyhive group/membership operation failed (group creation,
    /// peer registration, or `add_member` after the bar passed). Never raised
    /// for a policy refusal (that is `Policy`).
    #[error("membership: {0}")]
    Membership(String),
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

/// Run 36 (1b entry) — rooting level of the `identity` document (plan §4 H5;
/// memo v0.1.2 §6; spec v0.1.4 L-12). NOT RULED: memo §6 names the choice
/// and defers the ruling to after PR #230 (Keyline) merges. The level is
/// therefore SUPPLIED BY CONFIGURATION from the shell, never a constant in
/// Phase 1 code — there is deliberately no `Default` here. The two levels
/// are the two the memo names; their consequences (~ per memo §6): `Admin`
/// — the root edge grants Admin, any principal that has ever held apex
/// Admin can permanently brick the document; `Edit` — the root edge is
/// undeniable at Edit, a retained subject key can re-root out from under
/// old admins. The ceremony entry point (Run 37) takes this via
/// `IdentityConfig`, records the configured level in its log entry, and
/// cites L-12's floor-withdrawal caveat (`policy::delegation_floor_status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum RootingLevel {
    Admin,
    Edit,
}

/// Run 36 — the configuration path by which the rooting level reaches the
/// core. Crosses FFI as a record so each shell supplies it explicitly at
/// ceremony time (H5: "supplied by configuration, not a constant"). Held by
/// no `Core` field and consulted by no entry point yet: Run 37 adds the
/// ceremony entry point that accepts it. Additive to the FFI surface; the
/// Run 35 surface is unchanged.
/// Run 37: that entry point is `Core::run_identity_ceremony`; the record
/// crosses FFI unchanged (bindings additions-only, Run 35 docstring rule).
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct IdentityConfig {
    pub rooting_level: RootingLevel,
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
    /// Run 38 — the device hive behind the membership counter (Run 32's
    /// in-memory `membership` map is retired: the counter now reads Keyhive
    /// group membership — `groups` module docs). Lazily a session-only hive
    /// until the identity ceremony runs in this process (then rebuilt from
    /// the persisted rows). In-memory: group persistence is Run 39's.
    device: Mutex<Option<groups::DeviceHive>>,
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
        // Run 35: signature and behaviour unchanged (no clock supplied, so
        // no cleanup step runs — library code never reads the wall).
        self.open_typed_doc_inner(id, kind, None)
    }

    /// Run 35 (1a hardening) — `open_typed_doc` with the cleanup-on-load
    /// step (spec: "a cleanup observer removes expired entries on document
    /// load"). `now` is the injected clock, RFC 3339 (`Z` or numeric
    /// offset; fractional seconds accepted) — the caller supplies it, the
    /// core never reads the wall. For `kind = Pings`, every ping whose
    /// `expires_at` parses and is strictly earlier than `now` is deleted
    /// from its channel list in the opened document; a ping whose
    /// `expires_at` does not parse is retained. For other kinds `now` is
    /// validated and otherwise unused. The cleanup edits the in-memory
    /// document; like `put_*`, it reaches SQLite on the next `save`.
    /// Returns the handle; the removed count is not part of the surface.
    pub fn open_typed_doc_at(&self, id: String, kind: DocKind, now: String) -> Result<u64, CoreError> {
        let clock = docs::parse_rfc3339_utc(&now).ok_or_else(|| CoreError::InvalidTimestamp(now.clone()))?;
        self.open_typed_doc_inner(id, kind, Some(clock))
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
    /// Run 38: backed by Keyhive group membership (plan §9 row 4) — the
    /// version is the group's member count minus the device's own root
    /// membership; 0 when no group exists under `group_id`.
    pub fn membership_version(&self, group_id: String) -> u64 {
        self.device.lock().unwrap().as_ref().map(|d| d.membership_version(&group_id)).unwrap_or(0)
    }

    /// Run 32 — membership-version counter, event side. Records one
    /// (simulated) membership event for `group_id` and returns the new
    /// version. 1b replaces the simulation with real Keyhive membership
    /// events driving the same interface; the signature is the seam.
    /// Run 38: a real Keyhive membership event — the device creates the group
    /// on first use (device = Admin) and grants one simulated peer at `Read`
    /// through the H2 bar (`groups` module docs). The signature is unchanged
    /// (the seam); an event that fails to land leaves the version unchanged
    /// and the typed, error-bearing form is `grant_member`.
    pub fn record_membership_event(&self, group_id: String) -> u64 {
        match self.grant_member(group_id.clone(), policy::GrantLevel::Read) {
            Ok(v) => v,
            Err(_) => self.membership_version(group_id),
        }
    }

    /// Run 38 — the typed membership event: grant one simulated peer at
    /// `level` on the group named `group_id` (created by the device on first
    /// use), through the H2 grant bar (`policy::check_grant_bar`, first
    /// caller; a below-Admin granter is refused as `CoreError::Policy` before
    /// core is invoked). Returns the new membership version. Blocking;
    /// dispatch off-main like the ceremony.
    pub fn grant_member(&self, group_id: String, level: policy::GrantLevel) -> Result<u64, CoreError> {
        let mut guard = self.device.lock().unwrap();
        if guard.is_none() {
            *guard = Some(groups::DeviceHive::session_only()?);
        }
        guard.as_mut().expect("device hive present").record_membership_event(&group_id, level)
    }

    /// Run 38 — the device's own grant level on the group named `group_id`
    /// (`Admin` on every group it created), or `None` when no such group.
    pub fn device_grant_level(&self, group_id: String) -> Option<policy::GrantLevel> {
        self.device.lock().unwrap().as_ref().and_then(|d| d.device_grant_level(&group_id))
    }

    /// Run 37 — the identity ceremony (plan §9 row 3; `ceremony` module docs).
    /// Takes the rooting level as configuration (H5), creates the `identity`
    /// document with two admin delegations, enforces the floor as consumer
    /// policy (H2), persists the delegation proof bytes under the first
    /// Keyhive format tag, and returns the cold material once. Blocking;
    /// shells dispatch it off-main like `init_core`. Refuses a second
    /// enrollment (`IdentityAlreadyEnrolled`).
    /// Run 38: also enrolls the device (third delegation, `Edit`), persists the
    /// admin/device `KeyOp` row, and rebuilds the device hive from the two
    /// persisted rows (D-38-4 reload path) — the hive the counter runs on
    /// from here; groups created on a pre-ceremony session hive are dropped.
    pub fn run_identity_ceremony(&self, config: IdentityConfig) -> Result<ceremony::IdentityCeremonyRecord, CoreError> {
        let (record, material) = ceremony::run_retaining_device(self.store.as_ref(), config)?;
        let hive = groups::DeviceHive::from_ceremony(self.store.as_ref(), material)?;
        *self.device.lock().unwrap() = Some(hive);
        Ok(record)
    }

    /// Pin attestation for the observation log.
    pub fn pins(&self) -> String {
        // Run 36 H1 housekeeping (Run 30 flag): the Keyhive pin label is owned
        // by the storage adapter (storage::PIN_LABEL_KEYHIVE_CORE). Run 37
        // re-ruled its text (version + git rev) — shells' footer changes.
        format!(
            "{} automerge=0.12 samod=0.14 autosurgeon=0.14 atrium-api=0.25 subduction={}",
            storage::PIN_LABEL_KEYHIVE_CORE,
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
            device: Mutex::new(None),
        })
    }

    /// Shared open path (Run 33 body, unchanged) plus the Run 35 cleanup
    /// step, which runs only when a clock is supplied and the kind is Pings.
    fn open_typed_doc_inner(&self, id: String, kind: DocKind, now: Option<docs::UtcInstant>) -> Result<u64, CoreError> {
        let mut doc = match self.store.read(&id).map_err(|e| CoreError::Storage(e.to_string()))? {
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
        if let (DocKind::Pings, Some(clock)) = (kind, now) {
            remove_expired_pings(&mut doc, clock).map_err(CoreError::Automerge)?;
        }
        Ok(self.insert(Open { id, doc }))
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

/// Run 35 — the cleanup-on-load step. Walks `channels` → each channel list
/// → each ping's `expires_at` and deletes expired entries in place with
/// Automerge list deletes (not a whole-document reconcile), so the op
/// history records exactly the removals — the property that matters once
/// 1b sync arrives. Anything not shaped as `PingsDoc` (a missing
/// `channels` map, a non-list channel, a ping without a string
/// `expires_at`) is left untouched rather than errored: the shape is not
/// re-validated on open (Run 33 rule — the doc id is the shell's contract).
/// Returns the number of pings removed.
fn remove_expired_pings(doc: &mut automerge::AutoCommit, now: docs::UtcInstant) -> Result<u64, String> {
    use automerge::{transaction::Transactable, ReadDoc, Value, ROOT};
    let Some((Value::Object(automerge::ObjType::Map), channels)) =
        doc.get(ROOT, "channels").map_err(|e| e.to_string())?
    else {
        return Ok(0);
    };
    let channel_ids: Vec<String> = doc.keys(&channels).collect();
    let mut removed = 0u64;
    for ch in channel_ids {
        let Some((Value::Object(automerge::ObjType::List), list)) =
            doc.get(&channels, ch.as_str()).map_err(|e| e.to_string())?
        else {
            continue;
        };
        // walk from the end so deletes do not shift the indices still to visit
        for idx in (0..doc.length(&list)).rev() {
            let Some((Value::Object(automerge::ObjType::Map), ping)) =
                doc.get(&list, idx).map_err(|e| e.to_string())?
            else {
                continue;
            };
            let expires_at = match doc.get(&ping, "expires_at").map_err(|e| e.to_string())? {
                Some((Value::Scalar(v), _)) => match v.as_ref() {
                    automerge::ScalarValue::Str(s) => s.to_string(),
                    _ => continue,
                },
                _ => continue,
            };
            if docs::ping_expiry(&expires_at, now) == Some(true) {
                doc.delete(&list, idx).map_err(|e| e.to_string())?;
                removed += 1;
            }
        }
    }
    Ok(removed)
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

    /// Run 38 done-when (plan §9 row 4), through the FFI surface: the Run 32
    /// counter contract holds un-enrolled (session hive); `GrantLevel`
    /// crosses; the ceremony replaces the session hive with the reloaded
    /// identity hive (session groups dropped — in-memory by design); the
    /// typed event lands through the bar on the device's own group.
    #[test]
    fn groups_behind_the_counter_via_ffi_surface() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run38-groups.sqlite").to_string_lossy().to_string();
        let core = init_core(path).unwrap();
        assert_eq!(core.device_grant_level("g".into()), None);
        assert_eq!(core.record_membership_event("g".into()), 1);
        assert_eq!(core.device_grant_level("g".into()), Some(policy::GrantLevel::Admin));
        assert_eq!(core.grant_member("g".into(), policy::GrantLevel::Edit).unwrap(), 2);
        assert_eq!(core.membership_version("g".into()), 2);
        let rec = core.run_identity_ceremony(IdentityConfig { rooting_level: RootingLevel::Edit }).unwrap();
        assert_eq!(rec.admin_delegations, 2);
        assert_eq!(rec.device_key.device_secret.len(), 32);
        assert_eq!(rec.keyops_row_id, ceremony::IDENTITY_KEYOPS_ROW_ID);
        assert_eq!(core.membership_version("g".into()), 0, "session-hive groups do not survive the ceremony");
        assert_eq!(core.grant_member("g".into(), policy::GrantLevel::Read).unwrap(), 1);
        assert_eq!(core.device_grant_level("g".into()), Some(policy::GrantLevel::Admin));
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

    // ------------------------------------------------------------------
    // Run 35 — cleanup-on-load (1a hardening; closes the Run 33 §4.4
    // deferral). Clock always injected; nothing below reads the wall.
    // ------------------------------------------------------------------

    fn ping(id: &str, expires_at: &str) -> docs::Ping {
        docs::Ping {
            ping_id: id.into(),
            ping_type: "here".into(),
            sender_id: "contact-1".into(),
            sent_at: "2026-09-22T00:00:00Z".into(),
            expires_at: expires_at.into(),
            content: None,
        }
    }

    /// Seeds a pings doc under `id` with one expired, one active and one
    /// unparseable-expiry ping in `ch-a`, plus one active ping in `ch-b`,
    /// and saves it. Returns the path.
    fn seed_pings(dir: &tempfile::TempDir, file: &str) -> String {
        let path = dir.path().join(file).to_string_lossy().to_string();
        let mut channels = HashMap::new();
        channels.insert(
            "ch-a".into(),
            vec![
                ping("expired", "2026-09-22T11:59:59Z"),
                ping("active", "2026-09-29T12:00:00Z"),
                ping("garbled", "next tuesday"),
            ],
        );
        channels.insert("ch-b".into(), vec![ping("active-b", "2026-09-23T12:00:00.500Z")]);
        let core = init_core(path.clone()).unwrap();
        let h = core.open_typed_doc("pings".into(), DocKind::Pings).unwrap();
        core.put_pings(h, docs::PingsDoc { channels }).unwrap();
        core.save(h).unwrap();
        path
    }

    fn ids(doc: &docs::PingsDoc, ch: &str) -> Vec<String> {
        doc.channels[ch].iter().map(|p| p.ping_id.clone()).collect()
    }

    /// Run 35 done-when (1–3 of 5): after `open_typed_doc_at` a seeded
    /// expired ping is absent, an active one survives, an unparseable
    /// `expires_at` is retained.
    #[test]
    fn expired_ping_removed_on_load_active_and_unparseable_retained() {
        let dir = tempfile::tempdir().unwrap();
        let path = seed_pings(&dir, "run35-cleanup.sqlite");
        let core = init_core(path).unwrap();
        let h = core.open_typed_doc_at("pings".into(), DocKind::Pings, "2026-09-22T12:00:00Z".into()).unwrap();
        let got = core.get_pings(h).unwrap();
        assert_eq!(ids(&got, "ch-a"), vec!["active", "garbled"], "expired absent; active + unparseable retained, order kept");
        assert_eq!(ids(&got, "ch-b"), vec!["active-b"]);
        assert_eq!(got.channels["ch-a"][1].expires_at, "next tuesday", "unparseable expires_at retained verbatim");
    }

    /// The clock is the parameter, not the wall: the same stored document
    /// yields different survivors under different injected `now` values,
    /// and `open_typed_doc` (no clock) still returns every ping as stored.
    #[test]
    fn cleanup_follows_injected_clock_not_wall() {
        let dir = tempfile::tempdir().unwrap();
        let path = seed_pings(&dir, "run35-clock.sqlite");
        let core = init_core(path).unwrap();
        // clock before every expiry: nothing removed
        let h0 = core.open_typed_doc_at("pings".into(), DocKind::Pings, "2026-09-22T00:00:00Z".into()).unwrap();
        assert_eq!(ids(&core.get_pings(h0).unwrap(), "ch-a"), vec!["expired", "active", "garbled"]);
        // Kotlin-shaped fractional clock at the exact ch-b expiry: strict comparison, not expired yet
        let h1 = core.open_typed_doc_at("pings".into(), DocKind::Pings, "2026-09-23T12:00:00.500000000Z".into()).unwrap();
        assert_eq!(ids(&core.get_pings(h1).unwrap(), "ch-b"), vec!["active-b"]);
        // one nanosecond past it, with an offset-form clock: expired
        let h2 = core.open_typed_doc_at("pings".into(), DocKind::Pings, "2026-09-23T08:00:00.500000001-04:00".into()).unwrap();
        assert!(core.get_pings(h2).unwrap().channels["ch-b"].is_empty());
        // far future: only the unparseable one survives
        let h3 = core.open_typed_doc_at("pings".into(), DocKind::Pings, "2030-01-01T00:00:00Z".into()).unwrap();
        assert_eq!(ids(&core.get_pings(h3).unwrap(), "ch-a"), vec!["garbled"]);
        // Run 33 surface unchanged: no clock, no cleanup
        let h4 = core.open_typed_doc("pings".into(), DocKind::Pings).unwrap();
        assert_eq!(ids(&core.get_pings(h4).unwrap(), "ch-a"), vec!["expired", "active", "garbled"]);
    }

    /// Cleanup edits the opened document; it reaches SQLite on `save`
    /// (same contract as `put_*`), and a later plain open sees the
    /// smaller document.
    #[test]
    fn cleanup_persists_on_save_and_survives_core_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = seed_pings(&dir, "run35-persist.sqlite");
        {
            let core = init_core(path.clone()).unwrap();
            let h = core.open_typed_doc_at("pings".into(), DocKind::Pings, "2026-09-22T12:00:00Z".into()).unwrap();
            core.save(h).unwrap();
        }
        let core = init_core(path).unwrap();
        let h = core.open_typed_doc("pings".into(), DocKind::Pings).unwrap();
        assert_eq!(ids(&core.get_pings(h).unwrap(), "ch-a"), vec!["active", "garbled"]);
    }

    /// A clock that does not parse is an error, not a silent no-op; the
    /// document is not opened. Non-Pings kinds validate the clock and are
    /// otherwise untouched; a fresh Pings doc opens empty.
    #[test]
    fn invalid_clock_is_an_error_and_other_kinds_are_untouched() {
        let core = Core::new(":memory:".into()).unwrap();
        match core.open_typed_doc_at("pings".into(), DocKind::Pings, "now-ish".into()) {
            Err(CoreError::InvalidTimestamp(s)) => assert_eq!(s, "now-ish"),
            other => panic!("expected InvalidTimestamp, got {other:?}"),
        }
        assert!(matches!(
            core.open_typed_doc_at("profile".into(), DocKind::Profile, "".into()),
            Err(CoreError::InvalidTimestamp(_))
        ));
        let hp = core.open_typed_doc_at("profile".into(), DocKind::Profile, "2026-09-22T12:00:00Z".into()).unwrap();
        assert_eq!(core.get_profile(hp).unwrap(), docs::ProfileDoc::default());
        let hq = core.open_typed_doc_at("pings".into(), DocKind::Pings, "2026-09-22T12:00:00Z".into()).unwrap();
        assert_eq!(core.get_pings(hq).unwrap(), docs::PingsDoc::default());
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

    /// Run 37 — the ceremony crosses the exported surface through `init_core`
    /// (the path both shells call): record fields populated, the identity row
    /// survives a core restart under its tag, the Automerge rows are untouched.
    #[test]
    fn identity_ceremony_via_ffi_surface_persists_and_survives_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run37-ceremony.sqlite").to_string_lossy().to_string();
        let rec = {
            let core = init_core(path.clone()).unwrap();
            let hp = core.open_typed_doc("profile".into(), DocKind::Profile).unwrap();
            core.put_profile(hp, sample_profile()).unwrap();
            core.save(hp).unwrap();
            core.run_identity_ceremony(IdentityConfig { rooting_level: RootingLevel::Edit }).unwrap()
        };
        assert_eq!(rec.admin_delegations, 2);
        assert_eq!(rec.floor_status, policy::FloorStatus::Safe);
        let core = init_core(path.clone()).unwrap();
        assert!(matches!(
            core.run_identity_ceremony(IdentityConfig { rooting_level: RootingLevel::Edit }),
            Err(CoreError::IdentityAlreadyEnrolled)
        ));
        let hp = core.open_typed_doc("profile".into(), DocKind::Profile).unwrap();
        assert_eq!(core.get_profile(hp).unwrap().identity.handle, "@jedi", "automerge row untouched");
        let conn = rusqlite::Connection::open(&path).unwrap();
        let tags: Vec<(String, String)> = conn
            .prepare("SELECT id, format FROM docs ORDER BY id")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert_eq!(
            tags,
            vec![
                ("identity".to_string(), storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1.to_string()),
                // Run 38 (D-38-4): the KeyOp row — a SECOND test-line edit,
                // not named in the kickoff; a direct consequence of the ruled row.
                ("identity-keyops".to_string(), storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1.to_string()),
                ("profile".to_string(), storage::FORMAT_AUTOMERGE_SAVE_V1.to_string()),
            ]
        );
        assert_eq!(core.pins().split(' ').next().unwrap(), storage::PIN_LABEL_KEYHIVE_CORE);
        assert!(storage::PIN_LABEL_KEYHIVE_CORE.ends_with("+90fe4a51"), "label carries the git rev (Run 37 re-rule)");
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
