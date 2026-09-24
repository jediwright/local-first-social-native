//! Groups behind the membership counter — Run 38, Phase 1 Build Plan v0.1.1
//! §9 row 4 (D-38 record `1a354226…` §2 D-38-2/D-38-4 CARRIED).
//!
//! This is the HIVE-HOLDING module: the one place (besides `ceremony`, which
//! produces the identity bytes) that holds a live `keyhive_core` hive and
//! names its types. H1's grep therefore names `storage` + `policy` +
//! `ceremony` + `groups` from Run 38 on.
//!
//! What it does:
//!
//! 1. Holds the DEVICE HIVE — a `Keyhive` whose active agent is the device
//!    signer. After the identity ceremony the hive is rebuilt from the two
//!    persisted rows through `ceremony::reload_with` (D-38-4 path (B), the
//!    reload test green), so the identity document with its three members is
//!    live in memory; before any ceremony (the Run 32 shells call the counter
//!    without one — a host-build touches no shell) a per-session hive with an
//!    ephemeral device signer and NO identity document is used, so the
//!    counter always reads real Keyhive group membership (ruled in-run, ~).
//! 2. Backs `membership_version` / `record_membership_event` with Keyhive
//!    GROUPS: an app-owned `group_id` string names a group the device created
//!    (`generate_group`, device = `Admin`); a membership event is a real
//!    `add_member` delegation on that group; the version READ BACK is the
//!    group's `members()` count minus the device's own root membership.
//!    Version 0 = no group (no event seen) — the Run 32 contract, kept. The
//!    member a `record_membership_event` adds is a simulated peer (an
//!    in-memory individual introduced by contact card) at `Read`: real peers
//!    arrive with the connection seam (Phase 2), the signature is the seam.
//! 3. Wires the H2 GRANT BAR — the first caller of `policy::check_grant_bar`.
//!    The granter's level is the level it HOLDS on the resource
//!    (`get_capability(&granter)`, the member's highest delegation), mapped to
//!    `policy::GrantLevel` by the 1:1 table below (D-38-2), then checked; the
//!    check runs BEFORE `add_member`, so a below-Admin grant is refused by
//!    this app's policy with core never invoked and nothing written. The bar
//!    is per-resource. Its scope on the identity document is HELD (OR-2,
//!    §10 item 13, ruled before Phase 2): every grant this module issues is
//!    on a group the device created, where the device is `Admin`.
//!
//! H3: no Keyhive group/document id crosses FFI or lands in a log path — the
//! app-owned `group_id` string is the only name a shell sees.
//! H4: no `ed25519` literal here; the device signer is a `MemorySigner`.
//!
//! Run 39 — grant/revoke on device (plan §9 row 5; kickoff STOP 1 ruled (a),
//! STOP 2 ruled P-1; revoke bar named; custody ruled):
//!
//! 4. SEED → SIGNER (`signer_from_seed`): the custodied 32-byte device seed
//!    (`DeviceKeyExport.device_secret`, the raw Ed25519 seed) rebuilds the
//!    device `MemorySigner` through `ed25519_dalek::SigningKey::from_bytes`
//!    (2.2.0 `signing.rs` L104) — the pinned `keyhive_crypto` offers only
//!    `generate` and `From<SigningKey>` (memory.rs L49/L125, no seed path).
//!    This is the ONE `ed25519` literal in product code: H4 as amended
//!    ("no `ed25519` literal outside the one seed→signer constructor in the
//!    hive-holding module" — plan v0.1.2/v0.1.3 touch). The seed never lands
//!    in SQLite (H3 held); the FFI boundary checks its length only.
//! 5. GROUP PERSISTENCE (P-1): every device-created group is written as a
//!    ROW PAIR through the storage adapter (H1) under app-owned ids —
//!    `group:<app id>` (the group's current `members()` delegation set, the
//!    Run 37 v1 delegation type, same tag) and `group:<app id>.keyops` (the
//!    peers' `KeyOp`s as `PrekeysExpanded` events, the Run 38 v1 events type,
//!    same tag). No new tag. The row pair is a MEMBERSHIP SNAPSHOT: after a
//!    revoke the revoked member's delegations are absent and the revocation
//!    itself is not replayed (ruled in-run, ~ — sufficient for row 5's
//!    "kill/relaunch keeps it"; an event-log row is Phase 2's if sync needs
//!    it). Reload (`reload`) rebuilds the identity hive first
//!    (`ceremony::reload_with`), then each group through the same ingest
//!    path: KeyOps before delegations; `UnknownAgent`/pending is an error,
//!    never a silent skip. The group's Keyhive id is recovered from its root
//!    delegation's issuer exactly as the identity document's is (no map row);
//!    the app id is the row-id suffix. Groups on a pre-ceremony session hive
//!    are not persisted (no identity to reload them under — Run 38 shape).
//! 6. MEMBERSHIP QUERY (`members`): a typed per-member list — fingerprint
//!    (the member Identifier's verifying key, lowercase hex, the Run 37
//!    export convention) + `GrantLevel` + whether it is this device. No
//!    Keyhive id crosses (H3); the shells see ONE identifier format.
//! 7. REVOKE: `policy::check_revoke_bar` (Rule 2b, `REVOKE_BAR = Admin`) runs
//!    BEFORE `revoke_member` — its first caller. The member is located by
//!    fingerprint among `members()` (no key is parsed here). Core's own
//!    refusal at the pin is provenance-based (group.rs L629: the issuer may
//!    revoke what it issued at any level), so for a grant the device issued
//!    the bar is the ONLY thing standing between an Edit-level device and a
//!    landed revocation — the refusal test proves policy refused and core
//!    would have accepted. `membership_version` falls with `members()`; a
//!    separate removed-count is deferred (open item (d), deadline Run 40).

