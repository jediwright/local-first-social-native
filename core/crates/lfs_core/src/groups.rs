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

use crate::ceremony::{self, DeviceMaterial};
use crate::policy::{self, GrantLevel};
use crate::storage::DocStore;
use crate::CoreError;
use future_form::Sendable;
use keyhive_core::access::Access;
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
    #[allow(dead_code)]
    identity_doc: Option<DocumentId>,
    groups: HashMap<String, GroupId>,
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
        Ok(Self { hive, device, identity_doc: Some(doc_id), groups: HashMap::new() })
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
        Ok(Self { hive, device, identity_doc: None, groups: HashMap::new() })
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
    pub(crate) fn record_membership_event(&mut self, group_id: &str, level: GrantLevel) -> Result<u64, CoreError> {
        let gid = self.ensure_group(group_id)?;
        let peer = MemorySigner::generate(&mut OsRng);
        let peer_id = self.introduce(peer)?;
        self.grant(gid, peer_id, level)?;
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
    pub(crate) fn introduce(&self, peer: MemorySigner) -> Result<Identifier, CoreError> {
        let mer = |e: &dyn std::fmt::Debug| CoreError::Membership(format!("{e:?}"));
        crate::runtime::rt().block_on(async {
            let peer_hive: Hive = Keyhive::generate(peer, MemoryCiphertextStore::new(), NoListener, OsRng)
                .await
                .map_err(|e| mer(&e))?;
            let card = peer_hive.generate_contact_card().await.map_err(|e| mer(&e))?;
            let id = self.hive.receive_contact_card(&card).await.map_err(|e| mer(&e))?;
            Ok(id.into())
        })
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
    use keyhive_core::event::static_event::StaticEvent;
    use keyhive_core::principal::group::delegation::StaticDelegation;

    /// Plan §9 row 4 done-when (1): the counter reads Keyhive group
    /// membership — version 0 with no group; each event is a real delegation
    /// on a group the device created (device = Admin), and the version is
    /// read back from `members()`.
    #[test]
    fn counter_reads_keyhive_group_membership() {
        let mut d = DeviceHive::session_only().unwrap();
        assert!(!d.has_identity());
        assert_eq!(d.membership_version("group-a"), 0);
        assert_eq!(d.device_grant_level("group-a"), None, "no group yet");
        assert_eq!(d.record_membership_event("group-a", GrantLevel::Read).unwrap(), 1);
        assert_eq!(d.device_grant_level("group-a"), Some(GrantLevel::Admin), "creator is Admin");
        assert_eq!(d.record_membership_event("group-a", GrantLevel::Edit).unwrap(), 2);
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
        match d.record_membership_event("shared", GrantLevel::Read) {
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
        assert_eq!(d.record_membership_event("mine", GrantLevel::Read).unwrap(), 1);
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
