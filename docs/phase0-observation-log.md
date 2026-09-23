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

```
run:                 17
run_type:            binding-build
platform:            ios
started_at:          2026-09-20T18:56:00.000Z
ended_at:            2026-09-20T19:09:44.000Z
elapsed_min:         14
clean_checkout:      false
commit:              362487a
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) rustls=0.23.45 uniffi=0.32.1 rusqlite=0.40.2 cargo-make=0.37.24 xcode=27.0(27A266a) ios-sdk=27.0 ios-sim-runtime=27.0(24A434) swift-tools=5.9 rustc=1.98.1
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            iOS side of the L-14 asymmetry, for the LOE ratio: iOS binding-build 14 min / 0 repo defects / 0 host-env defects vs Android Run 13 16 min / 4 repo defects (3 build-system, 1 dependency) / 0 host-env. Pre-invocation read of the never-run swift-xcframework task (Run 13 lesson) predicted four items: P-1 iOS rustup targets absent (host env, added, not a defect); P-2 staticlib crate-type (already present -- cleared by read); P-3 line-56 create-xcframework placeholder (plan-anticipated fill, written 362487a, not a defect); P-4 uniffi-bindgen --library on a .a archive (~ -> ✓, works). Predicted-then-cleared is the difference from Run 13's found-by-failure.
loe_note:            Xcode 27.0 install from App Store ~18:50Z-18:56Z (~, screenshot to xcodebuild -version; simulator runtime 8.05 GB downloaded in parallel, complete before Run 18, not on the critical path). Device staticlib aarch64-apple-ios 41.61s cold (18:57:11Z-18:57:53Z), 51.0 MB; sim staticlib 43.69s cold (18:58:31Z-18:59:15Z), 51.0 MB; every cc-built crate (ring, blake3, libsqlite3-sys bundled, curve25519-dalek, chacha20poly1305) compiled against iPhoneOS/iPhoneSimulator 27.0 SDK first pass. Swift bindgen from the device .a: 963 + 610 lines, identical to Run 9 (Linux .so) -- three-target determinism. cargo make swift-xcframework end-to-end 4.10s warm: xcframework with ios-arm64 + ios-arm64-simulator slices; headers dir carries lfs_coreFFI.h + module.modulemap. Package.swift (LfsCore, iOS 17, binaryTarget lfs_coreFFI + target from generated/lfs_core.swift) written; NOT compile-verified on host (iOS-only binary target; expected) -- verified at Run 18 by the Xcode build, as Kotlin was by Gradle at Run 14. BUILDING.md iOS section already accurate; no edit. Harness defects H-9 (zsh nomatch glob, repeat of H-8, ~1 min), H-11 (nested backtick fences in an issued heredoc, render-only, caught pre-apply; rule: tilde outer fence). B6 iOS done-when met: one cargo make invocation, documented.
next:                Run 18 -- shell-run (ios): SwiftUI one-screen shell in apps/ios depending on the local bindings/swift package; round-trip on physical iPhone 17 Pro (UDID from xcrun devicectl list devices).
```

```
run:                 18
run_type:            shell-run
platform:            ios
started_at:          2026-09-20T19:19:00.000Z
ended_at:            2026-09-20T20:03:00.000Z
elapsed_min:         44
clean_checkout:      false
commit:              0b54903
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) rustls=0.23.45 uniffi=0.32.1 rusqlite=0.40.2 xcode=27.0(27A266a) ios-sdk=27.0 swift-tools=5.9 rustc=1.98.1 deployment-target=17.6 device-ios=26.6.1
outcome:             pass
defects:             1
defect_classes:      build-system
sentinel_state:      n/a
l14_note:            iOS side of the L-14 asymmetry, shell-run: ~24 min active wall-clock / 1 repo defect (warning-only) / 0 host-env vs Android Run 14 20 min / 0 repo / 1 host-env (JAVA_HOME). B7 pass on the physical iPhone 17 Pro (iPhone18,1, UDID [UDID redacted 2026-09-20, R-P0-17], iOS 26.6.1): launch 1 -> openDoc ok, text=0 chars, pins string identical to Run 14 (subduction=off, default feature set); typed "Hello phase 0" (iOS autocapitalised the H; 13 chars), Save -> saved: 159 bytes -- byte-identical to the Android Run 14 save; app-switcher kill -> relaunch from the home-screen icon -> field shows "Hello phase 0", text=13 chars. Core.save persists to Application Support/lfs.sqlite on iOS (A-O15 parity). A-O20 RESOLVED: Package.swift (LfsCore) imported and lfs_coreFFI.xcframework linked by the Xcode build first pass. Defect 1 (class build-system, repo, warning-only, NOT fixed this run): 22 linker warnings "object file was built for newer iOS version (27.0) than being linked (17.6)" on every cc-built object in liblfs_core.a (sqlite3, ring curve25519/aes/montgomery/poly1305/chacha, ...) -- the swift-xcframework task's cargo build lines carry no IPHONEOS_DEPLOYMENT_TARGET, so cc defaults C objects to the SDK version; Rust objects unaffected; app ran on 26.6.1. Fix (Makefile env prefix, both lines, rebuild ~90s x2) scheduled as its own commit before Run 19; Run 19's Xcode build verifies warning count -> 0. Android has no analogue (cargo-ndk -p 24 pins the API floor). A-O22 carried: Swift `CoreError` is not public (Kotlin CoreException is) -- Phase 1 error-mapping item; no Phase 0 effect. Xcode 27 debug-launch console noise (debug dylib, "No entry point found. Checking for alias", CA launch-metrics event failure) noted, not the app. Evidence: docs/evidence/run18/{run18-launch1,run18-saved,run18-relaunch}.png (phone screenshots 15:58/15:58/16:03 local, AirDropped).
loe_note:            started_at ~19:19Z from the wizard screenshots (A-O23 ~; date -u not captured at C1); elapsed_min 44 includes a ~19 min session break (C2 close 19:25Z -> C3 resume 19:44:02Z captured); active ~24 min. Operator-side stops: (1) Developer Mode not enabled on the phone -> enabled, reboot (~3 min); (2) Xcode Devices window "Screen Sharing Unavailable -- must be running iOS 27.0" read as a build blocker -> non-event, screen mirroring only, phone update not needed (~1 min); (3) deployment target set to 17.6 before the first green build (prior wizard default not recorded, ~); (4) Untrusted Developer trust prompt: did not fire (no Developer App entry under Settings > General > VPN & Device Management; home-screen relaunch opened directly); (5) Trust This Computer USB-pairing prompt approved on first connect (~, operator recollection; expected, not a stop). Signing: automatic, personal team, com.uxminds.lfs.LfsShell. Harness defect H-12 (class other, ~2 min, no repo effect): drafted ContentView.swift without `import Combine`; Xcode 27's template enables member-import visibility, so SwiftUI no longer re-exports ObservableObject/@Published -- 4 compile errors, one-line fix (68 lines final; handoff said 62, miscount). B7 done-when met: round-trip on a physical iPhone. iOS Unit 5 so far: Runs 16-18, 1 repo defect total vs Android Runs 13-15, 5 repo defects.
next:                Makefile IPHONEOS_DEPLOYMENT_TARGET fix + rebuild (own commit), then Run 19 -- network-probe (ios): resolvePds from the shell on the iPhone; B4 done-when on both platforms.
```

```
run:                 19
run_type:            network-probe
platform:            ios
started_at:          2026-09-20T20:15:12.000Z
ended_at:            2026-09-20T20:24:31.000Z
elapsed_min:         9
clean_checkout:      false
commit:              a2555e2
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) rustls=0.23.45 uniffi=0.32.1 rusqlite=0.40.2 blake3=1.8.7 xcode=27.0(27A266a) ios-sdk=27.0 swift-tools=5.9 rustc=1.98.1 deployment-target=17.6 iphoneos-deployment-target=17.0 device-ios=26.6.1
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            B4 on device, iOS, physical iPhone 17 Pro (UDID [UDID redacted 2026-09-20, R-P0-17], iOS 26.6.1, Wi-Fi): resolvePds(did:plc:z72i7hdynmk6r22z27h6tvur) -> "resolvePds ok in 342 ms: https://puffball.us-east.host.bsky.network" (cold, first tap after launch); taps 2-3: 118 ms, 260 ms, same PDS. Same host as Android Run 15 (1477 cold / 1241 warm on the emulator); iOS ~4x faster in absolute terms -- physical radio vs emulator NAT, not a runtime finding; cold-warm gap and tap-to-tap spread are network round-trip variance, consistent with the Run 15 note. rustls + webpki-roots + tokio via UniFFI/Swift resolved DID -> DID doc -> PDS first try; no throw, so nothing lands on Phase 1 pre-decision (vi). B4 done-when now met on BOTH platforms. Generated signature verified before drafting: func resolvePds(did: String) throws -> String, blocking -- dispatched on DispatchQueue.global(.userInitiated), result posted via DispatchQueue.main (Kotlin parity: named Thread + runOnUiThread). Shell class marked @unchecked Sendable pre-emptively (predicted P-1: Swift 6 strict-concurrency capture error) -- P-1 did not fire; language mode of the wizard target not read (~). No INTERNET-permission analogue on iOS (Run 15's C4a has no counterpart); ATS default allows the HTTPS calls; no Info.plist edit. Run 18 defect 1 VERIFIED FIXED from the Xcode side: warning count 22 -> 0 on this build after commit e6582d8 (export IPHONEOS_DEPLOYMENT_TARGET=17.0 in the swift-xcframework script); archive side: otool minos 17.0 on all 142 members. Fix needed one stale member cleared -- blake3_neon.o stayed at 27.0 after the env change because blake3's build.rs is not re-run on IPHONEOS_DEPLOYMENT_TARGET (ring, libsqlite3-sys, rustls chain did recompile); cargo clean -p blake3 for both iOS targets + re-run (20.2s) cleared it. Reinstall preserved lfs.sqlite: launch showed text=13 chars (B7 re-confirmed, as Run 15 did for B8). Evidence: docs/evidence/run19/{run19-launch,run19-resolve-cold-342ms,run19-resolve-warm-118ms,run19-resolve-warm-260ms}.png (16:16/16:18/16:20/16:22 local). Repo defects: 0. Host-env defects: 0.
loe_note:            Run 19 wall-clock 9 min (C1 apply 20:15:12Z -> commit 20:24:31Z), including the three taps and four screenshots; Xcode incremental build seconds. Makefile fix + rebuild before this run (own commit e6582d8): first cargo make swift-xcframework after the env change 47.2s (20:11:10Z-20:11:57Z; ring/sqlite/rustls chain recompiled), second after blake3 clean 20.2s. ContentView.swift 91 lines (harness estimate said 90). No harness defects this run. iOS Unit 5 complete: Runs 16-19, 1 repo defect total (warning-only, fixed same session) vs Android Unit 4 Runs 13-15, 5 repo defects; L-14 delta drafted at Unit 7 with both platforms' numbers.
next:                G13 checkpoint (Unit 5 closed: Runs 16-19). Then Unit 6 -- B12 OAuth-path smoke on both shells (Runs 20-21, oauth-smoke), B9 Bedrock common.nix note, B10 did:plc route note; Unit 7 exit check + LOE packet + close.
```

```
run:                 20
run_type:            oauth-smoke
platform:            ios
started_at:          2026-09-20T20:50:22.000Z
ended_at:            2026-09-20T20:56:15.000Z
elapsed_min:         6
clean_checkout:      false
commit:              8070059
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) rustls=0.23.45 uniffi=0.32.1 rusqlite=0.40.2 blake3=1.8.7 xcode=27.0(27A266a) ios-sdk=27.0 swift-tools=5.9 swift-version=5.0 rustc=1.98.1 deployment-target=17.6 iphoneos-deployment-target=17.0 device-ios=26.6.1 placeholder=https://jediwright.github.io/local-first-social-native/phase0/oauth.html@61905d5
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            B12 on iOS, physical iPhone 17 Pro (UDID [UDID redacted 2026-09-20, R-P0-17], iOS 26.6.1, Wi-Fi): OAuth tap -> ASWebAuthenticationSession(url: placeholder, callback: .customScheme("lfs")) presented the GitHub Pages placeholder in the system sheet (jediwright.github.io) -> Continue to app (href lfs://oauth/callback?code=phase0) -> sheet dismissed -> status "oauth-smoke ok in 8411 ms: lfs://oauth/callback?code=phase0 code=phase0". The 8411 ms is tap-to-tap human time inside the sheet, not a latency figure. No CFBundleURLTypes / Info.plist edit: ASWebAuthenticationSession intercepts its own callback scheme (assumption held; scheme registration proper is Phase 2). prefersEphemeralWebBrowserSession=true: no "wants to use github.io to sign in" consent sheet fired (a choice, not a workaround; Phase 2 decides ephemeral vs shared cookie jar). Predicted P-2 (new file not in target) did not fire: Xcode 27 synchronized folder picked up OAuthSmoke.swift unadded. Placeholder page is tap-to-continue, not auto-redirect, so the same page serves Android Custom Tabs (user-gesture requirement). Launch showed text=13 chars (B7 re-confirmed). Xcode warning count not read this run (null). Repo defects: 0. Host-env defects: 0. Rulings at Unit 6 open: R-P0-15 B12 Android side = emulator-5554 under D-8; R-P0-16 placeholder = GitHub Pages from main:/docs (public repo; docs/phase0/oauth.html, commit 61905d5; push also caught origin up from 3c81b22). A-O26 RESOLVED: SWIFT_VERSION = 5.0 both configs (Swift 5 language mode; explains P-1 never firing in Run 19). Evidence: docs/evidence/run20/{run20-launch,run20-placeholder-sheet,run20-callback}.png (16:53/16:54/16:54 local, AirDropped).
loe_note:            Run 20 wall-clock 6 min (C0 20:50:22Z -> commit 20:56:15Z) including ContentView read, one new 43-line file, three perl inserts (94 lines, verified), device build + run, three screenshots. Pages setup before the run (page commit 61905d5 + Settings > Pages + deploy + curl 200) ~5 min, operator-side, not in run wall-clock. Harness defects: 0. Operator-side stop: none on the device path; one harness-instruction clarification (cmd-R is Xcode on the Mac, not the phone) ~1 min, class other, no repo effect. iOS OAuth path needed zero workarounds -> LOE packet "OAuth-path readiness" iOS row: none.
next:                Run 21 -- oauth-smoke (android) on emulator-5554 (R-P0-15): Custom Tabs / ACTION_VIEW to the same placeholder, <intent-filter> for lfs://oauth/callback in AndroidManifest, status line on redirect. Check emulator state first.
```

