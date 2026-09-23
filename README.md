# local-first-social-native

A small native iOS and Android app, with a shared Rust core, for a local-first social network. Working name; the project will be renamed before any public release.

This is the native-device sibling of [`local-first-social-network`](https://github.com/jediwright/local-first-social-network) (the web app). The two share the social design and data model and make different choices for the device context: device-held keys, a native shell per platform, SQLite on disk. Neither is downstream of the other.

## What runs today

- **Rust core** (`core/crates/lfs_core`): three Automerge document types — profile, pings, threads — behind a typed [UniFFI](https://mozilla.github.io/uniffi-rs/) surface; SQLite persistence behind a single storage adapter that tags every row with its format. Expired pings are removed at the core on load when the caller supplies a clock (`open_typed_doc_at`); an unparseable expiry is retained, an unparseable clock is an error, and the clock-free `open_typed_doc` never reads the wall.
- **Consumer policy** (`core/crates/lfs_core/src/policy.rs`): the two-delegation floor and the "at least Admin to grant" bar, named as this app's own policy — not a call into `keyhive_core` enforcement, and not yet called by anything. A `RootingLevel` (Admin or Edit) is supplied by the shells as configuration, never fixed as a constant; the ceremony that consumes it is the next run.
- **Shells** (`apps/ios`, `apps/android`): SwiftUI and Jetpack Compose, each opening the typed documents through the core and rendering them. Seed → kill → cold relaunch restores content from SQLite on both platforms. The shells still call `open_typed_doc` and hide expired pings at display time; adopting the clocked entry point is a pending two-line change per shell.
- **Targets**: `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-linux-android`, `x86_64-linux-android`.
- **Dependencies of note** (see `core/crates/lfs_core/Cargo.toml`): `keyhive_core` pinned to git rev `90fe4a51`, no optional features enabled — linked and pin-verified, not yet used to issue grants; `samod` / `autosurgeon` 0.14 (Automerge 0.12); `rusqlite` 0.40 bundled; `uniffi` 0.32; `atrium-*` for atproto identity resolution; `reqwest` on `rustls` with webpki roots — a recorded choice (the OS trust store is not used), provisional until a physical Android device has run. Subduction is declared behind a feature flag and off by default.

Not yet: sync between devices, capability grants, or any identity ceremony. The next run is the identity ceremony — the first Keyhive bytes in SQLite, under their own format tag — followed by groups behind the membership counter and grant/revoke on device. All of it is scoped against the current `keyhive_core` encoding, and the evidence note below records what it would cost to migrate if that encoding changes.

## Building

See [`BUILDING.md`](BUILDING.md). One `cargo make` task per platform; host tests with `cargo make test`. Bindings regenerate with `cargo make android-so`, `bindgen-kotlin`, and `swift-xcframework` from the repo root; generated files are not committed.

## Docs

- [`docs/consumer-evidence-note-v0_2026-09-22.md`](docs/consumer-evidence-note-v0_2026-09-22.md) — what this app looks like as a consumer of the current `keyhive_core` encoding, how its storage layer is set up to absorb a format change, and what the two migration paths would cost. Written for the Keyhive / Onomancy maintainers.
- [`docs/phase0-observation-log.md`](docs/phase0-observation-log.md) — append-only log of every build run: toolchain, pins, outcome, defects. The record of how the repo got here.
- `docs/` also holds the level-of-effort packet and a few upstream read notes.

## Contributing

Pull requests follow `.github/PULL_REQUEST_TEMPLATE.md` and `.github/CONTRIBUTING.md`. Every build that changes core, bindings, or a shell gets an observation-log entry. Design specs and build plans live off-repo; the log and the evidence note are the public record.

## License

Apache-2.0 (see `LICENSE`).

---

*J. Wright / UX Minds, LLC. AI-collaborative development; human authorial responsibility.*
