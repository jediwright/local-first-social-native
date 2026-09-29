# Gap register

Gaps this app has found in the Ink & Switch local-first stack, Keyhive first. Each gap is worked around in the app where possible; see [`CHARTER.md`](CHARTER.md) for when a gap moves on to a local patch or an upstream pull request. Experiments that touch a gap are logged in [`frontier-log.md`](frontier-log.md) and cite it by number.

`main` pins Keyhive at `90fe4a51` (`keyhive_core`, `keyhive_crypto`); `frontier` moved to `35460ba` during F-1. File and line references below are at `90fe4a51` unless marked otherwise.

## Fields

Each gap records:

- **Gap:** what's missing or doesn't work, in one or two sentences.
- **Evidence:** file and line where it shows up, at the pinned version.
- **Latest Keyhive:** whether it still holds on Keyhive's current `main`, and when that was checked.
- **Workaround in the app:** what the app does instead today.
- **Possible fix:** the level (0, 1 or 2, as in the charter) and where the change would go.
- **Next step:** keep local · ask in `#keyhive` · open an issue · open a pull request.
- **Experiments:** the `F-n` entries that touched it.

---

## G-1 — Adding a member to a reloaded document fails

**Gap:** A document rebuilt from stored events has no group-encryption state, so adding a member at Read level or above fails with `CgkaError::NotInitialized`. The routes that would avoid this, reaching the document's group directly or `add_member_with_manual_content`, are crate-private.

**Evidence (at `90fe4a51`):**
- `keyhive_core/src/keyhive.rs` L524–529: `Keyhive::add_member` passes a document add to `Document::add_member`.
- `keyhive_core/src/principal/document.rs` L239–281: `Document::add_member` calls `add_cgka_members_from_prekeys` for readers and above.
- `document.rs` L122–125: `cgka_mut` returns `NotInitialized` when the state is missing.
- `document.rs` L88–93: `from_group` builds the document with `cgka: None`.
- `document.rs` L71: `Document.group` is `pub(crate)`.
- `keyhive_core/src/principal/group.rs` L482: `add_member_with_manual_content` is `pub(crate)`.

**Latest Keyhive:** Partly checked on 2026-09-28 against `main` at `35460ba`. `from_group` still sets `cgka: None`, and `Document.group` and `add_member_with_manual_content` are still crate-private. `keyhive.rs` and `document.rs` have changed a good deal since the pin (16 commits, several touching group encryption), so a full read is still needed before any patch.

**Workaround in the app:** The admin signs the delegation directly (`ceremony::issue_delegation_as_bytes`, `core/crates/lfs_core/src/ceremony.rs` L508) and the result is loaded through `ingest_unsorted_static_events`, the same path stored delegations already take. Added in Run 40.

**Possible fix:** A public way to add a delegation-only member to a reloaded document. Level 1 to test; upstream if Keyhive wants it.

**Next step:** Ask in `#keyhive` whether a public delegation-only add on a reloaded document is intended.

**Experiments:** F-1, related. With the document's group-encryption operations stored and replayed, a reloaded document does have that state (see G-3). Adding a member to such a document was not tested.

---

## G-2 — No way to build a signer from a seed

**Gap:** `MemorySigner` has no `from_seed`. To rebuild the device signer from its stored 32-byte seed, the app goes through `ed25519_dalek::SigningKey` and then converts. This is the one place the app names the signature curve directly.

**Evidence:**
- `keyhive_crypto/src/signer/memory.rs` L125 (at `90fe4a51`): the only route is `impl From<ed25519_dalek::SigningKey> for MemorySigner`.
- App side: `signer_from_seed`, `core/crates/lfs_core/src/groups.rs` L148.

**Latest Keyhive:** Checked on 2026-09-28 against `main` at `35460ba`. Still no `from_seed`; the same `From<SigningKey>` conversion is at the same line.

**Workaround in the app:** `signer_from_seed` builds the key with `ed25519-dalek` (already in the build through `keyhive_crypto`) and converts it.

**Possible fix:** A `MemorySigner::from_seed(&[u8; 32])` constructor in `keyhive_crypto`. With it, the app would no longer need to name the curve.

**Next step:** Ask in `#keyhive`.

**Experiments:** none yet.

---