```
run:                 21
run_type:            oauth-smoke
platform:            android
started_at:          2026-09-20T20:57:44.000Z
ended_at:            2026-09-20T21:03:38.000Z
elapsed_min:         6
clean_checkout:      false
commit:              4fb5eff
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 jna=5.17.0 kotlinx-coroutines=1.10.2 agp=9.4.1 kotlin=2.2.10 compileSdk=37 minSdk=24 jbr=25.0.3 emulator=Pixel_9(AVD)-17 placeholder=https://jediwright.github.io/local-first-social-native/phase0/oauth.html@61905d5
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            B12 on Android, emulator-5554 (Pixel_9 AVD, Android 17 image; R-P0-15 under D-8; emulator-5562 stale/offline entry present, every adb call pinned -s emulator-5554): OAuth tap -> Intent.ACTION_VIEW(OAUTH_PLACEHOLDER) opened Chrome -> placeholder page -> Continue to app (href lfs://oauth/callback?code=phase0) -> manifest <intent-filter> (VIEW, DEFAULT, BROWSABLE, scheme=lfs host=oauth path=/callback) + android:launchMode="singleTask" routed back via onNewIntent -> status "oauth-smoke ok in 51026 ms: lfs://oauth/callback?code=phase0 code=phase0". The 51 s is two Chrome first-run cards (Make Chrome your own -> Stay signed out; notifications -> No thanks) plus taps on a fresh AVD -- emulator environment, not latency, not a defect; a physical device with Chrome set up would not hit it. No "Open with" chooser fired. ACTION_VIEW chosen over Custom Tabs for the smoke (no androidx.browser dependency / catalog edit; same external user-agent per RFC 8252); Custom Tabs is Phase 2 wiring with atrium-oauth. Predicted P-1 (onNewIntent nullability) did not fire: override fun onNewIntent(intent: Intent) compiled on compileSdk 37. Predicted P-2 (no browser on image) did not fire; startActivity wrapped for ActivityNotFoundException regardless. Launch showed text=13 chars (B8 re-confirmed; Android's own lowercase save). Gradle installDebug 8 s incremental; first am start hit the pre-install instance ("intent delivered to currently running top-most instance") -> force-stop + restart before tapping (~30 s, class other, not a defect). Repo defects: 0. Host-env defects: 0. Evidence: docs/evidence/run21/run21-callback.png (adb screencap 17:03 local). B12 done-when now met on BOTH platforms.
loe_note:            Run 21 wall-clock 6 min (C0 20:57:44Z -> commit 21:03:38Z): manifest 2-substitution perl edit, MainActivity.kt full-file rewrite (157 lines; harness estimate said 152 -- miss logged as a note, heredoc governs), Gradle install, force-stop/restart, two Chrome first-run dismissals, callback, screencap. iOS Run 20 was also 6 min -> the OAuth-path smoke shows no L-14 asymmetry at all; the asymmetry lives in binding-build and shell-run (Runs 13/14 vs 17/18). Harness defects: 0. LOE packet "OAuth-path readiness" Android row: no workarounds; one environment note (Chrome first-run on fresh AVDs). Unit 6 build items closed (B12 both); B9 and B10 notes remain.
next:                B9 Bedrock common.nix fork-read note and B10 did:plc verification-method route note (docs/); harness-side reads need container egress or operator pastes. Then Unit 7: sentinel-check re-run (A-O3), exit check vs plan §0, LOE packet, L-14 delta, close.
```

```
run:                 22
run_type:            note
platform:            n/a
started_at:          2026-09-20T21:06:00.000Z
ended_at:            2026-09-20T21:09:51.000Z
elapsed_min:         4
clean_checkout:      n/a
commit:              00948e2
pins:                n/a
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            n/a
loe_note:            B9 and B10 filed as docs/bedrock-common-nix-read.md (34 lines) and docs/did-plc-route-oi-m4.md (31 lines), overwriting the repo-creation stubs at those paths (A-O35). Sources read harness-side from the container via codeload tarballs (connector declared at the Unit 6 midpoint: egress to github.com / api.github.com / codeload / raw, reads only, A-O34); GitHub API rate-limited on the shared IP after 3 calls, as plan §3 predicted — tarballs unaffected. B9 (inkandswitch/bedrock main, updated 2026-09-08, sha not taken ~): hosted default is Caddy TLS -> Subduction WS on localhost:8080, auth="open", first-boot 32-byte key seed as host identity, redb + resident-tree cache, memory caps from incidents, and Caddy access logs designed to join peer ID to client IP for 14 days -- the metadata-custody point the Phase 1 self-host decision turns on; staging host (4 GB) is the minimum-viable self-host shape; transport is WS not QUIC (L-5 fallback role). B10 (did-method-plc main, pushed 2026-09-01, spec v0.1): verificationMethods accept any did:key type with no DID control, each change is a rotation-key-signed PLC op with a 72 h window and a permanent public log; the Keyhive doc ID cannot be a verification method; ROUTE NAMED = record route (social.localfirst.identity under OAuth); OI-M4 CLOSES with the verification-method route verified-available-and-declined; residue (post-T4 DID-side key commitment, Phase 4b) queued for the spec v0.1.5 cut list. started_at is the first fetch, ~ (not captured by date -u). No harness defects. Unit 6 fully closed: B9, B10, B12 both platforms.
next:                Unit 6 close block + downloadable handoff. Unit 7 next session: sentinel-check re-run (A-O3, Run 0 now ~6 h old), exit check vs plan §0 verbatim, LOE packet (§5) from log fields, L-14 delta with both platforms' numbers, close handoff + LOE packet downloadable.
```

```
run:                 23
run_type:            sentinel-check
platform:            n/a
started_at:          2026-09-20T21:21:54.809Z
ended_at:            2026-09-20T21:22:31.804Z
elapsed_min:         1
clean_checkout:      n/a
commit:              ed47363
pins:                n/a
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=0.5.0@90fe4a51(main,2026-09-17) S-3=#230 open/DRAFT/unmerged(33 commits, head keyline_crate, updated 2026-09-14) S-4=kh0 S-5="0.5" | S-6=null(operator, before Phase 2) S-7=SyncpointMap pub(crate)@21b2e6b8 and @main S-8=samod 0.14.0/autosurgeon 0.14.0 S-9=api 0.25.8/oauth 0.1.7/identity 0.1.9 S-10=git-only, subduction_iroh 0.10.2, subduction_keyhive 0.8.2, workspace automerge 0.11.0 S-11=null(operator ruling: own repos, nothing beyond this session's work) S-12(a)=null(root + keyhive_core CHANGELOG 404; #keyhive not read) S-12(b)=branch `2026-sept-updates` EXISTS (head 881226c9, 2026-09-08), NOT merged (lib.rs 4-line diff, Cargo.toml 49-line diff vs main)
l14_note:            n/a
loe_note:            Unit 7 exit sentinel re-run, harness-side from the container (egress to github.com/raw/codeload + crates.io declared, A-O41; reads only). Every S-1..S-5, S-7..S-10, S-12 value identical to Run 0 (~6 h 14 min earlier): NO TRIGGER MOVED. GitHub API rate-limited after one call (S-3) as plan §3 predicted; raw/atom/HTML reads used for the rest (✓). 1ec1669d seen in the HTML commit list is the #227 PR head, not a new main commit (title-read ✓). S-12(b) stands → Phase 1 does not open before a T4 scoping ruling.
next:                Run 24 — exit check vs plan §0; LOE packet
```

```
run:                 24
run_type:            exit-check
platform:            n/a
started_at:          2026-09-20T21:23:31.697Z
ended_at:            2026-09-20T21:37:31.000Z
elapsed_min:         14
clean_checkout:      n/a
commit:              ed47363
pins:                keyhive_core=0.5.0 samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      (as Run 23)
l14_note:            L-14 quantified (LOE packet §4): run wall-clock to round-trip Android 36 min vs iOS 38 min active (≈1:1); repo defects 4:1 (Run 13 vs Run 18, all build-system/dependency, all fixed in-run); toolchain install ~45 vs ~6 min (~, ≈7.5:1); OAuth smoke 1:1. UniFFI Kotlin output from the keyhive_core-bearing workspace CONFIRMED on host, cross-compiled, loaded on device (Runs 9/13/14). Android side measured on emulator (D-8); device-level Android friction unmeasured.
loe_note:            Exit criterion (plan §0 verbatim: "Hello-world with an Automerge doc round-tripping through the core on a device") MET on iOS (Run 18, physical) and on Android (Run 14, emulator under D-8); both saves 159 bytes. B1-B12 all met. Phase 0 total ~6.3 h governed wall-clock (Run 0 start → this entry), ~51 min toolchain installs outside runs, 24 runs, 5 repo defects, 12 harness defects. LOE packet filed as docs/phase0-loe-packet_2026-09-20.md (plan §5 fields; estimate section ~). Phase 0 closes on the same upstream sentinel state it opened on. Log residue noted, not edited: Run 10 blank block; Run 12 timestamps unfilled.
next:                Phase 1 plan — not before the T4 scoping ruling (S-12(b)); physical Android device first Phase 1 item; spec v0.1.5 Lightweight cut; SL-0241 / B11 / L-14 deltas append in order, VERIFY tail.
```

```
run:                 25
run_type:            note
platform:            n/a
started_at:          2026-09-20T22:16:22.000Z
ended_at:            2026-09-20T22:16:22.000Z
elapsed_min:         0
clean_checkout:      n/a
commit:              n/a
pins:                n/a
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      n/a
l14_note:            n/a
loe_note:            RULING R-P0-17 (operator, 2026-09-20, post-close): a device identifier recorded in the Run 18, 19 and 20 l14_note fields is redacted from this log and from repository history (git filter-repo, replacement token in place). Append-only rule set aside for this one edit, by ruling, and recorded here rather than done silently. Commits from Run 18 forward were rewritten; the old-to-new sha map is held with the operator's session handoffs, not in this repository. No content other than the identifier changed.
next:                Ledger appends (SL-0241 -> B11 -> L-14) with tail verification; Discord/GitHub asks; T4 scoping session.
```

```
run:                 26
run_type:            pin-decision
platform:            n/a (container resolution + operator canonical apply)
started_at:          2026-09-20T23:42:00.000Z
ended_at:            2026-09-21T00:17:53.000Z
elapsed_min:         36
clean_checkout:      yes (fresh container; reproduction workspace)
commit:              f70889b
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 (H6 ADOPTED; was =0.5.0 crates.io) samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=main@90fe4a51(2026-09-17, no new commits) S-3=#230 OPEN/DRAFT/unmerged S-4=kh0 S-5="0.5" | S-12(a)=null (CHANGELOG 404 root + keyhive_core; #keyhive not read from container) S-12(b)=`2026-sept-updates` EXISTS (head 881226c9, unchanged since Run 23), NOT merged (branch-containment check: head on branch only, not main) S-13=no maintainer replies attached this session. NO STOP CONDITION FIRED. All values identical to Run 23. In-tree keyhive_core version = "0.5.0".
l14_note:            n/a
loe_note:            First Phase 1 run; wall-clock 36 min including container toolchain bootstrap and operator canonical apply. GitHub API rate-limited at first call (plan section 3 pattern); raw/atom/HTML fallback per Run 23 discipline. H6 executed twice: (1) container reproduction workspace (Ubuntu-packaged cargo/rustc 1.91.1, same minor as Phase 0 container toolchain) — baseline =0.5.0 locks 91 packages, exact LOE packet section 3 / Run 3 match; git pin locks 91 packages, lockfile name+version symmetric diff vs crates.io baseline NONE — identical dep set re-confirmed; (2) canonical apply same session: core/crates/lfs_core/Cargo.toml edited, commit f70889b (2 files, +72/-7, committed 2026-09-21T00:17:53Z); done-when test re-run on operator machine — PASS, identical tree line to container reproduction; lock churn limited to keyhive_core/beekem 0.3.0/keyhive_crypto 0.2.1 (versions match LOE section 3). DONE-WHEN TEST (plan section 4 H6) PASSES both sides: cargo tree -p keyhive_core -> keyhive_core v0.5.0 (https://github.com/inkandswitch/keyhive?rev=90fe4a51#90fe4a51). Declaring manifest is core/crates/lfs_core/Cargo.toml (one level deeper than plan section 7 sketch layout, ~ noted). Run close extended to the canonical apply commit; container work ended 00:10Z. Correctness fixes #226/#227/#232/#233 apply from first Phase 1 build per SL-0244/H6. NUMBERING: plan section 6 proposed "Run 25" for H6, but Run 25 was consumed by the R-P0-17 redaction note (SL-0245); log is append-only and sequential, so this run is 26 and section 6 proposed runs shift +1 (physical Android -> 27, runtime -> 28, sync-host -> 29). Operator accepted as silent carry 2026-09-20; plan text not amended.
next:                Run 27 -- physical-android (D-8 close), first device-level Android run, git pin in the lockfile it builds from.
```

