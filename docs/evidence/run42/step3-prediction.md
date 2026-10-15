# Step 3 prediction (written before the run)

Tracked files: unchanged from step 2 plus the approved CONTRIBUTING.md (3 modified: .github/CONTRIBUTING.md, .github/workflows/guard.yml, core/Cargo.lock). Build output lands only in git-ignored paths (core/target, bindings/kotlin/{jniLibs,generated}, bindings/swift/{generated,include}, *.xcframework, apps/android/app/build). Any other tracked change is a mismatch.

- `cargo make test`: 50 passed, 0 failed, 2 ignored (same as step 0). Build log shows `Compiling rustls v0.23.46`.
- `cargo make test-network`: `resolve_known_did_to_pds` passes (1 passed, 0 failed), resolving against plc.directory.
- `cargo audit`: no vulnerabilities; the same 5 allowed warnings as step 0 (bincode 1.3.3, derivative 2.2.0, paste 1.0.15 unmaintained; lru 0.16.4 unsound; yoke-derive 0.8.3 yanked). rustls absent from the report.
- `cargo make android-so`, `bindgen-kotlin`, `swift-xcframework`: succeed; each release build compiles rustls 0.23.46.
- Android emulator (emulator-5554), app `social.localfirst.shell`: one Resolve tap shows `resolvePds ok in <n> ms: https://…`.
- iPhone 17 Pro, bundle `com.uxminds.lfs.LfsShell`: one Resolve tap shows `resolvePds ok in <n> ms: https://…`.
