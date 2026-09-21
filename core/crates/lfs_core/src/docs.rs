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
use autosurgeon::{Hydrate, Reconcile};
use std::collections::BTreeMap;

// ---------------------------------------------------------------- profile

/// `profile.identity` (spec: displayName, handle, handleRegisteredAt,
/// avatarColor, createdAt).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
pub struct Identity {
    pub display_name: String,
    pub handle: String,
    pub handle_registered_at: String,
    pub avatar_color: String,
    pub created_at: String,
}

/// `profile.preferences` (spec: defaultPingType, notificationsEnabled,
/// discoverable).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
pub struct Preferences {
    pub default_ping_type: String,
    pub notifications_enabled: bool,
    pub discoverable: bool,
}

/// One `profile.trust_graph` entry (spec: contactId → { tier, connectedAt,
/// syncStatus }).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
pub struct TrustEntry {
    pub tier: String,
    pub connected_at: String,
    pub sync_status: String,
}

/// One `profile.channel_memberships` entry (spec: { channelId, joinedAt,
/// lastPingAt }).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
pub struct ChannelMembership {
    pub channel_id: String,
    pub joined_at: String,
    pub last_ping_at: String,
}

/// The profile document (spec `profile` map: identity, preferences,
/// trust_graph, ping_history, channel_memberships).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
pub struct ProfileDoc {
    pub identity: Identity,
    pub preferences: Preferences,
    pub trust_graph: BTreeMap<String, TrustEntry>,
    pub ping_history: Vec<Ping>,
    pub channel_memberships: Vec<ChannelMembership>,
}

// ------------------------------------------------------------------ pings

/// One ping (spec: { pingId, type, senderId, sentAt, expiresAt, content? }).
/// `type` is a reserved word in Rust; carried as `ping_type`.
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
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
/// shape property, and is out of this run's scope.
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
pub struct PingsDoc {
    pub channels: BTreeMap<String, Vec<Ping>>,
}

// ---------------------------------------------------------------- threads

/// One thread message (spec: { messageId, senderId, sentAt, content,
/// assetRef?, readAt? }).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
pub struct Message {
    pub message_id: String,
    pub sender_id: String,
    pub sent_at: String,
    pub content: String,
    pub asset_ref: Option<String>,
    pub read_at: Option<String>,
}

/// The threads document (spec `threads` map: contactId → array of messages).
#[derive(Default, Debug, Clone, PartialEq, Hydrate, Reconcile)]
pub struct ThreadsDoc {
    pub threads: BTreeMap<String, Vec<Message>>,
}