```
run:                 27
run_type:            runtime-decision
platform:            n/a (desk decision; container sentinel reads only)
started_at:          2026-09-21T00:25:00.000Z
ended_at:            2026-09-21T00:34:00.000Z
elapsed_min:         9
clean_checkout:      n/a (no workspace touched)
commit:              null (no code lands this run; repo unchanged at 93ae49c)
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 (H6, Run 26) samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 tokio=1.53.1
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=main@90fe4a51(2026-09-17, no new commits; A-R5 discharged — 1ec1669d confirmed ancestor by git merge-base) S-3=#230 OPEN/DRAFT/unmerged (HTML fallback; API rate-limited, plan section 3 pattern) S-4=kh0 S-5="0.5" # pre-alpha (in-tree 0.5.0) | S-12(a)=null (CHANGELOG 404 root + keyhive_core; #keyhive not read from container) S-12(b)=`2026-sept-updates` EXISTS (head 881226c9, unchanged since Run 23), NOT merged — git merge-base --is-ancestor decisive (bare blob-filtered clone; head on branch only, not main) S-13=no maintainer replies attached. NO STOP CONDITION FIRED. All values identical to Run 26.
l14_note:            n/a
loe_note:            RUNTIME DECISION (plan section 1 1a item, section 6 runtime-decision run): the core owns a LONG-LIVED multi-thread tokio runtime, adopted now — the Run 7 per-call shape (Runtime spun inside resolve_pds_blocking, "adequate for one probe, not for sync") does not carry past the implementing commit. Decision TAKEN FOR SYNC NOW, not staged: Subduction's periodic cache-refresh and caller-driven sync loop (watch note r4 section 1) need a host that outlives a call, and the lifecycle contract is cheapest written at 1a's FFI-init seam, not retrofitted at the sync run. Lifecycle contract: lazy core-internal construction (OnceLock), explicit shell-called init FFI entry point (natural host: the 1a storage-open call), init dispatched off-main on both shells, NO shutdown owner in Phase 1 (process lifetime; graceful close is a Subduction-run question). FFI shape UNCHANGED — blocking fns, RUNTIME.block_on replaces per-call construction; Runs 15/19 shell dispatch code untouched; INTERNET (C4a) and ATS posture unmoved. NOT RULED: all sync-specific machinery (connection lifecycle, background-execution strategy, Subduction runtime integration, shutdown/drain), async FFI, samod runtime sharing, worker tuning. Done-when for the implementing run: grep for Runtime construction returns one hit in the runtime/init module only; resolve_pds_blocking constructs nothing; cargo make test-network passes; two consecutive resolvePds calls in one process pass (reuse proven); binding signatures unchanged. VERDICT-BEARING: NO — build rule inside plan 1a / SL-0244 scope (plan pre-registered the decision with loe_note as its record; R9-R12 / Run 26 H6 precedent); no ledger delta. REORDER (first entry to record it): Run 26 close handoff section 6a, operator ruling 2026-09-20 — no physical device same-day; silent carry, same pattern as the section 2 numbering ruling: runtime-decision = Run 27, sync-host-decision = Run 28, physical-android (D-8, SL-0243 validation event) = Run 29 or device day, date-gated. Run 26's "next:" line (physical-android) is superseded by this reorder. Plan text not amended. Ledger tail SL-0245 confirmed by direct read at open; no delta emitted; next available SL-0246.
next:                Run 28 — sync-host-decision (desk; B9 Bedrock common.nix custody evidence). Runtime implementing run queued in 1a alongside/before CoreError mapping (A-O22). Run 29+ physical-android on device day (kickoff preserved, Run 26 handoff section 7b).
```

```
run:                 28
run_type:            sync-host-decision
platform:            n/a (desk decision; container sentinel reads only)
started_at:          2026-09-21T00:36:00.000Z
ended_at:            2026-09-21T00:45:00.000Z
elapsed_min:         9
clean_checkout:      n/a (no workspace touched)
commit:              null (no code lands this run; repo unchanged at 93ae49c)
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 (H6) subduction=21b2e6b8 (declared; not yet exercised)
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=main@90fe4a51(2026-09-17, no new commits; bare-clone refetch) S-3=#230 OPEN/DRAFT/unmerged (HTML read) S-4=kh0 S-5="0.5" # pre-alpha | S-12(a)=null (CHANGELOG 404 root + keyhive_core; #keyhive not read from container) S-12(b)=`2026-sept-updates` EXISTS (head 881226c9, unchanged), NOT merged (git merge-base --is-ancestor, decisive) S-13=no maintainer replies attached. NO STOP CONDITION FIRED. All values identical to Runs 26 and 27.
l14_note:            n/a
loe_note:            SYNC-HOST CUSTODY DECISION (plan section 1 1a item, section 6 sync-host-decision run): SELF-HOST — the first sync host the app syncs against in Phase 2 is operator-run, Bedrock-shape, staging (~4 GB) minimum-viable size. Grounds, citing the B9 metadata-custody fact per the plan's done-when: (1) DECISIVE — Caddy access logs on the Bedrock hosted default join peer ID <-> client IP for 14 days (Run 22 B9 read); self-hosting puts that join under operator custody and makes retention a deliberate deploy setting, not an inherited default; content is E2E Keyhive-encrypted either way — the decision is metadata custody, not confidentiality (LOE packet section 7). (2) Seed-file = host identity (B9 runbook line): self-host makes the sync layer's one identity-bearing artifact an operator-custodied key file, same custody class as cold admin keys. (3) Cost measured: B9 sized the minimum-viable shape; no unmeasured LOE added. Hosted relays not excluded from the later topology as untrusted peers; the ruling covers the FIRST host and the custody DEFAULT. Custody contract: seed backed up off-host under cold-key discipline; access-log retention set deliberately at deploy (14-day upstream figure is the ceiling to justify against); auth="open" hardening is a deploy-run decision, noted not ruled. NOT RULED: transport (WS host stays L-5 fallback per B9; QUIC-primary question stays with the Subduction integration run), deploy specifics (provider, retention number, auth posture), multi-host/federation, A-O38 rotation-key custody (1b item viii). VERDICT-BEARING: NO — build rule inside plan 1a / SL-0244 scope; plan pre-registered the decision with loe_note as its record (R9-R12 / Run 26 H6 / Run 27 precedent). OPEN ITEM flagged: custody posture as a spec claim is a candidate for the spec v0.1.5 cut list (OI-P0-3), not performed here. Evidence caveat (~): B9 facts taken from the Run 22 loe_note and LOE packet sections 7-8; the canonical docs/bedrock-common-nix-read.md was not re-read this session — it governs on any divergence. Ledger tail SL-0245 confirmed at session open; no delta; next available SL-0246.
next:                1a implementing runs (runtime ruling from Run 27; storage/CoreError per plan section 1). Run 29+ physical-android (D-8, SL-0243 validation event) on device day — kickoff preserved in Run 26 handoff section 7b. Deploy run for the self-host lands with Phase 2 Subduction work.
```

```
run:                 29
run_type:            host-build
platform:            host (container reproduction; canonical apply operator-side)
started_at:          2026-09-21T01:44:00.000Z
ended_at:            2026-09-22T00:20:00.000Z (operator apply close; in-container close 2026-09-21T01:49:10Z)
elapsed_min:         5 in-container; apply leg same evening EST
clean_checkout:      yes (fresh container clone of origin 4181da6 + reproduced H6 pin; canonical target main=93ae49c)
commit:              edb2f0e. batch-applied 2026-09-21: canonical was at pre-Run-29 state; delivered files hash-verified against session CHECKSUMS before drop-in; 12/0/2 on canonical post-apply.
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 (H6, cargo tree verified before run work) samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 (declared) tokio=1.53.1 uniffi=0.32.1
outcome:             pass (container legs; network legs null — see loe_note)
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=main@90fe4a51(no new commits; bare-clone rev-parse) S-3=#230 OPEN/DRAFT/unmerged (HTML fallback; API rate-limited, plan section 3 pattern) S-4=kh0 (carriage.rs:26) S-5="0.5" # pre-alpha | S-12(a)=null (CHANGELOG 404 root + keyhive_core; #keyhive not read from container) S-12(b)=`2026-sept-updates` EXISTS (head 881226c9, unchanged), NOT merged (git merge-base --is-ancestor, decisive) S-13=no maintainer replies attached. NO STOP CONDITION FIRED. All values identical to Runs 26/27/28.
l14_note:            n/a
loe_note:            RUN 27 RULING IMPLEMENTED (built, not re-argued): new src/runtime.rs — single multi-thread tokio runtime in a OnceLock, pub(crate) rt() accessor, new_multi_thread().enable_all() defaults, worker tuning default, no shutdown path, lazy construction as backstop (explicit init lands in Run 30 with storage-open per the lifecycle contract); resolve_pds_blocking's per-call Runtime construction replaced with rt().block_on(...), FFI signature unchanged. DONE-WHEN (Run 27 wording): (1) grep -rn "Runtime::new\|new_multi_thread" core/crates/lfs_core/src/ -> exactly ONE hit, runtime.rs:19, inside the runtime module — PASS; (2) resolve_pds_blocking contains no runtime construction — PASS (0 hits); (3) cargo make test-network — container NULL (plc.directory outside egress allowlist, Phase 0 precedent; observed error is Network("http client error ... plc.directory"), i.e. the request left through the SHARED runtime with no construction failure) — OPERATOR-SIDE RE-RUN REQUIRED at canonical apply; (4) two-consecutive-resolvePds-in-one-process — reuse leg PASS container-side (rt_is_shared_across_consecutive_calls: identical &'static Runtime across consecutive calls, work executes on both), live leg added as #[ignore] test resolve_pds_twice_in_one_process joining test-network, NULL in-container, operator-side; (5) regenerated Kotlin binding signatures vs the Run 26 workspace (uniffi-bindgen library mode, host .so per Makefile Run 13 note) — ZERO DIFF, PASS. Host suite 5 passed / 2 ignored. Toolchain: Ubuntu-packaged cargo/rustc 1.91.1 (Run 26 container match; rustup outside egress). H6 gate note: pin edit preceded the reproduction build; tree verification displayed after that build completed — pin in effect for every compile (ordering noted, no effect). NUMBERING: this is Run 29 per the kickoff rule (Runs 27–28 tail, no device run landed; verify against canonical log at append — the attached log copy ends at Run 26 and the Run 27/28 entries were presumed appended operator-side, ~).
next:                Run 30 — storage (same session): Phase 1 SQLite behind the Storage trait, stub removed, init_core entry point.
```

```
run:                 30
run_type:            storage
platform:            host (container reproduction; canonical apply operator-side)
started_at:          2026-09-21T01:49:30.000Z
ended_at:            2026-09-22T00:20:00.000Z (operator apply close; in-container close 2026-09-21T01:52:45Z)
elapsed_min:         3 in-container; apply leg same evening EST
clean_checkout:      yes (same session workspace, Run 29 state as base)
commit:              edb2f0e. batch-applied 2026-09-21: canonical was at pre-Run-29 state; delivered files hash-verified against session CHECKSUMS before drop-in; 12/0/2 on canonical post-apply.
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 (H6) rusqlite=0.40.2 (bundled, plan section 2) tokio=1.53.1 uniffi=0.32.1 samod=0.14.0 autosurgeon=0.14.0
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      Re-read from source at run open (~01:49:34Z): S-1=0.5.0 S-2=main@90fe4a51 (refetch) S-3=#230 OPEN/DRAFT/unmerged (HTML) S-4=kh0 S-5="0.5" # pre-alpha | S-12(a)=null (CHANGELOG 404 both) S-12(b)=`2026-sept-updates` head 881226c9, NOT merged (merge-base --is-ancestor false) S-13=no replies attached. NO STOP CONDITION FIRED. All values identical to Run 29 minutes earlier.
l14_note:            n/a
loe_note:            PHASE 1 STORAGE (plan section 1 1a item): Phase 0 stub REMOVED; SQLite storage implements the UNCHANGED DocStore trait (the seam the plan keeps). Schema docs(id TEXT PK, format TEXT NOT NULL, bytes BLOB NOT NULL) — app-owned format tag column present from day one so H1's grep test has its target at 1b open; 1a's single persisted format stamped adapter-side as FORMAT_AUTOMERGE_SAVE_V1 = "automerge.save.v1" (tag value chosen this run, ~; 1b tags unruled; rows never re-tagged). Additive migration: Phase 0-stub dbs (no format column) gain the column via ALTER TABLE on open, bytes untouched — existing device/emulator dbs keep working (test-proven). INIT ENTRY POINT — naming decided this run: init_core (Kotlin initCore(dbPath): Core); #[uniffi::export] free fn returning Arc<Core>; touches rt() (explicit-at-startup per Run 27 lifecycle contract) then opens/creates the db at the shell-provided path; blocking, shells dispatch off-main (shell wiring is A-O22, NOT this session). Core::new constructor RETAINED for Phase 0 shells until A-O22 migrates them — recorded as an A-O22 finding, not ruled. DONE-WHEN (plan section 1 + kickoff): INSERT/SELECT roundtrip under the trait PASS (format tag verified on row); stub removed PASS; existing lfs_core suite PASS (8 passed / 2 ignored network); db file created at the init path PASS (init_core_creates_db_at_path_and_inits_runtime). Bindings: diff vs Run 29 surface is exactly the initCore addition; every Phase 0 signature unchanged. A-O22 FINDINGS (recorded, not ruled): (1) Core::new retirement timing once shells migrate to initCore; (2) DocStore trait error type is still rusqlite::Result — whether it migrates to a storage-neutral error is a CoreError-seam question for A-O22; kept unchanged here per the kickoff's "existing Storage trait". H1 HOUSEKEEPING (pre-existing, untouched): lib.rs keyhive_core mentions (pin-link fn, pins() string) will hit H1's grep when 1b opens — flagged for the 1b entry housekeeping list.
next:                A-O22 — CoreError-mapping / shell-integration run (next 1a item): wire initCore into both shells off-main; then remaining 1a items per plan section 1. Run 31+ physical-android (D-8, SL-0243) on device day — kickoff preserved in Run 26 handoff section 7b (its "<N> = next sequential" now resolves against Runs 29–30).
```

