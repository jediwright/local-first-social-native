# Phase 0 Observation Log — local-first-social-native (placeholder name, R11)

**Instrument:** native-track-phase0-build-plan-v0-1_2026-09-18.md §6 (shape) · PC#8 §H.3 discipline
**Append-only.** One block per run. ISO 8601 UTC ms. `null` = applicable-but-unobserved; `n/a` = structurally inapplicable.
**Register:** CONTEXTUAL · ⚑ SINGLE-CONTEXT — NOT PANELED
**Environment for Runs 0–9:** Phase 0 build container (Ubuntu 24.04, 1 vCPU, 3 GB RAM, no rustup — Ubuntu `rustc-1.91.1`/`cargo-1.91.1`; egress allowlist: crates.io, github.com, raw.githubusercontent.com, codeload). No git repo initialised in the container (operator ruling 2026-09-20: no repo creation from the build session) — `commit` is `null` until the operator's first commit; the scaffold tarball sha256 is in the session handoff.

---

```
run:                 0
run_type:            sentinel-check
platform:            n/a
started_at:          2026-09-20T15:07:36.079Z
ended_at:            2026-09-20T15:31:00.000Z
elapsed_min:         24
clean_checkout:      n/a
commit:              n/a
pins:                n/a
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=0.5.0@90fe4a51(main,2026-09-17) S-3=#230 open/DRAFT/unmerged(33 commits, head keyline_crate) S-4=kh0 S-5="0.5" | S-6=null(operator reads before Phase 2) S-7=SyncpointMap pub(crate) S-8=samod 0.14.0/autosurgeon 0.14.0 (2026-09-17) S-9=api 0.25.8/oauth 0.1.7/identity 0.1.9/repo 0.1.8/crypto 0.1.3 S-10=git-only, subduction_iroh 0.10.2, subduction_keyhive 0.8.2, workspace automerge 0.11.0 S-11=null S-12(a)=null(no 0.6/codec statement in repo issues, branch .md, or CHANGELOG[404]; #keyhive channel not read) S-12(b)=branch `2026-sept-updates` EXISTS (7 commits, 2026-09-08, infra/CI/nix/docs), NOT merged into main (lib.rs + Cargo.toml differ)
l14_note:            n/a
loe_note:            No trigger moved; no halt. S-12(b) source-confirmed → Phase 1 does not open before a T4 scoping ruling (plan §3 S-12 rule). main is 12 commits past 0.5.0 (d12511de) under the same version string; the 09-11/09-17 commits are core-semantic (#226 concurrent-merge vulnerabilities, #227 empty BeeKEM tree, concurrent-remove ordering, #232 revocation-lock deadlock, #233 dup membership bfs) → B11 builds both (Runs 2–3). GitHub API unauthenticated limit exhausted mid-run; HTML/raw/codeload fallbacks used — reads are ✓, `updated_at` for S-3 is ~ (HTML timestamps).
next:                Run 1 — toolchain provisioning
```

```
run:                 1
run_type:            note
platform:            host
started_at:          2026-09-20T15:11:00.000Z
ended_at:            2026-09-20T15:13:30.000Z
elapsed_min:         3
clean_checkout:      true
commit:              null
pins:                n/a
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            n/a
loe_note:            rustup/static.rust-lang.org unreachable (egress). Ubuntu archive carries rustc-1.91 — meets keyhive_core's rust-version = "1.90.0" floor (0.5.0 crates.io AND main). Toolchain class: not a defect; a provisioning constraint of this environment only. Operator machines use rustup.
next:                Run 2 — B11 crates.io build
```

```
run:                 2
run_type:            pin-decision
platform:            host
started_at:          2026-09-20T15:14:31.814Z
ended_at:            2026-09-20T15:15:34.000Z
elapsed_min:         1
clean_checkout:      true
commit:              null
pins:                keyhive_core==0.5.0(crates.io) keyhive_crypto=0.2.1 beekem=0.3.0
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=0.5.0
l14_note:            n/a
loe_note:            B11 (a): keyhive_core =0.5.0 from crates.io builds on a fresh machine with today's toolchain in 1m02s (91 packages locked; dev profile; 1 vCPU). Publishability at the pinned version: ✓ (plan L-4 "?" resolved to ✓ for 0.5.0 on 2026-09-20; Time-sensitive against this lockfile).
next:                Run 3 — B11 main@90fe4a51
```