use crate::ceremony::{self, DeviceMaterial};
use crate::policy::{self, GrantLevel};
use crate::storage::DocStore;
use crate::CoreError;
use future_form::Sendable;
use keyhive_core::access::Access;
use keyhive_core::event::static_event::StaticEvent;
use keyhive_core::principal::group::delegation::StaticDelegation;
use keyhive_crypto::signed::Signed;
use keyhive_core::keyhive::Keyhive;
use keyhive_core::listener::no_listener::NoListener;
use keyhive_core::principal::document::id::DocumentId;
use keyhive_core::principal::group::id::GroupId;
use keyhive_core::principal::identifier::Identifier;
use keyhive_core::store::ciphertext::memory::MemoryCiphertextStore;
use keyhive_crypto::signer::memory::MemorySigner;
use keyhive_crypto::verifiable::Verifiable;
use rand::rngs::OsRng;
use std::collections::HashMap;

type Hive = Keyhive<Sendable, MemorySigner>;
type StaticDelegations = Vec<Signed<StaticDelegation<[u8; 32]>>>;
type StaticEvents = Vec<StaticEvent<[u8; 32]>>;

/// Run 39 — row-id prefix of the persisted group rows (app-owned; H3).
pub const GROUP_ROW_PREFIX: &str = "group:";
/// Run 39 — suffix of a group's `KeyOp` row.
pub const GROUP_KEYOPS_ROW_SUFFIX: &str = ".keyops";

/// Run 39 — one member of a group as the shells see it (H3: fingerprint is
/// the verifying key in lowercase hex — the same format `ColdKeyExport` /
/// `DeviceKeyExport` use; no Keyhive identifier type crosses).
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GroupMember {
    pub fingerprint: String,
    pub level: GrantLevel,
    pub is_device: bool,
}

/// Run 39 (STOP 1, ruled (a)) — the ONE seed→signer constructor (H4 as
/// amended). `seed` must be exactly 32 bytes (the length is fixed by the
/// crate: `SecretKey = [u8; 32]`, 2.2.0 `signing.rs` L59).
pub(crate) fn signer_from_seed(seed: &[u8]) -> Result<MemorySigner, CoreError> {
    let bytes: [u8; 32] = seed
        .try_into()
        .map_err(|_| CoreError::Ceremony(format!("device seed must be 32 bytes, got {}", seed.len())))?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&bytes).into())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn fingerprint_of(id: &Identifier) -> String {
    hex(&id.to_bytes())
}

/// Run 39 — the issuer of a persisted `KeyOp` event. At the pin a contact
/// card carries a ROTATE op (`keyhive.rs` L332–341), so the events these
/// rows hold are `PrekeyRotated`, not `PrekeysExpanded` — both are matched.
fn keyop_issuer(ev: &StaticEvent<[u8; 32]>) -> Option<Identifier> {
    match ev {
        StaticEvent::PrekeysExpanded(op) => Some(Identifier::from(op.issuer())),
        StaticEvent::PrekeyRotated(op) => Some(Identifier::from(op.issuer())),
        _ => None,
    }
}

/// D-38-2 — the 1:1 mapping, fixed at the pin (`access.rs` L19–40 ≡
/// `policy.rs` L42–47, verified at Run 38 open). Lives HERE, not in
/// `policy.rs` (H2: policy names no core type).
impl From<Access> for GrantLevel {
    fn from(a: Access) -> Self {
        match a {
            Access::Relay => GrantLevel::Relay,
            Access::Read => GrantLevel::Read,
            Access::Edit => GrantLevel::Edit,
            Access::Admin => GrantLevel::Admin,
        }
    }
}

/// The inverse — the app's level vocabulary onto the pin's `Access`.
pub(crate) fn access_of(level: GrantLevel) -> Access {
    match level {
        GrantLevel::Relay => Access::Relay,
        GrantLevel::Read => Access::Read,
        GrantLevel::Edit => Access::Edit,
        GrantLevel::Admin => Access::Admin,
    }
}

/// The device hive plus the app-owned names it knows its groups by.
pub(crate) struct DeviceHive {
    hive: Hive,
    device: Identifier,
    /// Present once an identity ceremony has run (or reloaded) in this
    /// process; absent for the pre-ceremony session hive. Never crosses FFI.
    /// Consumed by Run 39/40 (grant state on the identity document; recovery).
    identity_doc: Option<DocumentId>,
    groups: HashMap<String, GroupId>,
    /// Run 39 — the `KeyOp` (as its `PrekeysExpanded` event) of every peer
    /// this hive has been introduced to, by identifier: what a group's
    /// `.keyops` row is built from. Public prekey material only.
    known_ops: HashMap<Identifier, StaticEvent<[u8; 32]>>,
}