```
run:                 31
run_type:            shell-integration
platform:            container reproduction + shell wiring applied 2026-09-21 (macOS; Xcode + Gradle compiles)
started_at:          2026-09-21T02:20:00.000Z
ended_at:            2026-09-22T00:20:00.000Z (operator apply close; in-container close 2026-09-21T03:10Z approx)
elapsed_min:         session ~50 in-container; apply leg same-day, ~2026-09-21 evening EST
clean_checkout:      yes (fresh container; reproduction scaffold workspace)
commit:              edb2f0e (core, batch with Runs 29-30/32) / 0cfe677 (shell legs). batch-applied 2026-09-21: canonical was at pre-Run-29 state; delivered files hash-verified against session CHECKSUMS before drop-in; 12/0/2 on canonical post-apply (edb2f0e).
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 (H6) samod=0.14.0 autosurgeon=0.14.0 subduction=21b2e6b8 atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 rusqlite=0.40.2
outcome:             pass (in-container legs + operator legs (a)-(c); leg (d) round-trip pending)
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=main@90fe4a51(2026-09-17, no new commits) S-3=#230 Open/DRAFT/unmerged(33 commits) S-4=kh0 S-5="0.5" | S-12(a)=null(root+keyhive_core CHANGELOG 404; #keyhive not read) S-12(b)=2026-sept-updates EXISTS(head 881226c9, unchanged since Run 23), NOT merged S-13=no replies attached. NO STOP CONDITION FIRED. All values identical to Run 26. [Session-time state; S-12(b) fired later the same day — see Run 32.]
l14_note:            n/a
loe_note:            A-O22 run. CoreError mapping CONFIRMED (not repaired) by in-container bindings generation on the delivered Runs 29-30 state, uniffi-bindgen 0.32.1 library mode: Swift CoreError IS public — modifier emitted on its own line ("public\n enum CoreError: Swift.Error, Equatable, Hashable, LocalizedError"), 4 cases; Kotlin sealed class CoreException, 4 subclasses; one type per class, one-to-one both surfaces; no rusqlite/network type leak. Run 18's "not public" does not reproduce at the generator level; most plausible cause a single-line grep missing the line-broken modifier (~); CLOSED at apply: typed catch (catch let e as CoreError / catch CoreException) compiled on both shells 2026-09-21 — A-S31-5 discharged, grep-miss explanation stands. Finding (a) SETTLED: Core::new retired from FFI this run (constructor removed from the export block, pub(crate) internal; init_core sole FFI init path; both shells migrated at apply — shell-wide constructor grep = 0 hits, initCore present in both). Finding (b) SETTLED: DocStore keeps rusqlite::Result, mapped at the CoreError boundary; revisit trigger = second backend or 1b typed storage-failure classes; no spec claim touched. One file changed (lib.rs); suite 8 passed/2 ignored before AND after in-container; regenerated-bindings diff = constructor removal + doc updates + init_core metadata checksum (31484->19270), nothing else. Baseline reproduction independently corroborated A-S29-2 (all four delivered files hash-match; H6 gate line identical to Run 26; 8/2 on Ubuntu cargo/rustc 1.91.1). Shell wiring applied 2026-09-21 (0cfe677) from a patch cut against the canonical shell files (uploaded to the adoption session; record §7 shapes with real anchors): iOS Task.detached -> initCore -> MainActor publish (ContentView.swift; async doc-state publish added); Android lifecycleScope Dispatchers.IO -> initCore -> Main publish (MainActivity.kt; doc state hoisted to MutableState + remember keyed so late init recomposes). Bindings distributions regenerated on canonical via cargo-make (android-so, bindgen-kotlin, swift-xcframework); artifacts untracked by repo policy. Both shells compile (Xcode Build Succeeded; Gradle BUILD SUCCESSFUL). Session crossed one container tool-ceiling and resumed from its own G13 checkpoint; container state survived (HO-31-1, harness observation, no defect).
next:                Leg (d) round-trip both shells through initCore (pending, device/simulator); then local-UI run per Run 32 handoff; physical-android still date-gated (SL-0243).
```

```
run:                 32
run_type:            host-build
platform:            container (Ubuntu 24.04.4); applied to canonical 2026-09-21 (macOS)
started_at:          2026-09-21T16:59:00.000Z
ended_at:            2026-09-22T00:20:00.000Z (operator apply close; in-container close 2026-09-21T18:10Z approx)
elapsed_min:         71 in-container (includes one tool-ceiling interruption + operator-resume; in-container work ~55); apply leg same evening EST
clean_checkout:      yes (fresh container; reproduction workspace, P2 warm start)
commit:              edb2f0e. batch-applied 2026-09-21: canonical was at pre-Run-29 state; delivered files hash-verified against session CHECKSUMS before drop-in; 12/0/2 on canonical post-apply.
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 samod=0.14.0 autosurgeon=0.14.0 subduction=off atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1
outcome:             pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=main@8c631cdc(2026-09-21, MOVED past pin) S-3=#230 OPEN/DRAFT/unmerged/34-commits S-4=kh0 S-5="0.5" | S-12(a)=404 both, in-tree "0.5.0" S-12(b)=FIRED — 2026-sept-updates MERGED as PR #229 (9675a0fe, 2026-09-21T16:16:11Z, branch deleted) S-13=none. STOP CONDITION S-12(b) FIRED -> 1b HALTED; 1a run continued per plan discipline. H6 pin 90fe4a51 intact; gate line identical to Run 26. PR #237 merged same minute (8c631cdc), recorded. [Post-session footnote: PR #229's head branch was force-pushed 881226c9 -> 347cd57 ~26 min pre-merge; the head tracked since Run 23 does not appear in main's history. Content read (post-close): infra + docs modernization — CI/Nix/typos/wasm-bodge + threat_model.md, glossary.md, convergent_capabilities.md rewrite, ciphersuite.md corrections, BeeKEM no-FS note; no core-semantics changes; "more stacked on top" announced.]
l14_note:            n/a
loe_note:            Doc model + membership counter (plan §1 1a rows 2-3). Baseline reproduced first: Run 31 source hash-verified, H6 gate ok, 8/2 on Ubuntu cargo/rustc 1.91.1 (third independent corroboration of the delivered-state chain). docs.rs NEW: ProfileDoc/PingsDoc/ThreadsDoc from socialpings spec v1.0 data model, 1:1, no divergence; channels/assets maps plan-scoped out; string vocabulary carriage (build rule); HelloDoc KEPT (backs shell-wired KV surface). Counter: membership_version/record_membership_event on Core, in-memory per-group, interface only. Suite 12/0/2 in-container AND on canonical at apply (fourth reproduction, first outside a container; A-S32-2 and the A-S31-2/A-S29-2 chain discharged by hash match + green gate). Bindings regenerated (uniffi-bindgen 0.32.1 library mode): diff vs Run 31 = 146 additions / 0 deletions / 0 modifications, all counter; modulemap identical. Doc structs deliberately not FFI-exported (binding-diff discipline; typed surface deferred to local-UI run). One tool-ceiling interruption, G13 checkpoint recovery, container state intact (HO-32-1; second HO-31-1-pattern occurrence — P1/P2 case strengthened, adoption-gap evidence adds a third data point; still unruled). Preserve tarball produced (P1 shape, unruled). Delta SL-0246 drafted ([SENTINEL-STOP], confidence confirmed) — appended operator-side after tail verify.
next:                Run 32 apply DONE (edb2f0e). S-12(b) consequence ruling (1b/T4 re-scoping vs merged #229) — operator. Local-UI shell run next; physical-android date-gated.
```

```
run:                 33
run_type:            host-build
platform:            container (Ubuntu 24.04); applied to canonical 2026-09-22 (macOS; Xcode + Gradle compiles; iOS physical device + Android emulator legs)
started_at:          2026-09-22T03:02:00.000Z
ended_at:            2026-09-22T04:15:00.000Z (operator apply close; in-container close 2026-09-22T03:35Z approx)
elapsed_min:         33 in-container (includes one tool-ceiling interruption + operator-resume, HO-33-1); apply leg same evening EST, ~25
clean_checkout:      yes (fresh container; reproduction workspace, P2 warm start from run32 preserve)
commit:              a641eea (applied 2026-09-22; built against 14cc683). Canonical pre-hashed 5/5 = Run 32 state before drop-in; delivered files hash-verified post drop-in; 16/0/2 on canonical post-apply; bindings regenerated on canonical via cargo-make (bindgen-kotlin, swift-xcframework): diff vs Run 32 = 0 removed / +869 Swift / +814 Kotlin — byte-parity with the in-container run.
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 samod=0.14.0 autosurgeon=0.14.0 subduction=off atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 rusqlite=0.40.2 (scaffold Cargo.lock byte-identical to Run 31/32 preserve)
outcome:             pass (in-container + operator legs (a)-(c) both shells)
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=main@8c631cdc(unchanged since Run 32 read; no follow-on merges from the #229 stack yet) S-3=#230 OPEN/DRAFT/unmerged/34-commits S-4=kh0 (carriage.rs L26) S-5="0.5" (workspace Cargo.toml L59) | S-12(a)=404 both, in-tree "0.5.0" S-12(b)=FIRED-state persists (PR #229 merge commit 9675a0fe on main; 1b halted per operator ruling behind the T4 fence) S-13=none. NO NEW STOP CONDITION. H6 pin 90fe4a51 intact; gate line identical to Run 26. GitHub REST rate-limited from first call; atom/HTML/raw reads throughout.
l14_note:            n/a
loe_note:            Local-UI wiring (plan §1 1a: "Local UI on both shells — displays doc content from SQLite; no sync"). Open checks first: P3 apply-confirmation PASSED — attached Run 32 source = Run 32 CHECKSUMS (5/5 incl. docs.rs) = canonical main@14cc683 core/crates/lfs_core/src/ (5/5 byte-match by raw fetch); baseline 12/0/2 on Ubuntu cargo/rustc 1.91.1 (fifth independent reproduction of the delivered-state chain). Task 2 typed-FFI surface — BUILD RULE: typed uniffi records mirroring docs.rs 1:1; the docs.rs structs themselves derive uniffi::Record (one schema, no mirror types, no JSON copy per shell); display requirement decided it (typed lists, compile-time field access); JSON-over-FFI and KV-with-typed-accessors rejected (record §3). Consequence recorded: docs.rs maps BTreeMap->HashMap (uniffi 0.32.1 lowers HashMap only; autosurgeon supports both; Automerge maps unordered; shells sort keys). lib.rs: DocKind enum {Profile,Pings,Threads} + open_typed_doc(id,kind) (default of kind reconciled into a NEW doc; existing docs loaded as stored) + get_/put_ profile/pings/threads. HelloDoc + KV surface KEPT (Run 32 rule); no retirements -> no shell-migration step owed. Suite 16/0/2 (12 baseline intact + 4 new through init_core + exported methods incl. core restart on the same SQLite file and KV/typed coexistence). Bindings regenerated (uniffi-bindgen 0.32.1 library mode): diff vs bindings-run32-state = Swift +869 / Kotlin +814 / header +77 / 0 deletions / 0 modifications; modulemap identical; every hunk is the new surface (7 methods, 9 records, DocKind). Spec check: shape unchanged vs socialpings v1.0; one scope deferral recorded not adapted — spec's cleanup-observer-on-load for expired pings is NOT implemented (expired pings stay in the doc; shells filter at display). Task 3 shell wiring delivered against the attached post-Run-31 canonical (full files + run33-shell-wiring.patch, -p1 dry-run clean): typed docs opened on the existing off-main init path, published on main; Profile/Pings/Threads sections per spec display semantics (TTL indicator; expired hidden display-side; threads by last activity); "Seed demo" (put*+save) so kill->relaunch shows SQLite content; identifiers grep-verified against regenerated bindings. Two files changed in core (lib.rs, docs.rs); resolve/runtime/storage byte-identical to Run 32. HO-33-1: one tool-ceiling interruption after the preserve step, G13 checkpoint recovery on operator "Continue", container state intact (third HO-31-1-pattern occurrence; P1/P2/P3 now standing rules — the P1 tarball was already on disk). Preserve tarball produced (P1). RUN 31 LEG (d) CLOSED 2026-09-21 (verification note run31-legd-verification_2026-09-21.md): round-trip through initCore verified on both shells — iOS physical device (iOS 26.6.1, ios-arm64 slice; "openDoc ok, text=13 chars") and Android Pixel 9 emulator API 37.2 ("openDoc(\"note\") ok; text=13 chars"); no CoreError/CoreException at the boundary; core-reported pin footer byte-identical across both FFI surfaces (keyhive_core=0.5.0 automerge=0.12 samod=0.14 autosurgeon=0.14 atrium-api=0.25 subduction=off) — first on-device run of the Run 32 core; not verdict-bearing. Run 31's own entry keeps its "pending" (log not rewritten). RUN 33 LEGS (a)-(c) PASS 2026-09-22 (before append; recorded here): (a) Xcode Build Succeeded — A-S33-4 discharged (shell.oauth() lives in OAuthSmoke.swift); (b) Gradle BUILD SUCCESSFUL, installDebug on Pixel 9 AVD API 37.2 — A-S33-5/A-S33-6 discharged, no fallbacks (HorizontalDivider + java.time.Instant compiled); (c) both shells: first launch "docs ok: profile=(empty) trust=0 pings=0/0ch threads=0/0msg" -> Seed demo -> Profile Jedi @jedi / trust graph 2 / pings header "#ch-local-first — 2 active / 3 stored" (seeded expired ping hidden display-side) / threads contact-1 2 messages -> kill (iOS app-switcher swipe; Android am force-stop) -> cold relaunch -> "docs ok: profile=@jedi trust=2 pings=3/1ch threads=1/2msg" with content restored from SQLite, no seeded suffix; no CoreError/CoreException on either runtime; pin footer identical on both (keyhive_core=0.5.0 ... subduction=off). iOS on physical device (iOS 26.6.1, ios-arm64; identifier withheld per R-P0-17); Android Pixel 9 emulator API 37.2. Observations, not defects: seeded byte counts differ per platform (iOS 756+598+438, Android 650+640+460) — Kotlin Instant.toString() emits fractional seconds, Swift ISO8601DateFormatter does not; same shape, different timestamp string lengths; TTL label differs by one unit at the exact integer-second boundary at seed time (7d vs 6d), resolves within a second. Gradle strip warning on liblfs_core.so unchanged from Runs 15-31. Plan §1 1a row "Local UI on both shells — displays doc content from SQLite; no sync" DONE. Not verdict-bearing; no ledger delta.
next:                Run 33 apply DONE (a641eea). Run 34 shape ruling (operator; default: 1a hardening — ping cleanup-on-load at the core, closing the Run 33 §4.4 deferral). S-12(b) follow-on stack: watch at next open (S-2 movement). Physical-android date-gated (D-8, SL-0243) — next number when the device lands. Community asks unchanged (Discord §5; Onomancy Ask 2 / OI-M11).
```