```
run:                 3
run_type:            pin-decision
platform:            host
started_at:          2026-09-20T15:16:35.120Z
ended_at:            2026-09-20T15:17:38.601Z
elapsed_min:         1
clean_checkout:      true
commit:              null
pins:                keyhive_core=0.5.0(git rev 90fe4a51e38de44cfcef3ea7d504f6bcb2ccfe4b) keyhive_crypto=0.2.1 beekem=0.3.0
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-2=0.5.0@90fe4a51
l14_note:            n/a
loe_note:            B11 (b): main @ 90fe4a51 builds in 1m03s. Lockfile diff vs Run 2: dependency versions IDENTICAL (only the keyhive_core source line differs — main still resolves keyhive_crypto 0.2.1 and beekem 0.3.0 from crates.io). The L-4 gap is therefore semantic, not dependency-shaped: 12 commits of core correctness fixes under one version string. PIN DECISION (Phase 0): stay on crates.io =0.5.0 (plan B3 as written); record that a Phase 1 that needs #226/#232 fixes must pin the git rev, which is a Phase 1 pre-decision, not a Phase 0 change.
next:                Run 4 — samod/autosurgeon/subduction pin decision
```

```
run:                 4
run_type:            pin-decision
platform:            host
started_at:          2026-09-20T15:18:00.000Z
ended_at:            2026-09-20T15:19:30.000Z
elapsed_min:         2
clean_checkout:      n/a
commit:              null
pins:                samod=0.14.0 autosurgeon=0.14.0 automerge=0.12.0 subduction=21b2e6b8f757e9e5fc7a1e6d1306d946b834f645(main 2026-09-20) atrium-api=0.25.8 rusqlite=0.40.2 uniffi=0.32.1
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-8=0.14.0/0.14.0 S-10=21b2e6b8
l14_note:            n/a
loe_note:            B2 PIN DECISION: samod 0.14.0 + autosurgeon 0.14.0 (both → automerge 0.12.0, published 2026-09-16). samod 0.13.0 → automerge 0.11.0. Reason: 0.14 is current and the spec's "0.13" was descriptive (plan A2). Compatibility note (S-10): the Subduction workspace pins automerge 0.11.0, but `subduction_keyhive` and `subduction_iroh` do NOT depend on automerge (only the automerge_subduction_* crates do) — the pinned pair shares ONE keyhive_core 0.5.0 with lfs_core (lockfile: single keyhive_core entry) and pulls no second automerge. No duplication in Phase 0's declared set. Re-check if Phase 1 declares automerge_subduction_ingest.
next:                Run 5 — B1–B4 host build
```

```
run:                 5
run_type:            host-build
platform:            host
started_at:          2026-09-20T15:20:36.894Z
ended_at:            2026-09-20T15:25:06.541Z
elapsed_min:         4
clean_checkout:      true
commit:              null
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 automerge=0.12.0 subduction=off atrium-api=0.25.8 rusqlite=0.40.2 uniffi=0.32.1 tokio=1.53.1
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            n/a
loe_note:            lfs_core (B1 API, SQLite trait, B2 automerge+autosurgeon+samod[tokio], B3 pin, B4 atrium ×6 declared + resolve_pds) compiles green on first pass: 4m29s, 571 packages, 0 warnings, 1 vCPU dev profile. crate-type lib+staticlib+cdylib.
next:                Run 6 — host tests
```

```
run:                 6
run_type:            host-build
platform:            host
started_at:          2026-09-20T15:25:43.325Z
ended_at:            2026-09-20T15:26:07.008Z
elapsed_min:         1
clean_checkout:      false
commit:              null
pins:                (as Run 5)
outcome:             pass
defects:             1
defect_classes:      other
sentinel_state:      n/a
l14_note:            n/a
loe_note:            `cargo test -p lfs_core`: 4 passed (round_trip_in_memory; survives_save_load_through_sqlite_file; keyhive_pin_links; samod_repo_constructs_and_creates_doc), 1 ignored (B4 live). Defect: test code called samod `RepoBuilder::load_local()` on the tokio builder — bound is `LocalRuntimeHandle`; tokio path is `.load()`. Fixed in one line; class "other" (API misread, not a stack defect). B1 done-when met on host: a doc survives save/load, including through a SQLite file across Core instances.
next:                Run 7 — B4 host network probe
```

