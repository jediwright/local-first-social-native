# B10 — `did:plc` verification-method route (OI-M4) — decision note

**Source:** `did-method-plc/did-method-plc` `main` (pushed 2026-09-01T18:01:17Z ✓), `website/spec/v0.1/did-plc.md` (478 lines), codeload tarball 2026-09-20 ~21:08Z.
**Question (spec §3.2 / §10 OI-M4; plan §2 B10):** which `verificationMethod` entries can a Half A record reference, and does an *advisory* admin-key advertisement need one at all?
**Register:** CONTEXTUAL · SINGLE-CONTEXT — NOT PANELED · ✓ from the spec text unless tagged.

## What the PLC spec permits

- `verificationMethods` is a map *service-id → `did:key`*. Keys there **have no control over the DID**; control rests only in `rotationKeys`. Any syntactically valid `did:key` (`did:key:` + base58btc multibase) is accepted — key *type* is unrestricted for verification methods, whereas rotation keys must be k256 or p256. atproto's convention is one `atproto` entry (the repo signing key); other entries are permitted.
- Every change to `verificationMethods` is a **PLC operation**: DAG-CBOR, signed by a *rotation key*, submitted unauthenticated to `plc.directory`, subject to the 72-hour recovery window, and **permanently and publicly logged** — including after tombstone; the spec warns that anything PII-adjacent in the log cannot be redacted.
- The PLC server **does not cross-validate** `alsoKnownAs` or `services`; only signatures and rotation rules are checked.
- Doc-ID-shaped values (the Keyhive identity document ID, `automerge:` + bs58check) are not keys and cannot be a verification method. The only PLC fields that could carry one are `alsoKnownAs` (URI list) or `services` (type + endpoint) — both unvalidated free text.

## What Half A actually needs

Half A carries the Keyhive identity doc ID plus advisory admin verifying key(s); verification **chains to the doc ID** (§4.1), and the keys are an out-of-band comparison aid re-issued on every rotation (§3.2, F-1 converged). So the load-bearing datum is the doc ID, which PLC cannot express as a verification method at all. The advisory keys *could* be verification methods (an ed25519 `did:key` is syntactically valid; Keyhive admin keys are ed25519 ~ — verify against `keyhive_core` signing type at Phase 1), but doing so buys nothing the chain step doesn't already do and costs:

| Cost | Record route (current design) | Verification-method route |
|---|---|---|
| Write authority | OAuth session → PDS record (`social.localfirst.identity`, rkey `self`) | Rotation-key-signed PLC op. For hosted accounts the rotation key is typically held by the PDS (~ — general atproto practice, not in this spec; verify via `com.atproto.identity.signPlcOperation` flow at Phase 2); user-held rotation keys are opt-in |
| Per rotation | New record version, same AT-URI, nothing a contact holds changes (F-8) | New PLC op per admin-key rotation; 72 h window each; log grows forever |
| Privacy | Record is deletable from the PDS | Key history is public and unredactable in the PLC log |
| Verification value | Chain step to doc ID (signature-validity, L-17) | Same chain step; a DID-side commitment to a *key* the design treats as advisory |

## Decision

**Route named: record route. OI-M4 closes.** The verification-method route is *verified as available* (any `did:key` type; no control implications) and *not chosen*: the Half A doc ID cannot live there, the advisory keys gain nothing there, and the route adds rotation-key ceremony plus permanent public key history. Spec §3.2's "alternative route … to be verified against current PLC operation rules at Phase 0 (?)" becomes ✓-verified-and-declined; the (?) resolves.

**Narrowed residue (not a blocker; queue for the spec v0.1.5 cut list):** whether a *future* DID-side commitment — one `verificationMethods` entry naming the current Keyhive admin key — is worth adding as a Half-B-adjacent strengthening after T4, when the bind becomes bidirectional (L-8). That is a Phase 4b question, not Phase 2, and it inherits the rotation-key custody point above.

No record written. Nothing here amends the spec; §3.2 and §10 OI-M4 are edited at the v0.1.5 Lightweight cut.