impl DeviceHive {
    /// After the ceremony: rebuild the identity document into a device-active
    /// hive from the persisted rows (D-38-4 path (B)) — the same path Run 40's
    /// recovery re-uses with a re-imported admin.
    pub(crate) fn from_ceremony(store: &dyn DocStore, material: DeviceMaterial) -> Result<Self, CoreError> {
        let device: Identifier = (&material.signer.verifying_key()).into();
        let (hive, doc_id) = ceremony::reload_with(store, material.signer)?;
        if doc_id != material.identity_doc {
            return Err(CoreError::Ceremony("reload materialised a different identity document".into()));
        }
        Ok(Self { hive, device, identity_doc: Some(doc_id), groups: HashMap::new(), known_ops: HashMap::new() })
    }

    /// Run 39 — after a relaunch: rebuild the device hive from the custodied
    /// seed (STOP 1 (a)) and the persisted rows — identity first
    /// (`ceremony::reload_with`), then every `group:` row pair through the
    /// same ingest path (P-1). Errors if there is no identity row.
    pub(crate) fn reload(store: &dyn DocStore, device_seed: &[u8]) -> Result<Self, CoreError> {
        let signer = signer_from_seed(device_seed)?;
        let device: Identifier = (&signer.verifying_key()).into();
        let (hive, doc_id) = ceremony::reload_with(store, signer)?;
        let mut this = Self { hive, device, identity_doc: Some(doc_id), groups: HashMap::new(), known_ops: HashMap::new() };
        this.reload_groups(store)?;
        Ok(this)
    }

    /// Run 39 — P-1 reload of every persisted group. Row ids are enumerated
    /// through the adapter; a `.keyops` row without its delegation row (or
    /// vice versa), a wrong tag, or an unresolved member is an error.
    fn reload_groups(&mut self, store: &dyn DocStore) -> Result<(), CoreError> {
        let ser = |e: rusqlite::Error| CoreError::Storage(e.to_string());
        let ids = store.ids_with_prefix(GROUP_ROW_PREFIX).map_err(ser)?;
        for id in ids.iter().filter(|i| !i.ends_with(GROUP_KEYOPS_ROW_SUFFIX)) {
            let app_id = &id[GROUP_ROW_PREFIX.len()..];
            let (tag, bytes) = store.read_tagged(id).map_err(ser)?.ok_or_else(|| CoreError::Ceremony(format!("group row {id} vanished")))?;
            if tag != crate::storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1 {
                return Err(CoreError::Ceremony(format!("group row {id} under unexpected tag {tag}")));
            }
            let kid = format!("{id}{GROUP_KEYOPS_ROW_SUFFIX}");
            let (ktag, kbytes) = store.read_tagged(&kid).map_err(ser)?.ok_or_else(|| CoreError::Ceremony(format!("group row {id} has no {kid} row")))?;
            if ktag != crate::storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1 {
                return Err(CoreError::Ceremony(format!("group keyops row {kid} under unexpected tag {ktag}")));
            }
            let delegations = ceremony::decode(&bytes)?;
            let keyops = ceremony::decode_events(&kbytes)?;
            let gid = self.ingest_group(keyops, delegations)?;
            self.groups.insert(app_id.to_string(), gid);
        }
        Ok(())
    }

    /// Run 39 — ingest one persisted group (KeyOps first, then delegations)
    /// and recover its Keyhive id from the root delegation's issuer — the
    /// same recovery `ceremony::reload_with` performs for the identity
    /// document; no map row (ruled in-run).
    fn ingest_group(&mut self, keyops: StaticEvents, delegations: StaticDelegations) -> Result<GroupId, CoreError> {
        let root_issuer = delegations
            .iter()
            .find(|d| d.payload().proof.is_none())
            .map(|d| Identifier::from(d.issuer()))
            .ok_or_else(|| CoreError::Ceremony("group row holds no root delegation".into()))?;
        let gid = GroupId::from(root_issuer);
        for ev in &keyops {
            if let Some(id) = keyop_issuer(ev) {
                self.known_ops.insert(id, ev.clone());
            }
        }
        let mut events: StaticEvents = keyops
            .into_iter()
            .filter(|ev| keyop_issuer(ev) != Some(self.device))
            .collect();
        events.extend(delegations.into_iter().map(StaticEvent::Delegated));
        let pending = crate::runtime::rt().block_on(self.hive.ingest_unsorted_static_events(events));
        if !pending.is_empty() {
            return Err(CoreError::Ceremony(format!("group reload left {} event(s) pending: {pending:?}", pending.len())));
        }
        crate::runtime::rt()
            .block_on(self.hive.get_group(gid))
            .ok_or_else(|| CoreError::Ceremony("group not materialised".into()))?;
        Ok(gid)
    }

