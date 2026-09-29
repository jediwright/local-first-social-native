//! F-1 (frontier): encrypted content on a reloaded document, steps 2–5 at the
//! Rust-core level.
//!
//! Every identity here is a test identity generated at run time. The tests are
//! ignored by default; run them by name with
//! `cargo test -p lfs_core f1_step -- --ignored --nocapture --test-threads=1`.
//! Each prints what held and what broke. Asserts guard the setup and the
//! no-secrets-on-disk rule only, so an unexpected result shows up as output,
//! not as a failed setup.

use crate::storage::{DocStore, SqliteStore, FORMAT_KEYHIVE_DOC_EVENTS_V1};
use future_form::Sendable;
use keyhive_core::access::Access;
use keyhive_core::event::static_event::StaticEvent;
use keyhive_core::keyhive::Keyhive;
use keyhive_core::listener::no_listener::NoListener;
use keyhive_core::principal::document::id::DocumentId;
use keyhive_core::principal::document::EncryptedContentWithUpdate;
use keyhive_core::principal::identifier::Identifier;
use keyhive_core::store::ciphertext::memory::MemoryCiphertextStore;
use keyhive_crypto::signer::memory::MemorySigner;
use keyhive_crypto::verifiable::Verifiable;
use nonempty::nonempty;
use rand::rngs::OsRng;
use std::collections::BTreeMap;

type Hive = Keyhive<Sendable, MemorySigner>;
type Events = Vec<StaticEvent<[u8; 32]>>;

const ROW: &str = "f1-doc-events";
const OLD_TEXT: &[u8] = b"written before the restart";
const NEW_TEXT: &[u8] = b"written after the restart";

fn rt<F: std::future::Future>(f: F) -> F::Output {
    crate::runtime::rt().block_on(f)
}

fn new_hive(signer: MemorySigner) -> Hive {
    rt(Keyhive::generate(signer, MemoryCiphertextStore::new(), NoListener, OsRng)).expect("generate hive")
}

fn id_of(signer: &MemorySigner) -> Identifier {
    (&signer.verifying_key()).into()
}

/// A hive already knows its own prekey op, so replay skips it (same rule as
/// the identity reload).
fn not_own_key_op(ev: &StaticEvent<[u8; 32]>, own: &Identifier) -> bool {
    match ev {
        StaticEvent::PrekeysExpanded(op) => Identifier::from(op.issuer()) != *own,
        StaticEvent::PrekeyRotated(op) => Identifier::from(op.issuer()) != *own,
        _ => true,
    }
}

async fn events_for(hive: &Hive, who: Identifier) -> Events {
    hive.static_events_for_agent(who).await.into_values().collect()
}

async fn sync_into(from: &Hive, to: &Hive, to_signer: &MemorySigner) -> usize {
    let own = id_of(to_signer);
    let evs: Events = events_for(from, own).await.into_iter().filter(|e| not_own_key_op(e, &own)).collect();
    to.ingest_unsorted_static_events(evs).await.len()
}

fn report(what: &str, r: Result<Vec<u8>, impl std::fmt::Debug>) {
    match r {
        Ok(p) => println!("F-1: {what}: OK ({:?})", String::from_utf8_lossy(&p)),
        Err(e) => println!("F-1: {what}: FAILED: {e:?}"),
    }
}

struct Setup {
    device: MemorySigner,
    peer: MemorySigner,
    peer_hive: Hive,
    doc: DocumentId,
    old: EncryptedContentWithUpdate<[u8; 32]>,
    secrets: Vec<u8>,
}

fn temp_store() -> (tempfile::TempDir, SqliteStore, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("f1.sqlite").to_string_lossy().to_string();
    let s = SqliteStore::open(&path).unwrap();
    (dir, s, path)
}

