⚑ STAMP: SINGLE-CONTEXT — NOT PANELED
One governed context (Mode 1, Session Harness v0.2). Every number below is copied from a named field of `docs/phase0-observation-log.md` (Runs 0–23) or from the Unit 6 close handoff §5; nothing is from memory. The estimate in the last section is the packet's only inferred content and is tagged as such.

# Native Track — Phase 0 LOE Packet (plan §5)

**Date:** 2026-09-20 · **Register:** CONTEXTUAL · **Mode:** 1
**Instrument:** `native-track-phase0-build-plan-v0-1_2026-09-18.md` §5 (fields), §6 (log shape); spec v0.1.4 §8/§9
**Repo:** `jediwright/local-first-social-native`, `main` at `ed47363` before this unit (Runs 23–24 entries drafted below for operator append)
**Prepared by:** J. Wright / UX Minds, LLC — AI-collaborative synthesis; human authorial responsibility
Confidence key: ✓ from a log field · ~ inferred (basis stated) · ? assumed (verification named)

---

## 0. Exit check — plan §0 verbatim

> *Hello-world with an Automerge doc round-tripping through the core on a device.*

Plan §0 reads "a device" as **both** platforms (A1, ~ — operator may narrow).

| Platform | Round-trip evidence (log field) | Device | Status |
|---|---|---|---|
| iOS | Run 18 `l14_note`: typed "Hello phase 0", Save → `saved: 159 bytes`, app-switcher kill, relaunch → field shows text, `text=13 chars`; re-confirmed Runs 19, 20 | physical iPhone 17 Pro, iOS 26.6.1 | **MET ✓** |
| Android | Run 14 `l14_note`: typed "hello phase 0", Save → `saved: 159 bytes`, `am force-stop`, relaunch (new pid) → field shows text, `text=13 chars`; re-confirmed Runs 15, 21 | emulator-5554 (Pixel_9 AVD, arm64-v8a) under **D-8** | **MET ✓ under D-8** — no physical Android device was run; plan B8 "done when" says physical, D-8 accepted 2026-09-20 |

Both saves are byte-identical (159 bytes) from the same core, same document, same SQLite path pattern. **Exit criterion met on both platforms, with the Android side qualified by D-8.** A physical-Android run is the first carried Phase 1 item (handoff §4).

Sentinel re-run at exit (Run 23, this unit): **no sentinel moved** since Run 0 (see §7). Phase 0 closes on the same upstream state it opened on.

---

## 1. Hours per toolchain first-success, per platform

"First success" = clean checkout → round-trip on device. Run wall-clock is the log's `elapsed_min`; toolchain install is operator wall-clock outside run entries (~ where the log says so).

| | Android (Runs 13–15, 21) | iOS (Runs 16–20) | Host (Runs 1–9, 11, 12) |
|---|---|---|---|
| Toolchain install (outside runs) | ~45 min ✓-as-logged, ~ as a figure (Run 13 `loe_note`: Android Studio + SDK 37 + NDK r30 + Play arm64 image + AVD; not timed) | ~6 min Xcode 27 from App Store (Run 17 `loe_note`, ~); 8.05 GB simulator runtime in parallel, off the critical path | rustup + cargo-make 1m31s on macOS (Run 12); Ubuntu `rustc-1.91` in the container (Run 1, 3 min) |
| binding-build | **16 min** (Run 13) | **14 min** (Run 17) | 2 min (Run 9, Linux .so → Kotlin + Swift) |
| shell-run (to round-trip) | **20 min** (Run 14) | **44 min elapsed / ~24 min active** (Run 18; ~19 min session break inside `elapsed_min`) | n/a |
| network-probe (B4 on device) | 5 min (Run 15) | 9 min (Run 19) | Run 7 partial (egress 403) → Run 12 pass on macOS (`elapsed_min` `<fill>` — unfilled, see §8) |
| oauth-smoke (B12) | 6 min (Run 21) | 6 min (Run 20) | n/a |
| reference read | — | 1 min (Run 16, MeetingNotes) | — |
| **Run wall-clock to round-trip (binding-build + shell-run)** | **36 min** | **38 min active (58 elapsed)** | — |
| **Run wall-clock, all platform runs** | **47 min** | **54 min active (74 elapsed)** | 20 min Runs 1–9 (sum of `elapsed_min`) + 34 min Run 11 (first commit/push, operator-time ~20) |

