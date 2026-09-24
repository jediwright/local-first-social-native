//! Identity ceremony — Run 37, Phase 1 Build Plan v0.1.1 §9 row 3.
//!
//! Creates the participant's `identity` document on the pinned `keyhive_core`
//! (`90fe4a51`, H6) and is the FIRST caller of the consumer-policy delegation
//! floor (`policy::check_delegation_floor`). Everything that names Keyhive
//! outside the storage adapter and the policy module lives in this one file,
//! so H1's grep names exactly three modules from Run 37 on: `storage`
//! (owns the persisted bytes), `policy` (owns the rules), `ceremony` (owns
//! the one ceremony that produces the bytes the rules govern).
//!
//! What the ceremony does (spec v0.1.4 §3.2, §5.2; memo v0.1.2 §6; plan §4
//! H2/H3/H5):
//!
//! 1. Generates two cold admin signing keys in memory — the PRIMARY admin key
//!    and the RECOVERY key (spec §5.2: the recovery key is itself an admin
//!    delegation, not a data-recovery secret).
//! 2. Creates the `identity` document with `keyhive_core`'s own ceremony:
//!    `Document::generate` runs `EphemeralSigner::with_signer`, so the root
//!    signing key is destroyed at creation (spec §3.2: "the root cannot be
//!    lost or stolen as a key; only delegations can be"); every parent named
//!    at generation receives an `Admin` delegation from that ephemeral root
//!    (source read at the pin: `principal/group.rs`, `generate_after_content`).
//!    The primary key is the document's active parent and the recovery key its
//!    co-parent → exactly two admin delegations.
//! 3. Counts the Admin delegations the document actually holds and passes the
//!    count through `policy::check_delegation_floor` — the floor is enforced
//!    by THIS app's policy, not by core (H2). A refusal maps onto
//!    `CoreError::Policy` (ruled Run 37, floor only; the grant bar is Run 38).
//! 4. Persists the document's delegation heads — `Signed<StaticDelegation>`,
//!    the public proof bytes and nothing else — through the storage adapter
//!    under the new format tag `storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1`
//!    (the first Keyhive bytes in SQLite). The encoding is `bincode` over
//!    `keyhive_core`'s serde shape at `90fe4a51`; the tag is the seam that
//!    absorbs a change there (T4 re-encode lands under a new tag value; rows
//!    are never re-tagged — Run 30 rule). The full `Archive` was considered
//!    and NOT chosen: it carries the active agent's prekey SECRETS, and the
//!    active agent here is the cold primary admin.
//! 5. Returns the cold material to the caller as `ColdKeyExport` — produced,
//!    exported, never written by the core (plan §9 row 3 "cold key
//!    off-device"). Custody past the FFI boundary is the shell's.
//!
//! Rooting level (H5, memo §6, spec L-12): the level arrives from
//! configuration (`IdentityConfig`) and is RECORDED, together with
//! `policy::delegation_floor_status(level)`, in the returned record. It is
//! not enacted: `keyhive_core` at the pin has no rooting-level concept (that
//! is Keyline, PR #230, unmerged — S-3). Under `Admin` rooting the record
//! carries `FloorStatus::WithdrawnPendingL12`, i.e. spec L-12's caveat: the
//! floor is withdrawn under Admin-rooting unless the recovery delegation is a
//! FROST 2-of-3 share (not modelled in Phase 1).
//!
//! H3: the Keyhive document ID never crosses FFI and appears in no log
//! path; the record carries the storage row id.
//!
//! Run 38 (plan v0.1.1 §9 row 4; D-38 record `1a354226…` §2, rulings
//! CARRIED; amendment pack `473ed745…` §0 OR-1/OR-4):
//!
//! 6. D-38-3 / OR-1 — a THIRD delegation on the identity document: the DEVICE
//!    key at `Edit`, issued by the primary admin (the active agent) with
//!    `add_member` after `generate_doc` and before the heads are collected.
//!    The device's `Individual` is introduced by contact card exactly as the
//!    recovery key is. `admin_delegations` still counts `can == Admin` only,
//!    so the floor count stays at two; the persisted identity row now holds
//!    three delegations under the SAME v1 tag (same serde type; `can` is a
//!    field). Device-seed source ruled A1 in-run: the ceremony generates the
//!    device `MemorySigner` and its seed crosses FFI once as
//!    `DeviceKeyExport` beside the cold keys; the seed never lands in SQLite.
//! 7. D-38-4 / OR-4 — the admin `KeyOp`s (the primary and recovery contact
//!    cards) plus the device's own are persisted as a second row under a NEW
//!    tag, `storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1`, as
//!    `Vec<StaticEvent<[u8; 32]>>` (`PrekeysExpanded` variants) — the type
//!    `ingest_unsorted_static_events` consumes, not the v1 delegation type.
//!    Without them a device-active hive cannot resolve the two cold admin
//!    delegates (`UnknownAgent`); with them the identity document reloads
//!    from the two rows into a hive whose active agent is the device
//!    (`reload_with`), which is the path Run 40's recovery re-uses with a
//!    re-imported admin as active agent. The test
//!    `identity_row_reloads_into_device_hive` decides whether the reload
//!    lifts here (green) or at Run 39 (red) — OR-4.

use crate::policy::{self, FloorStatus};
use crate::storage::{self, DocStore};
use crate::{CoreError, IdentityConfig, RootingLevel};
use future_form::Sendable;
use keyhive_core::access::Access;
use keyhive_core::keyhive::Keyhive;
use keyhive_core::listener::no_listener::NoListener;
use keyhive_core::principal::group::delegation::StaticDelegation;
use keyhive_core::store::ciphertext::memory::MemoryCiphertextStore;
use keyhive_crypto::signed::Signed;
use keyhive_crypto::signer::memory::MemorySigner;
use keyhive_crypto::verifiable::Verifiable;
use nonempty::nonempty;
use rand::rngs::OsRng;
// Run 38 — same crate, same rev (D-38-4 "not a new dependency"): the event
// and id types the reload path ingests and resolves.
use keyhive_core::contact_card::ContactCard;
use keyhive_core::event::static_event::StaticEvent;
use keyhive_core::principal::document::id::DocumentId;
use keyhive_core::principal::identifier::Identifier;
use keyhive_core::principal::individual::op::KeyOp;

/// Storage row id of the persisted identity delegations. A stable, app-owned
/// name; NOT the Keyhive document ID (H3).
pub const IDENTITY_ROW_ID: &str = "identity";

/// Run 38 — storage row id of the persisted admin/device `KeyOp`s (D-38-4).
/// App-owned name; NOT a Keyhive identifier (H3).
pub const IDENTITY_KEYOPS_ROW_ID: &str = "identity-keyops";

