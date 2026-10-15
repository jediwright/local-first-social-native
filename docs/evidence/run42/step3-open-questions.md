# Step 3: open questions checked before the device builds

1. Host test log shows no `Compiling rustls v0.23.46`.
   Confirmed from file metadata: core/target is shared between branches, and the host debug folder already held a rustls 0.23.46 build (rustls-ffc54bcf1047227a, 2026-10-15T03:40Z), made by the frontier branch three minutes before its own fix commit. Cargo reused it; only atrium-repo, atrium-crypto, atrium-oauth and lfs_core recompiled.

2. Device builds will compile the new rustls.
   Confirmed from file metadata: the four device target folders (aarch64-linux-android, x86_64-linux-android, aarch64-apple-ios, aarch64-apple-ios-sim) hold rustls 0.23.45 only, so the release builds must compile 0.23.46.

3. paste 1.0.15 (unmaintained) in the default build.
   Lockfile: all 10 paths from lfs_core run through subduction_iroh, which is optional and off by default. Inferred not in the default build; `cargo tree` on the default features settles it.

4. lru 0.16.4 (unsound, RUSTSEC-2026-0253).
   Reached through atrium-common, which requires lru 0.16; the advisory's fix is lru >= 0.18.2, a semver-incompatible line for a 0.x crate, so no `cargo update` can move it. The lockfile already carries lru 0.18.4 for another dependent. The fault needs a cached key whose Drop panics, with the panic caught and execution continuing. lfs_core's own source has no catch_unwind, but the UniFFI scaffolding does catch panics at the FFI boundary, so the catch condition can be met. The remaining condition is a key type with a panicking Drop: not confirmed here, as atrium-common's key types were not read in this change. Exposure treated as low, open, and upstream's to fix. Fix lies upstream in atrium-common.

5. yoke-derive 0.8.3 (yanked).
   0.8.4 was published 2026-09-30 and is not yanked; a precise update is possible in a later change. Not in this change's scope.

6. quinn-udp re-link on device targets.
   Inferred unaffected: the changed link is to windows-sys, compiled only for Windows. `cargo tree --target` on each device target settles it.
