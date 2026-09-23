//! Run 32 (Phase 1 1a) — profile / pings / threads documents as plain
//! Automerge (plan §1 1a: "Three doc types serialize/deserialize via samod;
//! stored in SQLite"). Shapes derive from the socialpings spec v1.0 Data
//! Model (§Data Model: `profile`, `pings`, `threads` maps). No Keyhive
//! groups, no sync — the structs are CRDT document shapes only.
//!
//! Scope notes (recorded, not silent):
//! - The spec's data model has five maps; the plan's 1a row names three doc
//!   types. `channels` and `assets` are plan-scoped out of 1a, not dropped
//!   from the spec.
//! - Timestamps are ISO-8601 strings, as in the spec's JS model. Ping/trust
//!   vocabulary (`type`, `tier`) carries as strings, not Rust enums: the
//!   five ping types and four trust tiers are spec vocabulary; string
//!   carriage avoids an enum-encoding ruling this run has no mandate for.
//! - Rust field naming is snake_case; the spec's camelCase names map 1:1
//!   (displayName → display_name, etc.). Shape divergence: none intended;
//!   any found later is flagged against the spec, not silently adapted.
//!
//! Run 33 (local-UI wiring) — these structs ARE the typed FFI surface
//! (build rule, record §3): each derives `uniffi::Record` so the spec-derived
//! shape crosses the boundary once, from one definition, with no mirror
//! types and no JSON schema copy in either shell. The display requirement
//! decided it: both shells render typed lists (spec `ContactList` /
//! `PingFeed` / `ThreadList` semantics) and need compile-time field access,
//! not a parser. Consequence of the choice, recorded not silent: maps are
//! `HashMap`, not `BTreeMap` — uniffi 0.32.1 lowers `HashMap<K, V>` only
//! (uniffi_core `ffi_converter_impls.rs` L383/L405); autosurgeon supports
//! both. Automerge maps are unordered, so no CRDT ordering is lost; shells
//! sort keys for display. Shape vs spec: unchanged.
use autosurgeon::{Hydrate, Reconcile};
use std::collections::HashMap;

// ---------------------------------------------------------------- profile

/// `profile.identity` (spec: displayName, handle, handleRegisteredAt,
/// avatarColor, createdAt).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct Identity {
    pub display_name: String,
    pub handle: String,
    pub handle_registered_at: String,
    pub avatar_color: String,
    pub created_at: String,
}

/// `profile.preferences` (spec: defaultPingType, notificationsEnabled,
/// discoverable).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct Preferences {
    pub default_ping_type: String,
    pub notifications_enabled: bool,
    pub discoverable: bool,
}

/// One `profile.trust_graph` entry (spec: contactId → { tier, connectedAt,
/// syncStatus }).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct TrustEntry {
    pub tier: String,
    pub connected_at: String,
    pub sync_status: String,
}

/// One `profile.channel_memberships` entry (spec: { channelId, joinedAt,
/// lastPingAt }).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct ChannelMembership {
    pub channel_id: String,
    pub joined_at: String,
    pub last_ping_at: String,
}

/// The profile document (spec `profile` map: identity, preferences,
/// trust_graph, ping_history, channel_memberships).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct ProfileDoc {
    pub identity: Identity,
    pub preferences: Preferences,
    pub trust_graph: HashMap<String, TrustEntry>,
    pub ping_history: Vec<Ping>,
    pub channel_memberships: Vec<ChannelMembership>,
}

// ------------------------------------------------------------------ pings

/// One ping (spec: { pingId, type, senderId, sentAt, expiresAt, content? }).
/// `type` is a reserved word in Rust; carried as `ping_type`.
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct Ping {
    pub ping_id: String,
    pub ping_type: String,
    pub sender_id: String,
    pub sent_at: String,
    pub expires_at: String,
    pub content: Option<String>,
}