/// The initial content head the identity document is generated over. Run 37
/// binds no content to the document yet (the document's automerge content is
/// later work); the zero digest is the pinned crate's own placeholder shape
/// (its tests generate documents over `[0u8; 32]`).
const INITIAL_CONTENT_HEAD: [u8; 32] = [0u8; 32];

type Hive = Keyhive<Sendable, MemorySigner>;
type StaticDelegations = Vec<Signed<StaticDelegation<[u8; 32]>>>;
/// Run 38 — the serde type persisted under `FORMAT_KEYHIVE_STATIC_EVENTS_V1`.
type StaticEvents = Vec<StaticEvent<[u8; 32]>>;

/// Cold/admin material produced by the ceremony. Crosses FFI once; the core
/// keeps no copy and writes none of it. Secrets are the 32-byte Ed25519
/// seeds; fingerprints are the lowercase hex of the matching verifying keys
/// (what the persisted delegations name as their delegates).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct ColdKeyExport {
    pub primary_admin_secret: Vec<u8>,
    pub primary_admin_fingerprint: String,
    pub recovery_secret: Vec<u8>,
    pub recovery_fingerprint: String,
}

/// Run 38 (D-38-3, device-seed source A1) — the device key material. Crosses
/// FFI once beside `ColdKeyExport`; the core keeps the SIGNER in memory for
/// this process (it is the active agent of the device hive behind the
/// membership counter) and writes the seed nowhere. Custody past the FFI
/// boundary is the shell's (spec §5.2 hardware-backed storage — an Ed25519
/// seed under hardware-protected storage; the pin's only signer is
/// `MemorySigner`, Run 37 §4.1).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct DeviceKeyExport {
    pub device_secret: Vec<u8>,
    pub device_fingerprint: String,
}

/// What the ceremony records (plan §4 H5 done-when: "ceremony run records
/// the configured level and cites L-12"). `floor_status` IS the L-12 citation
/// in code form (`policy::FloorStatus` docs).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct IdentityCeremonyRecord {
    /// Storage row the delegations were written under (`IDENTITY_ROW_ID`).
    pub identity_row_id: String,
    /// The configured rooting level, echoed — recorded, not enacted.
    pub rooting_level: RootingLevel,
    /// `policy::delegation_floor_status(rooting_level)` — spec L-12 / §5.2.
    pub floor_status: FloorStatus,
    /// Admin delegations the identity document holds (floor is 2).
    pub admin_delegations: u32,
    /// The app-owned format tag the row was written under.
    pub format_tag: String,
    /// Size of the persisted bytes (for the observation log; no bytes cross).
    pub persisted_bytes: u64,
    pub cold_keys: ColdKeyExport,
    /// Run 38 — the device key (third delegation, `Edit`), exported once.
    pub device_key: DeviceKeyExport,
    /// Run 38 — the row the `KeyOp`s were written under (`IDENTITY_KEYOPS_ROW_ID`).
    pub keyops_row_id: String,
    /// Run 38 — the app-owned tag of the `KeyOp` row.
    pub keyops_format_tag: String,
    /// Run 38 — size of the `KeyOp` row (observation log; no bytes cross).
    pub keyops_persisted_bytes: u64,
}

/// Run 38 — what the ceremony hands back to the core beside the record: the
/// device signer (retained in memory as the device hive's active agent) and
/// the identity document's id (never crosses FFI — H3). Crate-internal.
pub(crate) struct DeviceMaterial {
    pub(crate) signer: MemorySigner,
    pub(crate) identity_doc: DocumentId,
}

/// Run the ceremony against `store`. Refuses (without touching the store) if
/// an identity row already exists — enrollment happens once; recovery and
/// rotation are their own paths (spec §5.2/§6; Run 40).
/// Run 38: the core now calls `run_retaining_device`; this form is kept for
/// the Run 37 tests (test-only from Run 38).
#[cfg(test)]
pub(crate) fn run(store: &dyn DocStore, config: IdentityConfig) -> Result<IdentityCeremonyRecord, CoreError> {
    run_retaining_device(store, config).map(|(record, _)| record)
}

/// Run 38 — the ceremony as the core calls it: the record for the shell plus
/// the `DeviceMaterial` the core retains in memory (device hive active agent).
/// `run` keeps the Run 37 signature for its callers and tests.
pub(crate) fn run_retaining_device(
    store: &dyn DocStore,
    config: IdentityConfig,
) -> Result<(IdentityCeremonyRecord, DeviceMaterial), CoreError> {
    if store.read_tagged(IDENTITY_ROW_ID).map_err(|e| CoreError::Storage(e.to_string()))?.is_some() {
        return Err(CoreError::IdentityAlreadyEnrolled);
    }
    let primary = MemorySigner::generate(&mut OsRng);
    let recovery = MemorySigner::generate(&mut OsRng);
    // Run 38 (A1): the device signer is generated here, in memory.
    let device = MemorySigner::generate(&mut OsRng);
    enroll_with_device(store, config, primary, Some(recovery), device)
}

/// The ceremony body, parametric on the keys so the floor path can be
/// exercised: `recovery = None` generates a document with ONE admin
/// delegation, which the floor refuses before anything is persisted.
/// Run 38: test-only (the floor test); the product path is `enroll_with_device`.
#[cfg(test)]
pub(crate) fn enroll_with(
    store: &dyn DocStore,
    config: IdentityConfig,
    primary: MemorySigner,
    recovery: Option<MemorySigner>,
) -> Result<IdentityCeremonyRecord, CoreError> {
    let device = MemorySigner::generate(&mut OsRng);
    enroll_with_device(store, config, primary, recovery, device).map(|(record, _)| record)
}

