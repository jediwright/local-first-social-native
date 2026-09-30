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

**Next step:** Keep local. F-1 showed the fix works on `frontier` under stated conditions and is parked until G-5's cause is settled. Whether the app adopts it also depends on Phase 2 deciding whether it needs content encryption on reloaded documents.

**Experiments:** F-1. At `35460ba`, a row holding a document's public events, group-encryption operations included (`keyhive.doc-events.bincode.v1`), rebuilds the document's encryption state on reload, given two conditions: the prekey events of members the device added are stored too (see G-5), and the device's current secrets are imported. Once the members' events are stored, the order of import and replay doesn't matter. Only a document's first encryption and an explicit key update change the secrets, so they have to be saved again after those; secrets saved earlier still read everything encrypted before the key update. Across restarts the device and a peer read old and new content, in the core and on an Android emulator and an iPhone (F-1 step 7), with no seed or secret on disk.

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

## G-5 — A device's own event export leaves out members it knows only by contact card

**Gap:** Before exporting, Keyhive sorts each agent's prekey events with `KeyOp::topsort`, which starts only from prekey adds. A contact card carries a single prekey rotate, so a member the device knows only from its card has no add to start from, and its prekey event is silently left out of `static_events_for_agent`. A device rebuilt from its own export then refuses its delegation of that member, the encryption operations that depend on it stay pending, and no content can be read. Members introduced with a prekey add are exported and are unaffected.

**Evidence (at `35460ba`):**
- A reproduction against Keyhive alone: [`patches/G-5-repro.patch`](patches/G-5-repro.patch) adds one test file and changes no Keyhive code. Its output is in `evidence/G-5/run2.txt`.
  - `KeyOp::topsort` keeps a lone prekey add (1 in, 1 out) and drops a lone prekey rotate (1 in, 0 out).
  - With the member introduced by contact card, the device's export has no prekey event from the member. An instance rebuilt from that export, with the same signer and the device's secrets imported, fails with `UnknownAgent`, `PendingCgkaAuthorization` and `OutOfOrderOperation`, and reads fail with `KeyNotFound`. With the member introduced by a prekey add, the export carries the member's event and the rebuilt instance reads. Leaving the device's own prekey events out of the replay, as the app does, changes neither result.
  - The relay case from Keyhive issue #206 fails at the same version: the relay can't apply three of the sender's events and never learns of the member.
- `keyhive_core/src/principal/individual/op.rs` L37–68 (`KeyOp::topsort`), applied by `reachable_prekey_ops_for_agent` (`keyhive.rs` L1246, L1337–1339). `Keyhive::generate_contact_card` returns a rotate (`keyhive.rs` L345–354).
- In the app: F-1 runs 3 and 4 and step 7d show the same failure.

**Latest Keyhive:** Reproduced on 2026-09-29 at `35460ba`, Keyhive's `main` as of that date.

**Workaround in the app:** Store the prekey event from each added member's contact card beside Keyhive's export, as the app already does for groups. This supplies the one event the sort leaves out, and the document replays completely (F-1 run 5).

**Possible fix:** In Keyhive, `KeyOp::topsort` could also start from rotations whose earlier key isn't in the set. Level 1 to try locally; upstream only by a deliberate decision. The app keeps its workaround (level 0) until Keyhive changes.

**Next step:** Commented on Keyhive issue #206 on 2026-09-29 with the reproduction and the version ([comment](https://github.com/inkandswitch/keyhive/issues/206#issuecomment-5902241890)); #206's sender also learns the member from a contact card, so the cause is likely the same. Wait for a maintainer's reply there. Confirming the cause with a local change to `KeyOp::topsort` (level 1, scratch clone only) is optional.

**Experiments:** F-1 (core runs 3 and 4; step 7d reproduces it on an Android emulator and an iPhone).