```
run:                 34
run_type:            consumer-evidence-note
platform:            n/a (writing session, Session Harness v0.2 Mode 1; macOS operator-side publish; GitHub source reads)
started_at:          2026-09-22T15:00:00Z (approx; live sentinel read ~11:00 EDT)
ended_at:            2026-09-22T17:15:00Z (approx; note corrections landed at 4324984 after ~13:15 EDT repo hygiene)
elapsed_min:         ~135 wall (writing session + post-close publish, corrections, hygiene; approx)
clean_checkout:      n/a
commit:              4324984 (docs only; code unchanged at a641eea). Chain: e0c2590 (log) -> 3a83905 (note published, docs/consumer-evidence-note-v0_2026-09-22.md, byte-identical to delivered) -> 1225a2a (header line stripped from public copy) -> 646741c (README replaced, LICENSE Apache-2.0 added; stale note copy caught by verification) -> 4324984 (note corrections +2/-2). Public copy = delivered note minus header line, verified by raw fetch.
pins:                unchanged from Run 33: keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 samod=0.14.0 autosurgeon=0.14.0 subduction=off atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 rusqlite=0.40.2
outcome:             published
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 S-2=main@e2789c93(2026-09-22T13:01:50Z, #228 "Fix handling of concurrent CGKA Adds" — MOVED past Run 33 read 8c631cdc; first core-semantics commit past the pin; CGKA change, not an encoding change; content not read; read before pinning against anything) S-3=#230 OPEN/DRAFT/unmerged/34-commits(head 62a3d3ac) S-4=kh0 (onomancy carriage.rs L26 at main e29ca5df) S-5="0.5" (onomancy workspace Cargo.toml L59) | S-12(a)=null S-12(b)=FIRED-state persists (PR #229 merge 9675a0fe; per plan v0.1.1 now a WATCH, not a stop condition — SL-0247) S-13=Discord #keyhive post 1 LIVE 2026-09-22T16:32Z, no reply at close. NO STOP CONDITION FIRED (S-3/S-4 unchanged). H6 pin 90fe4a51 intact; keyhive_core/Cargo.toml L4 at pin = "0.5.0". GitHub REST rate-limited after first PR call; heads from atom feeds.
l14_note:            n/a
loe_note:            CONSUMER EVIDENCE NOTE v0 (plan v0.1.1 §1 1b row, Addition A re-scoped "v0 at 1a; v1 at 1b close" — the v0.1 text covered only a 1b-close deliverable; registration was held (narrower reading, evidence-note record §5) until the v0.1.1 cut; this entry discharges that owed item). Four sections, per-claim ✓/~ tags, every upstream statement scoped to file + revision, no maintainer-intent claims: (a) existence proof — iOS + Android on keyhive_core 90fe4a51, four targets, typed UniFFI 0.32.1, SQLite persistence, kill/relaunch both shells (Runs 25–33, a641eea), all ✓ from log/record; (b) storage adapter re-encode path from source (DocStore trait, SqliteStore, FORMAT_AUTOMERGE_SAVE_V1, additive schema migration; carriage.rs kh0 envelope at onomancy e29ca5df), ✓ where file+function named; (c) decoder-retained vs decoder-absent migration cost, ~ throughout (no grant issued at a641eea); (d) the one question for Onomancy Ask 2. Register: publication (Lexicon v2.6 R-1 terms avoided). Companion: community-asks-rework_2026-09-22.md (Discord #1 posted as post 1; Onomancy Ask 2 HELD — trigger = reply to post 1 or ~1 week silence; Discord #2/#3 queued). Two post-close corrections (record §6a): (1) "default-features = false" on the keyhive_core dependency was carried from the kickoff without a manifest read — core/crates/lfs_core/Cargo.toml L25 has no features clause; corrected to "no optional features enabled" (effect nil: keyhive_core default = [] at 90fe4a51); (2) 1b described as "paused when upstream merged" — inaccurate attribution; 1b halted on the plan's own S-12(b) (SL-0246); corrected to "not yet started". Both landed at 4324984. Repo hygiene same day (record §7b): About updated, 10 topics added; GitHub Pages found enabled (main//docs, 13 deployments — Phase 0 leftover, not a deliberate publication) and UNPUBLISHED; docs/ public via repo only, Discord link unaffected; LICENSE (Apache-2.0) added — was absent despite license field in core/Cargo.toml. Source-version finding (record §6b): the session read spec v0.1.2 / memo v0.1.1 while v0.1.4 / v0.1.2 were current; Session 0 (2026-09-22 evening) byte-compared L-12, L-16, §10 items 8/13/14, §5.2 floor and memo §6 across versions — identical; effect on the note nil. This entry also carries the two "log line at next run open" items from record §7a/§7b (Discord permalink; Pages unpublish) so they are dated with the event rather than with the next build. Not verdict-bearing on its own; the 1b reopen it enabled is SL-0247 (Session 0). 
next:                Ledger append SL-0247 (verify tail). Run 35 — host-build, 1a hardening: ping cleanup-on-load at the core (handoff r2 §8 kickoff; its "verify 34 as next" line now reads 35). Then Run 36 — 1b entry, opens only on a clean S-3/S-4 read at open; read S-2 e2789c93 content before pinning against anything. Physical-android date-gated (D-8, SL-0243). S-13 watch on post 1; Onomancy Ask 2 held on trigger. v1 of the note at 1b close (Run 40 exit).
```

```
run:                 35
run_type:            host-build
platform:            container (Ubuntu 24.04, reproduction scaffold, P2 warm start from run33 preserve); apply leg on canonical (macOS) — see apply guide
started_at:          2026-09-22T21:34:00Z
ended_at:            2026-09-22T22:05:00Z (in-container close, approx; operator apply close: 2026-09-23T01:30:00Z)
elapsed_min:         ~31 in-container (no tool-ceiling interruption; G13 checkpoint emitted after the preserve step per HO-31-1/32-1/33-1 pattern); apply leg: pass
clean_checkout:      yes (fresh container; scaffold Cargo.toml + Cargo.lock from run33-container-preserve, both byte-identical to the Run 31/32/33 preserve)
commit:              ac914bd (applied 2026-09-23; built against 9d5def2 = a641eea code). Canonical pre-hashed 5/5 = Run 33 state before drop-in; delivered lib.rs 2849d0a8… docs.rs b3e18425…; resolve/runtime/storage byte-identical to Run 33.
pins:                unchanged from Run 33: keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 samod=0.14.0 autosurgeon=0.14.0 subduction=off atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 rusqlite=0.40.2 (scaffold manifest + lockfile byte-identical to Run 33 preserve; no dependency added — RFC 3339 parsing is in-core)
outcome:             pass (in-container); apply leg: pass
defects:             0
defect_classes:      n/a
sentinel_state:      S-1=0.5.0 (crates.io max/stable, updated 2026-06-26) S-2=main@e2789c93(2026-09-22T13:01:50Z, #228 "Fix handling of concurrent CGKA Adds" — UNCHANGED since the Run 34 read; no follow-on merges; CONTENT READ this run: 9 files +1033/−400, product-code hunks confined to beekem/src/{cgka,lib,tree}.rs, remainder test tooling (beekem/src/test_utils.rs new, two new beekem test files, keyhive_core/tests/beekem_concurrent.rs deleted/moved) + .cargo/mutants.toml + beekem/Cargo.toml dev-dep; touches encoding: NO — no serde/derive/struct/enum/encode/decode additions in product code (grep-level read, not a semantic read of tree.rs); not pinned against) S-3=#230 "Keyline crate" DRAFT/unmerged/34-commits, mergedTime null S-4=kh0 (onomancy main e29ca5df, onomancy_keyhive/src/carriage.rs L26) S-5="0.5" (onomancy workspace Cargo.toml L59) | S-12(a)=null (root + keyhive_core CHANGELOG 404; in-tree keyhive_core/Cargo.toml "0.5.0" at main and at pin) S-12(b)=FIRED-state persists (PR #229 merge 9675a0fe on main.atom; WATCH per plan v0.1.1 / SL-0247, no halt) S-13=NOT READ in-container (discord.com outside the declared egress set) — operator read at append: no reply (read 2026-09-23 ~01:25Z, ✓). NO STOP CONDITION FIRED (S-3/S-4 unchanged). H6 pin 90fe4a51 intact; cargo tree gate line identical to Run 26. GitHub REST not used; atom/HTML/raw/codeload reads throughout.
l14_note:            n/a
loe_note:            1a HARDENING — expired-ping cleanup-on-load at the core (plan v0.1.1 §1 1a last row; closes the Run 33 §4.4 spec deferral). Open checks first: P3 apply-confirmation PASSED — attached Run 33 source = Run 33 CHECKSUMS.txt (5/5) = canonical main@a641eea core/crates/lfs_core/src/ (5/5 by raw fetch) = main@9d5def2 (5/5; code unchanged); run33 preserve tarball 11/11 vs its own sheet; baseline 16/0/2 on Ubuntu cargo/rustc 1.91.1 (sixth independent reproduction of the delivered-state chain). Open-block gap: the preserve tarball and CHECKSUMS.txt were not in the first attach set; a substitution (canonical core/Cargo.lock + in-container regenerated a641eea bindings as baseline) was proposed, considered, and NOT TAKEN — operator attached both and P2/P3 ran as written. Task 2 — BUILD RULE (flagged, recorded): the kickoff's "a step in open_typed_doc" + "clock injected as a parameter" + "binding diff additions only" cannot all hold on one signature; resolved by a parallel entry point: NEW FFI method Core::open_typed_doc_at(id, kind, now: String) runs the shared typed-open path and, for kind=Pings, deletes every ping whose expires_at parses as RFC 3339 and is STRICTLY earlier than now; an unparseable expires_at is RETAINED (never silently dropped); an unparseable now is an ERROR — NEW CoreError::InvalidTimestamp (additive variant) — never a silent no-op; other kinds validate the clock and are untouched. open_typed_doc(id, kind) keeps its Run 33 signature and behaviour (no clock, no cleanup — library code never reads the wall). Removal uses Automerge list deletes walked from the end of each channel list (not a whole-document reconcile) so the op history records exactly the removals; persistence follows the existing save contract (cleanup reaches SQLite on the next save; test covers core restart). RFC 3339 parser in-core (docs.rs; Hinnant days_from_civil; Z or numeric offset; fractional seconds to ns, truncating beyond; leap second folded) — no chrono/time dependency, so the scaffold and canonical manifests stay pin-for-pin; covers the two shapes Run 33 leg (c) observed (Kotlin Instant.toString fractional, Swift ISO8601DateFormatter whole-second). Suite 23/0/2 (12+4 baseline intact + 7 new: expired_ping_removed_on_load_active_and_unparseable_retained, cleanup_follows_injected_clock_not_wall, cleanup_persists_on_save_and_survives_core_restart, invalid_clock_is_an_error_and_other_kinds_are_untouched, docs::expiry_tests x3). Build 0 warnings. Bindings regenerated (uniffi-bindgen 0.32.1 library mode, workspace cwd): diff vs bindings-run33-state = Swift +72 / Kotlin +92 / header +11 / modulemap identical / 0 deletions / 0 modifications — every hunk is openTypedDocAt or InvalidTimestamp (the PingsDoc docstring Run 33 wrote is kept verbatim so no generated line is removed). SHELL FILTER: KEPT (belt-and-braces); no shell change ships; shells stay on open_typed_doc until they adopt _at (two-line edit per shell, next shell run — open item). Spec check: shape unchanged vs socialpings v1.0; the spec's "cleanup observer removes expired entries on document load" is now satisfied at the core for callers supplying the clock; "expire at expiresAt" read as exclusive (boundary instant active) — ~ against the shells' filter sense, not re-read. Preserve tarball produced (P1): run35-container-preserve_2026-09-22.tar.gz sha 31f11f67… (scaffold + bindings-run35-state + bindings-run33-state carried). Delivered: lib.rs/docs.rs changed (+214/−15, +201/−1 incl. the comment-only rewrites), run35-cleanup-on-load.patch (-p1 dry-run clean against 9d5def2), CHECKSUMS.txt with (pre) baselines, apply guide + gated appender (F-33-1/2/4 shape). Not verdict-bearing; no ledger delta (build execution under plan 1a / SL-0244 / SL-0247).
next:                Run 35 apply (operator; guide). Run 36 — 1b entry, opens ONLY on a clean S-3/S-4 read at its open (SL-0247); attach spec v0.1.4 + memo v0.1.2 + plan v0.1.1 + this run's preserve (bindings-run35-state = next binding-diff baseline). Shell adoption of open_typed_doc_at at the next shell run. S-3 demotion candidate (post-Run-37) carried as an open item in the Run 35 record until the plan's next Lightweight touch. Physical-android date-gated (D-8, SL-0243). S-13 watch on post 1; Onomancy Ask 2 held on trigger.
```

