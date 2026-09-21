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