## G-3 — Group-encryption state isn't stored for reloaded documents

**Gap:** This is the app-side half of G-1. The app stores delegations and key events, not group-encryption state, so a document rebuilt from storage can't take part in content encryption. The current app doesn't need it. Phase 2 would, if reloaded documents need to encrypt content.

**Evidence:** Run 40 entry in `docs/phase0-observation-log.md`; the app's storage format tags in `core/crates/lfs_core/src/storage.rs` L27 and L36.

**Latest Keyhive:** Not applicable; this is an app design question. `cgka_members` and `merge_cgka_op` were already public at the pin; at `35460ba` both changed shape (`cgka_members` takes `&mut self`, `merge_cgka_op` takes an owner ID). The group-encryption operation type itself changed too, so a row storing those operations is tied to the Keyhive version that wrote it.

**Workaround in the app:** None yet. F-1 is trying the possible fix below on the `frontier` branch.

**Possible fix:** Store group-encryption operations as a new row type under a new format tag. Level 0, in the app. It would change the persistence-absence test (charter rule 3) and has consequences for sync.

**Next step:** Keep local. F-1 is testing the fix on `frontier`. Whether the app adopts it depends on Phase 2 deciding whether it needs content encryption on reloaded documents.

**Experiments:** F-1. At `35460ba`, a row holding a document's public events, group-encryption operations included (`keyhive.doc-events.bincode.v1`), rebuilds the document's encryption state on reload. It works only if the device's secrets are imported before replay and the prekey events of members the device added are stored too (see G-5). Across two restarts the device and a peer read old and new content, with no seed or secret on disk. The secrets have to be saved again after every change that rotates a key, including an ordinary encryption.

---

## G-4 — Stored bytes depend on Keyhive's internal format

**Gap:** Stored Keyhive data is `bincode` encoding of `keyhive_core`'s own types at the pinned version. Any upstream change to those types means migrating stored data.

**Evidence:** Format tags `keyhive.static-delegations.bincode.v1` and `keyhive.static-events.bincode.v1` (`core/crates/lfs_core/src/storage.rs` L27, L36); the consumer evidence notes in `docs/` measure what's stored and what a change would cost.

**Latest Keyhive:** Checked on 2026-09-29 against `35460ba`. The types behind both stored formats are unchanged. Rows written at `90fe4a51` decode and re-encode byte for byte at `35460ba`, and reload the same members at the same access levels (F-1 step 1). The group-encryption operation type did change, which matters for any row that stores those operations (see G-3).

**Workaround in the app:** Every row carries a format tag, so a future format can sit beside the old one and be migrated.

**Possible fix:** None needed now. Watch upstream, and compare the stored types whenever moving the pin is proposed.

**Next step:** Keep local; the evidence notes are already public.

**Experiments:** F-1.

---

## G-5 — A device's own event export leaves out the members it added

**Gap:** For a document the device made and shared, `static_events_for_agent` for the device returns the document's delegations, the device's own prekey events and the group-encryption operations, but not the prekey event of the member it added. A device rebuilt from that set alone refuses its own delegation of that member, and the encryption operations that depend on it stay pending, so no content can be read.

**Evidence (at `35460ba`):**
- F-1 runs 3 and 4 (`evidence/F-1/`): every stored prekey event was the device's own. The device's delegation of the peer was refused with `UnknownAgent` naming the peer, and the peer's add waited with `PendingCgkaAuthorization`.
- `keyhive_core/src/keyhive.rs` L1246: `reachable_prekey_ops_for_agent` reads as though it should include the prekey events of a document's members (L1310–1334). So the cause isn't known yet. It may be a bug, or a result of how the app uses the call.

**Latest Keyhive:** Found on 2026-09-29 at `35460ba`, Keyhive's `main` as of that date.

**Workaround in the app:** Store the prekey event from each added member's contact card beside Keyhive's export, as the app already does for groups. With that, the document replays completely (F-1 run 5). The member's event is a `PrekeyRotated`, as contact cards produced at the old pin too.

**Possible fix:** First find why the member's prekey event is left out. Level 0 while the cause is unknown; level 1 or upstream if it turns out to be in Keyhive.

**Next step:** Keep local until the cause is narrowed down, then ask in `#keyhive` or open an issue.

**Experiments:** F-1.