/// Run 38 — the ceremony body with the device signer supplied (A1: generated
/// by `run_retaining_device`; the reload test supplies its own so it can
/// open the device hive afterwards).
pub(crate) fn enroll_with_device(
    store: &dyn DocStore,
    config: IdentityConfig,
    primary: MemorySigner,
    recovery: Option<MemorySigner>,
    device: MemorySigner,
) -> Result<(IdentityCeremonyRecord, DeviceMaterial), CoreError> {
    let primary_fp = hex(&primary.verifying_key().to_bytes());
    let recovery_fp = recovery.as_ref().map(|r| hex(&r.verifying_key().to_bytes())).unwrap_or_default();
    let device_fp = hex(&device.verifying_key().to_bytes());

    let generated = crate::runtime::rt().block_on(generate_identity_doc(primary.clone(), recovery.clone(), device.clone()))?;
    let (delegations, admin_count, keyops, identity_doc) =
        (generated.delegations, generated.admin_count, generated.keyops, generated.identity_doc);

    // H2 — the floor is this app's policy; core enforced nothing here.
    policy::check_delegation_floor(admin_count).map_err(|v| CoreError::Policy(v.to_string()))?;

    // Every head must verify before it is persisted — the row is proof bytes.
    for d in &delegations {
        d.try_verify().map_err(|e| CoreError::Ceremony(format!("delegation signature: {e}")))?;
    }
    let bytes = bincode::serialize(&delegations).map_err(|e| CoreError::Ceremony(format!("encode: {e}")))?;
    store
        .write_keyhive_static_delegations(IDENTITY_ROW_ID, &bytes)
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    // Run 38 (D-38-4): the KeyOps — public prekey material only — under the
    // new tag, stamped inside the adapter (H1). Every op verifies first.
    for ev in &keyops {
        if let StaticEvent::PrekeysExpanded(op) = ev {
            op.try_verify().map_err(|e| CoreError::Ceremony(format!("keyop signature: {e}")))?;
        }
    }
    let keyops_bytes = bincode::serialize(&keyops).map_err(|e| CoreError::Ceremony(format!("encode keyops: {e}")))?;
    store
        .write_keyhive_static_events(IDENTITY_KEYOPS_ROW_ID, &keyops_bytes)
        .map_err(|e| CoreError::Storage(e.to_string()))?;

    let cold_keys = ColdKeyExport {
        primary_admin_secret: primary.0.to_bytes().to_vec(),
        primary_admin_fingerprint: primary_fp,
        recovery_secret: recovery.map(|r| r.0.to_bytes().to_vec()).unwrap_or_default(),
        recovery_fingerprint: recovery_fp,
    };
    let record = IdentityCeremonyRecord {
        identity_row_id: IDENTITY_ROW_ID.to_string(),
        rooting_level: config.rooting_level,
        floor_status: policy::delegation_floor_status(config.rooting_level),
        admin_delegations: admin_count as u32,
        format_tag: storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1.to_string(),
        persisted_bytes: bytes.len() as u64,
        cold_keys,
        device_key: DeviceKeyExport { device_secret: device.0.to_bytes().to_vec(), device_fingerprint: device_fp },
        keyops_row_id: IDENTITY_KEYOPS_ROW_ID.to_string(),
        keyops_format_tag: storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1.to_string(),
        keyops_persisted_bytes: keyops_bytes.len() as u64,
    };
    Ok((record, DeviceMaterial { signer: device, identity_doc }))
}

/// Run 38 — what the Keyhive half returns.
struct Generated {
    delegations: StaticDelegations,
    admin_count: usize,
    keyops: StaticEvents,
    identity_doc: DocumentId,
}

/// The Keyhive half: an in-memory hive whose active agent is the primary
/// admin key; the recovery key's individual is introduced through a contact
/// card (the pinned crate's only path to register a peer) and named as
/// co-parent of the new document. Returns the document's delegation heads in
/// static form plus the number of them that grant `Admin`.
/// Run 38: also the device's individual (same contact-card path), which the
/// primary admin then delegates at `Edit` on the new document (D-38-3); and
/// the three `KeyOp`s as `PrekeysExpanded` static events (D-38-4).
async fn generate_identity_doc(
    primary: MemorySigner,
    recovery: Option<MemorySigner>,
    device: MemorySigner,
) -> Result<Generated, CoreError> {
    let cer = |e: &dyn std::fmt::Debug| CoreError::Ceremony(format!("{e:?}"));
    let admin_hive: Hive = Keyhive::generate(primary, MemoryCiphertextStore::new(), NoListener, OsRng)
        .await
        .map_err(|e| cer(&e))?;
    let mut keyops: StaticEvents = Vec::new();
    let primary_card = admin_hive.generate_contact_card().await.map_err(|e| cer(&e))?;
    keyops.push(keyop_event(&primary_card));

    let mut coparents = Vec::new();
    if let Some(recovery) = recovery {
        let recovery_hive: Hive = Keyhive::generate(recovery, MemoryCiphertextStore::new(), NoListener, OsRng)
            .await
            .map_err(|e| cer(&e))?;
        let card = recovery_hive.generate_contact_card().await.map_err(|e| cer(&e))?;
        let recovery_id = admin_hive.receive_contact_card(&card).await.map_err(|e| cer(&e))?;
        coparents.push(recovery_id.into());
        keyops.push(keyop_event(&card));
    }

    // Run 38 (D-38-3): the device's individual, introduced the same way.
    let device_hive: Hive = Keyhive::generate(device, MemoryCiphertextStore::new(), NoListener, OsRng)
        .await
        .map_err(|e| cer(&e))?;
    let device_card = device_hive.generate_contact_card().await.map_err(|e| cer(&e))?;
    let device_id = admin_hive.receive_contact_card(&device_card).await.map_err(|e| cer(&e))?;
    keyops.push(keyop_event(&device_card));

    let doc_id = admin_hive
        .generate_doc(coparents, nonempty![INITIAL_CONTENT_HEAD])
        .await
        .map_err(|e| cer(&e))?;
    // Run 38 (D-38-3 / OR-1): third delegation — device at `Edit`, issued by
    // the primary admin (active agent), after generate_doc, before the heads
    // are collected. Admin count is unaffected (`can == Admin` only).
    admin_hive
        .add_member(device_id, doc_id, Access::Edit, &[])
        .await
        .map_err(|e| cer(&e))?;
    let doc = admin_hive.get_document(doc_id).await.ok_or_else(|| CoreError::Ceremony("document vanished".into()))?;
    let doc = doc.lock().await;

    let mut statics: StaticDelegations = Vec::new();
    let mut admin_count = 0usize;
    // Run 38 FINDING (against D-38 record §1 (iii)-8 as read): once the
    // primary admin issues the device delegation, its own root delegation is
    // the device's PROOF and is no longer a HEAD — `delegation_heads()` drops
    // to two (recovery Admin + device Edit) and the floor would read ONE
    // Admin. The membership map keeps every delegation, so the row and the
    // count are taken from `members()`: all delegations (three: two Admin
    // roots + the device Edit, whose proof is the primary's root), and the
    // Admin count = members whose highest capability is Admin (still two).
    // Reload needs the proof delegation in the row anyway.
    for (member, delegations) in doc.members() {
        if doc.get_capability(member).map(|d| d.payload().can()) == Some(Access::Admin) {
            admin_count += 1;
        }
        for signed in delegations.iter() {
            let s: Signed<StaticDelegation<[u8; 32]>> = (**signed).clone().map(StaticDelegation::from);
            statics.push(s);
        }
    }
    // Deterministic order for byte-stable rows: by delegate identifier bytes.
    statics.sort_by(|a, b| a.payload().delegate.to_bytes().cmp(&b.payload().delegate.to_bytes()));
    Ok(Generated { delegations: statics, admin_count, keyops, identity_doc: doc_id })
}

