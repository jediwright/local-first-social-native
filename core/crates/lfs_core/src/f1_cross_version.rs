//! F-1 (frontier): cross-version check for the two stored Keyhive formats.
//!
//! Run `f1_write_rows` on a build at the old Keyhive version, then
//! `f1_read_rows` on a build at the new one, both with `F1_XV_DIR` set to the
//! same directory outside the repo. Both tests are ignored by default.
//!
//! The identity rows hold public material only, and the reader rebuilds the
//! identity document with a freshly generated signer. Members are recorded
//! as 8-byte fingerprint prefixes.
//!
//! F-3 adds a second file, written through `Core`: the F-1 document rows
//! (`f1:doc`, `f1:doc.keyops`, `f1:doc.content`, after a write, a key update
//! and another write) and one group row pair (`group:xv`, two members
//! granted). The reader checks that each row decodes and re-encodes byte for
//! byte, then rebuilds both on a copy of the file and compares the members,
//! their levels and whether every stored ciphertext still decrypts. Reading
//! those back needs the run's device seeds and prekey secrets; they belong to
//! identities generated for that run only and are written to
//! `test-material.bin` in `F1_XV_DIR`, outside the repo and outside the
//! store. A directory written before F-3 has no second file; the reader then
//! checks the identity rows only.

use crate::ceremony::{decode, decode_events, reload_with, run, IDENTITY_KEYOPS_ROW_ID, IDENTITY_ROW_ID};
use crate::f1_device::{F1RestoreMode, ROW_CONTENT, ROW_DOC, ROW_KEYOPS};
use crate::groups::{GROUP_KEYOPS_ROW_SUFFIX, GROUP_ROW_PREFIX};
use crate::policy::GrantLevel;
use crate::storage::{DocStore, SqliteStore};
use crate::{Core, IdentityConfig, RootingLevel};
use beekem::encrypted::EncryptedContent;
use keyhive_core::event::static_event::StaticEvent;
use keyhive_core::principal::document::id::DocumentId;
use keyhive_crypto::signer::memory::MemorySigner;
use rand::rngs::OsRng;
use std::path::Path;

const DG_FILE: &str = "rows-doc-group.sqlite";
const DG_EXPECTED: &str = "expected-doc-group.txt";
const DG_MATERIAL: &str = "test-material.bin";
const DG_GROUP: &str = "xv";

type Events = Vec<StaticEvent<[u8; 32]>>;
type Stored = (DocumentId, Vec<(String, [u8; 32], EncryptedContent<Vec<u8>, [u8; 32]>)>);
/// (identity device seed, F-1 device seed, F-1 prekey secrets), test identities only.
type Material = (Vec<u8>, Vec<u8>, Vec<u8>);

fn xv_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var("F1_XV_DIR").expect("set F1_XV_DIR to a directory outside the repo"))
}

fn snapshot(s: &SqliteStore) -> Vec<String> {
    let (hive, doc_id) = reload_with(s, MemorySigner::generate(&mut OsRng)).expect("reload from stored rows");
    crate::runtime::rt().block_on(async {
        let doc = hive.get_document(doc_id).await.expect("identity document");
        let doc = doc.lock().await;
        let mut v: Vec<String> = doc
            .members()
            .keys()
            .map(|m| {
                let fp: String = m.to_bytes()[..8].iter().map(|b| format!("{b:02x}")).collect();
                format!("{fp} {:?}", doc.get_capability(m).unwrap().payload().can())
            })
            .collect();
        v.sort();
        v
    })
}

#[test]
#[ignore]
fn f1_write_rows() {
    let d = xv_dir();
    std::fs::create_dir_all(&d).unwrap();
    let path = d.join("rows.sqlite");
    assert!(!path.exists(), "rows.sqlite already exists; use a fresh F1_XV_DIR");
    let s = SqliteStore::open(&path.to_string_lossy()).unwrap();
    let record = run(&s, IdentityConfig { rooting_level: RootingLevel::Edit }).unwrap();
    drop(record);
    std::fs::write(d.join("expected.txt"), snapshot(&s).join("\n")).unwrap();
    std::fs::write(d.join("written-by.txt"), crate::storage::PIN_LABEL_KEYHIVE_CORE).unwrap();
    write_doc_group(&d);
    println!("wrote rows with {}", crate::storage::PIN_LABEL_KEYHIVE_CORE);
}

#[test]
#[ignore]
fn f1_read_rows() {
    let d = xv_dir();
    let s = SqliteStore::open(&d.join("rows.sqlite").to_string_lossy()).unwrap();
    let written_by = std::fs::read_to_string(d.join("written-by.txt")).unwrap();
    assert_ne!(
        written_by,
        crate::storage::PIN_LABEL_KEYHIVE_CORE,
        "the rows must come from the other Keyhive version"
    );
    let (_, bytes) = s.read_tagged(IDENTITY_ROW_ID).unwrap().expect("delegations row");
    let (_, kbytes) = s.read_tagged(IDENTITY_KEYOPS_ROW_ID).unwrap().expect("key-event row");
    assert_eq!(
        bincode::serialize(&decode(&bytes).unwrap()).unwrap(),
        bytes,
        "delegations row decodes and re-encodes byte for byte"
    );
    assert_eq!(
        bincode::serialize(&decode_events(&kbytes).unwrap()).unwrap(),
        kbytes,
        "key-event row decodes and re-encodes byte for byte"
    );
    let expected: Vec<String> =
        std::fs::read_to_string(d.join("expected.txt")).unwrap().lines().map(String::from).collect();
    assert_eq!(snapshot(&s), expected, "members and access levels match what was written");
    read_doc_group(&d);
    println!("read rows written by {written_by} with {}", crate::storage::PIN_LABEL_KEYHIVE_CORE);
}

