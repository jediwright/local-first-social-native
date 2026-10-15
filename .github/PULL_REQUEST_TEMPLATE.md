<!--
Pull request template · local-first-social-native
Governed derivative of the Governed PR Framework v0.3. Full rules: .github/CONTRIBUTING.md

1. Declare the tier. 2. Fill only the sections that tier needs. 3. Delete the rest.
Tier disputes resolve upward. Anything on a protected surface (CONTRIBUTING.md) is Critical,
and every dependency change is Critical.
This repository's review scale is S1 (solo author with an AI assistant): review happens in a
separate session from the one that wrote the change.
Target branch is main. Nothing from the frontier branch is merged into main.

Emergency (live security issue): act first, then open this as a retrospective Critical PR
within 1 business day, with honest after-the-fact tags.
-->

## Change tier
<!-- Pick ONE.
     Low-risk — failure local and visible: copy, comments, test-only change, shell layout, README.
     Feature  — failure could spread within one area: new document type, new FFI function
                (added, not changed), shell screen, module refactor.
     Critical — failure could be silent or unrecoverable, including:
                • stored formats, format tags, the SQLite schema (storage.rs)
                • key custody: ceremony seeds, signer_from_seed, recover_identity,
                  Keychain / AndroidKeyStore seed handling
                • consumer policy (policy.rs): delegation floor, grant and revoke bars
                • identity and group membership (ceremony.rs, groups.rs)
                • removing or changing an FFI item
                • any Cargo.toml or Cargo.lock change, including the Keyhive pin
                • TLS backend or trust roots (resolve.rs, reqwest config)
                • app identifiers or the Keychain service name
                • CONTRIBUTING.md, this template, .gitleaks.toml, guard.yml -->

**Tier:** Low-risk / Feature / Critical

---

## Intent
<!-- ALL TIERS. What this changes and why, readable without opening the diff.
     Example: "Adds a fourth document type, 'notes', behind open_typed_doc so the shells
     can store a free-text note per profile. New FFI record NotesDoc; no stored format
     changes for the existing three types." -->



## Checks
<!-- ALL TIERS. Paste result lines, don't tick boxes. Declare any skipped check and why. -->

- `gitleaks` (pre-commit hook; CI on push):
- `cargo make test` (pass / fail / ignored):
- `cargo audit` (any dependency change; every advisory looked at):
- Device checks (any change that reaches a shell; Android emulator / iPhone):
- Observation-log entry (any change to the core, bindings or a shell): Run

<!-- ============ LOW-RISK STOPS HERE. Below is Feature and Critical. ============ -->

## Acceptance criteria  <!-- Feature + Critical -->
<!-- Written BEFORE the code. Given / when / then, each pointing at where it is met.
     Example: "Given a store holding rows written before this change, when the core
     reopens it, then every row decodes under its existing tag and the profile reloads
     unchanged → met in storage.rs tests + device relaunch check." -->

- [ ] Given … when … then … → met in: …
- [ ] Given … when … then … → met in: …

## Verification status  <!-- Feature + Critical -->
<!-- Tag every substantive claim.
     Confirmed      — verified; say against WHAT (test run, device run, pinned version) and when
     Inferred       — follows from something confirmed; state the reasoning
     Unverified     — assumed; MUST include a closure plan (what you'll check, by when).
                      Not allowed on a protected surface.
     Time-sensitive — true now; name what will make it expire
     Claims confirmed against the pinned Keyhive version are Time-sensitive: name the rev. -->

| Claim | Tag | Evidence / reasoning / closure plan |
|---|---|---|
|  |  |  |

## Not in this PR
<!-- One line: what you left out on purpose, and why that is safe. -->

**Not in this PR:** … — **safe to defer because:** …

## Pre-mortem  <!-- Feature + Critical -->
<!-- "It's two weeks from now and this change caused an incident. What was it?"
     For storage or key changes, consider specifically: could a row written before this
     change fail to load after it? Could a seed or secret now reach the database, a log
     or a screenshot? Could a grant or revoke now pass a check it should fail? -->



<!-- ============ FEATURE STOPS HERE. Below is Critical only. ============ -->

## Convention check  <!-- Critical -->
<!-- Link the document that governs any stored format, key format, protocol or type convention
     touched: the format-tag comments in storage.rs, the consumer evidence note, the upstream
     Keyhive / Automerge / AT Protocol source. If none exists yet, writing it is part of this PR. -->

- Governing document(s):
- New or updated document in this PR (if a gap was found):

## Upstream check  <!-- Critical -->
<!-- At this scale the upstream project is the second reviewer for boundary claims.
     What was checked against Keyhive, Automerge, AT Protocol or platform documentation,
     at which version or commit, and what it showed. -->

- Checked against:
- Result:

## Dependency changes  <!-- Critical, any Cargo.toml or Cargo.lock change -->
<!-- List every lockfile change: packages added, removed, versions moved, and any existing
     entries re-linked. Paste the cargo audit summary. If the Keyhive rev moves, show that rows
     written at the old rev still load at the new one, and update PIN_LABEL_KEYHIVE_CORE. -->

- Lockfile changes:
- `cargo audit`:
- Stored rows from the previous pin (if the Keyhive rev moved):

## Decision record  <!-- Critical, only if this settles a question that would fail silently if wrong -->
<!-- Context / decision / alternatives / consequences. A new format tag, a change to what is
     persisted, or a change to the policy bars almost always needs one. -->

- Decision record:

## Boundary questions  <!-- Critical, protected surfaces only -->
<!-- Does this change what any party can SEE, FORGE, REPLAY or DENY?
     "No change" is a fine answer. A blank is not.
     For key custody: can any seed or secret now be read, copied or persisted where it couldn't
     before? For policy: can anyone below the bar now grant or revoke? -->

- See:
- Forge:
- Replay:
- Deny:

## Deletion justification  <!-- Critical, only if code is removed -->
<!-- Why did the removed code exist? "Nobody knows" is a documentation gap to record,
     not permission to delete. -->



## Verification chain  <!-- Critical, bug fixes only -->

- Symptom (precise):
- Root cause (confirmed, not guessed):
- How it was confirmed (hypothesis → minimal test → result):

---

## Review  <!-- completed in a separate session from the one that wrote the change -->

- **Tier matches the diff:** <!-- yes | disputed → resolves upward -->
- **Reviewed outside the authoring session, against:** <!-- the plan and its acceptance criteria -->
- **Checks green (not attested):** <!-- yes | skipped check declared above with reason -->
- **No Unverified tags on protected surfaces:** <!-- confirmed | n/a -->

---
<!-- Before merging:
     · Rebased across a Cargo.lock change? Confirmed tags that depend on the stack are stale; re-verify.
     · Open more than 14 days? Re-verify Confirmed tags rather than just rebasing.
     · Storage or key change: cold_keys_are_exported_and_never_persisted extended, never loosened. -->
