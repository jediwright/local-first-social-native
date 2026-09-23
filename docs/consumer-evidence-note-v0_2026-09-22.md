# Consumer Evidence Note v0.1 — a native app on `keyhive_core`, and what a signed-event encoding change would cost it

**Date:** 2026-09-23 (v0 issued 2026-09-22) · **Version:** v0.1 (interim; v1 follows once capability grants are exercised in this app)
**Author:** J. Wright / UX Minds, LLC
**Repository:** `jediwright/local-first-social-native`, `main` at `568bd17` (code at `33fb3d2`; v0 was read at `a641eea`)

**Confidence marks used below:** ✓ = read directly from source or from a recorded run; ~ = estimated from the code as it stands, not yet exercised.

**What this note is.** A small native iOS + Android app builds and runs today on `keyhive_core` at git revision `90fe4a51`. Keyhive's authors have said its event encoding is pre-alpha and may change. This note records, from the code and from recorded runs, what a consumer of the current encoding looks like, how its storage layer is set up to absorb a format change, and what the two plausible migration paths would cost it. It ends with one question. It makes no claim about what Keyhive's or Onomancy's maintainers intend; every statement about upstream code is scoped to a named file at a named revision.

**Change log, in plain language.**

- **v0.1 — 2026-09-23.** Since v0 the app has run its first identity ceremony (build Run 37, code commit `33fb3d2`). It now writes real Keyhive bytes to its database: the signed proof of two admin delegations, stored under the app's own tag `keyhive.static-delegations.bincode.v1`. So the part of section (b) that said "this is where Keyhive bytes *would* land" is now something the app has done, not a plan, and section (a) gains a short account of what the ceremony produced. Section (c), the migration-cost estimate, is still an estimate: the app has not yet issued, revoked, or recovered a capability grant, which is what the header says v1 waits on. The question in (d) is unchanged, except that the app is now itself a consumer with signed material on disk. Nothing from v0 was removed; the v0 readings stay pinned to the revision they were taken at.
- **v0 — 2026-09-22.** First issue. Read at `a641eea` / `e0c2590`, observation log through Run 33.

---

## (a) What runs today

All ✓ — from the repository at `a641eea` and the observation log through Run 33. These are the v0 readings; line numbers refer to `a641eea`. What Run 37 added is in (a′) below.

