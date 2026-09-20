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
