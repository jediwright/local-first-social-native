# Frontier log

One entry per experiment on the `frontier` branch. How experiments run and the checks each one ends with are in [`CHARTER.md`](CHARTER.md). Gaps found in Keyhive or the wider stack go in [`gap-register.md`](gap-register.md) and are linked from here by their `G-n` number.

Entries are added in order and not rewritten afterward. If a later experiment changes what an earlier one found, add a new entry that says so.

## Entry template

```
exp:          F-<n> — <short name>
date:         <YYYY-MM-DD, America/New_York>
base:         frontier@<short hash> · keyhive <pinned rev> (+ local patch patches/<G-n>.patch, if any)
question:     <one line: what this experiment is trying to find out>
tried:        <what was built or changed>
result:       <what broke and what held; broken states saved under evidence/F-<n>/>
gap:          <G-n, or none>
outcome:      promote | park | drop — <one line on why>
checks:       gitleaks <clean | findings> · cargo audit <clean | notes> ·
              cold_keys_are_exported_and_never_persisted <pass; extended? yes/no> ·
              keyhive version recorded <yes> · app IDs unchanged <yes>
```

Notes on the fields:

- **base:** the commit the experiment started from, plus the exact Keyhive version in use.
- **result:** say what failed as well as what worked. Screenshots and captured output go in `evidence/F-<n>/`, with key material removed and fingerprints shortened to prefixes.
- **checks:** all five must pass before the outcome is recorded.

## Entries

```
exp:          F-1 — encrypted content on a document rebuilt from storage
date:         2026-09-29
base:         frontier@23c7373 · keyhive 35460ba1 (moved from 90fe4a51 in step 1; no local patch)
question:     Can the app keep reading and writing encrypted Keyhive documents after a
              restart, without storing key secrets unprotected?
tried:        1. Moved keyhive_core and keyhive_crypto from 90fe4a51 to 35460ba1, Keyhive's
                 current main and 16 commits later, and fixed the build. Checked that rows
                 stored at 90fe4a51 decode, re-encode byte for byte, and reload the same
                 members at the same access levels.
              2. Added a stored row type, keyhive.doc-events.bincode.v1, holding a document's
                 public events, its group-encryption operations included. With test
                 identities, a device made a document, shared it with a second member,
                 encrypted content, and stored the row.
              3. Rebuilt the device from storage and replayed the row in several ways, one
                 change at a time:
                 a. without the device's secrets;
                 b. keeping the device's own key events in the replay, then importing its
                    secrets after the replay;
                 c. importing the device's secrets before the replay;
                 d. as c, and also storing the added member's key event, taken from its
                    contact card, beside Keyhive's own export for the device;
                 e. as d, across two restarts with a key update between them, restoring
                    secrets saved before the update and then secrets saved after it.
              4. After a reload without secrets, had the device issue a key update.
              5. After a replay, imported the device's exported secrets and retried reading.
              6. Extended cold_keys_are_exported_and_never_persisted to write the new row and
                 check that no seed or secret reaches the database. Every run also checked the
                 store file for the seed and each exported secret. Identifiers in the test
                 output and evidence were shortened to prefixes.
              7. Kept the device's secrets in each platform's protected storage in the app,
                 debug builds only: wrapped under an AndroidKeyStore key on the Android
                 emulator (software-backed there), and as Keychain items, class
                 WhenUnlockedThisDeviceOnly, on an iPhone 17 Pro. After every change the app
                 saved the secrets, then committed the rows; on relaunch it restored the
                 secrets before replay. On both devices, staged the failures: each replay mode
                 from step 3, rows committed without re-saving the secrets (after a write, and
                 after a key update), and secrets saved without committing the rows, each
                 followed by a kill and relaunch. Checked that nothing F-1 compiles into a
                 release build, and that no plaintext test content reaches the device store.
result:       Held:
              - Rows written at 90fe4a51 read back unchanged at 35460ba1 (step 1).
              - With the added member's key event stored beside Keyhive's export and the
                device's current secrets imported, a device rebuilt from storage reads content
                written before and after restarts, in the core (3d, 3e) and on both devices
                (7a, 7b). The order of import and replay doesn't matter then: importing after
                replay works as well as before (7c on both devices).
              - Only a document's first encryption and an explicit key update change the
                secrets; ordinary writes don't (7b, 7f write). Secrets saved before a key
                update read everything encrypted before it and nothing after it (3e, 7f
                rotate, extra run).
              - Saving the secrets before committing the rows survives a kill between the two
                (7e). Committing the rows first fails only when a key update landed unsaved
                (7f rotate).
              - No seed or secret in any store file; the extended key-safety test passes
                (step 6); no plaintext test content in either device store.
              Broke:
              - Without the device's secrets at replay (3a, 4, 5; 7c on both devices), or with
                them imported after replay while the member's key event was missing (3b), the
                encryption state doesn't rebuild: the device's own group-encryption operations
                stay pending (UnknownInvitePrekey, UnexpectedInitialOperation,
                OutOfOrderOperation; all of them, 4 at cgka 4 on Android and 5 at cgka 5 on
                the iPhone), reads fail with KeyNotFound, and a key update fails.
              - Without the member's key event, the device refuses its own delegation of the
                member (UnknownAgent), the member's add waits (PendingCgkaAuthorization), and
                reads fail (3c; 7d on both devices). This is G-5.
              Notes: each restore builds a fresh hive holding 7 new key pairs of its own, so
              the exported count grows by 7 per relaunch without affecting reads. Protection:
              software-backed AndroidKeyStore on the emulator; on the iPhone, Keychain items of
              class WhenUnlockedThisDeviceOnly, Secure Enclave present, passcode set.
              Evidence: evidence/F-1/ (steps2-5-core*, step7-android-*, step7-ios-*).
gap:          G-3, G-4, G-5
outcome:      park — works under the stated conditions, but depends on the G-5 workaround;
              promote once G-5's cause is settled by a Keyhive-only reproduction
checks:       gitleaks clean (76 commits) · cargo audit clean: no vulnerabilities; 4 allowed
              warnings (bincode, derivative, paste unmaintained; lru unsound), all also on
              main · cold_keys_are_exported_and_never_persisted pass; extended yes (steps 6
              and 7) · keyhive version recorded yes (keyhive_core, keyhive_crypto and beekem
              at 35460ba1; crates.io keyhive_core 0.5.0 locked only for the optional
              subduction feature, not in the default build) · app IDs unchanged yes
```