- **Dependency.** `keyhive_core` is pinned to `git+https://github.com/inkandswitch/keyhive?rev=90fe4a51` with no optional features enabled (`core/crates/lfs_core/Cargo.toml` L25; the crate's `default = []` at `90fe4a51`). `cargo tree -p keyhive_core --depth 0` on the canonical checkout reports `keyhive_core v0.5.0 (https://github.com/inkandswitch/keyhive?rev=90fe4a51#90fe4a51)` (Run 33 apply transcript §3). Reading `keyhive_core/Cargo.toml` at `90fe4a51`, L4: `version = "0.5.0"`.
- **The crate links, and nothing is constructed.** `core/crates/lfs_core/src/lib.rs` L254–256 (`keyhive_core_linked`) takes `type_name::<keyhive_core::access::Access>()` to force the link; the test `keyhive_pin_links` (L597) asserts it. No Keyhive group, document, delegation or event is created anywhere in the crate at this revision. The pin is proven; the API is not yet exercised.
- **Four targets.** The Rust core cross-compiles for `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`, `x86_64-linux-android` via `cargo make android-so` and `cargo make swift-xcframework` (transcript §4: all four `Finished release`).
- **Typed FFI.** UniFFI 0.32.1, library mode. The three document shapes in `docs.rs` (`ProfileDoc`, `PingsDoc`, `ThreadsDoc`, L37–128) derive `uniffi::Record` and cross the boundary from one definition; `lib.rs` exports `init_core` (L74–78), `open_typed_doc` (L140), and `get_`/`put_` for each shape (L158–188). Regenerated bindings vs the prior run: +869 Swift, +814 Kotlin, 0 removals (transcript §4).
- **SQLite persistence.** `storage.rs`: one `docs` table, `id TEXT PRIMARY KEY, format TEXT NOT NULL, bytes BLOB NOT NULL` (L33–37); rusqlite 0.40.2 bundled.
- **Kill / relaunch, both shells** (transcript §5 leg (c); log Run 33). First launch: `docs ok: profile=(empty) trust=0 pings=0/0ch threads=0/0msg`. After seeding, force-stop (Android `am force-stop`; iOS app-switcher swipe) and cold relaunch: `docs ok: profile=@jedi trust=2 pings=3/1ch threads=1/2msg`, content restored from SQLite, no error at the FFI boundary. iOS on a physical device (iOS 26.6.1); Android on a Pixel 9 emulator (API 37.2). Test suite on the canonical checkout: 16 passed / 0 failed / 2 ignored (network).
- **Pointers.** Commits `14cc683` → `a641eea` (code) → `e0c2590` (log). Observation log Runs 25–33; Run 33 apply-leg transcript, 2026-09-22.

### (a′) What Run 37 added — read at `33fb3d2`

All ✓ — from the Run 37 build record and apply-leg close note (2026-09-23) and the observation log through Run 37. File and function names are given; line numbers are not, since (a)'s numbers are pinned to `a641eea` and the two revisions should not be mixed.

- **The API is now exercised.** A new module `ceremony.rs` implements `Core::run_identity_ceremony(IdentityConfig) -> IdentityCeremonyRecord`. It creates the identity document through `keyhive_core`'s own ceremony and issues exactly two Admin delegations — a primary cold admin key and a recovery key — from one ephemeral root that is destroyed at creation. The count passes through the app's own policy floor (`policy::check_delegation_floor`, its first caller); a one-delegation ceremony returns `CoreError::Policy` and writes nothing. The device is not yet a member of the identity document; the identity document's content head is a placeholder (zero digest).
- **The first Keyhive bytes in SQLite.** The delegation proof — `Vec<Signed<StaticDelegation<[u8; 32]>>>`, sorted by delegate, each `try_verify()`'d before write — is serialised with `bincode` over `keyhive_core`'s serde types at `90fe4a51` and written through the same adapter as every other row, under `storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1 = "keyhive.static-delegations.bincode.v1"`. The `.v1` names the serde shape at that revision; rows are never re-tagged. Keyhive's `Archive` form was considered and rejected for this row because it carries prekey secrets. A `decode()` that proves the bytes round-trip is present but `#[cfg(test)]`-gated; live reload from the persisted delegations is a later run.
- **Secrets stay off disk.** The cold seeds cross the FFI boundary once, as a `ColdKeyExport`, and a test asserts they are absent from the SQLite file.
- **Dependencies.** The pin is unchanged (`keyhive_core` at `90fe4a51`, no optional features; the app's pin label reads `keyhive_core=0.5.0+90fe4a51`). Five companion crates became direct dependencies at the versions the pin already locked: `keyhive_crypto` (same rev), `future_form 0.3.1`, `rand 0.8`, `nonempty 0.10`, `bincode 1.3`. `core/Cargo.lock` grew by five packages (571 total).
- **Suite and bindings.** 35 passed / 0 failed / 2 ignored, zero warnings, on the canonical checkout and in the reproduction container. Regenerated bindings vs Run 36: +314 Swift, +292 Kotlin, +11 header lines (one new FFI function plus a checksum), 0 removals. Both shells build and launch against the Run 37 core on a physical iPhone and a Pixel 9 emulator. No shell surfaces the ceremony yet.
- **Pointers.** Commits `8950ed9` → `33fb3d2` (code, eight files) → `8e6b530` (log: Run 37) → `568bd17` (README, doc-only). Observation log Runs 25–37.

## (b) The storage adapter, and where re-encoded bytes would land

✓ where a file and function are named; ~ where the sentence describes what the grant stage would build. Line numbers in this section are pinned to `a641eea`.

**One adapter owns every persisted byte.** `Core` holds `store: Box<dyn DocStore>` (`lib.rs` L57). The trait is two functions — `read(id) -> Option<Vec<u8>>` and `write(id, bytes)` (`storage.rs` L18–21) — and `SqliteStore` is the only implementation (L23–69). Every read of stored bytes goes through `store.read` (`open_doc` L84, `open_typed_doc` L141); every write goes through `store.write` (`save` L113). ✓

**Every row carries an app-owned format tag.** `SqliteStore::write` (L58–65) upserts `(id, format, bytes)` with `format = FORMAT_AUTOMERGE_SAVE_V1` (`"automerge.save.v1"`, L16). The tag is stamped inside the adapter, not passed by callers. Today the app persists exactly one format: Automerge `save()` bytes. ✓

**The schema already migrated once, additively.** `SqliteStore::open` (L27–50) checks `pragma_table_info('docs')` for the `format` column and, if absent, runs `ALTER TABLE docs ADD COLUMN format TEXT NOT NULL DEFAULT 'automerge.save.v1'`. Bytes are untouched; the test `migrates_phase0_stub_schema` (L95–114) proves an older database opens and reads back its original bytes. ✓ The rule recorded in the source (L11–15): a tag names what the bytes *are*; existing rows are never rewritten to a new tag.

**Where Keyhive bytes would land (~ at `a641eea`; exercised at `33fb3d2` — next paragraph).** When the app starts persisting Keyhive material — signed delegations and the events that back a group's membership — those rows go through the same `write`, under their own tag values (e.g. one for the current encoding, another for whatever succeeds it). A signed-event encoding change would then appear in this app as a *new tag value on new rows*, read back through the same `read`, with the tag telling the core which decoder to hand the bytes to. No caller changes; the decoder dispatch lives beside the tag in the adapter.

**Where they did land (✓ as of `33fb3d2`).** Run 37 did what the paragraph above described, for one row type. The static-delegation proof is written through the same `write`, under a second tag value, `keyhive.static-delegations.bincode.v1`, stamped inside the adapter; Automerge rows keep `automerge.save.v1`. The existing tests held (`migrates_phase0_stub_schema`, `roundtrip_under_trait_with_format_tag`) and a new one, `keyhive_and_automerge_rows_coexist_under_their_own_tags`, proves the two formats sit in the same table without touching each other. The encoding under the tag is `bincode` over `keyhive_core`'s serde types at `90fe4a51` — not an `onomancy_keyhive` carriage — so the tag's `.v1` is this app's name for that serde shape. A signed-event encoding change would appear here exactly as predicted: new rows, new tag, old rows untouched, with the decoder dispatch keyed on the tag. Group-membership events are not yet persisted (that is the next run); when they are, they take their own tag unless the serde shape is the same type.

**How the current encoding is framed upstream.** Reading `onomancy_keyhive/src/carriage.rs` at onomancy `main` `e29ca5df` (2026-09-03): the envelope tag is `pub const ENVELOPE_TAG: [u8; 3] = *b"kh0"` (L26). `Carriage::parse` (L45–65) strips the tag from each entry and returns `ParseCarriageError::UnknownEnvelope` if the prefix does not match (L51–56), then `bincode::deserialize`s the payload. The file's own doc comment (L20–25) reads, in substance: the tag is the envelope version for the Keyhive 0.5 bincode encoding; when the encoding changes the tag bumps (`kh1`, …) and old entries fail loudly rather than misparse; carriages ride an unsigned attached region, so re-encoding a chain re-attaches evidence without touching any signature. The workspace `Cargo.toml` at the same revision pins `keyhive_core = { version = "0.5", default-features = false }` (L59). ✓ as a reading of the file; nothing here is a statement about what the maintainers plan.

## (c) Migration cost — decoder retained vs decoder absent

**Every line in this section is ~.** It was estimated from (b) and from the code at `a641eea`, in which no capability grant had been issued or stored. At `33fb3d2` one row type now exists (the static-delegation proof, (a′)), so the "stored rows" column has a concrete instance to be measured against — but no grant has been issued to a second party, revoked, or recovered, and those are the operations the estimates are about. The table stands as written. What would firm it up is stated at the end.

Two things are at stake for a device that established its identity under the current encoding: its **stored data** (rows in its own SQLite) and its **standing with other devices** (a signed successor statement that a verifier on the new encoding must be able to read).

| | Decoder for the old encoding retained in the new core / consumer library | Decoder absent — consumers re-sign under the new encoding |
|---|---|---|
| **Stored rows on the device** | Old rows keep their old tag; new rows get the new tag; both read back through the same adapter. Cost: one new tag value, one decoder-dispatch branch. No data rewrite. | Old rows are unreadable by the new core. Cost: a one-time re-encode pass on the device (read old tag → decode with a *vendored* old decoder the app must ship itself → re-sign → write new tag), or accept data loss for anything not re-signable. |
| **Signed material from the device's own keys** | Unchanged; still verifies under the old tag. | Every delegation the device issued must be re-issued under the new encoding by a key that still exists. Anything signed by a key since rotated or lost is not re-issuable. |
| **Standing with other devices** | A successor statement signed under the old encoding verifies on a new-encoding peer. The device keeps its history with every contact. | A new-encoding peer cannot read the old statement. The device presents as *new* to every contact at the same moment — a network-wide reset of standing, not a per-device inconvenience. |
| **Cross-version window** | Peers on different app versions fail the handshake on the *tag*, legibly ("update to reconnect"), and reconnect after both upgrade. | Same, plus: after upgrade, nothing carries over unless the re-sign pass above ran on every device before the old decoder disappeared from the app. |
| **What the app has to ship** | Nothing beyond taking the upgrade. | Its own copy of the old decoder, pinned forever, to run the migration pass — the consumer becomes the maintainer of the format the library dropped. |

The asymmetry is the point: with a retained decoder the migration is ordinary and mostly invisible; without one the *cheapest* correct path is for every consumer to vendor the old decoder itself, which is the same code, maintained N times instead of once.

**What would make this ✓.** The grant stage of this app's build plan, not yet started: two documents under real capability grants, grant and revoke observed, and a recovery path exercised. At that point the "stored rows" and "signed material" columns become measurements from real rows under real tags, and v1 of this note replaces the estimates.

## (d) The question

For a `kh0` consumer with signed material on devices — which this app now is, in the narrow sense that signed delegations under the `90fe4a51` serde shape are on disk — the two paths above are not two flavours of the same migration; one is routine and the other resets every device's standing with every contact at once. So the one thing this app needs to plan for is which of them applies. Reading `carriage.rs` at `e29ca5df`, an unknown envelope tag fails loudly (L51–56, L107) — which says what happens to old entries at a *verifier without* the old decoder, but not whether such a verifier is the intended post-change state or a transitional one. The question for Onomancy, then, is narrow: when `onomancy_keyhive` picks up a `keyhive_core` encoding change and `ENVELOPE_TAG` moves off `kh0`, is the direction "keep a `kh0` decoder so already-signed events still verify," "consumers re-sign," or "not decided yet"? Any of the three is a usable answer; the first makes the left column real, the second makes the right column the plan, and the third tells this app to ship its own `kh0` decoder now, while the code that produces it is still the current one.

---

*Consumer Evidence Note v0.1 · 2026-09-23 (v0 2026-09-22) · J. Wright / UX Minds, LLC*
*AI-collaborative synthesis; human authorial responsibility; intellectual direction held by the named author.*
*Upstream statements scoped to file + revision; nothing here asserts maintainer intent.*
