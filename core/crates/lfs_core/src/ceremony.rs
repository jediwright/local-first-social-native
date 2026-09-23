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

/// Storage row id of the persisted identity delegations. A stable, app-owned
/// name; NOT the Keyhive document ID (H3).
pub const IDENTITY_ROW_ID: &str = "identity";

/// The initial content head the identity document is generated over. Run 37
/// binds no content to the document yet (the document's automerge content is
/// later work); the zero digest is the pinned crate's own placeholder shape
/// (its tests generate documents over `[0u8; 32]`).
const INITIAL_CONTENT_HEAD: [u8; 32] = [0u8; 32];

type Hive = Keyhive<Sendable, MemorySigner>;
type StaticDelegations = Vec<Signed<StaticDelegation<[u8; 32]>>>;

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
}

/// Run the ceremony against `store`. Refuses (without touching the store) if
/// an identity row already exists — enrollment happens once; recovery and
/// rotation are their own paths (spec §5.2/§6; Run 40).
pub(crate) fn run(store: &dyn DocStore, config: IdentityConfig) -> Result<IdentityCeremonyRecord, CoreError> {
    if store.read_tagged(IDENTITY_ROW_ID).map_err(|e| CoreError::Storage(e.to_string()))?.is_some() {
        return Err(CoreError::IdentityAlreadyEnrolled);
    }
    let primary = MemorySigner::generate(&mut OsRng);
    let recovery = MemorySigner::generate(&mut OsRng);
    enroll_with(store, config, primary, Some(recovery))
}

/// The ceremony body, parametric on the keys so the floor path can be
/// exercised: `recovery = None` generates a document with ONE admin
/// delegation, which the floor refuses before anything is persisted.
pub(crate) fn enroll_with(
    store: &dyn DocStore,
    config: IdentityConfig,
    primary: MemorySigner,
    recovery: Option<MemorySigner>,
) -> Result<IdentityCeremonyRecord, CoreError> {
    let primary_fp = hex(&primary.verifying_key().to_bytes());
    let recovery_fp = recovery.as_ref().map(|r| hex(&r.verifying_key().to_bytes())).unwrap_or_default();

    let (delegations, admin_count) =
        crate::runtime::rt().block_on(generate_identity_doc(primary.clone(), recovery.clone()))?;

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

    let cold_keys = ColdKeyExport {
        primary_admin_secret: primary.0.to_bytes().to_vec(),
        primary_admin_fingerprint: primary_fp,
        recovery_secret: recovery.map(|r| r.0.to_bytes().to_vec()).unwrap_or_default(),
        recovery_fingerprint: recovery_fp,
    };
    Ok(IdentityCeremonyRecord {
        identity_row_id: IDENTITY_ROW_ID.to_string(),
        rooting_level: config.rooting_level,
        floor_status: policy::delegation_floor_status(config.rooting_level),
        admin_delegations: admin_count as u32,
        format_tag: storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1.to_string(),
        persisted_bytes: bytes.len() as u64,
        cold_keys,
    })
}

/// The Keyhive half: an in-memory hive whose active agent is the primary
/// admin key; the recovery key's individual is introduced through a contact
/// card (the pinned crate's only path to register a peer) and named as
/// co-parent of the new document. Returns the document's delegation heads in
/// static form plus the number of them that grant `Admin`.
async fn generate_identity_doc(
    primary: MemorySigner,
    recovery: Option<MemorySigner>,
) -> Result<(StaticDelegations, usize), CoreError> {
    let cer = |e: &dyn std::fmt::Debug| CoreError::Ceremony(format!("{e:?}"));
    let admin_hive: Hive = Keyhive::generate(primary, MemoryCiphertextStore::new(), NoListener, OsRng)
        .await
        .map_err(|e| cer(&e))?;

    let mut coparents = Vec::new();
    if let Some(recovery) = recovery {
        let recovery_hive: Hive = Keyhive::generate(recovery, MemoryCiphertextStore::new(), NoListener, OsRng)
            .await
            .map_err(|e| cer(&e))?;
        let card = recovery_hive.generate_contact_card().await.map_err(|e| cer(&e))?;
        let recovery_id = admin_hive.receive_contact_card(&card).await.map_err(|e| cer(&e))?;
        coparents.push(recovery_id.into());
    }

    let doc_id = admin_hive
        .generate_doc(coparents, nonempty![INITIAL_CONTENT_HEAD])
        .await
        .map_err(|e| cer(&e))?;
    let doc = admin_hive.get_document(doc_id).await.ok_or_else(|| CoreError::Ceremony("document vanished".into()))?;
    let doc = doc.lock().await;

    let mut statics: StaticDelegations = Vec::new();
    let mut admin_count = 0usize;
    for signed in doc.delegation_heads().values() {
        if signed.payload().can() == Access::Admin {
            admin_count += 1;
        }
        let s: Signed<StaticDelegation<[u8; 32]>> = (**signed).clone().map(StaticDelegation::from);
        statics.push(s);
    }
    // Deterministic order for byte-stable rows: by delegate identifier bytes.
    statics.sort_by(|a, b| a.payload().delegate.to_bytes().cmp(&b.payload().delegate.to_bytes()));
    Ok((statics, admin_count))
}

/// Decode a persisted identity row back into its delegations. Test-only in
/// Run 37 (the round-trip proof); the reload path into a live hive is
/// Run 40's work and lifts the gate then.
#[cfg(test)]
pub(crate) fn decode(bytes: &[u8]) -> Result<StaticDelegations, CoreError> {
    bincode::deserialize(bytes).map_err(|e| CoreError::Ceremony(format!("decode: {e}")))
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
        assert_eq!(dels.len(), 2);
        let mut delegates = Vec::new();
        for d in &dels {
            d.try_verify().expect("persisted delegation verifies");
            assert_eq!(d.payload().can, Access::Admin);
            assert!(d.payload().proof.is_none(), "root edge: delegated by the ephemeral root, no proof");
            delegates.push(hex(&d.payload().delegate.to_bytes()));
        }
        let issuers: std::collections::BTreeSet<_> = dels.iter().map(|d| d.issuer().to_bytes()).collect();
        assert_eq!(issuers.len(), 1, "both delegations issued by the one ephemeral root key");
        let mut want = vec![rec.cold_keys.primary_admin_fingerprint.clone(), rec.cold_keys.recovery_fingerprint.clone()];
        want.sort();
        assert_eq!(delegates, want, "delegates are the two cold keys");
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