/// Run 38 — a contact card is a `KeyOp`; the reload path ingests it as the
/// `PrekeysExpanded` static event (the card's op is always the add-key op).
/// Run 39 (source read at `90fe4a51`, keyhive.rs L332–341): the premise above
/// is wrong at the pin — `generate_contact_card` returns `KeyOp::Rotate`, so
/// the persisted events are `PrekeyRotated`; ingest accepts either, which is
/// why Run 38 reloaded. Readers match both variants (`groups::keyop_issuer`).
pub(crate) fn keyop_event(card: &ContactCard) -> StaticEvent<[u8; 32]> {
    match card.op() {
        KeyOp::Add(add) => StaticEvent::PrekeysExpanded(Box::new((**add).clone())),
        KeyOp::Rotate(rot) => StaticEvent::PrekeyRotated(Box::new((**rot).clone())),
    }
}

/// Run 38 (D-38-4 / OR-4) — rebuild the identity document inside a hive whose
/// active agent is `signer` (the device key today; a re-imported cold admin
/// at Run 40), from the two persisted rows and no other signer. Ingests the
/// `KeyOp`s first (so the cold delegates resolve), then the delegations; the
/// fixed-point ingest leaves nothing pending when the rows suffice. Returns
/// the hive and the identity document's id (never crosses FFI — H3).
pub(crate) fn reload_with(store: &dyn DocStore, signer: MemorySigner) -> Result<(Hive, DocumentId), CoreError> {
    let (tag, bytes) = store
        .read_tagged(IDENTITY_ROW_ID)
        .map_err(|e| CoreError::Storage(e.to_string()))?
        .ok_or_else(|| CoreError::Ceremony("no identity row".into()))?;
    if tag != storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1 {
        return Err(CoreError::Ceremony(format!("identity row under unexpected tag {tag}")));
    }
    let (ktag, kbytes) = store
        .read_tagged(IDENTITY_KEYOPS_ROW_ID)
        .map_err(|e| CoreError::Storage(e.to_string()))?
        .ok_or_else(|| CoreError::Ceremony("no identity keyops row".into()))?;
    if ktag != storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1 {
        return Err(CoreError::Ceremony(format!("keyops row under unexpected tag {ktag}")));
    }
    let delegations = decode(&bytes)?;
    let keyops = decode_events(&kbytes)?;
    crate::runtime::rt().block_on(reload_into(signer, keyops, delegations, own_op_by_issuer))
}

/// Run 40 (D-40-4) — the product own-op filter: drop the active agent's own
/// `KeyOp` by ISSUER, whichever variant carries it (`PrekeysExpanded` or
/// `PrekeyRotated`; rows hold the latter at the pin — note (o)). The Run 38
/// filter below it is kept, test-only, so B-5 can assert both reload to the
/// same `members()`.
fn own_op_by_issuer(ev: &StaticEvent<[u8; 32]>, own: &Identifier) -> bool {
    match ev {
        StaticEvent::PrekeysExpanded(op) => Identifier::from(op.issuer()) != *own,
        StaticEvent::PrekeyRotated(op) => Identifier::from(op.issuer()) != *own,
        _ => true,
    }
}

/// Run 38's filter, verbatim (matches `PrekeysExpanded` only) — retained for
/// the D-40-4 / B-5 assertion only.
#[cfg(test)]
fn own_op_run38(ev: &StaticEvent<[u8; 32]>, own: &Identifier) -> bool {
    match ev {
        StaticEvent::PrekeysExpanded(op) => Identifier::from(op.issuer()) != *own,
        _ => true,
    }
}

async fn reload_into(
    signer: MemorySigner,
    keyops: StaticEvents,
    delegations: StaticDelegations,
    keep_op: fn(&StaticEvent<[u8; 32]>, &Identifier) -> bool,
) -> Result<(Hive, DocumentId), CoreError> {
    let cer = |e: &dyn std::fmt::Debug| CoreError::Ceremony(format!("{e:?}"));
    let own: Identifier = (&signer.verifying_key()).into();
    let hive: Hive = Keyhive::generate(signer, MemoryCiphertextStore::new(), NoListener, OsRng)
        .await
        .map_err(|e| cer(&e))?;
    // The identity document's id is the issuer of its root delegations (the
    // destroyed ephemeral signer): recover it from the bytes, not from state.
    let root_issuer = delegations
        .iter()
        .find(|d| d.payload().proof.is_none())
        .map(|d| Identifier::from(d.issuer()))
        .ok_or_else(|| CoreError::Ceremony("identity row holds no root delegation".into()))?;
    let doc_id = DocumentId::from(root_issuer);
    let mut events: StaticEvents = keyops
        .into_iter()
        // the active agent's own KeyOp is already known to its hive
        // Run 40 (D-40-4): filtered by issuer, both variants (`own_op_by_issuer`)
        .filter(|ev| keep_op(ev, &own))
        .collect();
    events.extend(delegations.into_iter().map(StaticEvent::Delegated));
    let pending = hive.ingest_unsorted_static_events(events).await;
    if !pending.is_empty() {
        return Err(CoreError::Ceremony(format!("reload left {} event(s) pending: {pending:?}", pending.len())));
    }
    hive.get_document(doc_id).await.ok_or_else(|| CoreError::Ceremony("identity document not materialised".into()))?;
    Ok((hive, doc_id))
}

/// Run 40 — what the identity half of recovery hands back to the caller
/// (`groups::DeviceHive::recover`, which continues with the group rows).
/// Crate-internal; the hive is ADMIN-active and lives only for the recovery
/// call (D-40-3 step 6: no field outlives it).
pub(crate) struct Recovered {
    pub(crate) hive: Hive,
    pub(crate) doc_id: DocumentId,
    /// The imported admin's fingerprint (lowercase hex; crosses FFI in the
    /// `RecoveryReport`).
    pub(crate) admin_fingerprint: String,
    /// `members()` of the identity document after the re-delegation.
    pub(crate) members: u32,
    pub(crate) new_device: Identifier,
    /// Every `KeyOp` event now in the `identity-keyops` row (primary,
    /// recovery, old device(s), new device) — B-1: the group persist path
    /// seeds `known_ops` from these.
    pub(crate) identity_keyops: StaticEvents,
}

