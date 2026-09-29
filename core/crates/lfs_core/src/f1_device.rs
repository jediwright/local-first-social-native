//! F-1 (frontier), tried step 7: encrypted content on a document rebuilt from
//! storage, driven from a device shell.
//!
//! A separate F-1 hive, independent of the identity ceremony (plan D1(a)).
//! Its device signer and one test peer are generated at run time (charter
//! rule 5); the peer is added at Read and then dropped, so only the device is
//! restored across a relaunch (plan D3(a)).
//!
//! Three rows, all public material:
//! - `f1:doc`, tag `keyhive.doc-events.bincode.v1`: the device's own event
//!   export, merged with what is already stored and de-duplicated;
//! - `f1:doc.keyops`, tag `keyhive.static-events.bincode.v1`: the key event of
//!   each member the device added, from their contact card (the G-5
//!   workaround, the same shape as a group's `.keyops` row);
//! - `f1:doc.content`, tag `keyhive.encrypted-content.bincode.v1`: the
//!   document id and the stored ciphertexts, each with a label.
//!
//! Secrets never reach the store. After every change that may rotate a key
//! (setup, write, rotate) the core stages the rows and returns the device's
//! exported prekey secrets; the shell saves them, then calls `f1_commit`,
//! which writes the rows (plan D2(a): secrets first). On relaunch the shell
//! passes the seed and secrets back to `f1_restore`, which imports the secrets
//! before replaying the rows.

use crate::storage::{
    DocStore, FORMAT_KEYHIVE_DOC_EVENTS_V1, FORMAT_KEYHIVE_ENCRYPTED_CONTENT_V1, FORMAT_KEYHIVE_STATIC_EVENTS_V1,
};
use crate::{ceremony, groups, CoreError};
use beekem::encrypted::EncryptedContent;
use future_form::Sendable;
use keyhive_core::access::Access;
use keyhive_core::event::static_event::StaticEvent;
use keyhive_core::keyhive::Keyhive;
use keyhive_core::listener::no_listener::NoListener;
use keyhive_core::principal::document::id::DocumentId;
use keyhive_core::principal::identifier::Identifier;
use keyhive_core::store::ciphertext::memory::MemoryCiphertextStore;
use keyhive_crypto::signer::memory::MemorySigner;
use keyhive_crypto::verifiable::Verifiable;
use nonempty::nonempty;
use rand::rngs::OsRng;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

pub(crate) const ROW_DOC: &str = "f1:doc";
pub(crate) const ROW_KEYOPS: &str = "f1:doc.keyops";
pub(crate) const ROW_CONTENT: &str = "f1:doc.content";

type Hive = Keyhive<Sendable, MemorySigner>;
type Events = Vec<StaticEvent<[u8; 32]>>;
type Content = EncryptedContent<Vec<u8>, [u8; 32]>;
type Stored = (DocumentId, Vec<(String, [u8; 32], Content)>);

/// How `f1_restore` rebuilds the hive. `ImportFirst` is the working path;
/// the others reproduce the failures seen in the core runs, on the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum F1RestoreMode {
    /// Secrets imported, then member key events and stored events replayed (3d).
    ImportFirst,
    /// Stored events replayed, then secrets imported (3b).
    ImportAfter,
    /// Stored events replayed, no secrets (3a).
    NoSecrets,
    /// Secrets imported first, but the member key events left out (G-5, as 3c).
    SkipMemberKeyOps,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct F1Read {
    pub label: String,
    pub ok: bool,
    pub detail: String,
}

/// What a call did, for the shell's status text. Prefixes and counts only.
#[derive(Debug, Clone, uniffi::Record)]
pub struct F1Report {
    pub phase: String,
    pub device_prefix: String,
    pub doc_prefix: String,
    /// Event kinds still pending after replay, each with why it was refused.
    pub pending: Vec<String>,
    pub cgka_ops: Option<u32>,
    /// How many key pairs the device's secret export holds (a count only).
    pub secret_pairs: u32,
    pub reads: Vec<F1Read>,
    /// Whether rows are staged and waiting for `f1_commit`.
    pub staged: bool,
    /// For setup, write and rotate: whether the change produced a key update.
    pub key_update: Option<bool>,
}

