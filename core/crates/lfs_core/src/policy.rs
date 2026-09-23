//! Consumer policy — Run 36 (1b entry), Phase 1 Build Plan v0.1.1 §4 H2.
//!
//! This module is CONSUMER POLICY: the rules this application imposes on
//! its own identity ceremony and on who may grant, stated in the app's own
//! terms. It relies on NO enforcement inside `keyhive_core` — PR #230
//! ("Keyline") moves the grant-side `>= Admin` bar out of core semantics
//! into consumer policy, and the pinned rev `90fe4a51` predates that move;
//! either way the bar lives HERE so an upstream change touches one file
//! (plan §4 H2, v0.1.1 "named policy module" clause). Nothing in this file
//! calls into `keyhive_core`; the mirrored level vocabulary below is a
//! deliberate copy, not an import.
//!
//! Skeleton only (plan §9 row 2). The two rules are NAMED as types and
//! functions; nothing calls them yet. Run 37 (identity ceremony) is the first
//! caller of the delegation floor; Run 38 (groups behind the membership
//! counter) wires the grant bar so that a below-Admin grant attempt is
//! refused by this policy, not by core.
//!
//! Sources: spec v0.1.4 §5.2 (admin-key floor, two admin delegations, safe
//! under Edit-rooting only) and L-12 (floor withdrawn under Admin-rooting
//! unless the recovery delegation is a FROST 2-of-3 share; ruled after
//! Keyline merges); memo v0.1.2 §6 (rooting level named, not ruled);
//! plan v0.1.1 §4 H2/H5. The FROST-share replacement is a CONDITION named by
//! L-12, not a Phase 1 evaluation — it is not modelled here.

use crate::RootingLevel;

/// Rule 1 — the admin-key floor (spec §5.2): the `identity` document and
/// every document the participant owns are created with at least this many
/// admin delegations (the primary cold admin key and the recovery key, itself
/// an admin delegation). Enrollment is refused below the floor.
pub const DELEGATION_FLOOR: usize = 2;

/// Capability level vocabulary for the grant bar, in the app's own terms.
/// Order mirrors `keyhive_core::access::Access` at the H6 pin (`90fe4a51`,
/// source read in Run 36: `Relay < Read < Edit < Admin`, later levels imply
/// earlier ones). Mirrored, NOT imported: the policy is stated without a
/// dependency on core's type so a core-side rename or bar move is absorbed
/// by this file alone. The mapping from a live Keyhive capability to a
/// `GrantLevel` is Run 38's work (the first run that holds Keyhive groups).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GrantLevel {
    Relay,
    Read,
    Edit,
    Admin,
}

/// Rule 2 — "no grants from below Admin" (plan §4 H2). The minimum level a
/// granter must hold for this app to accept its grant as valid consumer-side.
pub const GRANT_BAR: GrantLevel = GrantLevel::Admin;

/// A consumer-policy refusal. Not a `CoreError` variant and not an FFI type
/// in Run 36: the policy is not yet reachable from any entry point. When
/// Run 37/38 wire it, the mapping onto `CoreError` is ruled there.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PolicyViolation {
    #[error("delegation floor: {have} admin delegation(s) present, floor is {floor}")]
    BelowDelegationFloor { have: usize, floor: usize },
    #[error("grant bar: granter holds {have:?}, bar is {bar:?}")]
    GranterBelowBar { have: GrantLevel, bar: GrantLevel },
}

/// Rule 1 as a function. `admin_delegations` is the count the caller has
/// established for the document being created (Run 37 supplies it from the
/// ceremony's own delegation list — this module does not read Keyhive state).
pub fn check_delegation_floor(admin_delegations: usize) -> Result<(), PolicyViolation> {
    if admin_delegations >= DELEGATION_FLOOR {
        Ok(())
    } else {
        Err(PolicyViolation::BelowDelegationFloor {
            have: admin_delegations,
            floor: DELEGATION_FLOOR,
        })
    }
}

/// Rule 2 as a function. Refuses a grant whose granter holds less than
/// [`GRANT_BAR`]. Consumer-side: nothing here asks core whether the grant is
/// otherwise valid; that composition is Run 38's.
pub fn check_grant_bar(granter: GrantLevel) -> Result<(), PolicyViolation> {
    if granter >= GRANT_BAR {
        Ok(())
    } else {
        Err(PolicyViolation::GranterBelowBar {
            have: granter,
            bar: GRANT_BAR,
        })
    }
}

/// Whether the delegation floor (Rule 1) is a SAFE guarantee under the
/// configured rooting level — spec L-12, stated as a status, not enforced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloorStatus {
    /// Edit-rooting: a compromised recovery key is evicted by the retained
    /// subject key (rotation, no identity change) — the floor holds.
    Safe,
    /// Admin-rooting: the recovery key is an ever-apex admin of equal
    /// standing that no rotation can remove; the floor is WITHDRAWN pending
    /// L-12's ruling unless the recovery delegation is a FROST 2-of-3 share
    /// (not modelled in Phase 1). The ceremony run records this status
    /// alongside the configured level and cites L-12 (plan §4 H5).
    WithdrawnPendingL12,
}

/// Spec L-12 / §5.2 floor-withdrawal caveat as a lookup. No rooting level is
/// chosen here (memo §6: unruled; validation event = PR #230 merge); the
/// level arrives from configuration (`crate::IdentityConfig`).
pub fn delegation_floor_status(level: RootingLevel) -> FloorStatus {
    match level {
        RootingLevel::Edit => FloorStatus::Safe,
        RootingLevel::Admin => FloorStatus::WithdrawnPendingL12,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegation_floor_is_two_and_refuses_below() {
        assert_eq!(DELEGATION_FLOOR, 2);
        assert!(check_delegation_floor(2).is_ok());
        assert!(check_delegation_floor(3).is_ok());
        assert_eq!(
            check_delegation_floor(1),
            Err(PolicyViolation::BelowDelegationFloor { have: 1, floor: 2 })
        );
        assert!(check_delegation_floor(0).is_err());
    }

    #[test]
    fn grant_bar_is_admin_and_refuses_below() {
        assert_eq!(GRANT_BAR, GrantLevel::Admin);
        assert!(check_grant_bar(GrantLevel::Admin).is_ok());
        for below in [GrantLevel::Relay, GrantLevel::Read, GrantLevel::Edit] {
            assert_eq!(
                check_grant_bar(below),
                Err(PolicyViolation::GranterBelowBar { have: below, bar: GrantLevel::Admin })
            );
        }
    }

    #[test]
    fn grant_levels_are_ordered_like_the_pinned_access_enum() {
        // Relay < Read < Edit < Admin (keyhive_core::access::Access at 90fe4a51).
        assert!(GrantLevel::Relay < GrantLevel::Read);
        assert!(GrantLevel::Read < GrantLevel::Edit);
        assert!(GrantLevel::Edit < GrantLevel::Admin);
    }

    #[test]
    fn floor_is_safe_under_edit_rooting_only() {
        assert_eq!(delegation_floor_status(RootingLevel::Edit), FloorStatus::Safe);
        assert_eq!(
            delegation_floor_status(RootingLevel::Admin),
            FloorStatus::WithdrawnPendingL12
        );
    }
}