```
run:                 36
run_type:            host-build
platform:            container (Ubuntu 24.04, reproduction scaffold, P2 warm start from run35 preserve); apply leg on canonical (macOS) — see apply guide
started_at:          2026-09-23T01:48:00Z
ended_at:            2026-09-23T02:35:00Z (in-container close, approx; operator apply close: 2026-09-23T04:10:00Z)
elapsed_min:         ~47 in-container (no tool-ceiling interruption; G13 checkpoint emitted after the preserve step per HO-31-1/32-1/33-1/35 pattern); apply leg: 40
clean_checkout:      yes (fresh container; scaffold Cargo.toml + Cargo.lock from run35-container-preserve, both byte-identical to the Run 31–35 preserve chain)
commit:              a8e94f9 (applied 2026-09-23; built against ac914bd = Run 35 code, main@34a0825). Canonical pre-hashed 5/5 = Run 35 state before drop-in; delivered lib.rs be26883d… storage.rs a4b391c0… policy.rs db945d6d… (NEW); docs/resolve/runtime byte-identical to Run 35.
pins:                unchanged from Run 35: keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 samod=0.14.0 autosurgeon=0.14.0 subduction=off atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 rusqlite=0.40.2 (scaffold manifest + lockfile byte-identical to Run 35 preserve, lock sha bc42e348…; no dependency added — tls-decision changed no pin)
outcome:             pass (in-container); apply leg: pass
defects:             0
defect_classes:      n/a
sentinel_state:      READ 2026-09-23T01:58Z. ENTRY GATE (plan §3 v0.1.1 / SL-0247): CLEAR — S-3=#230 "Keyline crate" DRAFT/unmerged/34-commits, mergedTime null; S-4=kh0 (onomancy main e29ca5df, onomancy_keyhive/src/carriage.rs L26) — SL-0247's reopen TAKES EFFECT; this is the first 1b run. S-1=0.5.0 (crates.io max/stable, updated 2026-06-26) S-2=main@e2789c93 (2026-09-22T13:01:50Z, #228 — UNCHANGED since the Run 35 read; head identity only, no content re-read this run; not pinned against) S-5="0.5" (onomancy workspace Cargo.toml L59) | S-12(a)=null (root + keyhive_core CHANGELOG 404; in-tree keyhive_core/Cargo.toml "0.5.0" at main and at pin) S-12(b)=FIRED-state persists (PR #229 merge 9675a0fe on main.atom; branch 2026-sept-updates 404; WATCH per plan v0.1.1 / SL-0247, no halt) S-13=NOT READ in-container (discord.com 403 / outside the declared egress set) — operator read at append: no reply (read 2026-09-23 ~04:00Z, ✓). NO STOP CONDITION FIRED. H6 pin 90fe4a51 intact; cargo tree gate line identical to Run 26. GitHub REST not used; atom/HTML/raw/codeload + crates.io API reads throughout. PAGES: operator step 0 (2026-09-22 ~21:53 EDT, before this run opened) found Settings → Pages source still main/docs after the Run 34 unpublish (site had rebuilt on 34a0825); branch set to None and saved; "GitHub Pages is currently disabled" confirmed by screenshot — site 404 is now the durable state.
l14_note:            n/a
loe_note:            1b ENTRY — plan v0.1.1 §9 row 2 (tls-decision + H2 policy skeleton + H5 RootingLevel plumbing). Open checks first: P3 apply-confirmation PASSED — attached Run 35 source = Run 35 CHECKSUMS.txt (5/5) = canonical main@ac914bd core/crates/lfs_core/src/ (5/5 by raw fetch); attached log byte-identical to docs/ @ 34a0825; main head 34a0825; run35 preserve 11/11 vs its own sheet; baseline 23/0/2 on Ubuntu cargo/rustc 1.91.1 (eighth reproduction of the delivered-state chain). (i) TLS-DECISION (recorded choice, not a ruling; changes no pin — no stop): STAY on reqwest 0.12.28 rustls-tls + webpki roots for 1b; the platform verifier (rustls-platform-verifier / OS trust store) is NOT adopted at 1b entry. Evidence: Run 13 (Android binding-build: reqwest default-tls → native-tls → openssl-sys failed the aarch64-linux-android cross-build; swapped to rustls-tls + webpki roots at 8dc6dab; pre-decision (vi) opened), Run 15 (Android emulator network-probe: DID → DID doc → PDS resolved first try on webpki roots, no CoreException, "no finding lands on (vi)"), Run 19 (physical iPhone network-probe: same, no throw), Runs 31/33/35 shells launched on the same stack; plan §1 1a row "stay on rustls-tls + webpki roots" (Step 3 Addition B researched ruling). D-8 CAVEAT: physical Android has not run (date-gated, SL-0243) — this choice is provisional against that evidence; revisit condition = a certificate-verification failure on a physical Android device OR a 1b need for OS-managed trust (enterprise/MDM roots); either reopens (vi) as a pin-decision run (would add rustls-platform-verifier + Gradle setup, plan §7 upper-bound item). Handoff pointer "Run 26 evidence" reads as the device-run evidence above; Run 26 is the H6 pin-decision run and holds no TLS evidence (~, pointer likely stale). (ii) H2 POLICY MODULE SKELETON: NEW core/crates/lfs_core/src/policy.rs (161 lines), registered `pub mod policy;` in lib.rs. Docstring names CONSUMER POLICY and states no call into keyhive_core enforcement (none exists in the file). Rule 1 = DELEGATION_FLOOR: usize = 2 + check_delegation_floor(count) → Result<(), PolicyViolation::BelowDelegationFloor{have,floor}> (spec §5.2). Rule 2 = GrantLevel {Relay, Read, Edit, Admin} (Ord; a MIRROR of keyhive_core::access::Access at 90fe4a51 by source read this run — Relay < Read < Edit < Admin — not an import, so a core-side rename/bar move touches this one file) + GRANT_BAR = GrantLevel::Admin + check_grant_bar(granter) → GranterBelowBar{have,bar}. FloorStatus {Safe, WithdrawnPendingL12} + delegation_floor_status(RootingLevel) encodes spec L-12 / §5.2 as a lookup (Edit → Safe; Admin → WithdrawnPendingL12; FROST 2-of-3 share replacement is L-12's condition, not modelled). NO CALLERS, NO FFI EXPOSURE (Run 37 first calls the floor; Run 38 wires the bar; PolicyViolation → CoreError mapping ruled there). 4 unit tests. (iii) H5 ROOTINGLEVEL + CONFIG PLUMBING: lib.rs gains RootingLevel {Admin, Edit} (uniffi::Enum; the two levels memo §6 names — read, not invented; consequences quoted ~ per memo) with NO Default impl (cannot silently become a constant), and IdentityConfig {rooting_level: RootingLevel} (uniffi::Record) = the configuration path each shell supplies at ceremony time; held by no Core field, consulted by no entry point (Run 37 adds the ceremony entry point that accepts it). "Config path" read as the FFI record (~, kickoff wording). H1 HOUSEKEEPING (flagged Run 30 for 1b entry; done): the Phase 0 pin-link smoke keyhive_core_linked() + its test moved lib.rs → storage.rs (pub(crate)); the pins() label literal moved to storage::PIN_LABEL_KEYHIVE_CORE — pins() output BYTE-IDENTICAL to Run 35 (shell pins footer unchanged). Pre-existing, NOT changed: that label reads "keyhive_core=0.5.0" while H6 is git 90fe4a51 — display string to re-rule at Run 37 with the format tag (open item). DONE-WHEN: H1 grep (`grep -r 'keyhive' src/ --include='*.rs' | grep -v '_test\|//\|storage'`) returns EMPTY — stricter than "storage + policy only" (raw hits: storage.rs ×3 product lines, filtered by the pattern; policy.rs comment lines only). Suite 27/0/2 (23 baseline intact, incl. the relocated keyhive_pin_links now under storage::tests, + 4 policy tests: delegation_floor_is_two_and_refuses_below, grant_bar_is_admin_and_refuses_below, grant_levels_are_ordered_like_the_pinned_access_enum, floor_is_safe_under_edit_rooting_only). Build 0 warnings. Bindings regenerated (uniffi-bindgen 0.32.1 library mode, workspace cwd; CLI built as a standalone crate outside the scaffold so the lockfile stayed untouched): diff vs bindings-run35-state = Swift +111 / Kotlin +68 / header 0 / modulemap 0 / 0 deletions / 0 modifications — every hunk is RootingLevel or IdentityConfig; no new FFI function (header identical). No spec/memo divergence found; none adapted around. Preserve tarball produced (P1): run36-container-preserve_2026-09-23.tar.gz sha 02061b8e… (scaffold + bindings-run36-state + bindings-run35-state carried). Delivered: lib.rs (+33/−10), storage.rs (+19/−0), policy.rs (new), run36-1b-entry.patch (-p1 dry-run clean against ac914bd), six-file CHECKSUMS.txt with (pre) baselines, apply guide + gated appender (F-33-1/2/4 shape; Run 35 apply observations applied — elapsed_min in minutes). Not verdict-bearing; no ledger delta (build execution under plan §9 row 2 / SL-0247).
next:                Run 36 apply (operator; guide). Run 37 — identity ceremony (plan §9 row 3): ceremony entry point accepts IdentityConfig; records configured RootingLevel + policy::delegation_floor_status; cites L-12; two delegations via check_delegation_floor; first Keyhive bytes in SQLite under a new format tag (tag ruled in-run; re-rule storage::PIN_LABEL_KEYHIVE_CORE display string alongside); shell adoption of open_typed_doc_at folds in (first shell-touching run) unless split out; attach spec v0.1.4 + memo v0.1.2 + plan v0.1.1 + this run's preserve (bindings-run36-state = next binding-diff baseline). S-2 e2789c93 semantic read of beekem/tree.rs owed only if pinning past 90fe4a51 is proposed. S-3 demotion candidate (post-Run-37) carried. Physical-android date-gated (D-8, SL-0243) — reopens tls-decision (vi) only on a verification failure. S-13 watch on post 1; Onomancy Ask 2 held on trigger.
```