    /// Run 39 — P-1 persist: the group's current `members()` delegation set
    /// (verified, sorted by delegate) + the peers' KeyOps, as a row pair.
    /// Only for a hive with an identity (a session-only hive's groups are not
    /// persisted — nothing to reload them under). Called after every landed
    /// grant and revoke.
    fn persist_group(&self, store: &dyn DocStore, group_id: &str, gid: GroupId) -> Result<(), CoreError> {
        if self.identity_doc.is_none() {
            return Ok(());
        }
        let cer = |e: &dyn std::fmt::Debug| CoreError::Ceremony(format!("{e:?}"));
        let (mut statics, member_ids): (StaticDelegations, Vec<Identifier>) = crate::runtime::rt().block_on(async {
            let g = self.hive.get_group(gid).await.ok_or_else(|| CoreError::Membership("group vanished".into()))?;
            let g = g.lock().await;
            let mut statics = Vec::new();
            for dels in g.members().values() {
                for s in dels.iter() {
                    statics.push((**s).clone().map(StaticDelegation::from));
                }
            }
            Ok::<_, CoreError>((statics, g.members().keys().copied().collect()))
        })?;
        for d in &statics {
            d.try_verify().map_err(|e| cer(&e))?;
        }
        statics.sort_by(|a, b| a.payload().delegate.to_bytes().cmp(&b.payload().delegate.to_bytes()));
        let keyops: StaticEvents = member_ids
            .iter()
            .filter(|m| **m != self.device)
            .map(|m| {
                self.known_ops
                    .get(m)
                    .cloned()
                    .ok_or_else(|| CoreError::Membership(format!("no KeyOp known for member {}", fingerprint_of(m))))
            })
            .collect::<Result<_, _>>()?;
        let bytes = bincode::serialize(&statics).map_err(|e| cer(&e))?;
        let kbytes = bincode::serialize(&keyops).map_err(|e| cer(&e))?;
        let id = format!("{GROUP_ROW_PREFIX}{group_id}");
        store.write_keyhive_static_delegations(&id, &bytes).map_err(|e| CoreError::Storage(e.to_string()))?;
        store
            .write_keyhive_static_events(&format!("{id}{GROUP_KEYOPS_ROW_SUFFIX}"), &kbytes)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(())
    }

    /// Run 39 — the membership query: every member of the group named
    /// `group_id` (fingerprint, level, is-this-device), sorted by fingerprint;
    /// empty when no such group.
    pub(crate) fn members(&self, group_id: &str) -> Vec<GroupMember> {
        let Some(gid) = self.groups.get(group_id) else { return vec![] };
        let mut out: Vec<GroupMember> = crate::runtime::rt().block_on(async {
            let Some(g) = self.hive.get_group(*gid).await else { return vec![] };
            let g = g.lock().await;
            g.members()
                .keys()
                .filter_map(|m| {
                    g.get_capability(m).map(|d| GroupMember {
                        fingerprint: fingerprint_of(m),
                        level: GrantLevel::from(d.payload().can()),
                        is_device: *m == self.device,
                    })
                })
                .collect()
        });
        out.sort_by(|a, b| a.fingerprint.cmp(&b.fingerprint));
        out
    }

    /// Run 39 — revoke the member whose fingerprint is `fingerprint` from the
    /// group named `group_id`, through the revoke bar (composed below);
    /// persists the row pair; returns the new membership version.
    pub(crate) fn revoke_member(&mut self, store: &dyn DocStore, group_id: &str, fingerprint: &str) -> Result<u64, CoreError> {
        let gid = *self
            .groups
            .get(group_id)
            .ok_or_else(|| CoreError::Membership(format!("no group named {group_id}")))?;
        let target = crate::runtime::rt().block_on(async {
            let g = self.hive.get_group(gid).await?;
            let g = g.lock().await;
            g.members().keys().find(|m| fingerprint_of(m) == fingerprint).copied()
        });
        let target = target.ok_or_else(|| CoreError::Membership(format!("no member {fingerprint} in {group_id}")))?;
        if target == self.device {
            return Err(CoreError::Membership("the device does not revoke itself".into()));
        }
        self.revoke(gid, target)?;
        self.persist_group(store, group_id, gid)?;
        Ok(self.membership_version(group_id))
    }

    /// H2 (Run 39) — the revoke bar, composed: the DEVICE is the revoker; its
    /// held level on the group is read back, mapped, and checked BEFORE
    /// `revoke_member`. A refusal maps to `CoreError::Policy` and core is
    /// never invoked — even though, for a delegation the device issued, core
    /// at the pin would accept (group.rs L629). Crate-internal; the
    /// refusal test calls it directly.
    pub(crate) fn revoke(&self, gid: GroupId, member: Identifier) -> Result<(), CoreError> {
        let held = self
            .level_on(gid, self.device)
            .ok_or_else(|| CoreError::Membership("device holds no delegation on this group".into()))?;
        policy::check_revoke_bar(held).map_err(|v| CoreError::Policy(v.to_string()))?;
        crate::runtime::rt()
            .block_on(self.hive.revoke_member(member, true, gid))
            .map_err(|e| CoreError::Membership(format!("{e:?}")))?;
        Ok(())
    }

