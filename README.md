# local-first-social-native

A small native iOS and Android app, with a shared Rust core, for a local-first social network. Working name; the project will be renamed before any public release.

This is the native-device sibling of [local-first-social-network](https://github.com/jediwright/local-first-social-network) (the web app). The two share the social design and data model and make different choices for the device context: device-held keys, a native shell per platform, SQLite on disk. Neither is downstream of the other.

## What runs today

- **Rust core** (`core/crates/lfs_core`): three Automerge document types — profile, pings, threads — behind a typed UniFFI surface; SQLite persistence behind a single storage adapter that tags every row with its format. Expired pings are removed at the core on load when the caller supplies a clock (`open_typed_doc_at`); an unparseable expiry is retained, an unparseable clock is an error, and the clock-free `open_typed_doc` never reads the wall.
- **Consumer policy** (`core/crates/lfs_core/src/policy.rs`): the two-delegation floor and the "at least Admin to grant" bar, named as this app's own policy — not a call into `keyhive_core` enforcement. Both now have callers: the floor gates the identity ceremony; the bar gates every group grant (the device's capability on the group is mapped to a `GrantLevel` — Relay, Read, Edit, Admin — and checked *before* `add_member` is called; a granter below the bar is refused as a policy error). A `RootingLevel` (Admin or Edit) is supplied by the shells as configuration, never fixed as a constant; the ceremony echoes the configured level and its floor status.
- **Identity ceremony** (`core/crates/lfs_core/src/ceremony.rs`): `Core::run_identity_ceremony` creates the identity document through `keyhive_core`'s own ceremony — two Admin delegations (a primary cold admin key and a recovery key) from one root that is destroyed at creation, then an Edit delegation from the primary admin to a device key generated in the ceremony. The Admin count passes through the delegation floor; a one-delegation ceremony is refused and writes nothing. The cold seeds and the device seed are returned once across the FFI boundary and are never persisted. Two kinds of Keyhive bytes land in SQLite: the three delegations under `keyhive.static-delegations.bincode.v1`, and the three prekey operations (primary, recovery, device) under `keyhive.static-events.bincode.v1`. A reload path rebuilds the device's hive from those two rows given the device signer; a device signer surviving a process restart is not yet built (see below). No shell surfaces the ceremony yet.
- **Groups** (`core/crates/lfs_core/src/groups.rs`): the membership counter is backed by a real Keyhive group on the device's hive. `record_membership_event` ensures the group (device as Admin) and adds a simulated peer at Read through the grant bar; `membership_version` is the group's member count less the device's own root membership, so 0 means no group. New FFI: `grant_member(group_id, level)` and `device_grant_level(group_id)`. Before the ceremony has run, a session-only hive with an ephemeral device signer serves the counter; it is dropped when the ceremony installs the reloaded hive. Groups live in memory — they do not survive a relaunch — and every grant so far is on a device-created group. Neither shell yet exposes a control for the counter or for grants.
- **Shells** (`apps/ios`, `apps/android`): SwiftUI and Jetpack Compose, each opening the typed documents through the core and rendering them. Seed → kill → cold relaunch restores content from SQLite on both platforms. Pings are opened through the clocked entry point (`open_typed_doc_at` with the shell's clock), so expired pings are removed on load; profile and threads stay on the clock-free `open_typed_doc`.
- **Targets:** `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`, `x86_64-linux-android`.
- **Dependencies of note** (see `core/crates/lfs_core/Cargo.toml`): `keyhive_core` pinned to git rev `90fe4a51`, no optional features enabled — used for the identity ceremony and for group membership on the device's hive — with its companion crates `keyhive_crypto` (same rev), `future_form`, `rand`, `nonempty`, and `bincode` as direct dependencies at the versions the pin already locked (the persisted bytes are bincode over `keyhive_core`'s serde types at that rev, which is exactly the encoding surface the evidence note describes); `samod` / `autosurgeon` 0.14 (Automerge 0.12); `rusqlite` 0.40 bundled; `uniffi` 0.32; `atrium-*` for atproto identity resolution; `reqwest` on rustls with webpki roots — a recorded choice (the OS trust store is not used), provisional until a physical Android device has run. Subduction is declared behind a feature flag and off by default.
- **Not yet:** sync between devices, group persistence across relaunch, revocation, a device signer restored from its seed after a process restart (that needs a seed→signer path the pinned crates do not expose; ruled at the next run), or recovery from the cold key. The next run is grant/revoke on device, followed by the recovery path. All of it is scoped against the current `keyhive_core` encoding, and the evidence note below records what it would cost to migrate if that encoding changes.

## Building

See [BUILDING.md](BUILDING.md). One `cargo make` task per platform; host tests with `cargo make test`. Bindings regenerate with `cargo make android-so`, `bindgen-kotlin`, and `swift-xcframework` from the repo root; generated files are not committed.

## Docs

- [`docs/consumer-evidence-note-v0_2026-09-22.md`](docs/consumer-evidence-note-v0_2026-09-22.md) — what this app looks like as a consumer of the current `keyhive_core` encoding, how its storage layer is set up to absorb a format change, and what the two migration paths would cost. Written for the Keyhive / Onomancy maintainers.
- [`docs/phase0-observation-log.md`](docs/phase0-observation-log.md) — append-only log of every build run: toolchain, pins, outcome, defects. The record of how the repo got here.
- `docs/` also holds the level-of-effort packet and a few upstream read notes.

## Contributing

Pull requests follow [`.github/PULL_REQUEST_TEMPLATE.md`](.github/PULL_REQUEST_TEMPLATE.md) and [`.github/CONTRIBUTING.md`](.github/CONTRIBUTING.md). Every build that changes core, bindings, or a shell gets an observation-log entry. Design specs and build plans live off-repo; the log and the evidence note are the public record.

## License

Apache-2.0 (see [LICENSE](LICENSE)).
