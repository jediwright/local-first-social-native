# Consumer Evidence Note v1 — a native app on `keyhive_core`, measured: what it stores, and what a signed-event encoding change would cost it

**Date:** 2026-09-26 · **Version:** v1 (supersedes the estimates in v0; v0 stays published unedited)
**Author:** J. Wright / UX Minds, LLC
**Repository:** `jediwright/local-first-social-native`; measurements taken against `main` at `99cdcb4` (code at `c5b8bec`).
**Previous version:** [`docs/consumer-evidence-note-v0_2026-09-22.md`](consumer-evidence-note-v0_2026-09-22.md)

**Confidence marks used below:** ✓ = read directly from source, from a recorded run, or measured from stored rows; ~ = estimated from the code as it stands, not yet exercised.

**What this note is.** v0 described a small native iOS + Android app on `keyhive_core` at git revision `90fe4a51` before it stored any Keyhive material, and estimated what a change to Keyhive's signed-event encoding would cost it. The app now creates identities and groups through `keyhive_core`, persists the signed results, and recovers from a cold key against them. v1 re-measures section (c) from those stored rows. Every estimate that could be replaced by a measurement has been; the rest stay marked ~ with the reason. It asks no new question. Every statement about upstream code is scoped to a named file at a named revision, and nothing here asserts what Keyhive's or Onomancy's maintainers intend.

---

## What changed since v0

**In the app** (✓, repository at `c5b8bec`):
- Keyhive material is now written to disk. An identity ceremony, groups with grants and revocations, and recovery from a cold key all run through `keyhive_core` at the same pin, `90fe4a51`.
- The app now writes two tag values in addition to the Automerge one: `keyhive.static-delegations.bincode.v1` and `keyhive.static-events.bincode.v1`.
- The recovery drill (enrol, grant, discard the device key, recover, grant again) has run on an Android emulator and on a physical iOS device.

**Upstream** (✓ as readings of the named sources, 2026-09-26):
- `keyhive_core` 0.6.0 was published to crates.io on 2026-09-25; 0.5.0 is not yanked.
- Between `90fe4a51` and keyhive `main` at `9a8c1d56`, the source files for every type this app persists are byte-identical:
  - in `keyhive_core`: `event/static_event.rs`, `principal/group/delegation.rs`, `principal/group/revocation.rs` and `principal/individual/op/add_key.rs`;
  - in `keyhive_crypto`: `signed.rs`.
  `principal/individual/op/rotate_key.rs` differs by a doc-comment typo only.
- Among the changes in that range, the payload of the CGKA `Add` operation changed (`beekem/src/operation.rs`). ✓ This app stores no CGKA operations: every one of the 9 key-operation entries in the measured store is a prekey rotation, and the other 11 entries are delegations (see (b) and (c)).
- PR #230 remains a draft.
- Onomancy `main` at `e29ca5df` still carries `ENVELOPE_TAG = *b"kh0"` (`onomancy_keyhive/src/carriage.rs` L26), and its workspace still pins `keyhive_core = "0.5"` (`Cargo.toml` L59).

**Not yet closed** (~): one item of this build stage stays provisional. The TLS choice — rustls with webpki roots, not the OS trust store — is recorded but has not yet run on a physical Android device.

---

## (a) What runs today

All ✓ — from the repository at `c5b8bec` and the observation log.

- **Dependency.** `keyhive_core` is pinned to `git+https://github.com/inkandswitch/keyhive?rev=90fe4a51` with no optional features enabled, unchanged since v0. `keyhive_crypto` is at the same revision. `bincode` and the other companion crates are direct dependencies at the versions the pin already locked.
- **The API is exercised, not just linked.** v0 recorded that nothing was constructed. The app now:
  - creates an identity document through `keyhive_core`'s own ceremony: two Admin delegations (a cold primary key and a cold recovery key) from a root destroyed at creation, then an Edit delegation to a device key;
  - creates groups and grants and revokes members on them;
  - recovers from either cold key onto a replacement device key in the same store.