/// The pings document (spec `pings` map: channelId → array of pings).
/// Ephemeral by spec design; expiry cleanup is a runtime behavior, not a
/// shape property. Run 33 scope: the spec's "cleanup observer removes
/// expired entries on document load" is NOT implemented here — expired
/// pings stay in the document and the shells filter them at display time
/// (`expires_at` vs now). Deferred, not adapted: the shape is unchanged and
/// a cleanup-on-load step can be added at the core later without touching
/// the FFI surface.
///
/// Run 35 (1a hardening) — that step now exists: `Core::open_typed_doc_at
/// (id, Pings, now)` removes expired entries on load (lib.rs). The Run 33
/// paragraph above is kept verbatim as history (binding-diff discipline:
/// additions only). The shape is unchanged; `open_typed_doc` without a
/// clock still returns pings as stored; the shells' display filter is
/// KEPT (belt-and-braces).
///
/// Run 35 expiry rules (`ping_expiry` below): a ping is expired iff its
/// `expires_at` parses as RFC 3339 and is strictly earlier than the
/// injected `now`; an `expires_at` that does not parse is RETAINED, never
/// silently dropped (the shell's filter decides what to show). The clock
/// is always a parameter — library code never reads the wall.
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct PingsDoc {
    pub channels: HashMap<String, Vec<Ping>>,
}

// ---------------------------------------------------------------- threads

/// One thread message (spec: { messageId, senderId, sentAt, content,
/// assetRef?, readAt? }).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct Message {
    pub message_id: String,
    pub sender_id: String,
    pub sent_at: String,
    pub content: String,
    pub asset_ref: Option<String>,
    pub read_at: Option<String>,
}

/// The threads document (spec `threads` map: contactId → array of messages).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile, uniffi::Record)]
pub struct ThreadsDoc {
    pub threads: HashMap<String, Vec<Message>>,
}

// ------------------------------------------------------------ ping expiry
//
// Run 35 — expiry decision for the cleanup-on-load step. Kept in the docs
// module beside the `Ping` shape it reads; no new file (five-file sheet
// discipline) and no new dependency (no chrono/time: the scaffold and
// canonical manifests stay pin-for-pin as Run 33 left them). RFC 3339 is
// the subset both shells emit — Kotlin `Instant.toString()` (fractional
// seconds, `Z`) and Swift `ISO8601DateFormatter` (whole seconds, `Z`) —
// plus numeric offsets, which the spec's ISO-8601 strings permit. Both
// timestamps are reduced to a UTC instant before comparison; comparing
// the strings lexically would mis-order across offsets and fractional
// digits (Run 33 leg (c) observed both shells emitting different lengths).

/// A UTC instant: whole seconds since 1970-01-01T00:00:00Z plus nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UtcInstant {
    pub secs: i64,
    pub nanos: u32,
}

/// Parse an RFC 3339 timestamp (`YYYY-MM-DDTHH:MM:SS[.frac](Z|±HH:MM)`)
/// into a [`UtcInstant`]. Returns `None` for anything else — the caller
/// decides what a non-parse means (for `expires_at`: retain the ping).
/// Leap seconds (`SS = 60`) are accepted and folded into the next second,
/// matching common `Instant` implementations. Lower-case `t`/`z` accepted
/// (RFC 3339 §5.6 permits them).
pub(crate) fn parse_rfc3339_utc(s: &str) -> Option<UtcInstant> {
    let b = s.as_bytes();
    if b.len() < 20 {
        return None;
    }
    let num = |i: usize, n: usize| -> Option<i64> {
        let sl = b.get(i..i + n)?;
        if !sl.iter().all(|c| c.is_ascii_digit()) {
            return None;
        }
        std::str::from_utf8(sl).ok()?.parse::<i64>().ok()
    };
    let year = num(0, 4)?;
    if b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let month = num(5, 2)?;
    let day = num(8, 2)?;
    if !(b[10] == b'T' || b[10] == b't') || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let hour = num(11, 2)?;
    let minute = num(14, 2)?;
    let second = num(17, 2)?;
    let mut i = 19;
    // optional fraction
    let mut nanos: u32 = 0;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start {
            return None;
        }
        let mut scale: u32 = 100_000_000;
        for &c in &b[start..i] {
            if scale == 0 {
                break; // beyond nanosecond precision: truncate
            }
            nanos += (c - b'0') as u32 * scale;
            scale /= 10;
        }
    }
    // offset
    let offset_secs: i64 = match b.get(i) {
        Some(b'Z') | Some(b'z') => {
            i += 1;
            0
        }
        Some(sign @ (b'+' | b'-')) => {
            let oh = num(i + 1, 2)?;
            if b.get(i + 3) != Some(&b':') {
                return None;
            }
            let om = num(i + 4, 2)?;
            if oh > 23 || om > 59 {
                return None;
            }
            i += 6;
            let o = oh * 3600 + om * 60;
            if *sign == b'-' { -o } else { o }
        }
        _ => return None,
    };
    if i != b.len() {
        return None;
    }
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    if day > days_in_month(year, month) {
        return None;
    }
    let days = days_from_civil(year, month, day);
    let local = days * 86_400 + hour * 3600 + minute * 60 + second;
    Some(UtcInstant { secs: local - offset_secs, nanos })
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 29 } else { 28 }
        }
        _ => 0,
    }
}

