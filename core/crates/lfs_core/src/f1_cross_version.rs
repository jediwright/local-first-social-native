//! F-1 (frontier): cross-version check for the two stored Keyhive formats.
//!
//! Run `f1_write_rows` on a build at the old Keyhive version, then
//! `f1_read_rows` on a build at the new one, both with `F1_XV_DIR` set to the
//! same directory outside the repo. Both tests are ignored by default.
//!
//! No seed is written anywhere: the rows hold public material only, and the
//! reader rebuilds the identity document with a freshly generated signer.
//! Members are recorded as 8-byte fingerprint prefixes.

use crate::ceremony::{decode, decode_events, reload_with, run, IDENTITY_KEYOPS_ROW_ID, IDENTITY_ROW_ID};
use crate::storage::{DocStore, SqliteStore};
use crate::{IdentityConfig, RootingLevel};
use keyhive_crypto::signer::memory::MemorySigner;
use rand::rngs::OsRng;

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
    println!("read rows written by {written_by} with {}", crate::storage::PIN_LABEL_KEYHIVE_CORE);
}