/// Step 2: the device makes a document, adds a peer at Read, encrypts content,
/// and stores the document's public event set (group-encryption operations
/// included) under its own tag. The peer syncs once, before any restart.
/// The device's secrets are exported and held in memory only.
fn setup(s: &SqliteStore, store_peer_key_op: bool) -> Setup {
    let device = MemorySigner::generate(&mut OsRng);
    let peer = MemorySigner::generate(&mut OsRng);
    let h1 = new_hive(device.clone());
    let peer_hive = new_hive(peer.clone());
    let (doc, old, secrets) = rt(async {
        let card = peer_hive.generate_contact_card().await.expect("peer contact card");
        let peer_id = h1.receive_contact_card(&card).await.expect("receive peer card");
        let doc = h1.generate_doc(vec![], nonempty![[1u8; 32]]).await.expect("generate doc");
        h1.add_member(peer_id, doc, Access::Read, &[]).await.expect("add peer at Read");
        let old = h1.try_encrypt_content(doc, &[2u8; 32], &vec![], OLD_TEXT).await.expect("encrypt");
        assert_eq!(h1.try_decrypt_content(doc, old.encrypted_content()).await.expect("decrypt before restart"), OLD_TEXT);
        println!("F-1 step 2: encrypt produced a key update: {}", old.update_op().is_some());
        let mut events = events_for(&h1, id_of(&device)).await;
        if store_peer_key_op {
            events.push(crate::ceremony::keyop_event(&card));
            println!("F-1 step 2: also storing the peer's key op from its contact card");
        }
        println!("F-1 step 2: device {:?}", id_of(&device));
        println!("F-1 step 2: peer   {:?}", id_of(&peer));
        println!("F-1 step 2: doc    {:?}", doc);
        let mut stored: Vec<String> = events.iter().map(|e| kind(e, &id_of(&device))).collect();
        stored.sort();
        println!("F-1 step 2: stored kinds {:?}", stored);
        let cgka = events.iter().filter(|e| matches!(e, StaticEvent::CgkaOperation(_))).count();
        println!("F-1 step 2: stored {} event(s), {} of them group-encryption operations", events.len(), cgka);
        let bytes = bincode::serialize(&events).expect("encode events");
        s.write_keyhive_doc_events(ROW, &bytes).expect("write doc events");
        let pending = sync_into(&h1, &peer_hive, &peer).await;
        println!("F-1 step 2: peer pending after first sync: {pending}");
        report("peer reads content before the restart", peer_hive.try_decrypt_content(doc, old.encrypted_content()).await);
        let secrets = h1.export_prekey_secrets().await.expect("export secrets");
        (doc, old, secrets)
    });
    Setup { device, peer, peer_hive, doc, old, secrets }
}

/// Short name for an event, marking the device's own key ops.
fn kind(ev: &StaticEvent<[u8; 32]>, own: &Identifier) -> String {
    let mine = |i: Identifier| if i == *own { " (own)" } else { "" };
    match ev {
        StaticEvent::PrekeysExpanded(op) => format!("PrekeysExpanded{}", mine(Identifier::from(op.issuer()))),
        StaticEvent::PrekeyRotated(op) => format!("PrekeyRotated{}", mine(Identifier::from(op.issuer()))),
        StaticEvent::CgkaOperation(op) => format!("CgkaOperation{}", mine(Identifier::from(op.issuer()))),
        StaticEvent::Delegated(op) => format!("Delegated{}", mine(Identifier::from(op.issuer()))),
        StaticEvent::Revoked(op) => format!("Revoked{}", mine(Identifier::from(op.issuer()))),
    }
}

fn kinds(evs: &[std::sync::Arc<StaticEvent<[u8; 32]>>], own: &Identifier) -> Vec<String> {
    let mut v: Vec<String> = evs.iter().map(|e| kind(e, own)).collect();
    v.sort();
    v
}

/// Rebuild the device's hive from the stored row only. `drop_own_key_ops`
/// follows the identity reload's rule; `false` replays everything.
fn reload(device: &MemorySigner, s: &SqliteStore, drop_own_key_ops: bool) -> (Hive, Vec<String>) {
    let (tag, bytes) = s.read_tagged(ROW).unwrap().expect("doc events row");
    assert_eq!(tag, FORMAT_KEYHIVE_DOC_EVENTS_V1);
    let own = id_of(device);
    let events: Events = bincode::deserialize::<Events>(&bytes)
        .expect("decode doc events")
        .into_iter()
        .filter(|e| !drop_own_key_ops || not_own_key_op(e, &own))
        .collect();
    let h = new_hive(device.clone());
    let pending = rt(h.ingest_unsorted_static_events(events));
    let k = kinds(&pending, &own);
    (h, k)
}

/// The rule every F-1 test ends with: the store file holds no seed and no
/// exported secret.
fn assert_no_secrets_on_disk(x: &Setup, s: SqliteStore, path: &str) {
    drop(s);
    let mut file = std::fs::read(path).unwrap();
    if let Ok(wal) = std::fs::read(format!("{path}-wal")) {
        file.extend(wal);
    }
    let contains = |n: &[u8]| file.windows(n.len()).any(|w| w == n);
    assert!(!contains(&x.device.0.to_bytes()), "device seed must not be on disk");
    assert!(!contains(&x.peer.0.to_bytes()), "peer seed must not be on disk");
    let pairs: BTreeMap<[u8; 32], [u8; 32]> = bincode::deserialize(&x.secrets).expect("secret export shape");
    assert!(pairs.values().all(|sk| !contains(sk)), "no exported secret may be on disk");
    println!("F-1: store file holds no seed and none of {} exported secret(s)", pairs.len());
}