/// Rebuild the group and the F-1 document on a copy of the file (reloading
/// may rewrite rows), then list the group's members and the document's reads.
fn dg_snapshot(path: &Path, m: &Material) -> Vec<String> {
    let tmp = tempfile::tempdir().unwrap();
    let copy = tmp.path().join(DG_FILE);
    std::fs::copy(path, &copy).unwrap();
    let core = Core::new(copy.to_string_lossy().to_string()).unwrap();
    core.reload_identity(m.0.clone()).expect("group rows reload");
    let mut v: Vec<String> = core
        .group_members(DG_GROUP.into())
        .iter()
        .map(|x| format!("group {} {:?}{}", &x.fingerprint[..16], x.level, if x.is_device { " (device)" } else { "" }))
        .collect();
    let r = core.f1_restore(m.1.clone(), Some(m.2.clone()), F1RestoreMode::ImportFirst).expect("document rows restore");
    v.push(format!("doc pending {} {:?}", r.pending.len(), r.pending));
    v.push(format!("doc cgka ops {:?}", r.cgka_ops));
    for rd in &r.reads {
        v.push(format!("doc read {}: {}", rd.label, if rd.ok { "OK" } else { "FAILED" }));
    }
    v
}

fn write_doc_group(d: &Path) {
    let path = d.join(DG_FILE);
    assert!(!path.exists(), "{DG_FILE} already exists; use a fresh F1_XV_DIR");
    let core = Core::new(path.to_string_lossy().to_string()).unwrap();
    let rec = core.run_identity_ceremony(IdentityConfig { rooting_level: RootingLevel::Edit }).unwrap();
    core.grant_member(DG_GROUP.into(), GrantLevel::Read).unwrap();
    core.grant_member(DG_GROUP.into(), GrantLevel::Edit).unwrap();
    let p = core.f1_setup().unwrap();
    let f1_seed = p.device_seed.clone().unwrap();
    let mut secrets = p.prekey_secrets;
    core.f1_commit().unwrap();
    for step in ["write", "rotate", "write"] {
        let p = if step == "write" { core.f1_write() } else { core.f1_rotate() }.unwrap();
        secrets = p.prekey_secrets;
        core.f1_commit().unwrap();
    }
    drop(core);
    let m: Material = (rec.device_key.device_secret.clone(), f1_seed, secrets);
    std::fs::write(d.join(DG_MATERIAL), bincode::serialize(&m).unwrap()).unwrap();
    let snap = dg_snapshot(&path, &m);
    for line in &snap {
        println!("F-3 xv write: {line}");
    }
    std::fs::write(d.join(DG_EXPECTED), snap.join("\n")).unwrap();
}

fn read_doc_group(d: &Path) {
    let path = d.join(DG_FILE);
    if !path.exists() {
        println!("no {DG_FILE} in this directory (written before F-3): identity rows only");
        return;
    }
    {
        let s = SqliteStore::open(&path.to_string_lossy()).unwrap();
        let row = |id: &str| s.read_tagged(id).unwrap().unwrap_or_else(|| panic!("no {id} row")).1;
        let gid = format!("{GROUP_ROW_PREFIX}{DG_GROUP}");
        let b = row(&gid);
        assert_eq!(bincode::serialize(&decode(&b).unwrap()).unwrap(), b, "group delegations row re-encodes byte for byte");
        let b = row(&format!("{gid}{GROUP_KEYOPS_ROW_SUFFIX}"));
        assert_eq!(bincode::serialize(&decode_events(&b).unwrap()).unwrap(), b, "group key-event row re-encodes byte for byte");
        let b = row(ROW_DOC);
        let ev: Events = bincode::deserialize(&b).expect("document-events row decodes");
        assert_eq!(bincode::serialize(&ev).unwrap(), b, "document-events row re-encodes byte for byte");
        let b = row(ROW_KEYOPS);
        let ev: Events = bincode::deserialize(&b).expect("member key-event row decodes");
        assert_eq!(bincode::serialize(&ev).unwrap(), b, "member key-event row re-encodes byte for byte");
        let b = row(ROW_CONTENT);
        let st: Stored = bincode::deserialize(&b).expect("encrypted-content row decodes");
        assert_eq!(bincode::serialize(&st).unwrap(), b, "encrypted-content row re-encodes byte for byte");
        println!("F-3 xv read: five rows decode and re-encode byte for byte");
    }
    let m: Material = bincode::deserialize(&std::fs::read(d.join(DG_MATERIAL)).unwrap()).unwrap();
    let expected: Vec<String> =
        std::fs::read_to_string(d.join(DG_EXPECTED)).unwrap().lines().map(String::from).collect();
    let got = dg_snapshot(&path, &m);
    for line in &got {
        println!("F-3 xv read: {line}");
    }
    assert_eq!(got, expected, "group members and levels, and every stored ciphertext's read, match what was written");
}