    /// Before any ceremony: a per-session hive with an ephemeral device
    /// signer and no identity document (ruled in-run; the Run 32 shells call
    /// the counter un-enrolled). Membership does not survive relaunch.
    pub(crate) fn session_only() -> Result<Self, CoreError> {
        let signer = MemorySigner::generate(&mut OsRng);
        let device: Identifier = (&signer.verifying_key()).into();
        let hive = crate::runtime::rt()
            .block_on(Keyhive::generate(signer, MemoryCiphertextStore::new(), NoListener, OsRng))
            .map_err(|e| CoreError::Membership(format!("{e:?}")))?;
        Ok(Self { hive, device, identity_doc: None, groups: HashMap::new(), known_ops: HashMap::new() })
    }

    #[cfg(test)]
    pub(crate) fn has_identity(&self) -> bool {
        self.identity_doc.is_some()
    }

    /// The counter, read side: the group's member count minus the device's
    /// own root membership; 0 when no group exists under `group_id`.
    pub(crate) fn membership_version(&self, group_id: &str) -> u64 {
        let Some(gid) = self.groups.get(group_id) else { return 0 };
        crate::runtime::rt().block_on(async {
            match self.hive.get_group(*gid).await {
                Some(g) => (g.lock().await.members().len() as u64).saturating_sub(1),
                None => 0,
            }
        })
    }

    /// The counter, event side: ensure the group, then grant one simulated
    /// peer at `level` through the bar. Returns the new version.
    /// Run 39: persists the group's row pair through `store` after the grant
    /// lands (P-1; no-op on a session-only hive).
    pub(crate) fn record_membership_event(&mut self, store: &dyn DocStore, group_id: &str, level: GrantLevel) -> Result<u64, CoreError> {
        let gid = self.ensure_group(group_id)?;
        let peer = MemorySigner::generate(&mut OsRng);
        let peer_id = self.introduce(peer)?;
        self.grant(gid, peer_id, level)?;
        self.persist_group(store, group_id, gid)?;
        Ok(self.membership_version(group_id))
    }

    /// The device's own level on the group named `group_id`, mapped.
    pub(crate) fn device_grant_level(&self, group_id: &str) -> Option<GrantLevel> {
        let gid = *self.groups.get(group_id)?;
        self.level_on(gid, self.device)
    }

    fn ensure_group(&mut self, group_id: &str) -> Result<GroupId, CoreError> {
        if let Some(gid) = self.groups.get(group_id) {
            return Ok(*gid);
        }
        let gid = crate::runtime::rt()
            .block_on(self.hive.generate_group(vec![]))
            .map_err(|e| CoreError::Membership(format!("{e:?}")))?;
        self.groups.insert(group_id.to_string(), gid);
        Ok(gid)
    }

    /// Register a peer's individual in the device hive by contact card (the
    /// pin's only peer-registration path). Test-visible for the bar test.
    /// Run 39: the peer's `KeyOp` is retained (as its event) so the group's
    /// `.keyops` row can name every member — hence `&mut self` from Run 39.
    pub(crate) fn introduce(&mut self, peer: MemorySigner) -> Result<Identifier, CoreError> {
        let mer = |e: &dyn std::fmt::Debug| CoreError::Membership(format!("{e:?}"));
        let (id, ev) = crate::runtime::rt().block_on(async {
            let peer_hive: Hive = Keyhive::generate(peer, MemoryCiphertextStore::new(), NoListener, OsRng)
                .await
                .map_err(|e| mer(&e))?;
            let card = peer_hive.generate_contact_card().await.map_err(|e| mer(&e))?;
            let id: Identifier = self.hive.receive_contact_card(&card).await.map_err(|e| mer(&e))?.into();
            Ok::<_, CoreError>((id, ceremony::keyop_event(&card)))
        })?;
        self.known_ops.insert(id, ev);
        Ok(id)
    }

    /// Adopt a group another hive created (its static events already
    /// ingested) under an app-owned name — the test path for a group where
    /// the device holds LESS than Admin. Crate-internal.
    #[cfg(test)]
    pub(crate) fn adopt_group(&mut self, group_id: &str, gid: GroupId) {
        self.groups.insert(group_id.to_string(), gid);
    }

    #[cfg(test)]
    pub(crate) fn hive(&self) -> &Hive {
        &self.hive
    }

    #[cfg(test)]
    pub(crate) fn device_id(&self) -> Identifier {
        self.device
    }

    fn level_on(&self, gid: GroupId, who: Identifier) -> Option<GrantLevel> {
        crate::runtime::rt().block_on(async {
            let g = self.hive.get_group(gid).await?;
            let g = g.lock().await;
            g.get_capability(&who).map(|d| GrantLevel::from(d.payload().can()))
        })
    }