#[test]
#[ignore]
fn f1_step3_reload_without_secrets() {
    let (_dir, s, path) = temp_store();
    let x = setup(&s, false);
    let (h2, pending) = reload(&x.device, &s, true);
    println!("F-1 step 3: pending after replay: {} {:?}", pending.len(), pending);
    rt(async {
        println!("F-1 step 3: document present: {}", h2.has_document(x.doc).await);
        println!(
            "F-1 step 3: group-encryption operations after replay: {:?}",
            h2.cgka_ops_for_doc(&x.doc).await.map(|o| o.map(|v| v.len()))
        );
        report("device reads content from before the restart", h2.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
    });
    assert_no_secrets_on_disk(&x, s, &path);
}

#[test]
#[ignore]
fn f1_step4_key_update_after_reload() {
    let (_dir, s, path) = temp_store();
    let x = setup(&s, false);
    let (h2, _) = reload(&x.device, &s, true);
    rt(async {
        match h2.force_pcs_update(x.doc).await {
            Ok((_, leaf)) => println!("F-1 step 4: key update issued; new leaf secret returned: {}", leaf.is_some()),
            Err(e) => {
                println!("F-1 step 4: key update FAILED: {e:?}");
                return;
            }
        }
        let new = match h2.try_encrypt_content(x.doc, &[3u8; 32], &vec![[2u8; 32]], NEW_TEXT).await {
            Ok(n) => n,
            Err(e) => {
                println!("F-1 step 4: encrypt after restart FAILED: {e:?}");
                return;
            }
        };
        report("device reads new content", h2.try_decrypt_content(x.doc, new.encrypted_content()).await);
        let pending = sync_into(&h2, &x.peer_hive, &x.peer).await;
        println!("F-1 step 4: peer pending after second sync: {pending}");
        report("peer reads new content", x.peer_hive.try_decrypt_content(x.doc, new.encrypted_content()).await);
        report("peer reads old content", x.peer_hive.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
        report("device reads old content", h2.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
    });
    assert_no_secrets_on_disk(&x, s, &path);
}

#[test]
#[ignore]
fn f1_step5_restore_exported_secrets() {
    let (_dir, s, path) = temp_store();
    let x = setup(&s, false);
    let (h3, _) = reload(&x.device, &s, true);
    rt(async {
        match h3.import_prekey_secrets(&x.secrets).await {
            Ok(p) => println!("F-1 step 5: secrets imported; still pending: {} {:?}", p.len(), kinds(&p, &id_of(&x.device))),
            Err(e) => println!("F-1 step 5: import FAILED: {e:?}"),
        }
        report("device reads content from before the restart", h3.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
    });
    assert_no_secrets_on_disk(&x, s, &path);
}

/// Diagnostic for step 3: replay everything, the device's own earlier key ops
/// included, then try again after importing the exported secrets.
#[test]
#[ignore]
fn f1_step3b_reload_keeping_own_key_ops() {
    let (_dir, s, path) = temp_store();
    let x = setup(&s, false);
    let (h2, pending) = reload(&x.device, &s, false);
    println!("F-1 step 3b: pending after full replay: {} {:?}", pending.len(), pending);
    rt(async {
        let own = id_of(&x.device);
        for ev in h2.ingest_unsorted_static_events(vec![]).await {
            match h2.receive_static_event((*ev).clone()).await {
                Ok(()) => println!("F-1 step 3b: {} now accepted on its own", kind(&ev, &own)),
                Err(e) => println!("F-1 step 3b: {} refused: {e:?}", kind(&ev, &own)),
            }
        }
        println!(
            "F-1 step 3b: group-encryption operations after replay: {:?}",
            h2.cgka_ops_for_doc(&x.doc).await.map(|o| o.map(|v| v.len()))
        );
        report("3b device reads old content, no secrets", h2.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
        match h2.import_prekey_secrets(&x.secrets).await {
            Ok(p) => println!("F-1 step 3b: secrets imported; still pending: {} {:?}", p.len(), kinds(&p, &id_of(&x.device))),
            Err(e) => println!("F-1 step 3b: import FAILED: {e:?}"),
        }
        report("3b device reads old content, secrets restored", h2.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
    });
    assert_no_secrets_on_disk(&x, s, &path);
}

/// Diagnostic: import the exported secrets into the fresh hive BEFORE replay,
/// then replay everything and report what is still refused, and why.
#[test]
#[ignore]
fn f1_step3c_import_secrets_before_replay() {
    let (_dir, s, path) = temp_store();
    let x = setup(&s, false);
    let (tag, bytes) = s.read_tagged(ROW).unwrap().expect("doc events row");
    assert_eq!(tag, FORMAT_KEYHIVE_DOC_EVENTS_V1);
    let events: Events = bincode::deserialize(&bytes).expect("decode doc events");
    let h = new_hive(x.device.clone());
    let own = id_of(&x.device);
    rt(async {
        match h.import_prekey_secrets(&x.secrets).await {
            Ok(p) => println!("F-1 step 3c: secrets imported first; pending: {}", p.len()),
            Err(e) => println!("F-1 step 3c: import FAILED: {e:?}"),
        }
        let pending = h.ingest_unsorted_static_events(events).await;
        println!("F-1 step 3c: pending after replay: {} {:?}", pending.len(), kinds(&pending, &own));
        for ev in pending {
            match h.receive_static_event((*ev).clone()).await {
                Ok(()) => println!("F-1 step 3c: {} now accepted on its own", kind(&ev, &own)),
                Err(e) => println!("F-1 step 3c: {} refused: {e:?}", kind(&ev, &own)),
            }
        }
        println!(
            "F-1 step 3c: group-encryption operations after replay: {:?}",
            h.cgka_ops_for_doc(&x.doc).await.map(|o| o.map(|v| v.len()))
        );
        report("3c device reads old content", h.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
    });
    assert_no_secrets_on_disk(&x, s, &path);
}

/// The fix under test: store the peer's key op beside Keyhive's own export,
/// import the device's secrets before replay, then run the full path:
/// read old content, issue a key update, write new content, sync the peer.
#[test]
#[ignore]
fn f1_step3d_store_peer_key_op_and_import_first() {
    let (_dir, s, path) = temp_store();
    let x = setup(&s, true);
    let (tag, bytes) = s.read_tagged(ROW).unwrap().expect("doc events row");
    assert_eq!(tag, FORMAT_KEYHIVE_DOC_EVENTS_V1);
    let own = id_of(&x.device);
    let events: Events = bincode::deserialize::<Events>(&bytes)
        .expect("decode doc events")
        .into_iter()
        .filter(|e| not_own_key_op(e, &own))
        .collect();
    let h = new_hive(x.device.clone());
    rt(async {
        if let Err(e) = h.import_prekey_secrets(&x.secrets).await {
            println!("F-1 step 3d: import FAILED: {e:?}");
        }
        let pending = h.ingest_unsorted_static_events(events).await;
        println!("F-1 step 3d: pending after replay: {} {:?}", pending.len(), kinds(&pending, &own));
        for ev in pending {
            if let Err(e) = h.receive_static_event((*ev).clone()).await {
                println!("F-1 step 3d: {} refused: {e:?}", kind(&ev, &own));
            }
        }
        println!(
            "F-1 step 3d: group-encryption operations after replay: {:?}",
            h.cgka_ops_for_doc(&x.doc).await.map(|o| o.map(|v| v.len()))
        );
        report("3d device reads old content", h.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
        match h.force_pcs_update(x.doc).await {
            Ok((_, leaf)) => println!("F-1 step 3d: key update issued; new leaf secret returned: {}", leaf.is_some()),
            Err(e) => {
                println!("F-1 step 3d: key update FAILED: {e:?}");
                return;
            }
        }
        let new = match h.try_encrypt_content(x.doc, &[3u8; 32], &vec![[2u8; 32]], NEW_TEXT).await {
            Ok(n) => n,
            Err(e) => {
                println!("F-1 step 3d: encrypt after restart FAILED: {e:?}");
                return;
            }
        };
        report("3d device reads new content", h.try_decrypt_content(x.doc, new.encrypted_content()).await);
        let pending = sync_into(&h, &x.peer_hive, &x.peer).await;
        println!("F-1 step 3d: peer pending after second sync: {pending}");
        report("3d peer reads new content", x.peer_hive.try_decrypt_content(x.doc, new.encrypted_content()).await);
        report("3d peer reads old content", x.peer_hive.try_decrypt_content(x.doc, x.old.encrypted_content()).await);
    });
    assert_no_secrets_on_disk(&x, s, &path);
}
