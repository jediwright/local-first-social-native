# Consumer Evidence Note v0 — a native app on `keyhive_core`, and what a signed-event encoding change would cost it

**Date:** 2026-09-22 · **Version:** v0 (interim; v1 follows once capability grants are exercised in this app)
**Author:** J. Wright / UX Minds, LLC
**Repository:** `jediwright/local-first-social-native`, `main` at `e0c2590` (code at `a641eea`)

**Confidence marks used below:** ✓ = read directly from source or from a recorded run; ~ = estimated from the code as it stands, not yet exercised.

**What this note is.** A small native iOS + Android app builds and runs today on `keyhive_core` at git revision `90fe4a51`. Keyhive's authors have said its event encoding is pre-alpha and may change. This note records, from the code and from recorded runs, what a consumer of the current encoding looks like, how its storage layer is set up to absorb a format change, and what the two plausible migration paths would cost it. It ends with one question. It makes no claim about what Keyhive's or Onomancy's maintainers intend; every statement about upstream code is scoped to a named file at a named revision.

---

## (a) What runs today

All ✓ — from the repository at `a641eea` and the observation log through Run 33.

- **Dependency.** `keyhive_core` is pinned to `git+https://github.com/inkandswitch/keyhive?rev=90fe4a51` with `default-features = false`. `cargo tree -p keyhive_core --depth 0` on the canonical checkout reports `keyhive_core v0.5.0 (https://github.com/inkandswitch/keyhive?rev=90fe4a51#90fe4a51)` (Run 33 apply transcript §3). Reading `keyhive_core/Cargo.toml` at `90fe4a51`, L4: `version = "0.5.0"`.
- **The crate links, and nothing is constructed.** `core/crates/lfs_core/src/lib.rs` L254–256 (`keyhive_core_linked`) takes `type_name::<keyhive_core::access::Access>()` to force the link; the test `keyhive_pin_links` (L597) asserts it. No Keyhive group, document, delegation or event is created anywhere in the crate at this revision. The pin is proven; the API is not yet exercised.
- **Four targets.** The Rust core cross-compiles for `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`, `x86_64-linux-android` via `cargo make android-so` and `cargo make swift-xcframework` (transcript §4: all four `Finished release`).
- **Typed FFI.** UniFFI 0.32.1, library mode. The three document shapes in `docs.rs` (`ProfileDoc`, `PingsDoc`, `ThreadsDoc`, L37–128) derive `uniffi::Record` and cross the boundary from one definition; `lib.rs` exports `init_core` (L74–78), `open_typed_doc` (L140), and `get_`/`put_` for each shape (L158–188). Regenerated bindings vs the prior run: +869 Swift, +814 Kotlin, 0 removals (transcript §4).
- **SQLite persistence.** `storage.rs`: one `docs` table, `id TEXT PRIMARY KEY, format TEXT NOT NULL, bytes BLOB NOT NULL` (L33–37); rusqlite 0.40.2 bundled.
- **Kill / relaunch, both shells** (transcript §5 leg (c); log Run 33). First launch: `docs ok: profile=(empty) trust=0 pings=0/0ch threads=0/0msg`. After seeding, force-stop (Android `am force-stop`; iOS app-switcher swipe) and cold relaunch: `docs ok: profile=@jedi trust=2 pings=3/1ch threads=1/2msg`, content restored from SQLite, no error at the FFI boundary. iOS on a physical device (iOS 26.6.1); Android on a Pixel 9 emulator (API 37.2). Test suite on the canonical checkout: 16 passed / 0 failed / 2 ignored (network).
- **Pointers.** Commits `14cc683` → `a641eea` (code) → `e0c2590` (log). Observation log Runs 25–33; Run 33 apply-leg transcript, 2026-09-22.

## (b) The storage adapter, and where re-encoded bytes would land

✓ where a file and function are named; ~ where the sentence describes what the grant stage would build.

**One adapter owns every persisted byte.** `Core` holds `store: Box<dyn DocStore>` (`lib.rs` L57). The trait is two functions — `read(id) -> Option<Vec<u8>>` and `write(id, bytes)` (`storage.rs` L18–21) — and `SqliteStore` is the only implementation (L23–69). Every read of stored bytes goes through `store.read` (`open_doc` L84, `open_typed_doc` L141); every write goes through `store.write` (`save` L113). ✓

**Every row carries an app-owned format tag.** `SqliteStore::write` (L58–65) upserts `(id, format, bytes)` with `format = FORMAT_AUTOMERGE_SAVE_V1` (`"automerge.save.v1"`, L16). The tag is stamped inside the adapter, not passed by callers. Today the app persists exactly one format: Automerge `save()` bytes. ✓

**The schema already migrated once, additively.** `SqliteStore::open` (L27–50) checks `pragma_table_info('docs')` for the `format` column and, if absent, runs `ALTER TABLE docs ADD COLUMN format TEXT NOT NULL DEFAULT 'automerge.save.v1'`. Bytes are untouched; the test `migrates_phase0_stub_schema` (L95–114) proves an older database opens and reads back its original bytes. ✓ The rule recorded in the source (L11–15): a tag names what the bytes *are*; existing rows are never rewritten to a new tag.