Phase 0 total governed wall-clock: Run 0 `started_at` 2026-09-20T15:07:36Z → Run 24 `ended_at` (below) ≈ **6.3 h**, single day, one operator, one harness context per unit (7 units). Toolchain installs (~51 min) sit outside that figure.

---

## 2. Build-system defects hit and their class

From `defects` / `defect_classes` of `binding-build` entries, plus the shell-run entries the plan's field sends here by implication (l14 asymmetry).

| Run | Type / platform | Repo defects | Class (log's own labels) | Fix |
|---|---|---|---|---|
| 9 | binding-build / host | 0 | n/a | — |
| 13 | binding-build / android | **4** | 3 × `other` (build-system: `cargo metadata` from a root with no Cargo.toml in `android-so` / `bindgen-kotlin`; `.so` vs `.dylib` host cdylib path) + 1 × `dependency` (reqwest `default-tls` → openssl-sys, no Android sysroot → swapped to `rustls-tls`) | `6cfee9e`, `8dc6dab`; ~12 of 16 min |
| 14 | shell-run / android | 0 repo; 1 host-env (`other`: no JAVA_HOME → Studio JBR) | — | shell-session export, no repo change |
| 17 | binding-build / ios | **0** | n/a — four predicted items cleared by pre-read (P-1..P-4) | — |
| 18 | shell-run / ios | **1** | `build-system` (warning-only: cc objects built at SDK 27.0, linked at 17.6; no `IPHONEOS_DEPLOYMENT_TARGET` in the xcframework task) | `e6582d8`; verified 22 → 0 warnings at Run 19 (blake3 needed `cargo clean -p`) |

Plan §5's class vocabulary (UniFFI / linker / target / dependency) maps: **UniFFI 0**, **linker 1** (Run 18, warning-only), **target 0** (NDK r30 16 KB pages and arm64 cross-compile were first-pass green), **dependency 1** (Run 13 TLS backend), **build-system/task-wiring 3** (Run 13 — the never-invoked `cargo make` tasks). Every defect was fixed inside its run or before the next; none is open. Host-side non-build defects (Runs 6, 11, 12: API misread, harness path instruction, `cargo make` from the wrong directory) are `other` and not build-system.

---

## 3. Crate-pin risk register (from `pin-decision` entries)

| Crate / rev | Pin | Run | Risk note (log text governs) |
|---|---|---|---|
| `keyhive_core` | **`=0.5.0` (crates.io)** — `keyhive_crypto 0.2.1`, `beekem 0.3.0` | 2, 3 (B11) | Builds on a fresh machine, 1m02s, 91 packages ✓. `main @ 90fe4a51` builds in 1m03s with an **identical** dependency set — the L-4 gap is **semantic** (12 commits of core correctness fixes: #226, #227, #232, #233, concurrent-remove ordering) under one version string. **Phase 1 pre-decision:** a Phase 1 that needs #226/#232 fixes pins the git rev. Publishability ✓ (L-4 "?" → ✓ for 0.5.0 on 2026-09-20; Time-sensitive against the committed lockfile). Sentinels S-1..S-5 unmoved at Run 23. |
| `samod` / `autosurgeon` | **0.14.0 / 0.14.0** → `automerge 0.12.0` | 4 (B2) | Spec's "0.13" was descriptive (A2). One `keyhive_core`, one `automerge` in the lockfile (573 packages with Subduction). Re-check if Phase 1 declares `automerge_subduction_ingest` (Subduction workspace pins `automerge 0.11.0`; the two Subduction crates Phase 0 declares do not depend on automerge). |
| Subduction | git rev **`21b2e6b8`** (`main` 2026-09-20); `subduction_keyhive 0.8.2`, `subduction_iroh 0.10.2`, `iroh 1.2.0` | 4, 8 (B5) | Declared and compiling behind `--features subduction` (+222 packages, 4m51s on 1 vCPU; 39s full test on macOS). Not exercised. `main` still at 0.8.2 / 0.10.2 / automerge 0.11.0 at Run 23 (S-10). |
| atrium | `atrium-api 0.25.8`, `-oauth 0.1.7`, `-identity 0.1.9`, `-repo 0.1.8`, `-crypto 0.1.3`, `-xrpc-client 0.5.15` | 4, 7 | Stable since 2026-03-26; unchanged at Run 23 (S-9). Default features **off** on `atrium-oauth` / `atrium-xrpc-client` (Run 13 TLS fix). |
| TLS backend | `reqwest 0.12.28` **`rustls-tls`** + webpki roots, `rustls 0.23.45` | 13 | The one dependency defect of Phase 0. **Phase 1 pre-decision:** rustls + platform verifier vs webpki roots vs vendored OpenSSL (Run 13 `loe_note`). |
| Bindings / build | `uniffi 0.32.1` (proc-macro, library mode); `cargo-ndk 4.1.2`, NDK `30.0.16248370`; `cargo-make 0.37.24`; Xcode 27.0 (27A266a), iOS SDK 27.0; `rusqlite 0.40.2` bundled; `tokio 1.53.1` | 5, 9, 13, 17 | Generated bindings are deterministic across three targets (Linux .so, Android arm64 ELF, iOS .a: 1,687-line Kotlin / 963 + 610-line Swift every time — Runs 9, 13, 17). |
| Toolchain floor | rustc 1.91.1 (container) / 1.98.1 (macOS); `rust-version = 1.90.0` | 1, 12 | D-3; both above floor. |

---

## 4. Android / iOS asymmetry — L-14 quantified

| Measure | Android | iOS | Ratio A:I | Source |
|---|---|---|---|---|
| Run wall-clock to round-trip | 36 min | 38 min active | **≈ 1 : 1** | §1 |
| Run wall-clock, all platform runs | 47 min | 54 min active | 0.9 : 1 | §1 |
| Repo defects, all platform runs | **4** (Run 13) | **1** (Run 18, warning-only) | **4 : 1** | §2 |
| Host-env defects | 1 (JAVA_HOME) | 0 | — | Runs 14, 17–20 |
| Toolchain install, operator wall-clock | ~45 min (~) | ~6 min (~) | **≈ 7.5 : 1** (~) | Runs 13, 17 |
| binding-build first-pass | 4 defects, ~12 min diagnosing | 0 defects, 4 predicted items cleared by pre-read | — | Runs 13, 17 |
| OAuth-path smoke | 6 min, 0 defects | 6 min, 0 defects | **1 : 1** | Runs 20, 21 |
| Live network across FFI (cold / warm) | 1477 / 1241 ms (emulator NAT) | 342 / 118–260 ms (physical Wi-Fi) | not a runtime finding (Run 19 `l14_note`) | Runs 15, 19 |
| Device class | emulator (D-8) | physical | — | — |

**Reading (✓ from the fields above):** L-14's "Android is the thinner side" holds on **defect count** (4:1, all build-system/dependency, all fixed within the run) and **toolchain setup** (~7.5:1, ~), and does **not** hold on run wall-clock (≈1:1) or on the OAuth path (1:1). L-14's second clause — "UniFFI Kotlin output from the Keyhive workspace unconfirmed (~)" — is **confirmed ✓**: Kotlin generated from a `keyhive_core 0.5.0`-bearing workspace on three hosts (Runs 9, 13), cross-compiled for arm64-v8a + x86_64 with every crypto/-sys crate clean (Run 13), loaded via JNA and round-tripping on device (Run 14). This is the content of the L-14 ledger delta (close handoff).

One structural caveat the ratio inherits: Android ran on an emulator (D-8), iOS on a physical device; the 4:1 defect ratio is a build-system ratio and does not depend on device class, but any device-level Android friction (Keystore, WorkManager, OEM background limits) is **unmeasured** and is Phase 1's.

---

## 5. Live-network-across-FFI result (B4 on device) and async-runtime shape

- **Result:** pass on both platforms, first try, no `CoreException` / throw. Android Run 15: `resolvePds(did:plc:z72i7hdynmk6r22z27h6tvur)` → `https://puffball.us-east.host.bsky.network` in 1477 ms cold / 1241 ms warm. iOS Run 19: same DID → same PDS, 342 ms cold / 118, 260 ms warm. Host Run 12: 0.22–0.55 s. Cold–warm gaps are network round-trip variance, not runtime/TLS setup (Runs 15, 19 `l14_note`).
- **Shape used:** blocking FFI call (`fun resolvePds(did: String): String` / `func resolvePds(did:) throws -> String`) that spins a **per-call multi-thread tokio runtime** inside `resolve_pds_blocking` (Run 7 `loe_note`); shells dispatch on a background thread (named `Thread` + `runOnUiThread`; `DispatchQueue.global(.userInitiated)` + main) — parity on both.
- **Phase 1 decision named by the log:** whether the core owns a **long-lived runtime** (Run 7) — adequate for one probe, not for sync. Android needed `INTERNET` permission (Run 15 C4a); iOS needed nothing (ATS default).
- Nothing landed on Phase 1 pre-decision (vi) from either probe (Runs 15, 19).

---

## 6. OAuth-path readiness (B12)

| Platform | Mechanism | Workarounds | Notes |
|---|---|---|---|
| iOS (Run 20) | `ASWebAuthenticationSession(url:callback: .customScheme("lfs"))`, ephemeral | **none** | No `CFBundleURLTypes` edit (session intercepts its own scheme; scheme registration proper is Phase 2); ephemeral = no consent sheet (a choice; Phase 2 decides cookie-jar sharing). |
| Android (Run 21) | `Intent.ACTION_VIEW` → Chrome; `<intent-filter>` (VIEW/DEFAULT/BROWSABLE, `lfs://oauth/callback`) + `launchMode="singleTask"` + `onNewIntent` | **none** | One environment note: two Chrome first-run cards on a fresh AVD (51 s of the run; not a defect). `ACTION_VIEW` used where R9 names Custom Tabs — **accepted as-is by operator ruling this unit (A-O32; no D-10)**; Custom Tabs is Phase 2 wiring with `atrium-oauth` (same external-user-agent class, RFC 8252). After `installDebug`, force-stop before `am start` (the pre-install instance can survive). |

Placeholder: `docs/phase0/oauth.html` @ `61905d5`, served from GitHub Pages (R-P0-16); tap-to-continue link (user gesture) so the same page serves both platforms.

---

## 7. The two notes (B9, B10) and the sentinel state at close

- **B9 — `docs/bedrock-common-nix-read.md`** (Run 22): a Bedrock host is a public, `auth = "open"`, TLS-fronted WebSocket relay with a first-boot key-seed identity; Caddy access logs are designed to join peer ID ↔ client IP for 14 days. **The Phase 1 self-host decision is about metadata custody, not confidentiality**; staging host (~4 GB) is the minimum-viable self-host shape; WS-only, so a Bedrock-style host is the L-5 fallback, not the primary QUIC path.
- **B10 — `docs/did-plc-route-oi-m4.md`** (Run 22): **OI-M4 closed — record route named**; the `did:plc` verification-method route is verified-available-and-declined (any `did:key` type, no DID control, rotation-key ceremony + 72 h window + permanent public log; the Keyhive doc ID cannot be a verification method). Residue (post-T4 DID-side key commitment) → Phase 4b, spec v0.1.5 cut list.

**Run 23 sentinel-check (this unit, 2026-09-20T21:21:54Z–21:22:31Z, raw/codeload + crates.io API; GitHub API rate-limited after one call as plan §3 predicted):**

| # | Run 0 (15:07Z) | Run 23 (21:22Z) | Moved |
|---|---|---|---|
| S-1 `keyhive_core` crates.io | 0.5.0 (2026-06-26) | 0.5.0 (2026-06-26) | no |
| S-2 `keyhive_core` main | 0.5.0 @ `90fe4a51` (2026-09-17) | 0.5.0; main head `90fe4a51` "Permit an empty BeeKEM tree (#227)" 2026-09-17 (atom feed) | no |
| S-3 PR #230 | open / DRAFT / unmerged, 33 commits, head `keyline_crate` | open / draft / unmerged, 33 commits, `updated_at` 2026-09-14 (API, one call before the limit) | no |
| S-4 `ENVELOPE_TAG` | `kh0` | `kh0` (`carriage.rs` L26) | no |
| S-5 onomancy `keyhive_core` pin | `"0.5"` pre-alpha; `onomancy_keyhive` 0.3.0 | same | no |
| S-6 `_lexicon` TXT | null (operator) | null (no DNS from the container; operator, before Phase 2) | — |
| S-7 `SyncpointMap` | `pub(crate)` | `pub(crate)` at `21b2e6b8` **and** at `main` | no |
| S-8 samod / autosurgeon | 0.14.0 / 0.14.0 | 0.14.0 / 0.14.0 | no |
| S-9 atrium | api 0.25.8 / oauth 0.1.7 / identity 0.1.9 | same | no |
| S-10 Subduction main | git-only; iroh 0.10.2, keyhive 0.8.2, automerge 0.11.0 | same | no |
| S-11 employment-seam HEAD | null | null — operator ruled this unit: own repos, nothing beyond the session's work | — |
| S-12(a) 0.6 / codec statement | null (CHANGELOG 404; `#keyhive` not read) | null — root and `keyhive_core/` CHANGELOG both 404; `#keyhive` not read (?) | — |
| S-12(b) `2026-sept-updates` | exists, 7 commits (2026-09-08), NOT merged | exists (HTTP 200), head `881226c9` 2026-09-08, NOT merged (`keyhive_core/src/lib.rs` diff 4 lines, root `Cargo.toml` diff 49 lines vs main) | no |

**No trigger moved. T4 remains partial-fire (announced, unmerged). S-12(b) stands → Phase 1 does not open before a T4 scoping ruling (plan §3 S-12 rule; restated verbatim in the close handoff).**

---

## 8. Recommended Phase 1 pins and the list of things Phase 1 must decide first

**Pins (recommend carrying Phase 0's lockfile unchanged unless a listed decision moves one):** `keyhive_core =0.5.0` (crates.io) **or** git `90fe4a51` if #226/#232 fixes are required — decide, don't drift; `samod`/`autosurgeon` 0.14.0; Subduction `21b2e6b8`; atrium as pinned; `reqwest rustls-tls`; `uniffi 0.32.1`; `cargo-ndk 4.1.2` / NDK r30; Xcode 27.0 with `IPHONEOS_DEPLOYMENT_TARGET=17.0` in the xcframework task.

**Phase 1 must decide first (each with its source run):**

| # | Decision | Source |
|---|---|---|
| (i)–(vi) | Carried from earlier handoffs by number (~ — not restated verbatim in the Unit 6 handoff; the ones the log names are listed individually below; A-O42) | Unit 6 handoff §4 |
| — | **Gate: T4 scoping ruling before Phase 1 opens** (S-12(b) source-confirmed at Runs 0 and 23) | plan §3; Run 0, 23 |
| — | Rooting level L-12: named interim or wait on §10 item 8 — **ruled at T4 from a re-read of memo §6, never assumed by the build** (carried constraint ii) | plan §8 |
| — | `keyhive_core` crates.io `=0.5.0` vs git rev (#226/#232) | Run 3 |
| — | Mobile TLS backend (rustls + platform verifier / webpki roots / vendored OpenSSL) | Run 13 |
| — | Core-owned long-lived async runtime vs per-call | Run 7 |
| — | iroh FFI in the shells (plan A3: one FFI layer to size) | plan §5 |
| (vii) | **Sync-host custody: self-host vs hosted**, now with the B9 metadata-custody fact; seed-file = host identity (runbook line) | Run 22, B9 |
| (viii) | `keyhive_core` admin signing-key type (A-O37, ed25519 ~) before Half A design; hosted-account rotation-key custody (A-O38) — Phase 2 verify | Run 22, B10 |
| — | Error mapping across FFI: Swift `CoreError` not public vs Kotlin `CoreException` (A-O22) | Run 18 |
| — | Physical Android device (D-8 closes) | Run 14 |
| — | Custom Tabs wiring with `atrium-oauth`; ephemeral vs shared cookie jar on iOS | Runs 20, 21 |
| — | D-5: templates carry TS paths — Phase 1 governance PR | handoff §3 |
| — | `automerge_subduction_ingest` declared? → re-check the 0.11/0.12 automerge pair | Run 4 |

**Carried Phase 1 constraints (operator, 2026-09-18; ~ until Phase 1's own plan adopts them) — verbatim from plan §5:**
> (i) Phase 1's Keyhive surface stays as thin as the pure-verifier "Ask 2" shape allows, so a `kh0`→`kh1` / 0.6 codec migration costs a re-encode of persisted delegation bytes, not a redesign — the DID→key half (Half A) survives such a migration by construction; (ii) **T4 firing is a forced re-read of memo §6, not a green light** — the rooting level (L-12, §10 item 8) is ruled *at* T4 from that re-read, never assumed by the build.

---

## 9. The estimate — Phases 1–3 in ranges (~ throughout; the packet's only inferred section)

**Calibration statement.** Plan v0.1 issued **no planned figure** for Phase 0 (§4: "the estimate is Phase 0's output, not its input"), so the plan-§5 "planned vs actual ratio" **cannot be computed** (✓ — plan text). What can be stated: Phase 0 was scoped as 12 build items across 11 sequence steps and closed in **one operator-day** — ~6.3 h governed wall-clock across 7 harness units, ~51 min toolchain installs outside it, 24 log runs, 5 repo defects (all fixed in-run), 12 harness-instruction defects (H-1..H-12, none touching the repo). If a later plan wants a ratio, the denominator to use is this packet, not an impression.

**Basis for the ranges (~):** Phase 0 is plumbing with no design residue; Phases 1–3 each carry design decisions that Phase 0 explicitly deferred, plus at least one pre-registered validation event. The rate observed here (~2 build items/hour on a governed day, near-zero rework) applies to the plumbing share of each phase only; decisions, ceremonies, and validation events are not rate-limited by toolchain friction and are estimated as sessions, not hours.

| Phase | Scope (spec §8) | Plumbing share, harness-days | Decision / VE share, governed sessions | Range (operator-days) |
|---|---|---|---|---|
| 1 — Private substrate | Keyhive groups, `identity` doc ceremony with cold admin keys, two-delegation floor, profile/pings/threads docs, membership counter, local UI, SQLite storage | 3–5 (first real Keyhive objects on the pinned surface; two shells; Keystore/Keychain — Phase 0 measured none of this) | 3–5 sessions (T4 scoping gate *before* open; rooting interim; TLS backend; runtime; sync-host custody) + physical Android | **5–9**, not opening before the T4 scoping ruling |
| 2 — Identity + rendezvous | atrium OAuth via system browser, DID resolution, Half A record, profile record with iroh ticket, second-device enrollment, direct-QUIC private sync, single-half re-bind text | 4–7 (iroh FFI in shells — the largest unmeasured surface; Custom Tabs; PDS record write; first live sync) | 3–4 sessions + **external recovery drill (VE-6, 10 testers, pre-registered)** | **7–12**, drill scheduling external to the count |
| 3 — Connection seam | `let's-connect` public record, handshake, chain step (§4.1), grant issuance between two users; L-8, L-15, L-17 VEs | 3–5 (two-user, two-device test rig doubles device count) | 3–5 sessions (three VEs, each a run with a pass criterion) | **6–10** |

**Phases 1–3 total: ~18–31 operator-days (~), on the Phase 0 method (one governed context per unit, operator applies, append-only log). Phases 4–6 not estimated from Phase 0 evidence (plan §5).**

What would move these ranges: T4 merging (Phase 1 becomes a re-encode plus a rooting ruling — add 2–4 days and a scoping session); a physical Android device surfacing Keystore/background friction the emulator hid (Phase 1 +1–3); iroh FFI needing a second binding layer rather than riding UniFFI (Phase 2 +2–4).

---

## 10. Observation-log residue (not fixed — append-only)

Noted for the record, no edit proposed: **Run 10** is a blank template block followed by a stray footer (the run was never executed; Run 11 is the first commit); **Run 12** `started_at` / `ended_at` / `elapsed_min` carry `<fill>` placeholders; Run 15's missing closing fence was repaired by Run 16's leading fence (H-10). None affects a number above.

---

## 11. Log entries drafted for operator append (Runs 23–24)

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
ended_at:            <operator: time of append>
elapsed_min:         <operator>
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

---

*Phase 0 LOE Packet · 2026-09-20 · J. Wright / UX Minds, LLC · CONTEXTUAL · SINGLE-CONTEXT — NOT PANELED*
*Delivery-not-application. Canonical files on the operator's machine. Every figure traces to a log field or the plan; §9 is inferred and tagged.*