/// Days since 1970-01-01 for a proleptic-Gregorian civil date
/// (Howard Hinnant's `days_from_civil`; exact for all i64-safe years).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Expiry decision for one ping. `Some(true)` = expired (strictly before
/// `now`), `Some(false)` = active, `None` = `expires_at` did not parse —
/// the caller RETAINS the ping (never silently dropped).
pub(crate) fn ping_expiry(expires_at: &str, now: UtcInstant) -> Option<bool> {
    parse_rfc3339_utc(expires_at).map(|e| e < now)
}

#[cfg(test)]
mod expiry_tests {
    use super::*;

    fn at(s: &str) -> UtcInstant {
        parse_rfc3339_utc(s).expect("fixture parses")
    }

    #[test]
    fn parses_both_shell_formats_and_offsets() {
        // Swift ISO8601DateFormatter shape (whole seconds, Z)
        let swift = at("2026-09-22T12:00:00Z");
        // Kotlin Instant.toString shape (fractional seconds, Z)
        let kotlin = at("2026-09-22T12:00:00.123456789Z");
        assert_eq!(swift.secs, kotlin.secs);
        assert_eq!(kotlin.nanos, 123_456_789);
        assert!(swift < kotlin);
        // numeric offset reduces to the same instant
        assert_eq!(at("2026-09-22T08:00:00-04:00").secs, swift.secs);
        assert_eq!(at("2026-09-22T14:00:00+02:00").secs, swift.secs);
        // epoch anchor
        assert_eq!(at("1970-01-01T00:00:00Z"), UtcInstant { secs: 0, nanos: 0 });
        // known instant: 2000-03-01T00:00:00Z = 951868800
        assert_eq!(at("2000-03-01T00:00:00Z").secs, 951_868_800);
        // more than nine fractional digits truncate rather than fail
        assert_eq!(at("2026-09-22T12:00:00.1234567891Z").nanos, 123_456_789);
    }

    #[test]
    fn rejects_non_rfc3339() {
        for bad in [
            "", "soon", "2026-09-22", "2026-09-22T12:00Z", "2026-09-22 12:00:00Z",
            "2026-09-22T12:00:00", "2026-09-22T12:00:00+0400", "2026-13-01T00:00:00Z",
            "2026-02-30T00:00:00Z", "2026-09-22T24:00:00Z", "2026-09-22T12:00:00.Z",
            "1695384000", "2026-09-22T12:00:00Z ",
        ] {
            assert!(parse_rfc3339_utc(bad).is_none(), "should not parse: {bad:?}");
        }
    }

    #[test]
    fn expiry_is_strict_and_none_on_unparseable() {
        let now = at("2026-09-22T12:00:00Z");
        assert_eq!(ping_expiry("2026-09-22T11:59:59Z", now), Some(true));
        assert_eq!(ping_expiry("2026-09-22T12:00:00Z", now), Some(false)); // boundary: not yet expired
        assert_eq!(ping_expiry("2026-09-22T12:00:00.000000001Z", now), Some(false));
        assert_eq!(ping_expiry("2026-09-29T12:00:00Z", now), Some(false));
        assert_eq!(ping_expiry("whenever", now), None);
    }
}