- **Tests.** 50 passed / 0 failed / 2 ignored (network) on the canonical checkout at `c5b8bec`, 0 warnings.
- **Four targets, typed FFI, SQLite.** Same as v0:
  - `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`, `x86_64-linux-android`;
  - UniFFI 0.32, library mode;
  - one `docs` table of `(id, format, bytes)`.
- **On device.** The recovery drill ran on an Android emulator and on a physical iOS device. On both platforms:
  - the recovered device holds Admin on the group;
  - the state survives a kill and cold relaunch;
  - the recovered device can grant again.

## (b) The storage adapter, and what now lands in it

✓ where a file and function are named; ~ where the sentence describes code not yet written.

**One adapter still owns every persisted byte.** ✓ As in v0, every read and write goes through the `DocStore` trait's `read` and `write`, and every row carries an app-owned format tag. The rule stands that existing rows are never rewritten to a new tag.

**What Keyhive material looks like on disk.** ✓ Four row kinds under two tags:
- `identity` (the identity document's signed delegations) and `group:<id>` (one per group) under `keyhive.static-delegations.bincode.v1`;
- `identity-keyops` and `group:<id>.keyops` (signed key operations) under `keyhive.static-events.bincode.v1`.

Each row is a bincode-encoded list of signed entries over `keyhive_core`'s serde types at `90fe4a51`. On relaunch the app rebuilds its hive from these rows. Recovery rewrites the identity pair and each group pair in place under the same tags. Sizes are in (c).

**Where a second encoding would land.** ~ Unchanged from v0: as a new tag value on new rows, read through the same adapter, with the tag selecting the decoder. Not yet written — today each tag has one encoding.

**What the app does not persist.** ✓ A document rebuilt from persisted static events has no CGKA state: at `90fe4a51` it is materialised with `cgka: None`, and a reader-level `add_member` on it returns `CgkaError::NotInitialized` (`document.rs` L88–93, L122–125, L239–281). Two things follow.
- Recovery adds the replacement device to a reloaded identity document another way. The cold key signs a static delegation, and the hive ingests it through the same path the stored rows take.
- The app persists no CGKA operations. Its signed material is delegations and key operations only.

Both points are ✓ by line and by execution in the app's test suite, and the second is ✓ in the measured store: no stored entry is a CGKA operation.

## (c) Migration cost — measured where it can be, estimated where it can't

v0 estimated this section before any Keyhive bytes reached the app's store. They have now. This section rebuilds it from real rows under real tags. Every cell in the table at the end is marked: ✓ measured or read, ~ estimated.

**Where the numbers come from.** ✓ The primary source is the SQLite store of an Android emulator running the app at commit `c5b8bec` (`keyhive_core` at `90fe4a51`). It was pulled after the app's recovery drill:

1. Enrol an identity.
2. Create a group and grant one member Read and one Edit.
3. Kill and relaunch.
4. Discard the device key.
5. Recover from the cold recovery key.
6. Grant two more members Read from the recovered device.

✓ A replay of the same sequence on a scratch build of the same commit matched the device store on row ids, tags, byte sizes and entry counts. The bytes themselves differ, because every run generates fresh keys. The replay supplies the earlier states. ✓ The store holds only Keyhive rows: the drill seeded no Automerge documents.

### (c1) What is on disk

✓ Bytes / entries per row, by state:

| State | `identity` | `identity-keyops` | `group:<id>` | `group:<id>.keyops` | Rows | Bytes |
|---|---|---|---|---|---|---|
| Fresh enrol | 743 / 3 | 524 / 3 | — | — | 2 | 1,267 |
| One group, two grants (unchanged by kill/relaunch) | 743 / 3 | 524 / 3 | 732 / 4 | 352 / 2 | 4 | 2,351 |
| After recovery | 988 / 4 | 696 / 4 | 929 / 5 | 696 / 4 | 4 | 3,309 |
| After one post-recovery grant | 988 / 4 | 696 / 4 | 1,126 / 6 | 696 / 4 | 4 | 3,506 |
| After two post-recovery grants | 988 / 4 | 696 / 4 | 1,323 / 7 | 868 / 5 | 4 | 3,875 |

✓ Tags: the `identity` and `group:<id>` rows carry `keyhive.static-delegations.bincode.v1`; both `-keyops` rows carry `keyhive.static-events.bincode.v1`. Recovery rewrites existing rows in place: it adds no row and no tag.

✓ Per entry, every row accounts for its size to the byte:
- an 8-byte entry count;
- 245 bytes per identity-document delegation;
- 197 bytes per group delegation, or 165 bytes for a group root delegation that carries no proof;
- 172 bytes per key operation.

### (c2) Whether the current release reads these rows

✓ `keyhive_core` 0.6.0 was published to crates.io on 2026-09-25. A scratch build against it decoded every stored row, 8 of 8 (the four device rows and the four replay rows), and re-encoded each byte-identical. The scratch build resolved its dependencies fresh from crates.io; bincode came out at 1.3.3, the same version the app's lock pins.

~ Not exercised: signature verification and ingest under 0.6.0. This note makes no claim about either.

~ So for this release, the stored-rows question did not arise at the encoding level. What a future encoding change would do to these rows is still an estimate.

### (c3) Who can re-sign what

✓ The measured store holds 20 signed entries. Of these, 8 were issued by keys the user still holds: the two cold keys (5 entries) and the current device key (3). The other 12 were not:

| Issuer | Entries | Re-signable by this user |
|---|---|---|
| The two cold keys | 5 | yes ✓ |
| The current device key | 3 | yes ✓ |
| The lost device's key: its two grants and two key operations | 4 | no ✓ — the key is gone |
| The identity root: two Admin root delegations | 2 | no ✓ — destroyed at creation by design |
| The group's root key: two Admin root delegations | 2 | no ~ — assumed not retained |
| Other members' own key operations | 4 | no ~ — simulated members here; in real use, held by other people |

✓ Two of the lost device's four entries are the grants that admitted two members. ~ Without a retained decoder, those members' standing rests on entries no one can re-issue.

### (c4) The two paths, re-marked

| | Decoder for the old encoding retained | Decoder absent — consumers re-sign |
|---|---|---|
| **Stored rows on the device** | ~ Old rows keep their tag and new rows get a new one, read through the same adapter. ✓ Under 0.6.0, every stored row decodes as it stands. | ~ A one-time re-encode pass using a vendored old decoder; not written, so its cost is not measured. ✓ Its scope in the measured store: 4 rows, 20 signed entries, 3,875 bytes. |
| **Signed material** | ~ Unchanged; still verifies under the old tag (verification under a newer core not exercised). | ✓ 8 of 20 re-issuable by the user; of the 12 that are not, 6 ✓ and 6 ~ (per (c3)). |
| **Standing with other devices** | ~ Unchanged from v0: the app does not yet sync between devices. | ~ Unchanged from v0, same reason. |
| **Cross-version window** | ~ Unchanged from v0. | ~ Unchanged from v0. |
| **What the app has to ship** | ~ Nothing beyond taking the upgrade. | ~ Its own pinned copy of the old decoder. |

The asymmetry v0 described holds, and one part of it is now counted: on the re-sign path, 12 of this store's 20 entries are not the user's to re-sign — 6 measured as such, 6 resting on the two assumptions stated in (c3).

## (d) The question

Unchanged from v0. Section (d) of v0 asked the Onomancy maintainers one question about what happens to `kh0`-encoded material when `ENVELOPE_TAG` moves; it stands as written and is not re-asked here. ✓ What v1 adds is the measured store that question applies to in one app: four rows, twenty signed entries and 3,875 bytes after one recovery, of which eight entries are the user's to re-issue.

---

*Consumer Evidence Note v1 · 2026-09-26 · J. Wright / UX Minds, LLC*
*AI-collaborative synthesis; human authorial responsibility; intellectual direction held by the named author.*
*Upstream statements scoped to file + revision; nothing here asserts maintainer intent. Measurements from the app at `c5b8bec`; the scratch decode build is not committed.*