```
run:                 7
run_type:            network-probe
platform:            host
started_at:          2026-09-20T15:26:07.008Z
ended_at:            2026-09-20T15:26:07.597Z
elapsed_min:         1
clean_checkout:      false
commit:              null
pins:                atrium-identity=0.1.9 atrium-xrpc-client=0.5.15 reqwest=0.12.28
outcome:             partial
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            n/a
loe_note:            B4 host resolve of did:plc:z72i7hdynmk6r22z27h6tvur: the full call path (Did parse → CommonDidResolver → reqwest → tokio runtime block_on) executed to the network boundary and returned `http status: 403` — attributed to the container's egress proxy (`x-deny-reason: host_not_allowed` on plc.directory, verified with curl), not to the PLC directory. Result recorded as **null** per operator ruling 2026-09-20 (deferred to the operator's machine: `cargo make test-network`). Async-runtime shape used: a per-call multi-thread tokio runtime inside `resolve_pds_blocking` — adequate for one probe; Phase 1 decides whether the core owns a long-lived runtime (LOE packet field).
next:                Run 8 — B5 feature build
```

```
run:                 8
run_type:            host-build
platform:            host
started_at:          2026-09-20T15:26:18.252Z
ended_at:            2026-09-20T15:31:09.662Z
elapsed_min:         5
clean_checkout:      false
commit:              null
pins:                (Run 5) + subduction_keyhive=0.8.2@21b2e6b8 subduction_iroh=0.10.2@21b2e6b8 iroh=1.2.0
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-10=21b2e6b8
l14_note:            n/a
loe_note:            `--features subduction`: 222 additional packages compile (iroh 1.2.0 tree), 4m51s incremental over Run 5, green first pass. B5 done-when met on host; device targets pending (operator Units 4–5). Lockfile: 573 packages total, one keyhive_core, one automerge.
next:                Run 9 — UniFFI bindgen on host
```

```
run:                 9
run_type:            binding-build
platform:            host
started_at:          2026-09-20T15:35:23.933Z
ended_at:            2026-09-20T15:37:45.210Z
elapsed_min:         2
clean_checkout:      false
commit:              null
pins:                uniffi=0.32.1 (proc-macro, library mode)
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            FIRST L-14 EVIDENCE: `uniffi-bindgen generate --library target/debug/liblfs_core.so --language kotlin` succeeds from a workspace that includes keyhive_core 0.5.0 — 1,687-line `lfs_core.kt` with `openDoc/put/get/save/load/resolvePds/pins` and `CoreException`. ktlint absent (formatting warning only). What this does NOT show: JNA load of the arm64 .so on a device, cargo-ndk cross-compile of keyhive_core's crypto deps (curve25519-dalek, blake3, chacha20poly1305 — all pure Rust or cc-built), or Compose integration. L-14 narrows from "UniFFI Kotlin output unconfirmed" to "Kotlin output confirmed on host; device link unconfirmed".
loe_note:            Swift: `lfs_core.swift` (963 lines) + `lfs_coreFFI.h` (610) + modulemap generated on Linux from the same .so — swift-format absent (warning only). Elapsed includes compiling the uniffi CLI (~2 min); bindgen itself is seconds. Generated files are NOT committed (.gitignore); they regenerate per platform build.
next:                Run 10 — operator: cargo make android-so + bindgen-kotlin on the operator machine (Unit 4; L-14 first)
```

```
run:                 10
run_type:            
platform:            android
started_at:          
ended_at:            
elapsed_min:         
clean_checkout:      
commit:              
pins:                
outcome:             
defects:             
defect_classes:      
sentinel_state:      n/a
l14_note:            
loe_note:            
next:                
```

---

*Append-only. Canonical copy: operator's machine → `docs/` in `jediwright/local-first-social-native` at first commit.*

run:                 11
run_type:            note
platform:            n/a
started_at:          2026-09-20T16:06:43.000Z
ended_at:            2026-09-20T16:40:00.000Z
elapsed_min:         34
clean_checkout:      false
commit:              da8ca7e
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8
outcome:             pass
defects:             1
defect_classes:      other
sentinel_state:      n/a
l14_note:            n/a
loe_note:            First commit and push to jediwright/local-first-social-native over HTTPS (D-1 discharged). Runs 0-9 `commit: null` -> da8ca7e by this note; prior entries not edited (D-4 discharged). Root commit amended pre-push to extend .gitignore with Android/iOS shell exclusions (5eb7f4b -> da8ca7e). Defect: harness extraction instruction omitted the ~/Downloads path; git init briefly created ~/.git in the home directory, removed before any add/commit; nothing outside the project tree was touched. SSH auth absent on this machine; HTTPS matches the other track repos. Operator-time cost ~20 min.
next:                cargo make test; cargo make test-network (Run 12, network-probe, host)