/// Run 40 (STOP 1, ruled (a) in-session) — the admin issues a delegation on
/// the identity document AS BYTES: a `StaticDelegation` signed with the
/// admin's `MemorySigner` (`try_sign_sync`, keyhive_crypto memory.rs L73),
/// verified (`try_verify`) before it is handed to `ingest_unsorted_static_events`
/// — the path the persisted rows already take. Why not `Document::add_member`:
/// at `90fe4a51` its `can.is_reader()` branch adds the delegate to the
/// document's CGKA (document.rs L262–281) and a document materialised from
/// static events has none (`from_group`, L88–93 → `CgkaError::NotInitialized`,
/// L122–125); `Document.group` / `add_member_with_manual_content` are
/// `pub(crate)`. Ingest re-applies core's own checks (state.rs L150–183:
/// non-escalation, signer == the proof's delegate) — a non-admin signer is
/// refused there too; this app refuses it earlier, before signing.
/// `proof` = digest of the admin's own root delegation from `row`;
/// `after_revocations` empty; `after_content` copied from the row's existing
/// device `Edit` edge (shape parity with the edge core produced at the
/// ceremony — not required by ingest, which reads `after_content` only when
/// materialising a NEW subject from a root, keyhive.rs L1826–1845).
/// No `ed25519` literal here (H4 as amended: the one literal stays in
/// `groups::signer_from_seed`).
pub(crate) fn issue_delegation_as_bytes(
    admin: &MemorySigner,
    row: &StaticDelegations,
    delegate: Identifier,
    can: Access,
) -> Result<Signed<StaticDelegation<[u8; 32]>>, CoreError> {
    let admin_id: Identifier = (&admin.verifying_key()).into();
    let root = row
        .iter()
        .find(|d| d.payload().proof.is_none() && d.payload().delegate == admin_id)
        .ok_or_else(|| CoreError::Recovery("row holds no root delegation to this admin".into()))?;
    let after_content = row
        .iter()
        .find(|d| d.payload().proof.is_some() && d.payload().can == Access::Edit)
        .map(|d| d.payload().after_content.clone())
        .unwrap_or_default();
    let payload = StaticDelegation { can, proof: Some(root.digest()), delegate, after_revocations: Vec::new(), after_content };
    let signed = admin.try_sign_sync(payload).map_err(|e| CoreError::Recovery(format!("sign delegation: {e}")))?;
    signed.try_verify().map_err(|e| CoreError::Recovery(format!("issued delegation does not verify: {e}")))?;
    Ok(signed)
}

/// Run 40 (plan v0.1.2 §9 row 6; D-40 record §4 steps 2–4; D-40-1(a),
/// D-40-2, D-40-3(i)) — the identity half of recovery from the cold key.
/// `admin` is the re-imported cold signer (primary or recovery — either is an
/// Admin delegate; built by the one seed→signer constructor,
/// `groups::signer_from_seed`, H4 as amended). Steps: reload the identity
/// document into an admin-active hive from the two rows (`reload_with`,
/// K1: delegate-is-active ingest); verify the admin HOLDS `Admin` on it
/// (else `CoreError::Recovery`, nothing written); introduce `new_device` by
/// contact card and delegate it `Edit` on the document (K2: the admin proves
/// with its own capability); rewrite the identity row from `members()` (the
/// Run 38 shape, sorted by delegate) and the `identity-keyops` row with the
/// new device's op appended — SAME tags (Run 30 rule; rows never re-tagged).
/// The admin seed is never persisted (test
/// `cold_keys_are_exported_and_never_persisted`, extended).
pub(crate) fn recover_identity(store: &dyn DocStore, admin: MemorySigner, new_device: MemorySigner) -> Result<Recovered, CoreError> {
    let admin_id: Identifier = (&admin.verifying_key()).into();
    let admin_fingerprint = hex(&admin.verifying_key().to_bytes());
    let admin_for_edge = admin.clone();
    let (hive, doc_id) = reload_with(store, admin)?;
    let held = crate::runtime::rt().block_on(async {
        let doc = hive.get_document(doc_id).await?;
        let doc = doc.lock().await;
        doc.get_capability(&admin_id).map(|d| d.payload().can())
    });
    if held != Some(Access::Admin) {
        return Err(CoreError::Recovery("seed is not an admin of this identity".into()));
    }
    // the row's ops, to be extended — read back so the rewrite carries every op
    let (_, kbytes) = store
        .read_tagged(IDENTITY_KEYOPS_ROW_ID)
        .map_err(|e| CoreError::Storage(e.to_string()))?
        .ok_or_else(|| CoreError::Ceremony("no identity keyops row".into()))?;
    let mut keyops = decode_events(&kbytes)?;

    // the row's delegations, read back: the admin's own root is the PROOF of
    // the edge issued below; the existing device edge is its shape model
    let (_, dbytes) = store
        .read_tagged(IDENTITY_ROW_ID)
        .map_err(|e| CoreError::Storage(e.to_string()))?
        .ok_or_else(|| CoreError::Ceremony("no identity row".into()))?;
    let row = decode(&dbytes)?;

    let rer = |e: &dyn std::fmt::Debug| CoreError::Recovery(format!("{e:?}"));
    let (new_device_id, new_device_op, statics, members) = crate::runtime::rt().block_on(async {
        // D-40-3 step 4: introduce the replacement device (the `introduce` shape)
        let device_hive: Hive = Keyhive::generate(new_device, MemoryCiphertextStore::new(), NoListener, OsRng)
            .await
            .map_err(|e| rer(&e))?;
        let card = device_hive.generate_contact_card().await.map_err(|e| rer(&e))?;
        // condition 2 (STOP 1 ruling): the device's KeyOp lands in the hive
        // BEFORE its delegation — `receive_contact_card` registers the
        // individual; the delegation below resolves it through `get_agent`.
        let new_device_id: Identifier = hive.receive_contact_card(&card).await.map_err(|e| rer(&e))?.into();
        // STOP 1 ruled (a): the admin ISSUES the Edit delegation as bytes and
        // the hive ingests it — not `Document::add_member`, whose reader branch
        // needs a CGKA a reloaded document does not hold (document.rs L239–281,
        // L122–125 at the pin). Same ingest path as the rows.
        let edge = issue_delegation_as_bytes(&admin_for_edge, &row, new_device_id, Access::Edit)?;
        let pending = hive.ingest_unsorted_static_events(vec![StaticEvent::Delegated(edge)]).await;
        if !pending.is_empty() {
            return Err(CoreError::Recovery(format!("re-delegation left {} event(s) pending: {pending:?}", pending.len())));
        }
        let doc = hive.get_document(doc_id).await.ok_or_else(|| CoreError::Recovery("identity document vanished".into()))?;
        let doc = doc.lock().await;
        let mut statics: StaticDelegations = Vec::new();
        for delegations in doc.members().values() {
            for signed in delegations.iter() {
                statics.push((**signed).clone().map(StaticDelegation::from));
            }
        }
        Ok::<_, CoreError>((new_device_id, keyop_event(&card), statics, doc.members().len() as u32))
    })?;
    let mut statics = statics;
    for d in &statics {
        d.try_verify().map_err(|e| CoreError::Recovery(format!("delegation signature: {e}")))?;
    }
    statics.sort_by(|a, b| a.payload().delegate.to_bytes().cmp(&b.payload().delegate.to_bytes()));
    keyops.push(new_device_op);
    let bytes = bincode::serialize(&statics).map_err(|e| CoreError::Recovery(format!("encode: {e}")))?;
    let kbytes = bincode::serialize(&keyops).map_err(|e| CoreError::Recovery(format!("encode keyops: {e}")))?;
    store
        .write_keyhive_static_delegations(IDENTITY_ROW_ID, &bytes)
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    store
        .write_keyhive_static_events(IDENTITY_KEYOPS_ROW_ID, &kbytes)
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(Recovered { hive, doc_id, admin_fingerprint, members, new_device: new_device_id, identity_keyops: keyops })
}