**Where Keyhive bytes would land (~, not yet written).** When the app starts persisting Keyhive material — signed delegations and the events that back a group's membership — those rows go through the same `write`, under their own tag values (e.g. one for the current encoding, another for whatever succeeds it). A signed-event encoding change would then appear in this app as a *new tag value on new rows*, read back through the same `read`, with the tag telling the core which decoder to hand the bytes to. No caller changes; the decoder dispatch lives beside the tag in the adapter.

**How the current encoding is framed upstream.** Reading `onomancy_keyhive/src/carriage.rs` at onomancy `main` `e29ca5df` (2026-09-03): the envelope tag is `pub const ENVELOPE_TAG: [u8; 3] = *b"kh0"` (L26). `Carriage::parse` (L45–65) strips the tag from each entry and returns `ParseCarriageError::UnknownEnvelope` if the prefix does not match (L51–56), then `bincode::deserialize`s the payload. The file's own doc comment (L20–25) reads, in substance: the tag is the envelope version for the Keyhive 0.5 bincode encoding; when the encoding changes the tag bumps (`kh1`, …) and old entries fail loudly rather than misparse; carriages ride an unsigned attached region, so re-encoding a chain re-attaches evidence without touching any signature. The workspace `Cargo.toml` at the same revision pins `keyhive_core = { version = "0.5", default-features = false }` (L59). ✓ as a reading of the file; nothing here is a statement about what the maintainers plan.

## (c) Migration cost — decoder retained vs decoder absent

**Every line in this section is ~.** It is estimated from (b) and from the code at `a641eea`, in which no capability grant has yet been issued or stored. What would firm it up is stated at the end.

Two things are at stake for a device that established its identity under the current encoding: its **stored data** (rows in its own SQLite) and its **standing with other devices** (a signed successor statement that a verifier on the new encoding must be able to read).

| | Decoder for the old encoding retained in the new core / consumer library | Decoder absent — consumers re-sign under the new encoding |
|---|---|---|
| **Stored rows on the device** | Old rows keep their old tag; new rows get the new tag; both read back through the same adapter. Cost: one new tag value, one decoder-dispatch branch. No data rewrite. | Old rows are unreadable by the new core. Cost: a one-time re-encode pass on the device (read old tag → decode with a *vendored* old decoder the app must ship itself → re-sign → write new tag), or accept data loss for anything not re-signable. |
| **Signed material from the device's own keys** | Unchanged; still verifies under the old tag. | Every delegation the device issued must be re-issued under the new encoding by a key that still exists. Anything signed by a key since rotated or lost is not re-issuable. |
| **Standing with other devices** | A successor statement signed under the old encoding verifies on a new-encoding peer. The device keeps its history with every contact. | A new-encoding peer cannot read the old statement. The device presents as *new* to every contact at the same moment — a network-wide reset of standing, not a per-device inconvenience. |
| **Cross-version window** | Peers on different app versions fail the handshake on the *tag*, legibly ("update to reconnect"), and reconnect after both upgrade. | Same, plus: after upgrade, nothing carries over unless the re-sign pass above ran on every device before the old decoder disappeared from the app. |
| **What the app has to ship** | Nothing beyond taking the upgrade. | Its own copy of the old decoder, pinned forever, to run the migration pass — the consumer becomes the maintainer of the format the library dropped. |

The asymmetry is the point: with a retained decoder the migration is ordinary and mostly invisible; without one the *cheapest* correct path is for every consumer to vendor the old decoder itself, which is the same code, maintained N times instead of once.

**What would make this ✓.** The grant stage of this app's build plan, which was paused on 2026-09-21 when upstream merged its September infrastructure update and has not resumed: two documents under real capability grants, grant and revoke observed, and a recovery path exercised. At that point the "stored rows" and "signed material" columns become measurements from real rows under real tags, and v1 of this note replaces the estimates.

## (d) The question

For a `kh0` consumer with signed material on devices, the two paths above are not two flavours of the same migration; one is routine and the other resets every device's standing with every contact at once. So the one thing this app needs to plan for is which of them applies. Reading `carriage.rs` at `e29ca5df`, an unknown envelope tag fails loudly (L51–56, L107) — which says what happens to old entries at a *verifier without* the old decoder, but not whether such a verifier is the intended post-change state or a transitional one. The question for Onomancy, then, is narrow: when `onomancy_keyhive` picks up a `keyhive_core` encoding change and `ENVELOPE_TAG` moves off `kh0`, is the direction "keep a `kh0` decoder so already-signed events still verify," "consumers re-sign," or "not decided yet"? Any of the three is a usable answer; the first makes the left column real, the second makes the right column the plan, and the third tells this app to ship its own `kh0` decoder now, while the code that produces it is still the current one.

---

*Consumer Evidence Note v0 · 2026-09-22 · J. Wright / UX Minds, LLC*
*AI-collaborative synthesis; human authorial responsibility; intellectual direction held by the named author.*
*Upstream statements scoped to file + revision; nothing here asserts maintainer intent.*