run:                 12
run_type:            network-probe
platform:            host
started_at:          <ISO UTC — start of rustup install>
ended_at:            <ISO UTC — resolve pass>
elapsed_min:         <fill>
clean_checkout:      true
commit:              1386e82
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8
outcome:             pass
defects:             1
defect_classes:      other
sentinel_state:      n/a
l14_note:            n/a
loe_note:            Fresh macOS toolchain (aarch64-apple-darwin): rustup stable rustc 1.98.1 (2026-09-01) + cargo-make 0.37.24 (1m31s). Build session ran on Ubuntu-packaged rustc 1.91.1 (D-3); both above the 1.90.0 floor. Host re-ground: lfs_core 4 passed / 1 ignored, first pass, 39s with --all-features (Subduction included) vs 4m51s in the Linux container. B4 live DID->PDS resolve passes twice: direct cargo test from core/ (0.32s) and canonical `cargo make test-network` from root (0.22s; 16.5s rebuild without the subduction feature) -- D-2 closed; first real network result across atrium-identity + tokio on host. Defect: harness instruction ran cargo make from core/; cargo-make fell back to built-in workspace recursion and the root Makefile.toml (which already guards with default_to_workspace=false and says "run from repo root") was never read. No build-system change. B4 on device is Run 15 (Android) and Unit 5 (iOS).
next:                Unit 4 -- rustup target add x86_64-linux-android aarch64-linux-android; cargo install cargo-ndk; AVD (Google Play, x86_64); Run 13 binding-build