```
run:                 37
run_type:            host-build
platform:            container (Ubuntu 24.04, reproduction scaffold, P2 warm start from run36 preserve); apply leg on canonical (macOS) — see apply guide
started_at:          2026-09-23T13:38:00Z
ended_at:            2026-09-23T14:40:00Z (in-container close, approx; operator apply close: 2026-09-23T16:15:00Z)
elapsed_min:         ~62 in-container (no tool-ceiling interruption; one dependency STOP-and-ask mid-run, ruled by the operator in-session; G13 checkpoint emitted after the preserve step per HO-31-1/32-1/33-1/35/36 pattern); apply leg: 117
clean_checkout:      yes (fresh container; scaffold Cargo.toml + Cargo.lock from run36-container-preserve, both byte-identical to the Run 31–36 preserve chain BEFORE this run's five-dependency addition)
commit:              33fb3d2 (applied 2026-09-23; built against a8e94f9 = Run 36 code, main@8950ed9). Canonical pre-hashed 6/6 = Run 36 state before drop-in; delivered lib.rs 89a7c66e… storage.rs 7b9e639d… policy.rs aa8127b3… ceremony.rs 221db206… (NEW); docs/resolve/runtime byte-identical to Run 36. Canonical core/crates/lfs_core/Cargo.toml + core/Cargo.lock changed this run (first manifest touch since Run 26) — gated files in the apply guide with (pre) hashes.
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 (UNCHANGED; `cargo tree -p keyhive_core` line identical to Run 26) samod=0.14.0 autosurgeon=0.14.0 subduction=off atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 rusqlite=0.40.2 | FIVE DIRECT DEPENDENCIES ADDED (ruled in-run by the operator, Option 1 with three conditions): keyhive_crypto=git 90fe4a51 (same source as the pin; default features kept — std, as keyhive_core itself uses it) future_form=0.3.1 rand=0.8 (0.8.8 locked; default features KEPT — std → getrandom → OsRng; verified against rand's manifest, not blanket default-features=false) nonempty=0.10 bincode=1.3 (1.3.3). ZERO new packages: every one was already a keyhive_core transitive at the locked version; Cargo.lock delta = +5/−0 in lfs_core's own dependencies list (predicted before the edit, matched exactly; 363 packages before and after). Pin label re-ruled: `Core::pins()` now leads `keyhive_core=0.5.0+90fe4a51` (was `=0.5.0`; SL-0242 — the version string alone does not identify the pinned code) — shells' pins footer changes, predicted in the guide.
outcome:             pass (in-container); apply leg: pass
defects:             0
defect_classes:      n/a
sentinel_state:      READ 2026-09-23T13:49Z. PLAN §3 PER-RUN CHECK (1b OPEN under SL-0247): CLEAR — S-3=#230 "Keyline crate" DRAFT/unmerged/34-commits, mergedTime null (UNCHANGED since Run 36); S-4=kh0 (onomancy main e29ca5df, 2026-09-03, onomancy_keyhive/src/carriage.rs L26 — UNCHANGED). S-1=0.5.0 (crates.io max/stable, updated 2026-06-26) S-2=main@e2789c93 (2026-09-22T13:01:50Z, #228 — UNCHANGED since the Run 35 content read; head identity only; not pinned against) S-5="0.5" (onomancy workspace Cargo.toml L59) | S-12(a)=null (root + keyhive_core CHANGELOG 404; in-tree keyhive_core/Cargo.toml "0.5.0" at main and at pin) S-12(b)=FIRED-state persists (PR #229 merge 9675a0fe on main.atom; branch 2026-sept-updates 404; WATCH per plan v0.1.1 / SL-0247, no halt) S-13=NOT READ in-container (discord.com 403 / outside the declared egress set) — operator read at append: no reply (read 2026-09-23 16:13Z, ✓). NO STOP CONDITION FIRED. H6 pin 90fe4a51 intact. GitHub REST not used; atom/HTML/raw/codeload + crates.io API reads throughout.
l14_note:            n/a
loe_note:            IDENTITY CEREMONY — plan v0.1.1 §9 row 3 (H2 first caller, H3, H5; spec v0.1.4 §3.2, §5.2, L-12; memo v0.1.2 §6). Open checks first: P3 apply-confirmation PASSED — attached Run 36 source = Run 36 CHECKSUMS.txt (6/6) = canonical main@a8e94f9 core/crates/lfs_core/src/ (6/6 by raw fetch); shell files ContentView.swift 0ea4123f… / MainActivity.kt 33ced8fa… = canonical @8950ed9 (2/2 by raw fetch); attached log tail Run 36, fence count 65 (odd residue, untouched); ledger tail SL-0247 by direct read; run36 preserve 11/11 vs its own sheet; baseline 27/0/2 on Ubuntu cargo/rustc 1.91.1 (ninth reproduction of the delivered-state chain). DEPENDENCY STOP (kickoff clause "no new dependency unless ruled"): source read at the pin showed Keyhive::generate / generate_doc require a signer (keyhive_crypto::signer::memory::MemorySigner), a future form (future_form::Sendable), a CSPRNG (rand::rngs::OsRng), NonEmpty (nonempty) and an encoder for the persisted bytes (bincode) — keyhive_core re-exports only keyhive_crypto::digest; no in-crate workaround (implementing a signer still names the AsyncSigner/Verifiable traits). Session halted before touching Cargo.toml; operator ruled Option 1 — the predictable consequence of H6 (a git pin on a workspace crate whose companions are not re-exported; the pin decision is not reopened, only its surface made explicit) — see pins: for versions/features. CONDITION 1 RECORDED: bincode over keyhive_core's serde shape at 90fe4a51 is now the app's stored-bytes encoding surface — exactly the surface the Consumer Evidence Note describes, now concrete; the format tag (below) is the seam that absorbs a change there; this sharpens Discord asks #2/#3 and the migration-cost note v1 (carried as a note, not a task). (i) CEREMONY ENTRY POINT: NEW core/crates/lfs_core/src/ceremony.rs (350 lines) + NEW FFI method Core::run_identity_ceremony(config: IdentityConfig) → IdentityCeremonyRecord (uniffi::Record: identity_row_id, rooting_level, floor_status, admin_delegations, format_tag, persisted_bytes, cold_keys). Parametric on RootingLevel — the level arrives from the shell via IdentityConfig, is ECHOED in the record and NOT ENACTED (keyhive_core at the pin has no rooting-level concept; that is Keyline #230, S-3); the record's floor_status = policy::delegation_floor_status(level) IS the L-12 citation (Edit → Safe; Admin → WithdrawnPendingL12 — the floor is withdrawn under Admin-rooting unless the recovery delegation is a FROST 2-of-3 share, not modelled). policy::FloorStatus gains uniffi::Enum and crosses FFI; GrantLevel does NOT cross (Run 38). Document creation is keyhive_core's own ceremony: Document::generate runs EphemeralSigner::with_signer (root key destroyed at creation — spec §3.2 ✓ by source read) and grants each generation-time parent Admin from that root (principal/group.rs generate_after_content ✓). (ii) TWO DELEGATIONS: the primary cold admin key is the hive's active agent and the recovery key (a second MemorySigner, introduced to the admin hive through a contact card — the pinned crate's only peer-registration path) is the co-parent → the identity document holds exactly two Admin delegations from one ephemeral root, counted from doc.delegation_heads() and passed through policy::check_delegation_floor — first caller of the floor; core enforced nothing. PolicyViolation → CoreError mapping RULED (floor only): additive variant CoreError::Policy(String) carrying the violation's Display; a below-floor ceremony (test hook: no recovery key) returns Policy("delegation floor: 1 admin delegation(s) present, floor is 2") and writes nothing. Two more additive variants: CoreError::Ceremony(String) (Keyhive/signature/encode failure) and CoreError::IdentityAlreadyEnrolled (a second ceremony is refused without touching the existing row). (iii) COLD KEY OFF-DEVICE: export shape = ColdKeyExport { primary_admin_secret: Vec<u8> (32-byte Ed25519 seed), primary_admin_fingerprint: String (hex verifying key = the persisted delegate), recovery_secret, recovery_fingerprint } — returned once across FFI as a field of the record; the core keeps no copy and writes none; test proves both seeds absent from the SQLite file bytes while both public keys are present. Custody past the boundary is the shell's; the shells do NOT surface the ceremony this run (no UI; Run 39 shell-integration). H3 held: the Keyhive document ID never crosses FFI (the record carries the storage row id `identity`) and appears in no log path. (iv) FIRST KEYHIVE BYTES IN SQLITE — FORMAT TAG RULED: storage::FORMAT_KEYHIVE_STATIC_DELEGATIONS_V1 = "keyhive.static-delegations.bincode.v1"; bytes = bincode::serialize(Vec<Signed<StaticDelegation<[u8;32]>>>) of the identity doc's delegation heads, sorted by delegate id, each try_verify()'d before write — public PROOF bytes only. The full Archive was considered and REJECTED for the row: it carries the active agent's prekey SECRETS and the active agent here is the cold primary admin. `.v1` names the serde shape at 90fe4a51; a T4 re-encode (kh0 → kh1, L-16) or any upstream serde change lands under a new value; rows are never re-tagged (Run 30 rule). Storage adapter: two additive DocStore methods (read_tagged, write_keyhive_static_delegations — tag stamped inside the adapter, H1) sharing one upsert; `read`/`write` unchanged; Phase 0 stub migration and automerge.save.v1 rows untouched (roundtrip_under_trait_with_format_tag + migrates_phase0_stub_schema green). Pin label re-ruled alongside (pins:). Initial content head of the identity document = the zero digest (the pinned crate's own placeholder shape; content binding is later work — ~). (v) SHELL ADOPTION of open_typed_doc_at FOLDED IN (first shell-touching run of 1b): two-line edit per shell — the pings open becomes openTypedDocAt("pings", …, now) with the shell's clock (Swift ISO8601DateFormatter whole-second Z; Kotlin Instant.now().toString()); profile/threads stay on openTypedDoc (no clock, no cleanup — Run 35 rule); run37-shell-open-typed-doc-at.patch -p1 dry-run clean against 8950ed9; identifiers grep-verified against the regenerated bindings. Visible effect predicted for leg (c): expired pings are removed on load, so the shells' Pings "stored" count can drop to the active count. DONE-WHEN: H1 grep (`grep -r 'keyhive' src/ --include='*.rs' | grep -v '_test\|//\|storage'`) returns ceremony.rs ONLY — 10 lines (8 `use keyhive_core/keyhive_crypto` lines, the adapter call write_keyhive_static_delegations, one test name); plan §4 H1 done-when text still reads "storage + policy" — ceremony.rs is the named exception the Run 37 kickoff anticipated; v0.1.2 Lightweight touch candidate, NOT adapted here. Suite 35/0/2 (27 baseline intact + 8 new: ceremony::tests ×6 — two_admin_delegations, admin_rooting_records_floor_withdrawn_pending_l12, keyhive_bytes_land_under_new_format_tag_and_verify, cold_keys_are_exported_and_never_persisted, floor_violation_maps_to_policy_error_and_persists_nothing, second_ceremony_is_refused_and_leaves_the_row_intact; storage::tests::keyhive_and_automerge_rows_coexist_under_their_own_tags; tests::identity_ceremony_via_ffi_surface_persists_and_survives_restart). Build 0 warnings. Bindings regenerated (uniffi-bindgen 0.32.1 library mode, workspace cwd; CLI built standalone outside the scaffold): diff vs bindings-run36-state = Swift +314 / Kotlin +292 / header +11 (uniffi_lfs_core_fn_method_core_run_identity_ceremony + its checksum fn) / modulemap 0 / 0 deletions / 0 modifications — the IdentityConfig docstring kept verbatim (Run 35 rule) so nothing generated is removed. No spec/memo divergence adapted around; one plan-text divergence flagged (H1 done-when wording). Preserve tarball produced (P1): run37-container-preserve_2026-09-23.tar.gz sha 92205960… (scaffold manifest + lockfile AS CHANGED + bindings-run37-state + bindings-run36-state carried). Delivered: lib.rs (+67/−2), storage.rs (+67/−8), policy.rs (+8/−4), ceremony.rs (new), scaffold Cargo.toml (+7 incl. comment) / Cargo.lock (+5), run37-identity-ceremony.patch (-p1 dry-run clean against the a8e94f9 tree), seven-file CHECKSUMS.txt with (pre) baselines + scaffold manifest/lock (pre), shell files + patch, apply guide + gated appender (Run 36 apply rules applied: paths quoted; adb by SDK path; head -1 gate before every cp; diff-stat miss = STOP; this header carries no marker literal and no slot count). Not verdict-bearing; no ledger delta (build execution under plan §9 row 3 / SL-0247; H6 pin unchanged; H6 was never SL-ruled on its own — plan hedge under SL-0244 executed at Run 26).
next:                Run 37 apply (operator; guide — first apply that touches Cargo.toml/Cargo.lock since Run 26; lockfile prediction +5/−0 is a STOP if missed). Run 38 — groups behind the counter (plan §9 row 4): Keyhive groups back membership_version / record_membership_event; H2 grant bar wired (check_grant_bar first caller; GrantLevel ↔ live capability mapping; PolicyViolation::GranterBelowBar → CoreError::Policy); the device-key delegation and the live-hive reload path (ceremony bytes → ingest) are Run 38/40 design questions, not pre-decided here; attach spec v0.1.4 + memo v0.1.2 + plan v0.1.1 + this run's preserve (bindings-run37-state = next binding-diff baseline; scaffold manifest/lock AS CHANGED). Plan v0.1.2 Lightweight touch candidates: H1 done-when wording (+ ceremony), §5 run_type `ceremony` vs the kickoff's host-build for this run (flagged, not adapted). S-2 e2789c93 semantic read of beekem/tree.rs owed only if pinning past 90fe4a51 is proposed. S-3 demotion candidate (post-Run-37) carried. Physical-android date-gated (D-8, SL-0243). S-13 watch on post 1; Onomancy Ask 2 on trigger (~2026-09-29); Discord #2/#3 now sharpened by the concrete bincode/archive-shape coupling (note).
```