/// Decode a persisted identity row back into its delegations. Test-only in
/// Run 37 (the round-trip proof); the reload path into a live hive is
/// Run 40's work and lifts the gate then.
/// Run 38 (OR-4): the gate is lifted HERE because
/// `identity_row_reloads_into_device_hive` is green — `reload_with` is the
/// first product caller.
pub(crate) fn decode(bytes: &[u8]) -> Result<StaticDelegations, CoreError> {
    bincode::deserialize(bytes).map_err(|e| CoreError::Ceremony(format!("decode: {e}")))
}

/// Run 38 — decode the persisted `KeyOp` row (`FORMAT_KEYHIVE_STATIC_EVENTS_V1`).
pub(crate) fn decode_events(bytes: &[u8]) -> Result<StaticEvents, CoreError> {
    bincode::deserialize(bytes).map_err(|e| CoreError::Ceremony(format!("decode keyops: {e}")))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::SqliteStore;

    fn store(dir: &tempfile::TempDir, file: &str) -> (String, SqliteStore) {
        let path = dir.path().join(file).to_string_lossy().to_string();
        (path.clone(), SqliteStore::open(&path).unwrap())
    }

    /// Plan §9 row 3 done-when (1): two admin delegations; parametric on the
    /// configured level; Edit-rooting records the floor as Safe.
    #[test]
    fn ceremony_creates_identity_doc_with_two_admin_delegations() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "c1.sqlite");
        let rec = run(&s, IdentityConfig { rooting_level: RootingLevel::Edit }).unwrap();
        assert_eq!(rec.admin_delegations, 2);
        assert_eq!(rec.rooting_level, RootingLevel::Edit);
        assert_eq!(rec.floor_status, FloorStatus::Safe);
        assert_eq!(rec.identity_row_id, IDENTITY_ROW_ID);
        assert_eq!(rec.format_tag, storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1);
        assert!(rec.persisted_bytes > 0);
    }

    /// H5 done-when: the configured level is recorded and L-12 is cited —
    /// Admin-rooting carries the floor-withdrawn status; the ceremony still
    /// completes (L-12 is unruled; the status is the citation).
    #[test]
    fn admin_rooting_records_floor_withdrawn_pending_l12() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "c2.sqlite");
        let rec = run(&s, IdentityConfig { rooting_level: RootingLevel::Admin }).unwrap();
        assert_eq!(rec.rooting_level, RootingLevel::Admin);
        assert_eq!(rec.floor_status, FloorStatus::WithdrawnPendingL12);
        assert_eq!(rec.admin_delegations, 2);
    }

    /// Plan §9 row 3 done-when (2): first Keyhive bytes in SQLite under the
    /// NEW tag; they decode, every signature verifies, both delegations grant
    /// Admin from the same ephemeral root, and their delegates are exactly
    /// the two exported fingerprints.
    #[test]
    fn keyhive_bytes_land_under_new_format_tag_and_verify() {
        let dir = tempfile::tempdir().unwrap();
        let (path, s) = store(&dir, "c3.sqlite");
        let rec = run(&s, IdentityConfig { rooting_level: RootingLevel::Edit }).unwrap();
        let conn = rusqlite::Connection::open(&path).unwrap();
        let fmt: String = conn
            .query_row("SELECT format FROM docs WHERE id = ?1", rusqlite::params![IDENTITY_ROW_ID], |r| r.get(0))
            .unwrap();
        assert_eq!(fmt, storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1);
        let (tag, bytes) = s.read_tagged(IDENTITY_ROW_ID).unwrap().unwrap();
        assert_eq!(tag, storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1);
        assert_eq!(bytes.len() as u64, rec.persisted_bytes);
        let dels = decode(&bytes).unwrap();
        // Run 38 (D-38-3; record §3(c)-6, the ONE named test-line edit of the
        // run): three delegations — two Admin root edges (no proof) plus the
        // device at Edit, issued by the primary admin with a proof.
        assert_eq!(dels.len(), 3);
        let mut delegates = Vec::new();
        let mut admin_delegates = Vec::new();
        let mut device_delegates = Vec::new();
        for d in &dels {
            d.try_verify().expect("persisted delegation verifies");
            match d.payload().can {
                Access::Admin => {
                    assert!(d.payload().proof.is_none(), "root edge: delegated by the ephemeral root, no proof");
                    admin_delegates.push(hex(&d.payload().delegate.to_bytes()));
                }
                Access::Edit => {
                    assert!(d.payload().proof.is_some(), "device edge: delegated by the primary admin, with proof");
                    assert_eq!(hex(&d.issuer().to_bytes()), rec.cold_keys.primary_admin_fingerprint, "issued by the primary admin");
                    device_delegates.push(hex(&d.payload().delegate.to_bytes()));
                }
                other => panic!("unexpected level on the identity document: {other:?}"),
            }
            delegates.push(hex(&d.payload().delegate.to_bytes()));
        }
        let issuers: std::collections::BTreeSet<_> =
            dels.iter().filter(|d| d.payload().can == Access::Admin).map(|d| d.issuer().to_bytes()).collect();
        assert_eq!(issuers.len(), 1, "both admin delegations issued by the one ephemeral root key");
        let mut want = vec![rec.cold_keys.primary_admin_fingerprint.clone(), rec.cold_keys.recovery_fingerprint.clone()];
        want.sort();
        admin_delegates.sort();
        assert_eq!(admin_delegates, want, "admin delegates are the two cold keys");
        assert_eq!(device_delegates, vec![rec.device_key.device_fingerprint.clone()], "the Edit delegate is the device key");
        assert_eq!(rec.admin_delegations, 2, "the floor count is unaffected by the device delegation");
    }

    /// Run 38 — D-38-4 / OR-4: the test that decides the reload timing.
    /// Run the ceremony with a retained device signer; open a FRESH hive whose
    /// active agent is the device; ingest the persisted KeyOps + delegations;
    /// nothing pending, the document materialises, three members, device at
    /// Edit, admins at Admin.
    #[test]
    fn identity_row_reloads_into_device_hive() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "c7.sqlite");
        let primary = MemorySigner::generate(&mut OsRng);
        let recovery = MemorySigner::generate(&mut OsRng);
        let device = MemorySigner::generate(&mut OsRng);
        let (rec, material) =
            enroll_with_device(&s, IdentityConfig { rooting_level: RootingLevel::Edit }, primary.clone(), Some(recovery.clone()), device.clone())
                .unwrap();
        assert_eq!(rec.keyops_format_tag, storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1);
        assert_eq!(s.read_tagged(IDENTITY_KEYOPS_ROW_ID).unwrap().unwrap().0, storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1);

        let (hive, doc_id) = reload_with(&s, device.clone()).expect("reload from the two rows, no admin signer");
        assert_eq!(doc_id, material.identity_doc, "the identity document id is recovered from the root issuer");
        crate::runtime::rt().block_on(async {
            let doc = hive.get_document(doc_id).await.expect("identity document materialised");
            let doc = doc.lock().await;
            assert_eq!(doc.members().len(), 3, "primary admin + recovery + device");
            let level = |signer: &MemorySigner| {
                let id: Identifier = (&signer.verifying_key()).into();
                doc.get_capability(&id).map(|d| d.payload().can())
            };
            assert_eq!(level(&device), Some(Access::Edit));
            assert_eq!(level(&primary), Some(Access::Admin));
            assert_eq!(level(&recovery), Some(Access::Admin));
        });
    }

    /// Run 40 (plan v0.1.2 §9 row 6; D-40 record `586e73f9…` §4 steps 2–4;
    /// SL-0249 VE(1)) — WRITTEN FIRST. The device seed is LOST (the ceremony's
    /// device signer is dropped); a fresh hive is built from a RE-IMPORTED
    /// cold admin seed (the recovery key — D-40-2) and the two rows alone:
    /// the identity document is found, `members()` reads three, the admin
    /// holds `Admin` (K1: delegate-is-active ingest). Then the re-delegation
    /// (D-40-3 step 4): a new device is introduced and delegated `Edit` by the
    /// admin; both rows are rewritten under the SAME tags (Run 30 rule) and
    /// `members()` reads four; a device-active reload from the NEW seed sees
    /// the same four with the new device at `Edit`.
    #[test]
    fn identity_row_recovers_with_reimported_admin() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "c40.sqlite");
        let primary = MemorySigner::generate(&mut OsRng);
        let recovery = MemorySigner::generate(&mut OsRng);
        let device = MemorySigner::generate(&mut OsRng);
        let (rec, material) =
            enroll_with_device(&s, IdentityConfig { rooting_level: RootingLevel::Edit }, primary.clone(), Some(recovery.clone()), device.clone())
                .unwrap();
        let row_before = s.read_tagged(IDENTITY_ROW_ID).unwrap().unwrap();
        drop(material); // the device seed is gone

        // (i) recovery as read: a fresh hive from the imported admin seed alone
        let admin = crate::groups::signer_from_seed(&rec.cold_keys.recovery_secret).unwrap();
        let (hive, doc_id) = reload_with(&s, admin.clone()).expect("reload from the two rows with the cold admin as active agent");
        assert_eq!(doc_id, material_doc_id(&row_before.1));
        crate::runtime::rt().block_on(async {
            let doc = hive.get_document(doc_id).await.expect("identity document found");
            let doc = doc.lock().await;
            assert_eq!(doc.members().len(), 3, "primary + recovery + (lost) device");
            let id: Identifier = (&admin.verifying_key()).into();
            assert_eq!(doc.get_capability(&id).map(|d| d.payload().can()), Some(Access::Admin), "admin capability Admin");
        });
        drop(hive);

        // (ii) re-delegation: the admin enrolls a replacement device at Edit
        let new_device = MemorySigner::generate(&mut OsRng);
        let new_seed = new_device.0.to_bytes().to_vec();
        let recovered = recover_identity(&s, admin.clone(), new_device.clone()).expect("re-delegation from the cold admin");
        assert_eq!(recovered.doc_id, doc_id);
        assert_eq!(recovered.members, 4, "primary + recovery + old device + new device");
        assert_eq!(recovered.admin_fingerprint, rec.cold_keys.recovery_fingerprint);
        // a seed that is NOT an admin of this identity is refused before anything is written
        let stranger = MemorySigner::generate(&mut OsRng);
        match recover_identity(&s, stranger, MemorySigner::generate(&mut OsRng)) {
            Err(CoreError::Recovery(msg)) => assert_eq!(msg, "seed is not an admin of this identity"),
            Err(other) => panic!("expected CoreError::Recovery, got {other:?}"),
            Ok(_) => panic!("expected CoreError::Recovery, got Ok"),
        }
        // rows rewritten under the same tags
        let row_after = s.read_tagged(IDENTITY_ROW_ID).unwrap().unwrap();
        assert_eq!(row_after.0, storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1);
        assert_ne!(row_after.1, row_before.1);
        let dels_after = decode(&row_after.1).unwrap();
        assert_eq!(dels_after.len(), 4);
        // STOP 1 (a), condition 4: the ingested edge is the admin's — its issuer
        // fingerprint is the imported admin's, it delegates the new device at Edit
        let new_id: Identifier = (&new_device.verifying_key()).into();
        let edge = dels_after.iter().find(|d| d.payload().delegate == new_id).expect("new device edge persisted");
        assert_eq!(hex(&edge.issuer().to_bytes()), rec.cold_keys.recovery_fingerprint, "issued by the imported admin");
        assert_eq!(edge.payload().can, Access::Edit);
        assert!(edge.payload().proof.is_some(), "proof = the admin's root");
        assert_eq!(recovered.members, 4);
        let kops = s.read_tagged(IDENTITY_KEYOPS_ROW_ID).unwrap().unwrap();
        assert_eq!(kops.0, storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1);
        assert_eq!(decode_events(&kops.1).unwrap().len(), 4);

        // (iii) the Run 39 device path from the NEW seed sees the re-delegated document
        let (hive2, doc_id2) = reload_with(&s, crate::groups::signer_from_seed(&new_seed).unwrap()).unwrap();
        assert_eq!(doc_id2, doc_id);
        crate::runtime::rt().block_on(async {
            let doc = hive2.get_document(doc_id2).await.unwrap();
            let doc = doc.lock().await;
            assert_eq!(doc.members().len(), 4);
            let level = |signer: &MemorySigner| {
                let id: Identifier = (&signer.verifying_key()).into();
                doc.get_capability(&id).map(|d| d.payload().can())
            };
            assert_eq!(level(&new_device), Some(Access::Edit));
            assert_eq!(level(&device), Some(Access::Edit), "the lost device's edge stays (item (p), HELD)");
            assert_eq!(level(&primary), Some(Access::Admin));
            assert_eq!(level(&recovery), Some(Access::Admin));
        });
    }

    /// Test helper: the identity document id as `reload_with` recovers it —
    /// the issuer of the row's root delegations.
    fn material_doc_id(row: &[u8]) -> DocumentId {
        let dels = decode(row).unwrap();
        let root = dels.iter().find(|d| d.payload().proof.is_none()).unwrap();
        DocumentId::from(Identifier::from(root.issuer()))
    }

    /// Run 40 (D-40-4, B-5) — the reload's own-op filter is hygiene, not
    /// behaviour: the same rows reloaded with the Run 38 filter (matches
    /// `PrekeysExpanded` only) and with the product filter (by issuer, both
    /// variants) materialise the identity document with identical
    /// `members()` and capabilities.
    #[test]
    fn own_op_filter_by_issuer_reloads_identically_to_run38_filter() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "c40b.sqlite");
        let device = MemorySigner::generate(&mut OsRng);
        enroll_with_device(
            &s,
            IdentityConfig { rooting_level: RootingLevel::Edit },
            MemorySigner::generate(&mut OsRng),
            Some(MemorySigner::generate(&mut OsRng)),
            device.clone(),
        )
        .unwrap();
        let (_, bytes) = s.read_tagged(IDENTITY_ROW_ID).unwrap().unwrap();
        let (_, kbytes) = s.read_tagged(IDENTITY_KEYOPS_ROW_ID).unwrap().unwrap();
        let own: Identifier = (&device.verifying_key()).into();
        let keyops = decode_events(&kbytes).unwrap();
        // the rows hold PrekeyRotated at the pin (note (o)): the Run 38 filter drops nothing, the product filter drops the own op
        assert_eq!(keyops.iter().filter(|e| own_op_run38(e, &own)).count(), 3);
        assert_eq!(keyops.iter().filter(|e| own_op_by_issuer(e, &own)).count(), 2);
        let snapshot = |keep: fn(&StaticEvent<[u8; 32]>, &Identifier) -> bool| {
            let (hive, doc_id) = crate::runtime::rt()
                .block_on(reload_into(device.clone(), decode_events(&kbytes).unwrap(), decode(&bytes).unwrap(), keep))
                .unwrap();
            crate::runtime::rt().block_on(async {
                let doc = hive.get_document(doc_id).await.unwrap();
                let doc = doc.lock().await;
                let mut v: Vec<(Vec<u8>, Access)> = doc
                    .members()
                    .keys()
                    .map(|m| (m.to_bytes().to_vec(), doc.get_capability(m).unwrap().payload().can()))
                    .collect();
                v.sort();
                (doc_id, v)
            })
        };
        assert_eq!(snapshot(own_op_run38), snapshot(own_op_by_issuer));
    }

    /// Plan §9 row 3 done-when (3): cold key off-device — the exported seeds
    /// are 32 bytes, differ, and appear NOWHERE in the SQLite file.
    #[test]
    fn cold_keys_are_exported_and_never_persisted() {
        let dir = tempfile::tempdir().unwrap();
        let (path, s) = store(&dir, "c4.sqlite");
        let rec = run(&s, IdentityConfig { rooting_level: RootingLevel::Edit }).unwrap();
        drop(s);
        let ck = &rec.cold_keys;
        assert_eq!(ck.primary_admin_secret.len(), 32);
        assert_eq!(ck.recovery_secret.len(), 32);
        assert_ne!(ck.primary_admin_secret, ck.recovery_secret);
        assert_eq!(ck.primary_admin_fingerprint.len(), 64);
        let file = std::fs::read(&path).unwrap();
        let contains = |needle: &[u8]| file.windows(needle.len()).any(|w| w == needle);
        assert!(!contains(&ck.primary_admin_secret), "primary seed must not be on disk");
        assert!(!contains(&ck.recovery_secret), "recovery seed must not be on disk");
        // and the public halves ARE there (the delegates of the persisted rows)
        let pk = |fp: &str| (0..fp.len()).step_by(2).map(|i| u8::from_str_radix(&fp[i..i + 2], 16).unwrap()).collect::<Vec<u8>>();
        assert!(contains(&pk(&ck.primary_admin_fingerprint)));
        assert!(contains(&pk(&ck.recovery_fingerprint)));

        // Run 40 (D-40-2; kickoff done-when (iii)) — the IMPORT path: the
        // admin seed crosses into recovery and out again without landing;
        // after step 8 the store holds neither cold seed nor either device seed.
        let s = SqliteStore::open(&path).unwrap();
        let (report, live) = crate::groups::DeviceHive::recover(&s, &ck.recovery_secret).unwrap();
        drop(live);
        drop(s);
        let file = std::fs::read(&path).unwrap();
        let contains = |needle: &[u8]| file.windows(needle.len()).any(|w| w == needle);
        assert!(!contains(&ck.primary_admin_secret), "primary seed must not be on disk after recovery");
        assert!(!contains(&ck.recovery_secret), "the IMPORTED recovery seed must not be on disk");
        assert!(!contains(&rec.device_key.device_secret), "the lost device's seed is not on disk either");
        assert!(!contains(&report.device_key.device_secret), "the new device seed is exported, never persisted");
        assert!(contains(&pk(&report.device_key.device_fingerprint)), "its public half is in the rewritten row");
    }

    /// H2 — first caller of the floor: with one admin delegation the policy
    /// refuses, the refusal maps to `CoreError::Policy`, and nothing is
    /// written.
    #[test]
    fn floor_violation_maps_to_policy_error_and_persists_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "c5.sqlite");
        let primary = MemorySigner::generate(&mut OsRng);
        match enroll_with(&s, IdentityConfig { rooting_level: RootingLevel::Edit }, primary, None) {
            Err(CoreError::Policy(msg)) => assert_eq!(msg, "delegation floor: 1 admin delegation(s) present, floor is 2"),
            other => panic!("expected CoreError::Policy, got {other:?}"),
        }
        assert!(s.read_tagged(IDENTITY_ROW_ID).unwrap().is_none(), "refused ceremony writes nothing");
    }

    /// Enrollment happens once: a second ceremony is refused and the existing
    /// row is byte-identical afterwards.
    #[test]
    fn second_ceremony_is_refused_and_leaves_the_row_intact() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "c6.sqlite");
        run(&s, IdentityConfig { rooting_level: RootingLevel::Edit }).unwrap();
        let before = s.read_tagged(IDENTITY_ROW_ID).unwrap().unwrap();
        assert!(matches!(
            run(&s, IdentityConfig { rooting_level: RootingLevel::Admin }),
            Err(CoreError::IdentityAlreadyEnrolled)
        ));
        assert_eq!(s.read_tagged(IDENTITY_ROW_ID).unwrap().unwrap(), before);
    }
}