run:                 13
run_type:            binding-build
platform:            android
started_at:          2026-09-20T16:50:58.000Z
ended_at:            2026-09-20T17:07:04.000Z
elapsed_min:         16
clean_checkout:      false
commit:              8dc6dab
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) rustls=0.23.45 uniffi=0.32.1 cargo-ndk=4.1.2 ndk=30.0.16248370
outcome:             pass
defects:             4
defect_classes:      other, dependency
sentinel_state:      n/a
l14_note:            emulator arm64-v8a (Apple-silicon host; D-8 -- R15's x86_64 premise inverted: arm64 is the run target, x86_64 compiled/unrun). Defects 1-3 (class other, build-system; fixed 6cfee9e): android-so and bindgen-kotlin ran cargo metadata from the repo root, which has no Cargo.toml -- cargo-ndk and uniffi-bindgen do not honour --manifest-path for that step; bindgen-kotlin also pointed at a .so host cdylib that is .dylib on macOS. Neither task had ever been invoked via cargo make (Run 9 ran uniffi-bindgen directly). Defect 4 (class dependency; fixed 8dc6dab): reqwest default-tls -> native-tls -> openssl-sys, no aarch64-linux-android sysroot; swapped to rustls-tls (webpki roots) with atrium-oauth/atrium-xrpc-client default-features off; lockfile -32 lines, openssl chain gone. NDK r30 -> 16 KB page alignment by default; no alignment friction. After fixes: cross-compile first-pass green for both ABIs (arm64 36.25s, x86_64 38.84s, 75.49s total); every crypto/-sys crate (ring, blake3, curve25519-dalek, chacha20poly1305, libsqlite3-sys bundled) built clean; liblfs_core.so 12.9 MB per ABI unstripped. Kotlin generated from the Android arm64 ELF on the macOS host: 1,687 lines, same count as Run 9 from the Linux host .so. ktlint absent (warning only). No AAR task exists; the AAR is the Gradle build's output in Run 14.
loe_note:            Android toolchain install (rustup targets, cargo-ndk 16s, Android Studio Quail 4 2026.1.4p1 + SDK 37 + NDK r30 + Play arm64 image 2.2 GB + AVD Pixel 9) ~45 min operator wall-clock (~ estimated; not timed). Run itself 16 min, of which ~12 min was diagnosing and fixing four defects, ~2.5 min compiling. Fix commits 6cfee9e, 8dc6dab. Harness defects H-3 (placeholder in a runnable block, ~1 min) and H-4 (two tasks in one block after saying stop-at-first-failure, ~0 min) logged; class other, no repo change. Phase 1 pre-decision added: mobile TLS backend (rustls + platform verifier vs webpki roots vs vendored OpenSSL). Host test-network under rustls NOT yet re-run -- Run 15 covers B4 on device; a host re-verify rides in Run 15's loe_note.
next:                Run 14 -- shell-run: Compose shell on AVD Pixel 9 (arm64, API 37.2): type -> save -> kill -> reload -> text returns

run:                 14
run_type:            shell-run
platform:            android
started_at:          2026-09-20T17:13:00.000Z
ended_at:            2026-09-20T17:33:00.000Z
elapsed_min:         20
clean_checkout:      false
commit:              d4dd532
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 jna=5.17.0 kotlinx-coroutines=1.10.2 agp=9.4.1 kotlin=2.2.10 compileSdk=37 minSdk=24 jbr=25.0.3
outcome:             pass
defects:             1
defect_classes:      other
sentinel_state:      n/a
l14_note:            AVD Pixel_9 = emulator-5554, sdk_gphone16k_arm64 (Play image android-37.2, 16 KB pages), ro.product.cpu.abi=arm64-v8a. D-8 accepted (operator ruling this session): arm64 exercised on emulator; x86_64 .so packaged, unrun; no physical device. B8 pass: launch 1 -> openDoc("note") ok, text=0 chars, pins string from the core matches Run 13 (device shows subduction=off -- shell built from the default feature set); typed "hello phase 0", Save -> saved: 159 bytes; am force-stop -> pidof empty; relaunch (new pid) -> field shows "hello phase 0", text=13 chars. A-O15 resolved: Core.save persists to the core's SQLite file on-device. Gradle: C1 (build.gradle.kts, jniLibs + generated srcDirs from ../../bindings/kotlin) and C2 (MainActivity.kt) applied as drafted; :app:assembleDebug BUILD SUCCESSFUL in 33s first pass (17:25:30Z-17:26:04Z), neither predicted failure (kotlin.srcDir on the android source set; @aar JNA coordinate) fired. Note, not a defect: stripDebugDebugSymbols could not strip libjnidispatch.so / liblfs_core.so / libandroidx.graphics.path.so (llvm-strip not resolved from NDK r30); app-debug.apk 38.7 MB unstripped. logcat: libjnidispatch.so loaded via nativeloader ok; liblfs_core.so dlopen'd lazily by JNA (no nativeloader line, expected); one SIGABRT in an emulator HAL process (pid != app), noise. Evidence: docs/evidence/run14/{run14-launch1,run14-saved,run14-relaunch}.png. Repo defects: 0. Defect 1 (class other, host env, ~1 min): gradlew found no Java runtime -- resolved by exporting JAVA_HOME to Android Studio's bundled JBR 25.0.3 for the shell session only; no repo change, no shell-profile change.
loe_note:            Wizard-to-relaunch ~20 min operator wall-clock (started_at ~ from wizard launch, A-O14; ended_at ~ from relaunch screenshot mtime), of which 33s Gradle compile (first run, includes distribution + dependency download) and ~2 min AVD first boot. Harness defects: H-5 quoted $! in an issued echo left the operator's zsh at a dquote> prompt (~1 min); H-6 first adb block not serial-targeted, tripped on a ghost offline emulator-5562 (~1 min); rule: every adb/emulator block carries -s <serial>. Host cargo make test-network under rustls still NOT re-run -- rides in Run 15's loe_note. Emulator left running for Run 15.
next:                Run 15 -- network-probe: resolvePds from the shell on emulator-5554 (rustls + tokio via JNA), plus host `cargo make test-network` re-verify; then G13 checkpoint (Run 16 collapsed).

run:                 15
run_type:            network-probe
platform:            android
started_at:          2026-09-20T18:01:53.000Z
ended_at:            2026-09-20T18:06:55.000Z
elapsed_min:         5
clean_checkout:      false
commit:              5d8abc7
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 jna=5.17.0 kotlinx-coroutines=1.10.2 agp=9.4.1 kotlin=2.2.10 compileSdk=37 minSdk=24 jbr=25.0.3
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            emulator-5554 (AVD Pixel_9, sdk_gphone16k_arm64, arm64-v8a) re-verified BOOTED at open; D-8 stands. B4 on device pass: resolvePds(did:plc:z72i7hdynmk6r22z27h6tvur) -> "resolvePds ok in 1477 ms: https://puffball.us-east.host.bsky.network" (cold, first tap after launch); second tap 1241 ms, same PDS. rustls + webpki-roots + tokio via JNA resolved DID -> DID doc -> PDS from the emulator first try; no CoreException, so no finding lands on Phase 1 pre-decision (vi). Cold-warm gap ~240 ms -> cost is network round-trips, not runtime/TLS setup (note, not a finding). Generated signature verified before drafting: fun resolvePds(did: String): String, @Throws(CoreException::class), blocking -- call runs on a named background Thread, result posted via runOnUiThread. C4a: wizard manifest lacked INTERNET; added ahead of <application> (guaranteed first failure otherwise). C4b: MainActivity.kt 116 lines. C5: :app:assembleDebug BUILD SUCCESSFUL in 2s (18:02:53Z-18:02:56Z, warm daemon, config cache reused). Install -r 18:03:19Z, prior lfs.sqlite survived -> relaunch showed text=13 chars (B8 re-confirmed for free). Probe DID: the bsky.app fallback (A-O16) -- R15-0 grep of core/src + core/tests found no DID literal although the host test is named resolve_known_did_to_pds; host DID not identified, no action. Evidence: docs/evidence/run15/run15-resolve-ok.png (18:05:52Z). Repo defects: 0. Host-env defects: 0.
loe_note:            Run 15 wall-clock 5 min from C4a apply to host re-verify done. Host cargo make test-network under rustls re-run: resolve_known_did_to_pds ok, 0.55s test / 11.0s cargo-make (18:06:44Z-18:06:55Z) -- item carried since 8dc6dab CLOSED. Host 0.55s vs device 1.24s warm / 1.48s cold. Harness defects: H-7 bare `adb` in the open block, command not found in a fresh shell (~1 min; rule: full platform-tools path on every adb block, same class as JAVA_HOME); draft defect caught pre-apply: handoff's C4a used `sed -i ''` with \n in the replacement, which BSD sed writes literally -- swapped to perl -pi, no repo effect. Note: main is ahead of origin/main by 5 (now 6) Phase 0 commits, unpushed; not a plan requirement. Emulator left running.
next:                G13 checkpoint (Unit 4 closed: Runs 13-15). Then Unit 5 -- iOS: read MeetingNotes first and log the read; Units 6-7 per the 09-20 build handoff section 7.
```

```
run:                 16
run_type:            note
platform:            ios
started_at:          2026-09-20T18:30:58.615Z
ended_at:            2026-09-20T18:31:14.712Z
elapsed_min:         1
clean_checkout:      n/a
commit:              n/a
pins:                n/a
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            n/a
loe_note:            MeetingNotes read (plan B7 / spec §5.1, sweep F-6 Action 3): harness source-read of automerge/MeetingNotes via codeload tarball of main (ref ~, no sha in tarball; A-O18: not a repo file, upstream reference app). Findings: (M-1) document-based app (DocumentGroup + ReferenceFileDocument, CBOR-wrapped Automerge bytes + DocumentId via Files) -- B7 does NOT copy this; B7 = one screen over UniFFI with core-owned SQLite, parity with the B8 Compose shell so L-14 is measured like-for-like. (M-2) sync via AutomergeRepo WebSocket + PeerToPeer (Bonjour _automergesync._tcp) -- out of Phase 0. (M-3) no BackgroundTasks, no Keychain/Secure Enclave in the reference; spec §5.1 iOS-row prescriptions do not derive from it (~, OI-P0-3 candidate note; no Phase 0 effect). (M-4) packaging precedent = SPM package wrapping a binary xcframework (automerge-swift upToNextMajor 0.5.8), iOS 16.4 / Swift 5.0, CI on xcodebuild -destination 'platform=iOS Simulator' -- confirms plan B6 bindings/swift layout; B7 deployment target set at iOS 17. Host-env at Unit 5 open: Xcode NOT INSTALLED (xcode-select -p = CommandLineTools; no /Applications/Xcode*.app; mdfind empty) -- install is Run 17's toolchain cost. Physical iPhone 17 Pro available (operator ruling); no D-9 needed. Harness defects: H-8 (zsh nomatch glob in a read-only block, no effect); H-10 (Run 15 entry committed at 96220b7 without its closing code fence -- repaired by this append's leading fence, prior entry text untouched).
next:                Run 17 -- binding-build (ios): Xcode install + xcode-select + iOS platform; rustup aarch64-apple-ios + aarch64-apple-ios-sim; cargo make swift-xcframework; uniffi Swift bindgen; Package.swift.
```