```
exp:          F-2 — G-5's cause, tested with a local change to Keyhive
date:         2026-09-30
base:         frontier@be2b685 · keyhive 35460ba1 (+ local patch patches/G-5-topsort-fix.patch,
              with patches/G-5-repro.patch; in a Keyhive checkout only, no app code)
question:     Is KeyOp::topsort leaving out a contact card's lone rotate the whole cause of G-5?
tried:        In a local Keyhive checkout at 35460ba1, changed KeyOp::topsort to also start from
              rotations whose earlier key isn't in the set (10 lines added, none removed). Reran
              the G-5 reproduction and all of keyhive_core's own tests. No app code or app build
              changed.
result:       Held:
              - All 9 reproduction tests pass; 4 failed without the change. The sort keeps a
                card's lone rotate (1 in, 1 out). The device's export carries the key event of a
                member introduced by contact card. An instance rebuilt from that export applies
                every event (0 pending), the member has Edit, and content decrypts.
              - The relay case from Keyhive issue #206: the relay applies all of the sender's
                events (0 pending), learns the member, and the member reads.
              - keyhive_core's own tests: 269 pass, 0 fail, 3 already ignored upstream.
              Broke: nothing.
              Evidence: evidence/G-5/run3.txt, evidence/G-5/run3-suite.txt.
gap:          G-5
outcome:      park — the fix belongs in Keyhive, not this app; offered on #206; the app keeps its
              workaround until Keyhive changes
checks:       gitleaks clean (80 commits) · cargo audit clean: no vulnerabilities; 4 allowed
              warnings, unchanged from F-1 · cold_keys_are_exported_and_never_persisted pass;
              extended no (no app change) · keyhive version recorded yes (35460ba1; the patch
              applied only in a local Keyhive checkout, not in the app's build) · app IDs
              unchanged yes
```

```
exp:          F-1 — outcome revised: promote
date:         2026-09-30
base:         frontier@be2b685 · keyhive 35460ba1 (no local patch in the app)
question:     F-1 was parked until G-5's cause was settled. Is it settled?
tried:        Nothing new in the app. F-2 tested the cause in Keyhive alone.
result:       G-5's cause is KeyOp::topsort leaving out a contact card's lone rotate; one change
              to the sort clears every failing case (F-2). The app's workaround, storing each
              added member's contact-card event beside Keyhive's export, supplies exactly the
              event the sort leaves out. If Keyhive fixes the sort, those stored events become
              redundant, not wrong.
gap:          G-3, G-5
outcome:      promote — the condition F-1 was parked on is met; a fresh build on main would keep
              the workaround and cite G-5 and Keyhive issue #206
checks:       gitleaks clean (80 commits) · cargo audit clean: no vulnerabilities; 4 allowed
              warnings, unchanged from F-1 · cold_keys_are_exported_and_never_persisted pass;
              extended no (no app change) · keyhive version recorded yes (35460ba1; the patch
              applied only in a local Keyhive checkout, not in the app's build) · app IDs
              unchanged yes
```

