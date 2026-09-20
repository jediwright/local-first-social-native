# BUILDING — Phase 0 (skeleton)

One command per platform, via `cargo make` (Phase 0 build plan §2 B6). Host commands below were exercised in the
Phase 0 build container on 2026-09-20; device commands are declared and complete on first device run.

## Toolchain
- Rust ≥ 1.90.0 (`keyhive_core` 0.5.0 floor). Container used Ubuntu `rustc-1.91` (rustup unreachable there).
- `cargo-make`, `cargo-ndk` (Android), Xcode + `aarch64-apple-ios{,-sim}` targets (iOS).
- Android NDK r26+; JNA at runtime.

## Host
```
cargo make test            # B1/B2/B3: round-trip, SQLite persistence, keyhive pin links, samod repo constructs
cargo make test-network    # B4 live resolve (egress required)
cargo make build-subduction # B5 compiles, feature-gated
```

## Android (first — L-14)
```
cargo make android-so
cargo make bindgen-kotlin
# open apps/android in Android Studio; the Compose shell links bindings/kotlin/generated + jniLibs
```

## iOS
```
cargo make swift-xcframework
# open apps/ios; the SwiftUI shell depends on the local Swift package in bindings/swift
```

Every build run is an observation-log entry (`docs/phase0-observation-log.md`), `clean_checkout: true` only from a
fresh clone + fresh toolchain.