/// Returned by every change that may rotate a key. Save `prekey_secrets`
/// (and `device_seed`, when present) before calling `f1_commit`.
#[derive(Debug, Clone, uniffi::Record)]
pub struct F1Pending {
    pub report: F1Report,
    /// Set by `f1_setup` only.
    pub device_seed: Option<Vec<u8>>,
    pub prekey_secrets: Vec<u8>,
}

struct Staged {
    doc_events: Vec<u8>,
    keyops: Vec<u8>,
    content: Vec<u8>,
}

pub(crate) struct F1Hive {
    hive: Hive,
    device: Identifier,
    doc: DocumentId,
    /// Everything stored or exported so far, own key events included.
    doc_events: Events,
    /// Key events of the members the device added.
    keyops: Events,
    contents: Vec<(String, [u8; 32], Content)>,
    staged: Option<Staged>,
}

fn rt<F: std::future::Future>(f: F) -> F::Output {
    crate::runtime::rt().block_on(f)
}

fn fr(e: impl std::fmt::Debug) -> CoreError {
    CoreError::Frontier(format!("{e:?}"))
}

async fn new_hive(signer: MemorySigner) -> Result<Hive, CoreError> {
    Keyhive::generate(signer, MemoryCiphertextStore::new(), NoListener, OsRng).await.map_err(fr)
}

/// A hive already holds its own key events, so replay skips them.
pub(crate) fn not_own_key_op(ev: &StaticEvent<[u8; 32]>, own: &Identifier) -> bool {
    match ev {
        StaticEvent::PrekeysExpanded(op) => Identifier::from(op.issuer()) != *own,
        StaticEvent::PrekeyRotated(op) => Identifier::from(op.issuer()) != *own,
        _ => true,
    }
}

/// Short name for an event, marking the device's own.
pub(crate) fn kind(ev: &StaticEvent<[u8; 32]>, own: &Identifier) -> String {
    let mine = |i: Identifier| if i == *own { " (own)" } else { "" };
    match ev {
        StaticEvent::PrekeysExpanded(op) => format!("PrekeysExpanded{}", mine(Identifier::from(op.issuer()))),
        StaticEvent::PrekeyRotated(op) => format!("PrekeyRotated{}", mine(Identifier::from(op.issuer()))),
        StaticEvent::CgkaOperation(op) => format!("CgkaOperation{}", mine(Identifier::from(op.issuer()))),
        StaticEvent::Delegated(op) => format!("Delegated{}", mine(Identifier::from(op.issuer()))),
        StaticEvent::Revoked(op) => format!("Revoked{}", mine(Identifier::from(op.issuer()))),
    }
}

/// Public identifiers as prefixes only (charter rule 10).
pub(crate) fn short(id: &impl std::fmt::Debug) -> String {
    let full = format!("{id:?}");
    let cut: String = full.chars().take(full.find("0x").map(|i| i + 18).unwrap_or(24)).collect();
    format!("{cut}…")
}