```
run:                 38
run_type:            host-build
platform:            container (Ubuntu 24.04, reproduction scaffold, P2 warm start from run37 preserve); apply leg on canonical (macOS) — see apply guide
started_at:          2026-09-23T20:03:00Z
ended_at:            2026-09-23T20:55:00Z (in-container close, approx; operator apply close: 2026-09-23T22:13:00Z)
elapsed_min:         ~52 in-container (one tool-ceiling interruption before the P1 preserve step — G13 checkpoint emitted early, resumed on "Continue"; no operator STOP-and-ask this run); apply leg: 28
clean_checkout:      yes (fresh container; scaffold Cargo.toml + Cargo.lock from run37-container-preserve AS CHANGED — 211d8724… / d513b5c9…, byte-identical before and after this run)
commit:              3e6ff66 (built against 33fb3d2 = Run 37 code, main@7a9566b; the two doc-only commits 568bd17/8e6b530/7a9566b change nothing under core/). Canonical pre-hashed 7/7 = Run 37 state before drop-in; delivered lib.rs e476ed19… storage.rs 08001d51… policy.rs 245a2d5a… ceremony.rs 950f2241… groups.rs 764f0745… (NEW); docs/resolve/runtime byte-identical to Run 37. Canonical core/crates/lfs_core/Cargo.toml (79f3b645…) + core/Cargo.lock (d651affb…, 571) UNCHANGED — gated files in the apply guide, pre = post.
pins:                keyhive_core=git+https://github.com/inkandswitch/keyhive?rev=90fe4a51 (UNCHANGED; `cargo tree -p keyhive_core` line identical to Run 26) keyhive_crypto=git 90fe4a51 future_form=0.3.1 rand=0.8 nonempty=0.10 bincode=1.3 samod=0.14.0 autosurgeon=0.14.0 subduction=off atrium-api=0.25.8 reqwest=0.12.28(rustls-tls) uniffi=0.32.1 rusqlite=0.40.2 | NO NEW DEPENDENCY: the additional keyhive_core paths named this run (event::static_event::StaticEvent, contact_card::ContactCard, principal::individual::op::KeyOp, principal::document::id::DocumentId, principal::group::id::GroupId, principal::identifier::Identifier) are re-exports of the same crate at the same rev (D-38-4 "not a new dependency"). Lockfile delta 0/0; 363 packages before and after.
outcome:             pass (in-container); apply leg: pass
defects:             0
defect_classes:      n/a
sentinel_state:      READ 2026-09-23T20:06Z. PLAN §3 PER-RUN CHECK (1b OPEN under SL-0247): CLEAR — S-3=#230 "Keyline crate" DRAFT/unmerged/34-commits, mergedTime null (UNCHANGED since Run 37 and the D-38 17:53Z read); S-4=kh0 (onomancy main e29ca5df, 2026-09-03, onomancy_keyhive/src/carriage.rs L26 — UNCHANGED). S-1=0.5.0 (crates.io max/stable, updated 2026-06-26) S-2=main@e2789c93 (2026-09-22T13:01:50Z, #228 — UNCHANGED; head identity only; not pinned against) S-5="0.5" (onomancy workspace Cargo.toml L59) | S-12(a)=null (root + keyhive_core CHANGELOG 404; in-tree keyhive_core/Cargo.toml "0.5.0" at main and at pin) S-12(b)=FIRED-state persists (PR #229 MERGED 2026-09-21T16:16:11Z on main; branch 2026-sept-updates 404; WATCH per plan v0.1.1 / SL-0247, no halt) S-13=NOT READ in-container (discord.com 403 / outside the declared egress set) — operator read at append: no reply (read 2026-09-23T22:12Z). NO STOP CONDITION FIRED. H6 pin 90fe4a51 intact. GitHub REST not used; atom/HTML/raw + crates.io API reads throughout.
l14_note:            n/a
loe_note:            GROUPS BEHIND THE COUNTER — plan v0.1.1 §9 row 4 (H1, H2 first caller of the bar, H3; D-38 record 1a354226… rulings D-38-1..4 CARRIED and verified at open; amendment pack 473ed745… §0 OR-1/OR-3/OR-4 applied). Open checks first: P3 apply-confirmation PASSED — attached Run 37 source = Run 37 CHECKSUMS.txt (7/7) = canonical main@33fb3d2 core/crates/lfs_core/src/ (7/7 by raw fetch; byte-identical at main@7a9566b); attached log tail Run 37, fence count 67 (odd residue, untouched); ledger tail SL-0248 by direct read (ARCH-PAIR); run37 preserve 11/11 vs its own sheet; baseline 35/0/2 on Ubuntu cargo/rustc 1.91.1 (tenth reproduction of the delivered-state chain). D-38 open verifications: access.rs L19–40 ≡ policy.rs L42–47 (Relay/Read/Edit/Admin); add_member (keyhive.rs L450–456) (to_add, resource, can, other_relevant_docs); receive_contact_card L356; ingest_unsorted_static_events L2460; StaticEvent five variants; group.rs L183 roots every parent at Admin; no rooting_level write path in storage.rs/ceremony.rs (D-38-1 — floor_status derived only). (i)(a) DEVICE DELEGATION (D-38-3, OR-1 Edit ✓ operator-ruled): inside generate_identity_doc the device's Individual is introduced by contact card exactly as the recovery key is, then, after generate_doc and before the delegations are collected, the primary admin (active agent) issues add_member(device, identity_doc, Access::Edit, &[]) — a third delegation with a proof; Admin count unchanged at two. Device-seed source RULED A1 in-run: the ceremony generates the device MemorySigner; the seed crosses FFI once as DeviceKeyExport { device_secret, device_fingerprint } beside ColdKeyExport (new field IdentityCeremonyRecord.device_key); the core retains the SIGNER in memory as the device hive's active agent; the seed never lands in SQLite (cold_keys_are_exported_and_never_persisted extended by design to the row set). FINDING (source, mechanism only — the ruling stands): once the primary admin issues the device delegation, its own root delegation is the device's PROOF and is no longer a HEAD — delegation_heads() drops to two (recovery Admin + device Edit) and the floor read ONE Admin on the first run (Policy refusal). The D-38 record §2 D-38-3 sentence "delegation_heads() now yields three entries (§1 (iii)-8)" is wrong at the pin (the DelegationStore never prunes, but heads are heads). The identity row and the Admin count are therefore taken from members(): every delegation of every member (three — two Admin root edges with no proof, the device Edit edge with the primary's root as proof), Admin count = members whose highest capability is Admin (two). Reload needs the proof delegation in the row anyway. Flagged for the D-38 record's companion note; not adapted around — the persisted set is the ruled set. (i)(b) RELOAD (D-38-4, OR-4 DECIDED HERE): identity_row_reloads_into_device_hive WRITTEN AND RUN FIRST — GREEN. Fresh Keyhive::generate(device_signer, MemoryCiphertextStore, NoListener, OsRng); ingest [PrekeysExpanded(primary KeyOp), PrekeysExpanded(recovery KeyOp), Delegated ×3 from decode(identity row)] → nothing pending; get_document Some; members()==3; device capability Edit; both admins Admin; the document id is recovered from the root delegations' issuer (the destroyed ephemeral key) and checked equal to the ceremony's. RELOAD LIFTS AT RUN 38: decode() loses #[cfg(test)]; ceremony::reload_with(store, signer) → (Hive, DocumentId) is the product path (parametric on the signer — Run 40's recovery re-uses it with a re-imported admin as active agent); Core::run_identity_ceremony now installs the hive it rebuilt from the two persisted rows. (i)(c) rooting level not persisted (D-38-1); no write path. Pack §4 docstring appends applied to policy.rs (below L11: the pin finding; below L40: mapping/FFI note; below L97: the discharged "does NOT cross FFI" line kept verbatim + appended) — never a generated-docstring line edited; the one non-docstring change in policy.rs is the derive line gaining uniffi::Enum (D-38-2). TEST-LINE EDITS: (1) the NAMED one — ceremony.rs keyhive_bytes_land_under_new_format_tag_and_verify: len()==3, Admin edges proof.is_none() + issued by the one ephemeral root, the Edit edge proof.is_some() + issued by the primary admin + delegate = device fingerprint, admin_delegations==2; (2) a SECOND, NOT named in the kickoff and stated here — lib.rs identity_ceremony_via_ffi_surface_persists_and_survives_restart's expected tag list gains the ("identity-keyops", static-events v1) row, a direct consequence of the ruled KeyOp row (D-38-4) that record §3(c)-6 did not anticipate. (ii) GROUPS BEHIND THE COUNTER: NEW core/crates/lfs_core/src/groups.rs (329 lines) = the hive-holding module (registered `pub mod groups;`). DeviceHive { hive, device: Identifier, identity_doc: Option<DocumentId>, groups: HashMap<app group_id → GroupId> }: from_ceremony(store, material) = reload_with (D-38-4 path (B)); session_only() = a per-session hive with an ephemeral device signer and NO identity document — RULED in-run (~): the Run 32 shells call the counter un-enrolled (a host-build touches no shell), so the counter always reads real Keyhive group membership, never the retired simulation; membership does not survive relaunch (group persistence is Run 39's — flagged). membership_version(group_id) = the group's members().len() − 1 (the device's own root membership), 0 = no group (the Run 32 "0 = no event seen" contract kept); record_membership_event(group_id) = ensure_group (generate_group(vec![]) → device = Admin) + one simulated peer (in-memory MemorySigner, contact card, receive_contact_card — the pin's only peer-registration path) granted at Read through the bar; real peers arrive with the connection seam (Phase 2), the FFI signature is the seam and is UNCHANGED (docstrings appended). Core's Run 32 in-memory `membership: Mutex<HashMap<String,u64>>` RETIRED (field + init removed — product lines, not FFI docstrings) → `device: Mutex<Option<groups::DeviceHive>>`. (iii) H2 GRANT BAR WIRED — first caller of policy::check_grant_bar: mapping `impl From<Access> for GrantLevel` + access_of(GrantLevel) → Access lives in groups.rs, NOT policy.rs (H2; D-38-2 1:1 table, round-trip test grant_level_maps_one_to_one_onto_access); granter = the device (the active agent signs every delegation the hive issues); its held level = get_capability(&device).payload().can() on the group, mapped; check_grant_bar runs BEFORE add_member; PolicyViolation::GranterBelowBar → CoreError::Policy(v.to_string()) (Run 37 variant, reused). REFUSAL TEST below_admin_grant_is_refused_by_policy_before_core: another hive creates a group and delegates the device at Edit; the device hive ingests that group (other's KeyOp + two delegations, nothing pending — the same ingest path as the reload) and adopts it under an app name; device_grant_level == Edit; membership_version == 1; the grant attempt returns Policy("grant bar: granter holds Edit, bar is Admin"); members() and total delegation count unchanged before/after (core never invoked — at the pin core would have ACCEPTED the non-escalating Read grant, D-38 record §1 (ii)-5); the same call on a device-created group passes. GrantLevel CROSSES FFI (D-38-2): derive appended; NEW FFI Core::grant_member(group_id: String, level: GrantLevel) → Result<u64, CoreError> (the typed, error-bearing event; record_membership_event routes through it at Read and leaves the version unchanged on error) and Core::device_grant_level(group_id) → Option<GrantLevel>. Bar scope on the identity document HELD (OR-2) — every grant this run issues is on a group the device created. NEW additive CoreError::Membership(String) (group creation / peer registration / add_member after the bar passed; never a policy refusal). (iv) PERSISTENCE — NEW TAG RULED: storage::FORMAT_KEYHIVE_STATIC_EVENTS_V1 = "keyhive.static-events.bincode.v1": bincode over Vec<keyhive_core::event::static_event::StaticEvent<[u8;32]>> at 90fe4a51 — the exact type ingest_unsorted_static_events consumes and NOT the v1 delegation type (a different serde enum), so a new value; `.v1` names that shape; never re-tagged (Run 30 rule). Row id ceremony::IDENTITY_KEYOPS_ROW_ID = "identity-keyops" (app-owned; H3). Contents: PrekeysExpanded events for the primary, recovery AND device KeyOps (each try_verify()'d before write) — the device's own is included beyond the kickoff's "two admin KeyOps" because Run 40's recovery path (re-imported admin as active agent) must resolve the DEVICE delegate (~, ruled in-run; the reload path filters the active agent's own op out before ingest). Storage adapter: one additive DocStore method write_keyhive_static_events (tag stamped inside the adapter, H1) on the shared upsert; read/write/read_tagged/write_keyhive_static_delegations unchanged; the identity row stays under keyhive.static-delegations.bincode.v1 (same serde type; three entries now); Phase 0 stub schema and automerge rows untouched (roundtrip_under_trait_with_format_tag, migrates_phase0_stub_schema, keyhive_and_automerge_rows_coexist_under_their_own_tags green). Record gains keyops_row_id / keyops_format_tag / keyops_persisted_bytes. DONE-WHEN: H1 grep (`grep -r 'keyhive' src/ --include='*.rs' | grep -v '_test\|//\|storage'`) returns ceremony.rs ×16 (13 `use keyhive_core/keyhive_crypto` lines, two adapter calls, one test name; Run 37 baseline ×10) + groups.rs ×12 (9 `use` lines, two test-module `use` lines, one test name) — groups.rs is the module this run names, as the kickoff done-when anticipated; policy.rs comment lines only. H3: no Keyhive doc/group id on any FFI or log path (records carry row ids and app-owned group names). H4: no `ed25519` literal in product code. Suite 40/0/2 (35 baseline intact + 5 new: ceremony::tests::identity_row_reloads_into_device_hive; groups::tests::counter_reads_keyhive_group_membership, below_admin_grant_is_refused_by_policy_before_core, grant_level_maps_one_to_one_onto_access; tests::groups_behind_the_counter_via_ffi_surface). Build 0 warnings. Bindings regenerated (uniffi-bindgen 0.32.1 library mode, workspace cwd; CLI built standalone outside the scaffold — an unpinned "0.32.1" resolved to 0.32.2 on the first attempt; pinned "=0.32.1"): diff vs bindings-run37-state = Swift +290/−5 / Kotlin +258/−4 / header +22 (uniffi_lfs_core_fn_method_core_grant_member, _device_grant_level + their two checksum fns) / modulemap 0. THE DELETIONS ARE STRUCTURAL, NOT DOCSTRINGS: IdentityCeremonyRecord's generated init/read/allocationSize lines (record gained four fields) and the checksum constants for membership_version / record_membership_event / run_identity_ceremony (uniffi hashes the docstring into the checksum; the appended docstrings change the constant, not the signature — all Run 37 docstring lines present verbatim, 7/7). No spec/memo divergence adapted around; one D-38 record source-reading correction flagged (above). STOP-AND-RULE QUEUED FOR RUN 39 (not touched here): rebuilding the device MemorySigner from the custodied seed across a relaunch needs ed25519_dalek::SigningKey::from_bytes — a NEW direct dependency AND an H4 literal; keyhive_crypto exposes no seed→signer path at the pin; a reload_identity(device_secret) FFI is therefore NOT added this run. Preserve tarball produced (P1): run38-container-preserve_2026-09-23.tar.gz sha 7437bf0a… (scaffold manifest/lock UNCHANGED + bindings-run38-state + bindings-run37-state carried, 11/11 sheet). Delivered: lib.rs (+78/−12), storage.rs (+14/−0), policy.rs (+11/−1), ceremony.rs (+294/−18), groups.rs (new), run38-groups-behind-the-counter.patch (-p1 dry-run clean against the 33fb3d2 tree), eight-file CHECKSUMS.txt with (pre) baselines + scaffold manifest/lock (pre = post) + canonical gated-file hashes, apply guide + gated appender (Run 37 apply rules applied). Not verdict-bearing; no ledger delta (build execution under plan §9 row 4 / SL-0247; the design claims are SL-0248's — its validation event (1) "reload test green" is MET this run, an operator note on SL-0248 at append, not a new entry).
next:                Run 38 apply (operator; guide — no manifest/lockfile change; gated files must hash pre = post). OR-4 OUTCOME: RELOAD LIFTED AT RUN 38 (identity_row_reloads_into_device_hive green; decode() ungated; reload_with is product code) — Run 39 does NOT carry the reload; Run 40's recovery re-uses reload_with with the re-imported admin as active agent and the persisted KeyOp row (device op included). Run 39 — grant/revoke on device (plan §9 row 5): BEFORE code, two rulings — (1) DEPENDENCY: seed→signer for the custodied device seed needs ed25519_dalek (new direct dep + H4 literal) — STOP-and-rule shape as Run 37's; (2) GROUP PERSISTENCE: groups are in-memory this run (session hive dropped at ceremony; "kill/relaunch keeps it" needs group delegations persisted — candidate: same v1 delegation type under keyhive.static-delegations.bincode.v1 with app-owned row ids, decided there). Membership query for the shells (per-member level list) is Run 39's design; removed-count across FFI (open item (d)) still open. D-38 record companion note: §2 D-38-3 "three heads" → "three delegations via members(); two heads" (correction, ruling unchanged). Plan v0.1.2 touch now also carries: H1 done-when "+ groups"; §9 row 4 done. Attach spec v0.1.4 (v0.1.5 when cut) + memo v0.1.2 + plan v0.1.1 + this run's preserve (bindings-run38-state = next binding-diff baseline; scaffold unchanged). S-2 e2789c93 semantic read only if pinning past 90fe4a51 is proposed. S-3 demotion candidate carried. Physical-android date-gated (D-8, SL-0243). S-13 watch on post 1; Onomancy Ask 2 on trigger (~2026-09-29); Discord #2/#3 sharpened.
```
