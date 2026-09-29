# Frontier branch — charter

This branch is a working space for trying things ahead of the tested app on `main`. It exists for two reasons:

1. **To move quickly.** On `main`, every build goes through a slow, step-by-step review. Here, most of those checks are relaxed so ideas can be tried and thrown away cheaply. A short set of checks still runs whenever an experiment ends (see [Checks at the end of every experiment](#checks-at-the-end-of-every-experiment)).
2. **To find gaps in the tools this app is built on.** Where the app runs into a limit in the [Ink & Switch](https://www.inkandswitch.com/) local-first stack (Keyhive first), the limit is written down in [`gap-register.md`](gap-register.md), worked around in the app where possible, and shared upstream when that would help.

The frontier sits beside the main line of work, not in place of it. Tested work continues on `main` as before.

## How it relates to `main`

- This branch starts from `main` and shares its history. The Phase 1 end point is tagged `phase1-exit`.
- **Nothing from this branch is ever merged into `main`.** Work that proves out here is rebuilt on `main` from scratch, through `main`'s normal process (see [Bringing work back to `main`](#bringing-work-back-to-main)).
- Builds from this branch install as separate apps with their own storage. They cannot read or change the data of the app built from `main`.

## Rules that always hold here

These are never relaxed, however experimental the work.

1. **No merge into `main`.** Anything kept goes back as a fresh build on `main`.
2. **Secrets are scanned automatically:** by a pre-commit hook on the developer's machine, by CI on every push, at the end of every experiment, and before anything is offered upstream. The scanner config (`.gitleaks.toml`) includes a rule for bare 32-byte hex seeds, which the default rules miss.
3. **The test `cold_keys_are_exported_and_never_persisted` is only ever extended.** If an experiment changes what gets stored, the test grows to cover it. It is never loosened or skipped.
4. **Separate app IDs on every build.** Android `social.localfirst.shell.frontier`; iOS `com.uxminds.lfs.LfsShell.frontier`; iOS Keychain service `social.localfirst.shell.frontier`.
5. **Test identities only.** No real identity is set up, and no real AT Protocol account is signed into, on a frontier build.
6. **No seed is ever written into the code.** Tests generate seeds when they run, or derive them from a label when they run. Screenshots and debug output that show key material stay off the repo.
7. **Debug screens stay out of release builds.** Any screen that shows or deletes secrets is behind a build flag and never compiles into a release configuration.
8. **The Keyhive version is recorded for every experiment,** including any local patch applied on top of it.
9. **Failures are kept.** Every experiment captures its broken states as well as its working end point, under [`evidence/`](evidence/).
10. **Public identifiers are shortened.** Key fingerprints appear as prefixes only. No full public key tied to a real person is published.
11. **Dependencies are audited** (`cargo audit`) at the end of every experiment and in CI on this branch.

## What's relaxed here

On `main`, each build step is checked before it lands: predicted file changes and checksums are compared, a change to the dependency manifest or lockfile stops the build, the Keyhive version is held fixed, and every run is written to a shared observation log. None of that applies on this branch:

- file-by-file checksum checks on each step;
- predicted file and bindings changes;
- stopping when the manifest or lockfile changes (replaced by the dependency audit in rule 11);
- holding the Keyhive version fixed (it may move, but it is recorded, per rule 8);
- the source-code search counts `main` uses to confirm certain names stay in one place;
- requiring changes to the generated bindings to be additions only;
- the shared observation log. This branch keeps its own log, [`frontier-log.md`](frontier-log.md); `docs/phase0-observation-log.md` is not written to from here;
- the read-back after each step.

## How an experiment runs

**Question → build or break freely → capture what happened → outcome.**

Each experiment ends with one of three outcomes:

- **Promote:** it becomes a candidate for a fresh build on `main`.
- **Park:** it stays on this branch, logged, and isn't pursued for now.
- **Drop:** it's reverted or left in place, logged with the reason.

Every experiment gets one entry in [`frontier-log.md`](frontier-log.md), using the template there.

## Checks at the end of every experiment

These run before an outcome is logged. All must pass. If one fails, stop and fix it before going on.

1. `gitleaks git . -c .gitleaks.toml` reports no leaks.
2. `cargo audit` in `core/` reports no advisories that haven't been looked at.
3. `cold_keys_are_exported_and_never_persisted` passes, and has been extended if the experiment changed what gets stored.
4. The Keyhive version, and any local patch, is recorded in the log entry.
5. `git diff --stat main...frontier -- apps/` still shows the frontier app IDs in place (rule 4).

## Working with Keyhive gaps

Go only as far down this list as a gap needs.

| Level | What | Where it lives | When |
|---|---|---|---|
| 0 | Work around it in the app | Code on this branch | The default |
| 1 | Patch a local copy of Keyhive at the pinned version; frontier builds use it through a Cargo `[patch]` override | The developer's machine; the diff is saved to `patches/<gap-id>.patch` | When testing a gap needs a change inside Keyhive |
| 2 | Fork Keyhive on GitHub and open a pull request upstream | A fork | Only by deliberate choice |

**Before a level 1 patch:**

- Check whether the gap still exists on Keyhive's latest code. It may already be fixed or redesigned.
- Ask in the `#keyhive` channel whether the behavior is intended.
- Check Keyhive's license terms before a patch file is published on this branch. This repo is Apache-2.0.

**Before level 2:** scan the fork's history for secrets (rule 2).

## Bringing work back to `main`

A promoted experiment becomes an ordinary build on `main`:

- it starts with a plan that cites the experiment (`F-n`) and any gap (`G-n`);
- the code is written fresh from the current head of `main`, not cherry-picked from this branch, so `main`'s checks see a clean change;
- it goes through `main`'s full build process;
- it gets an entry in `docs/phase0-observation-log.md`.

The frontier code is an input to that build, never its output.

## Why this is safe

Code here may be deliberately insecure, for example skipping a permission check or signing a delegation by hand. That is acceptable only because four protections hold together:

- rule 1 keeps it off `main`;
- rule 4 keeps it away from the tested app's stored data;
- rule 5 keeps it away from real identities;
- the notice at the top of the README keeps readers from mistaking this branch for the tested app.

Drop any one of them and the risk comes back.

Secrets are protected by tools rather than by review (rules 2, 3 and 6). Dependency review moves from stopping on lockfile changes to auditing (rule 11).

**If a secret ever reaches this public branch:** rotate the key first, then rewrite history. Rewriting history can't recall clones, caches or forks, which is why rules 5 and 6 are the main defense.

## Keeping this private

Anything involving a real identity, a real AT Protocol account, or a Keyhive patch whose license handling isn't settled stays on the developer's machine, or in a private scratch repo, until it's cleared for this branch.

## Files in `docs/frontier/`

| File or folder | Holds |
|---|---|
| `CHARTER.md` | This document |
| `frontier-log.md` | One entry per experiment |
| `gap-register.md` | Gaps found in the Ink & Switch stack |
| `patches/` | Level 1 Keyhive diffs, each with a header naming its gap |
| `evidence/` | Status text and redacted screenshots only |