/// Shorten every `0x…` run of hex in a message to 16 hex digits (charter
/// rule 10): Keyhive's error text carries full keys and ids.
pub(crate) fn scrub(msg: &str) -> String {
    let b = msg.as_bytes();
    let mut out = String::with_capacity(msg.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'0' && i + 1 < b.len() && b[i + 1] == b'x' {
            let start = i + 2;
            let mut end = start;
            while end < b.len() && b[end].is_ascii_hexdigit() {
                end += 1;
            }
            out.push_str("0x");
            out.push_str(&msg[start..end.min(start + 16)]);
            if end - start > 16 {
                out.push('…');
            }
            i = end;
        } else {
            let ch = msg[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Merge events, dropping exact duplicates (by encoded bytes), order kept.
fn merge(into: &mut Events, more: Events) -> Result<(), CoreError> {
    let mut seen: BTreeSet<Vec<u8>> = into.iter().map(|e| bincode::serialize(e).map_err(fr)).collect::<Result<_, _>>()?;
    for ev in more {
        if seen.insert(bincode::serialize(&ev).map_err(fr)?) {
            into.push(ev);
        }
    }
    Ok(())
}

fn read_row(store: &dyn DocStore, id: &str, tag: &str) -> Result<Vec<u8>, CoreError> {
    let (got, bytes) = store
        .read_tagged(id)
        .map_err(|e| CoreError::Storage(e.to_string()))?
        .ok_or_else(|| CoreError::Frontier(format!("no {id} row")))?;
    if got != tag {
        return Err(CoreError::Frontier(format!("{id} row under unexpected tag {got}")));
    }
    Ok(bytes)
}

pub(crate) fn present(store: &dyn DocStore) -> bool {
    matches!(store.read_tagged(ROW_DOC), Ok(Some(_)))
}

impl F1Hive {
    /// A new device and document, one test peer added at Read and dropped,
    /// and the first content. Rows staged; secrets returned.
    pub(crate) fn setup() -> Result<(Self, F1Pending), CoreError> {
        let device = MemorySigner::generate(&mut OsRng);
        let seed = device.0.to_bytes().to_vec();
        let device_id: Identifier = (&device.verifying_key()).into();
        let peer = MemorySigner::generate(&mut OsRng);
        let (hive, doc, keyops) = rt(async {
            let hive = new_hive(device).await?;
            let peer_hive = new_hive(peer).await?;
            let card = peer_hive.generate_contact_card().await.map_err(fr)?;
            let peer_id = hive.receive_contact_card(&card).await.map_err(fr)?;
            let doc = hive.generate_doc(vec![], nonempty![[1u8; 32]]).await.map_err(fr)?;
            hive.add_member(peer_id, doc, Access::Read, &[]).await.map_err(fr)?;
            Ok::<_, CoreError>((hive, doc, vec![ceremony::keyop_event(&card)]))
        })?;
        let mut this = Self { hive, device: device_id, doc, doc_events: Vec::new(), keyops, contents: Vec::new(), staged: None };
        let updated = this.encrypt()?;
        let mut pending = this.stage("setup", Some(updated))?;
        pending.device_seed = Some(seed);
        Ok((this, pending))
    }

    /// Encrypt the next labelled content. Every encryption may rotate a key.
    pub(crate) fn write(&mut self) -> Result<F1Pending, CoreError> {
        let updated = self.encrypt()?;
        self.stage("write", Some(updated))
    }

    /// Issue a key update on the document.
    pub(crate) fn rotate(&mut self) -> Result<F1Pending, CoreError> {
        rt(self.hive.force_pcs_update(self.doc)).map_err(fr)?;
        self.stage("rotate", Some(true))
    }

    /// Write the staged rows. Call only after the secrets are saved.
    pub(crate) fn commit(&mut self, store: &dyn DocStore) -> Result<F1Report, CoreError> {
        let st = self.staged.take().ok_or_else(|| CoreError::Frontier("nothing staged".into()))?;
        let ser = |e: rusqlite::Error| CoreError::Storage(e.to_string());
        store.write_keyhive_static_events(ROW_KEYOPS, &st.keyops).map_err(ser)?;
        store.write_keyhive_encrypted_content(ROW_CONTENT, &st.content).map_err(ser)?;
        store.write_keyhive_doc_events(ROW_DOC, &st.doc_events).map_err(ser)?;
        self.report("commit", vec![], false, None)
    }

    /// Rebuild from the seed, the secrets and the rows, then read every
    /// stored ciphertext. `ImportFirst` and `SkipMemberKeyOps` need secrets.
    pub(crate) fn restore(
        store: &dyn DocStore,
        seed: &[u8],
        secrets: Option<&[u8]>,
        mode: F1RestoreMode,
    ) -> Result<(Self, F1Report), CoreError> {
        let doc_events: Events = bincode::deserialize(&read_row(store, ROW_DOC, FORMAT_KEYHIVE_DOC_EVENTS_V1)?).map_err(fr)?;
        let keyops: Events = bincode::deserialize(&read_row(store, ROW_KEYOPS, FORMAT_KEYHIVE_STATIC_EVENTS_V1)?).map_err(fr)?;
        let (doc, contents): Stored =
            bincode::deserialize(&read_row(store, ROW_CONTENT, FORMAT_KEYHIVE_ENCRYPTED_CONTENT_V1)?).map_err(fr)?;
        let signer = groups::signer_from_seed(seed)?;
        let device: Identifier = (&signer.verifying_key()).into();
        let needs = matches!(mode, F1RestoreMode::ImportFirst | F1RestoreMode::ImportAfter | F1RestoreMode::SkipMemberKeyOps);
        let secrets = match (needs, secrets) {
            (true, Some(s)) => Some(s),
            (true, None) => return Err(CoreError::Frontier(format!("{mode:?} needs secrets"))),
            (false, _) => None,
        };
        let replay: Events = {
            let mut v: Events = if mode == F1RestoreMode::SkipMemberKeyOps { Vec::new() } else { keyops.clone() };
            v.extend(doc_events.iter().filter(|e| not_own_key_op(e, &device)).cloned());
            v
        };
        let (hive, pending) = rt(async {
            let hive = new_hive(signer).await?;
            let mut pending: Vec<Arc<StaticEvent<[u8; 32]>>>;
            match mode {
                F1RestoreMode::ImportFirst | F1RestoreMode::SkipMemberKeyOps => {
                    hive.import_prekey_secrets(secrets.unwrap()).await.map_err(fr)?;
                    pending = hive.ingest_unsorted_static_events(replay).await;
                }
                F1RestoreMode::ImportAfter => {
                    hive.ingest_unsorted_static_events(replay).await;
                    pending = hive.import_prekey_secrets(secrets.unwrap()).await.map_err(fr)?;
                }
                F1RestoreMode::NoSecrets => {
                    pending = hive.ingest_unsorted_static_events(replay).await;
                }
            }
            pending.sort_by_key(|e| kind(e, &device));
            Ok::<_, CoreError>((hive, pending))
        })?;
        let refused: Vec<String> = rt(async {
            let mut out = Vec::new();
            for ev in pending {
                let why = match hive.receive_static_event((*ev).clone()).await {
                    Ok(()) => "accepted on its own".to_string(),
                    Err(e) => scrub(&format!("{e:?}")),
                };
                out.push(format!("{}: {why}", kind(&ev, &device)));
            }
            out
        });
        let this = Self { hive, device, doc, doc_events, keyops, contents, staged: None };
        let report = this.report(&format!("restore {mode:?}"), refused, true, None)?;
        Ok((this, report))
    }

    /// Returns whether the encryption produced a key update.
    fn encrypt(&mut self) -> Result<bool, CoreError> {
        let n = self.contents.len() + 1;
        let label = if n == 1 { "content 1 (setup)".to_string() } else { format!("content {n}") };
        let r = [(n as u8).wrapping_add(1); 32];
        let pred: Vec<[u8; 32]> = self.contents.last().map(|c| vec![c.1]).unwrap_or_default();
        let out = rt(self.hive.try_encrypt_content(self.doc, &r, &pred, format!("F-1 {label}").as_bytes())).map_err(fr)?;
        self.contents.push((label, r, out.encrypted_content().clone()));
        Ok(out.update_op().is_some())
    }

    fn stage(&mut self, phase: &str, key_update: Option<bool>) -> Result<F1Pending, CoreError> {
        let export: Events = rt(self.hive.static_events_for_agent(self.device)).into_values().collect();
        merge(&mut self.doc_events, export)?;
        let stored: (DocumentId, &Vec<(String, [u8; 32], Content)>) = (self.doc, &self.contents);
        self.staged = Some(Staged {
            doc_events: bincode::serialize(&self.doc_events).map_err(fr)?,
            keyops: bincode::serialize(&self.keyops).map_err(fr)?,
            content: bincode::serialize(&stored).map_err(fr)?,
        });
        let prekey_secrets = rt(self.hive.export_prekey_secrets()).map_err(fr)?;
        let report = self.report(phase, vec![], false, key_update)?;
        Ok(F1Pending { report, device_seed: None, prekey_secrets })
    }

    fn report(&self, phase: &str, pending: Vec<String>, read: bool, key_update: Option<bool>) -> Result<F1Report, CoreError> {
        let secrets = rt(self.hive.export_prekey_secrets()).map_err(fr)?;
        let pairs: BTreeMap<[u8; 32], [u8; 32]> = bincode::deserialize(&secrets).map_err(fr)?;
        let cgka_ops = rt(self.hive.cgka_ops_for_doc(&self.doc)).ok().flatten().map(|v| v.len() as u32);
        let reads = if read {
            self.contents
                .iter()
                .map(|(label, _, c)| match rt(self.hive.try_decrypt_content(self.doc, c)) {
                    Ok(p) => F1Read { label: label.clone(), ok: true, detail: String::from_utf8_lossy(&p).into_owned() },
                    Err(e) => F1Read { label: label.clone(), ok: false, detail: scrub(&format!("{e:?}")) },
                })
                .collect()
        } else {
            Vec::new()
        };
        Ok(F1Report {
            phase: phase.to_string(),
            device_prefix: short(&self.device),
            doc_prefix: short(&self.doc),
            pending,
            cgka_ops,
            secret_pairs: pairs.len() as u32,
            reads,
            staged: self.staged.is_some(),
            key_update,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Core;

    fn show(r: &F1Report) {
        println!(
            "F-1 step 7: {} · doc {} · key update {:?} · pending {} {:?} · cgka {:?} · secret pairs {} · staged {}",
            r.phase, r.doc_prefix, r.key_update, r.pending.len(), r.pending, r.cgka_ops, r.secret_pairs, r.staged
        );
        for rd in &r.reads {
            println!("F-1 step 7:   read {}: {} ({})", rd.label, if rd.ok { "OK" } else { "FAILED" }, rd.detail);
        }
    }

    /// Runs 7a–7f through `Core`, a new `Core` on the same file standing in
    /// for a relaunch. Prints what held and what broke; asserts only the
    /// working path (7a, 7b).
    #[test]
    #[ignore]
    fn f1_step7_core_dry_run() {
        let dir = tempfile::tempdir().unwrap();
        let path = |n: &str| dir.path().join(n).to_string_lossy().to_string();
        let fresh = |n: &str| {
            let c = Core::new(path(n)).unwrap();
            let p = c.f1_setup().unwrap();
            c.f1_commit().unwrap();
            (p.device_seed.unwrap(), p.prekey_secrets)
        };

        // 7a, then 7b on the same store
        let (seed, secrets) = fresh("a.sqlite");
        let c = Core::new(path("a.sqlite")).unwrap();
        let r = c.f1_restore(seed.clone(), Some(secrets), F1RestoreMode::ImportFirst).unwrap();
        show(&r);
        assert!(r.pending.is_empty() && r.reads.iter().all(|x| x.ok), "7a");
        let mut last = Vec::new();
        for step in ["write", "rotate", "write"] {
            let p = if step == "write" { c.f1_write() } else { c.f1_rotate() }.unwrap();
            last = p.prekey_secrets;
            c.f1_commit().unwrap();
        }
        drop(c);
        let c = Core::new(path("a.sqlite")).unwrap();
        let r = c.f1_restore(seed.clone(), Some(last.clone()), F1RestoreMode::ImportFirst).unwrap();
        show(&r);
        assert!(r.pending.is_empty() && r.reads.len() == 3 && r.reads.iter().all(|x| x.ok), "7b");
        drop(c);

        // 7c, 7d
        for (n, mode) in [
            ("c1.sqlite", F1RestoreMode::NoSecrets),
            ("c2.sqlite", F1RestoreMode::ImportAfter),
            ("d.sqlite", F1RestoreMode::SkipMemberKeyOps),
        ] {
            let (seed, secrets) = fresh(n);
            let c = Core::new(path(n)).unwrap();
            match c.f1_restore(seed, Some(secrets), mode) {
                Ok(r) => show(&r),
                Err(e) => println!("F-1 step 7: restore {mode:?} FAILED: {e:?}"),
            }
        }

        // 7e: secrets saved, rows not committed, relaunch
        let (seed, secrets) = fresh("e.sqlite");
        let c = Core::new(path("e.sqlite")).unwrap();
        c.f1_restore(seed.clone(), Some(secrets), F1RestoreMode::ImportFirst).unwrap();
        let ahead = c.f1_write().unwrap().prekey_secrets;
        drop(c);
        let c = Core::new(path("e.sqlite")).unwrap();
        let mut r = c.f1_restore(seed, Some(ahead), F1RestoreMode::ImportFirst).unwrap();
        r.phase = format!("7e {}", r.phase);
        show(&r);
        drop(c);

        // 7f: a change committed without re-saving the secrets, relaunch.
        // Two cases: an ordinary write, and a key update followed by a write.
        for (n, rotate) in [("f1.sqlite", false), ("f2.sqlite", true)] {
            let (seed, secrets) = fresh(n);
            let c = Core::new(path(n)).unwrap();
            c.f1_restore(seed.clone(), Some(secrets.clone()), F1RestoreMode::ImportFirst).unwrap();
            if rotate {
                show(&c.f1_rotate().unwrap().report);
            }
            show(&c.f1_write().unwrap().report);
            c.f1_commit().unwrap();
            drop(c);
            let c = Core::new(path(n)).unwrap();
            let mut r = c.f1_restore(seed, Some(secrets), F1RestoreMode::ImportFirst).unwrap();
            r.phase = format!("7f ({}) {}", if rotate { "rotate + write" } else { "write" }, r.phase);
            show(&r);
        }
    }
}
