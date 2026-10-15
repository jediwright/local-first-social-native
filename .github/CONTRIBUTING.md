# Contributing to local-first-social-native

This repository follows the [Governed PR Framework v0.3](https://github.com/jediwright/governed-pr-framework). This file is a governed derivative of that framework; inheritance rules are in §8 of its `FRAMEWORK.md`. You can submit a correct change from this file and the pull request template alone.

---

## What this repository is

A small native iOS and Android app with a shared Rust core, for a local-first social network (working name). The core (`core/crates/lfs_core`) holds Automerge documents, SQLite storage, a Keyhive identity and group membership, and a UniFFI surface. The shells (`apps/ios`, `apps/android`) render it and hold the device's keys in platform storage.

Two branches, with different rules:

- **`main`** is the tested app. Everything in this file applies to it.
- **`frontier`** is for experiments. It has its own rules in [`docs/frontier/CHARTER.md`](../docs/frontier/CHARTER.md), and **nothing from it is ever merged into `main`.** Work that proves out there comes back as a fresh build on `main`, written from `main`'s current head and citing the experiment (`F-n`) and any gap (`G-n`) it relies on.

---

## Review scale

This repository operates at **S1: solo author with an AI assistant** (framework §3). I am the only author. A change is reviewed in a separate session from the one that produced it, against the written plan and its acceptance criteria. That is how K2 (reviewed outside the authoring context) is met here.

A fresh session is independent in context but not in kind. For Critical changes, the compensating check is external: the design claim is checked against the upstream project concerned (Keyhive, Automerge, AT Protocol, the platform documentation) before the change lands. For anything network-facing, at least one check involves a party outside the local environment before the phase closes.

Outside contributions are welcome as pull requests. They are reviewed the same way, by me, in a separate session.

---

## How a build lands on `main`

Most work on `main` lands as a numbered **build run**, committed directly by me, not as a pull request:

1. **Plan first.** A run starts from a written plan with acceptance criteria. Design plans live off-repo; the observation log and the evidence notes are the public record.
2. **Step by step.** Each step's expected file changes are written down before it is applied, then compared with what actually changed, and read back before the next step.
3. **Stop on a dependency change.** Any change to `core/Cargo.toml`, `core/crates/*/Cargo.toml` or `core/Cargo.lock` stops the build until it has been looked at and ruled on, with the lockfile diff listed in the log.
4. **The Keyhive version is held fixed** within a run. Moving it is its own Critical change.
5. **Generated bindings change by addition only.** Removing or changing an exported item is a Critical change.
6. **One observation-log entry per run.** Every build that changes the core, the bindings or a shell gets an entry in [`docs/phase0-observation-log.md`](../docs/phase0-observation-log.md): toolchain, pins, outcome, defects. The log is append-only.

Commit messages follow the existing history: `Run N: <what changed>` for a build run, `Log: Run N` for its log entry, `Docs: …` for documentation and `Chore: …` for tooling. A pull request, when one is used, carries the same information in the template.

---

## Four requirements on every change

These apply to every change, whatever its tier (framework §1):

- **K1, tier declared honestly.** The change states its tier. The reviewer's first check is whether that tier matches the diff.
- **K2, reviewed outside the authoring context.** Review in the session that wrote the change is proofreading, not review (see Review scale above).
- **K3, automated checks green, not attested.** Every check in the mechanized layer below passes. A check that is skipped is declared, with the reason.
- **K4, no `Unverified` claim on a protected surface.** See Verification tags below.

---

## Change tiers

Tier is set by **how far a failure would spread**, not by line count.

| Tier | Use it when | Examples here |
|---|---|---|
| **Low-risk** | Failure would be local and visible. | Copy, a comment, a test-only change, a shell layout tweak, README edits. |
| **Feature** | Failure could spread within one area. | A new document type, a new FFI function (added, not changed), a shell screen, a module refactor. |
| **Critical** | Failure could be silent or unrecoverable, or the change touches a protected surface below. | Anything in the protected surfaces list, including every dependency change. |

**Tier disputes resolve upward.** If the author's tier and the reviewer's tier differ, including across two sessions, the higher one applies.

---

## Protected surfaces

A change touching any of these is **automatically Critical**, whatever its size. K4 applies: no claim about a protected surface may carry an `Unverified` tag.

**Stored formats and the storage adapter**

- `core/crates/lfs_core/src/storage.rs`: the `DocStore` trait, the SQLite schema and its migration, and the format-tag constants (`automerge.save.v1`, `keyhive.static-delegations.bincode.v1`, `keyhive.static-events.bincode.v1`).
- Every persisted byte reaches SQLite through this one adapter, which stamps the tag. A tag names what the bytes are. A new shape gets a new tag value, and existing rows are never re-tagged.
- Anything that changes which rows are written or what they hold: `identity`, `identity-keyops`, and the group pair `group:<id>` / `group:<id>.keyops`.

**Key custody**

- The identity ceremony (`ceremony.rs`): the cold admin and recovery seeds and the device seed are returned once across the FFI boundary and never persisted.
- The single seed-to-signer constructor (`signer_from_seed` in `groups.rs`) and recovery from the cold key (`recover_identity`).
- Seed custody in the shells: the Keychain items on iOS (`ContentView.swift`, service `social.localfirst.shell`) and the AndroidKeyStore-wrapped seed on Android (`MainActivity.kt`, alias `lfs-device-seed-wrap`).
- The test `cold_keys_are_exported_and_never_persisted` is only ever extended. If a change alters what gets stored, the test grows to cover it. It is never loosened or skipped.

**Consumer policy**

- `policy.rs`: the two-delegation floor, the "at least Admin to grant" bar and the revoke bar. These are this app's own policy, checked before Keyhive is called.
- `RootingLevel` arrives as configuration from the shells and is never a constant in the core.

**Identity and group membership**

- `ceremony.rs` and `groups.rs`: who is delegated what at the ceremony, how groups persist and reload, how the identity document co-parents groups, and how recovery migrates them.

**The FFI surface**

- Every `#[uniffi::export]` function and every UniFFI record and enum. Adding is Feature tier; removing or changing one is Critical. Generated bindings are not committed.

**Dependency versions**

- Any change to a manifest or the lockfile is Critical and stops the build (see above). Record the lockfile diff and the `cargo audit` result.
- The pinned stack, held fixed between deliberate moves (read from `core/crates/lfs_core/Cargo.toml` at `24c31a0`):

  | Dependency | Pin |
  |---|---|
  | `keyhive_core`, `keyhive_crypto` | git `inkandswitch/keyhive` rev `90fe4a51` |
  | `automerge` / `samod` / `autosurgeon` | `0.12` / `0.14` / `0.14` |
  | `subduction_keyhive`, `subduction_iroh` | git `inkandswitch/subduction` rev `21b2e6b8`, optional, off by default |
  | `uniffi` | `0.32` |

- `PIN_LABEL_KEYHIVE_CORE` in `storage.rs` must name the same Keyhive version and rev.

**Network and TLS**

- `resolve.rs` and the `reqwest` configuration: rustls with webpki roots, not the OS trust store. This is a recorded choice; changing the TLS backend or the trust roots is Critical.

**App identity**

- Android `applicationId` `social.localfirst.shell`, the iOS bundle identifier `com.uxminds.lfs.LfsShell` and the Keychain service name. Changing any of them orphans stored data and custodied seeds. The `frontier` branch uses its own `.frontier` identifiers so its builds can never touch this app's data.

**Governance configuration**

- This file, `.github/PULL_REQUEST_TEMPLATE.md`, `.gitleaks.toml` and `.github/workflows/guard.yml`. Declaring, widening or changing a protected surface or a pre-cleared class is itself Critical (framework §2.3).

---

## Pre-cleared change classes

**None declared.** A class is created only by a Critical change that declares it, its machine-checkable boundary and its review interval (framework §2.3).

---

## The mechanized layer

Enforced by tooling:

- **Secrets scan, CI:** `.github/workflows/guard.yml` runs `gitleaks` over the full history on every push to `main` and `frontier`. The config (`.gitleaks.toml`) adds a rule for bare 32-byte hex seeds, which the default rules miss.
- **Dependency audit, CI:** the same workflow runs `cargo audit` (cargo-audit 0.22.2) in `core/` on every push to `main` and `frontier`. A vulnerability advisory fails the job. Warnings (unmaintained, unsound, yanked) are allowed and do not fail it, so each one is still looked at and recorded by hand.
- **Runner image:** on `main`, both jobs run on `ubuntu-24.04` rather than `ubuntu-latest`, which GitHub moves to Ubuntu 26.04 between 2026-10-19 and 2026-11-19. Pinned 2026-10-15. Time-sensitive: review the pin when GitHub announces the deprecation of `ubuntu-24.04`, or on 2027-01-15, whichever comes first. The pin fixes the OS image, not the Rust version: the audit job builds `cargo-audit` with whatever Rust that image carries, which can still change.
- **Secrets scan, local:** a `gitleaks` pre-commit hook on my machine scans each staged change. Hooks are not versioned; this one is expected on any machine that commits here.

Run by hand before a change lands, with the result recorded in the log entry or the pull request:

- **Core tests:** `cargo make test` (`cargo test -p lfs_core`). All must pass; record the pass, fail and ignored counts.
- **Dependency audit:** `cargo audit` in `core/` on any dependency change, before the change is committed. CI repeats it after the push; the hand run is the one the record cites. Every advisory is looked at and recorded.
- **Device builds:** `cargo make android-so`, `bindgen-kotlin` and `swift-xcframework` (see [`BUILDING.md`](../BUILDING.md)), then the run's device checks on the Android emulator and an iPhone for any change that reaches a shell.

Known gaps, each a candidate to move into CI: core tests do not run in CI on either branch. Until they do, they are run by hand and their results are written down. If you find another rule held by memory alone, adding it here is part of your change (see Self-healing conventions below).

---

## Verification tags

Every substantive claim in a log entry or pull request is tagged:

- **Confirmed:** verified. Say against what (a test run, a device run, a pinned version) and when.
- **Inferred:** follows from something confirmed. State the reasoning.
- **Unverified:** assumed. Must carry a closure plan: what will be checked, and by when. Not allowed on a protected surface.
- **Time-sensitive:** true now. Name what will make it expire.

A claim confirmed against the pinned Keyhive version is time-sensitive: it holds for that version only. Name the version.

---

## Self-healing conventions (framework §7)

If a change surfaces something undocumented (a convention, a failure mode, a protected surface not listed above, a dependency quirk), updating this file or the relevant document is part of the same change. Code and documentation ship together.

---

## Emergency changes

For a live security issue that cannot wait: act first, then record it within **1 business day** as a Critical change with honest after-the-fact tags, in the observation log or a retrospective pull request. The emergency path moves the order of the steps; it does not remove any of them. See framework §10.1.

---

*This file is a governed derivative of the Governed PR Framework v0.3 (J. Wright · UX Minds, LLC). Revised 2026-10-14 to match this repository; revised 2026-10-15 to record the dependency audit in CI on `main` and the runner-image pin.*