    /// H2 — the grant bar, composed: the DEVICE is the granter (the active
    /// agent signs every delegation this hive issues); its held level on the
    /// group is read back, mapped, and checked BEFORE `add_member`. A refusal
    /// maps to `CoreError::Policy` (Run 37 variant, reused) and core is never
    /// invoked.
    pub(crate) fn grant(&self, gid: GroupId, member: Identifier, level: GrantLevel) -> Result<(), CoreError> {
        let held = self
            .level_on(gid, self.device)
            .ok_or_else(|| CoreError::Membership("device holds no delegation on this group".into()))?;
        policy::check_grant_bar(held).map_err(|v| CoreError::Policy(v.to_string()))?;
        crate::runtime::rt()
            .block_on(self.hive.add_member(member, gid, access_of(level), &[]))
            .map_err(|e| CoreError::Membership(format!("{e:?}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ceremony::keyop_event;
    use crate::ceremony::enroll_with_device;
    use crate::{IdentityConfig, RootingLevel};
    use crate::storage::SqliteStore;

    fn store(dir: &tempfile::TempDir, file: &str) -> (String, SqliteStore) {
        let path = dir.path().join(file).to_string_lossy().to_string();
        (path.clone(), SqliteStore::open(&path).unwrap())
    }

    /// Run 39 — WRITTEN FIRST (P-1, plan §9 row 5 "kill/relaunch keeps it").
    /// A ceremony enrolls the device; the device hive grants two peers on a
    /// group (row pair persisted through the adapter under `group:demo` /
    /// `group:demo.keyops`, no new tag); the hive is DROPPED; a fresh hive is
    /// rebuilt from the custodied seed (STOP 1 (a): `signer_from_seed`) and
    /// the rows alone — identity first, then the group through the same
    /// ingest path — and reads the same members at the same levels. A revoke
    /// before the drop is kept too (snapshot semantics).
    #[test]
    fn device_created_group_reloads_from_row_pair_after_relaunch() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "g39.sqlite");
        let primary = MemorySigner::generate(&mut OsRng);
        let recovery = MemorySigner::generate(&mut OsRng);
        let device = MemorySigner::generate(&mut OsRng);
        let seed = device.0.to_bytes().to_vec();
        let (_rec, material) = enroll_with_device(
            &s,
            IdentityConfig { rooting_level: RootingLevel::Edit },
            primary,
            Some(recovery),
            device,
        )
        .unwrap();
        let (before, revoked_fp) = {
            let mut d = DeviceHive::from_ceremony(&s, material).unwrap();
            assert!(d.has_identity());
            assert_eq!(d.record_membership_event(&s, "demo", GrantLevel::Read).unwrap(), 1);
            assert_eq!(d.record_membership_event(&s, "demo", GrantLevel::Edit).unwrap(), 2);
            assert_eq!(d.record_membership_event(&s, "demo", GrantLevel::Read).unwrap(), 3);
            let ids = s.ids_with_prefix(GROUP_ROW_PREFIX).unwrap();
            assert_eq!(ids, vec!["group:demo".to_string(), "group:demo.keyops".to_string()]);
            assert_eq!(s.read_tagged("group:demo").unwrap().unwrap().0, crate::storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1);
            assert_eq!(s.read_tagged("group:demo.keyops").unwrap().unwrap().0, crate::storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1);
            // revoke one Read peer before the "relaunch"
            let victim = d.members("demo").into_iter().find(|m| m.level == GrantLevel::Read && !m.is_device).unwrap();
            assert_eq!(d.revoke_member(&s, "demo", &victim.fingerprint).unwrap(), 2);
            (d.members("demo"), victim.fingerprint)
        }; // hive dropped here — the "kill"
        assert_eq!(before.len(), 3, "device + two remaining peers");
        assert!(before.iter().any(|m| m.is_device && m.level == GrantLevel::Admin));

        let d2 = DeviceHive::reload(&s, &seed).expect("reload from seed + rows; no admin signer, no map row");
        assert!(d2.has_identity());
        assert_eq!(d2.membership_version("demo"), 2, "relaunch keeps it");
        assert_eq!(d2.device_grant_level("demo"), Some(GrantLevel::Admin));
        let after = d2.members("demo");
        assert_eq!(after, before, "same members, same levels, same fingerprints");
        assert!(!after.iter().any(|m| m.fingerprint == revoked_fp), "the revoked peer stays revoked");
        assert_eq!(d2.membership_version("other"), 0);
        // and the reloaded hive can keep granting (the device is Admin again)
        let mut d2 = d2;
        assert_eq!(d2.record_membership_event(&s, "demo", GrantLevel::Read).unwrap(), 3);
    }

    /// Run 39 — STOP 1 (a): the seed round-trips to the SAME verifying key
    /// (what `from_bytes` guarantees and option (d) did not); wrong lengths
    /// are refused at the boundary.
    #[test]
    fn seed_rebuilds_the_same_device_signer() {
        let device = MemorySigner::generate(&mut OsRng);
        let seed = device.0.to_bytes();
        let again = signer_from_seed(&seed).unwrap();
        assert_eq!(again.verifying_key(), device.verifying_key());
        assert!(matches!(signer_from_seed(&seed[..31]), Err(CoreError::Ceremony(_))));
        assert!(matches!(signer_from_seed(&[0u8; 33]), Err(CoreError::Ceremony(_))));
    }

    /// Run 39 — plan §9 row 5: revoke observed; the revoke bar is POLICY, not
    /// core. Another hive creates a group and delegates the device at `Edit`;
    /// the device (bypassing the bar, test-only, straight into core) grants a
    /// peer at `Read` — a delegation the DEVICE ISSUED. Its revoke attempt
    /// through the bar returns `CoreError::Policy` with the group untouched;
    /// the CONTROL calls core's `revoke_member` directly and core ACCEPTS
    /// (branch `to_revoke.issuer == vk`, group.rs L629 — ✓ on the text;
    /// `NoProof` is never raised), proving the bar was the only refusal.
    #[test]
    fn below_admin_revoke_is_refused_by_policy_and_core_would_have_accepted() {
        let dir = tempfile::tempdir().unwrap();
        let (_, s) = store(&dir, "g39b.sqlite");
        let mut d = DeviceHive::session_only().unwrap();
        let other = MemorySigner::generate(&mut OsRng);
        let (gid, events) = crate::runtime::rt().block_on(async {
            let other_hive: Hive =
                Keyhive::generate(other, MemoryCiphertextStore::new(), NoListener, OsRng).await.unwrap();
            let other_card = other_hive.generate_contact_card().await.unwrap();
            let device_card = d.hive().generate_contact_card().await.unwrap();
            let device_id = other_hive.receive_contact_card(&device_card).await.unwrap();
            let gid = other_hive.generate_group(vec![]).await.unwrap();
            other_hive.add_member(device_id, gid, Access::Edit, &[]).await.unwrap();
            let g = other_hive.get_group(gid).await.unwrap();
            let g = g.lock().await;
            let mut events: Vec<StaticEvent<[u8; 32]>> = vec![keyop_event(&other_card)];
            for dels in g.members().values() {
                for s in dels.iter() {
                    events.push(StaticEvent::Delegated((**s).clone().map(StaticDelegation::from)));
                }
            }
            (gid, events)
        });
        let pending = crate::runtime::rt().block_on(d.hive().ingest_unsorted_static_events(events));
        assert!(pending.is_empty());
        d.adopt_group("shared", gid);
        assert_eq!(d.device_grant_level("shared"), Some(GrantLevel::Edit));
        // the device issues a Read grant STRAIGHT INTO CORE (test-only bypass of the grant bar)
        let peer = d.introduce(MemorySigner::generate(&mut OsRng)).unwrap();
        crate::runtime::rt().block_on(d.hive().add_member(peer, gid, Access::Read, &[])).unwrap();
        assert_eq!(d.membership_version("shared"), 2, "other admin + device + peer, minus the device");
        let peer_fp = fingerprint_of(&peer);
        assert!(d.members("shared").iter().any(|m| m.fingerprint == peer_fp && m.level == GrantLevel::Read));

        // through the bar: refused by POLICY, core never invoked
        match d.revoke_member(&s, "shared", &peer_fp) {
            Err(CoreError::Policy(msg)) => assert_eq!(msg, "revoke bar: revoker holds Edit, bar is Admin"),
            other => panic!("expected CoreError::Policy, got {other:?}"),
        }
        assert_eq!(d.membership_version("shared"), 2, "untouched");
        assert!(d.members("shared").iter().any(|m| m.fingerprint == peer_fp));

        // CONTROL: core alone, same revoker, same delegation — accepted (issuer branch)
        let core_result = crate::runtime::rt().block_on(d.hive().revoke_member(peer, true, gid));
        assert!(core_result.is_ok(), "core accepts the issuer's own revocation at Edit: {core_result:?}");
        assert_eq!(d.membership_version("shared"), 1, "core landed it — the bar was the only refusal");
        assert!(!d.members("shared").iter().any(|m| m.fingerprint == peer_fp));

        // and on a device-created group the same revoke passes the bar
        assert_eq!(d.record_membership_event(&s, "mine", GrantLevel::Read).unwrap(), 1);
        let mine = d.members("mine").into_iter().find(|m| !m.is_device).unwrap();
        assert_eq!(d.revoke_member(&s, "mine", &mine.fingerprint).unwrap(), 0, "members() − 1 falls");
        assert!(matches!(d.revoke_member(&s, "mine", &mine.fingerprint), Err(CoreError::Membership(_))), "already gone");
        let me = d.members("mine").into_iter().find(|m| m.is_device).unwrap();
        assert!(matches!(d.revoke_member(&s, "mine", &me.fingerprint), Err(CoreError::Membership(_))), "no self-revoke");
        assert!(s.ids_with_prefix(GROUP_ROW_PREFIX).unwrap().is_empty(), "session-only hive persists nothing");
    }

    /// Plan §9 row 4 done-when (1): the counter reads Keyhive group
    /// membership — version 0 with no group; each event is a real delegation
    /// on a group the device created (device = Admin), and the version is
    /// read back from `members()`.
    #[test]
    fn counter_reads_keyhive_group_membership() {
        let s = SqliteStore::open(":memory:").unwrap();
        let mut d = DeviceHive::session_only().unwrap();
        assert!(!d.has_identity());
        assert_eq!(d.membership_version("group-a"), 0);
        assert_eq!(d.device_grant_level("group-a"), None, "no group yet");
        assert_eq!(d.record_membership_event(&s, "group-a", GrantLevel::Read).unwrap(), 1);
        assert_eq!(d.device_grant_level("group-a"), Some(GrantLevel::Admin), "creator is Admin");
        assert_eq!(d.record_membership_event(&s, "group-a", GrantLevel::Edit).unwrap(), 2);
        assert_eq!(d.membership_version("group-a"), 2);
        assert_eq!(d.membership_version("group-b"), 0, "per-group isolation");
        // read back from the live group, not a counter
        let gid = *d.groups.get("group-a").unwrap();
        crate::runtime::rt().block_on(async {
            let g = d.hive().get_group(gid).await.unwrap();
            let g = g.lock().await;
            assert_eq!(g.members().len(), 3, "device + two peers");
            let levels: Vec<Access> = g.members().keys().map(|m| g.get_capability(m).unwrap().payload().can()).collect();
            assert_eq!(levels.iter().filter(|c| **c == Access::Admin).count(), 1);
            assert_eq!(levels.iter().filter(|c| **c == Access::Read).count(), 1);
            assert_eq!(levels.iter().filter(|c| **c == Access::Edit).count(), 1);
        });
    }

    /// Plan §9 row 4 done-when (2), H2: a below-Admin grant attempt is refused
    /// by POLICY, not by core. Another hive creates a group and delegates the
    /// device at `Edit`; the device hive ingests that group (KeyOp + the two
    /// delegations, nothing pending); the device's held level reads `Edit`;
    /// its grant attempt returns `CoreError::Policy` and the group is
    /// untouched — core's `add_member` was never reached (member count and
    /// delegation count unchanged; at the pin core would have ACCEPTED the
    /// non-escalating Read grant — D-38 record §1 (ii)-5).
    #[test]
    fn below_admin_grant_is_refused_by_policy_before_core() {
        let s = SqliteStore::open(":memory:").unwrap();
        let mut d = DeviceHive::session_only().unwrap();
        let other = MemorySigner::generate(&mut OsRng);
        let (gid, events) = crate::runtime::rt().block_on(async {
            let other_hive: Hive =
                Keyhive::generate(other, MemoryCiphertextStore::new(), NoListener, OsRng).await.unwrap();
            let other_card = other_hive.generate_contact_card().await.unwrap();
            // the device is introduced to the other hive by contact card
            let device_card = d.hive().generate_contact_card().await.unwrap();
            let device_id = other_hive.receive_contact_card(&device_card).await.unwrap();
            let gid = other_hive.generate_group(vec![]).await.unwrap();
            other_hive.add_member(device_id, gid, Access::Edit, &[]).await.unwrap();
            let g = other_hive.get_group(gid).await.unwrap();
            let g = g.lock().await;
            let mut events: Vec<StaticEvent<[u8; 32]>> = vec![keyop_event(&other_card)];
            for dels in g.members().values() {
                for s in dels.iter() {
                    events.push(StaticEvent::Delegated((**s).clone().map(StaticDelegation::from)));
                }
            }
            (gid, events)
        });
        let pending = crate::runtime::rt().block_on(d.hive().ingest_unsorted_static_events(events));
        assert!(pending.is_empty(), "group ingested with nothing pending: {pending:?}");
        d.adopt_group("shared", gid);
        assert_eq!(d.device_grant_level("shared"), Some(GrantLevel::Edit), "the device holds Edit here");
        assert_eq!(d.membership_version("shared"), 1, "other admin + device, minus the device");

        let before = crate::runtime::rt().block_on(async {
            let g = d.hive().get_group(gid).await.unwrap();
            let g = g.lock().await;
            (g.members().len(), g.members().values().map(|v| v.len()).sum::<usize>())
        });
        match d.record_membership_event(&s, "shared", GrantLevel::Read) {
            Err(CoreError::Policy(msg)) => assert_eq!(msg, "grant bar: granter holds Edit, bar is Admin"),
            other => panic!("expected CoreError::Policy, got {other:?}"),
        }
        let after = crate::runtime::rt().block_on(async {
            let g = d.hive().get_group(gid).await.unwrap();
            let g = g.lock().await;
            (g.members().len(), g.members().values().map(|v| v.len()).sum::<usize>())
        });
        assert_eq!(before, after, "core never invoked: members and delegations unchanged");
        assert_eq!(d.membership_version("shared"), 1);
        // and on a group the device created, the same call passes the bar
        assert_eq!(d.record_membership_event(&s, "mine", GrantLevel::Read).unwrap(), 1);
        let _ = d.device_id();
    }

    /// D-38-2: the mapping is 1:1 and round-trips.
    #[test]
    fn grant_level_maps_one_to_one_onto_access() {
        for (a, l) in [
            (Access::Relay, GrantLevel::Relay),
            (Access::Read, GrantLevel::Read),
            (Access::Edit, GrantLevel::Edit),
            (Access::Admin, GrantLevel::Admin),
        ] {
            assert_eq!(GrantLevel::from(a), l);
            assert_eq!(access_of(l), a);
        }
    }
}