```
exp:          F-3 — Keyhive moved to the G-5 fix
date:         2026-10-14
base:         frontier@3fc4f0e · keyhive 35460ba1 → 2bb83d6 (no local patch)
question:     At 2bb83d6, does the app build, do its stored rows load, and does Keyhive's own
              export now carry members the device knows only by contact card?
tried:        1. At 35460ba1, extended the cross-version check. Besides the identity rows, it
                 now writes and reloads one group row pair with two grants, and the F-1
                 document rows (events, member key events, stored ciphertexts) after a write,
                 a key update and a write. The test identities' seeds and secrets it needs to
                 read them back are kept outside the repository. Wrote the rows, and ran the
                 step 7 dry run as the baseline.
              2. Moved keyhive_core, keyhive_crypto and beekem from 35460ba1 to 2bb83d6, the
                 merge of Keyhive pull request #260, and fixed the build.
              3. Read the rows written at 35460ba1 with the build at 2bb83d6.
              4. Ran the full lfs_core suite.
              5. Reran the step 7 dry run, and made it assert that a restore leaving out the
                 stored member key events has nothing pending and reads.
              6. In a Keyhive checkout at 2bb83d6, ran the seven-case G-5 reproduction and
                 all of keyhive_core's own tests.
              7. On the Android emulator and the iPhone 17 Pro, installed the new build over
                 the F-1 install and relaunched, then ran 7a, 7b and 7d from fresh setups.
              The stored member key events (the G-5 workaround) were kept throughout.
result:       Held:
              - The build broke only where expected: Keyhive now takes nonempty 0.12, and the
                app passed a 0.10 NonEmpty to generate_doc at four sites. Moving the app's
                direct nonempty to 0.12 fixed it. One package added to the lockfile,
                keyhive_codec 0.1.0; none removed (step 2).
              - Rows written at 35460ba1 load at 2bb83d6. The identity rows, the group row
                pair, and the document's events, member key-event and ciphertext rows all
                decode and re-encode byte for byte; the group has the same three members at
                the same levels; the document replays with nothing pending and all three
                stored contents decrypt (step 3). On both devices, the document F-1 left
                there reads exactly as it did at 35460ba1, including a read F-1 broke on
                purpose.
              - lfs_core: 50 pass, 0 fail, 12 ignored, as at 35460ba1, including the grant
                and revoke, group reload, recovery and key-safety tests (step 4).
              - G-5 is fixed for the app. With the stored member key events left out, the
                device's own export now carries the member it knows only by contact card:
                the restore has nothing pending and reads, in the core (step 5) and on both
                devices (7d). At 35460ba1 the same restore left 3 events pending and failed
                with KeyNotFound (step 1). The other restore modes behave as before.
              - In Keyhive alone at 2bb83d6, all seven reproduction cases pass, the relay
                case included. keyhive_core: 275 pass, 0 fail, 3 ignored upstream, not
                counting the seven. The 3 more than at a691f34 are one new test each from
                pull requests #250 and #254 and the relay test added on #260 (step 6).
              Broke: nothing in the app. The first 7d attempt on the Android emulator closed
              the app before setup while an emulator system service was crash-looping; it was
              rerun from a clear, and both attempts are kept.
              Not followed: Keyhive's main moved to 585e6fc (keyhive_core 0.7.0, beekem 0.5.0)
              on 2026-10-13. Those merges add fields to the group-encryption operations that
              the document-events row stores, so that row will very likely not decode there.
              Moving past 2bb83d6 is a separate experiment.
              Evidence: evidence/F-3/.
gap:          G-5
outcome:      promote — the app builds and its stored rows load at 2bb83d6, and Keyhive's own
              export now carries what the G-5 workaround stores; the workaround stays until a
              separate experiment removes it
checks:       gitleaks clean (87 commits) · cargo audit: 4 advisories in rustls 0.23.45
              (RUSTSEC-2026-0346 to -0349, published 2026-10-14), looked at. The same version
              is on main and was here before this experiment, reached through reqwest (and
              iroh/quinn under the optional subduction feature). None applies as built: the
              app has no custom TLS session store (0346), configures no ECH (0347), doesn't
              compile the brotli feature (0348), and uses the ring provider, which offers no
              post-quantum group to lose (0349). Fixed in 0.23.46; to be moved separately, on
              both branches. Allowed warnings: the 4 from F-1 and F-2, plus yoke-derive 0.8.3
              yanked, also on main. None from keyhive_codec or nonempty 0.12 ·
              cold_keys_are_exported_and_never_persisted pass; extended no · keyhive version
              recorded yes (keyhive_core, keyhive_crypto and beekem at 2bb83d6; no local
              patch) · app IDs unchanged yes
```
