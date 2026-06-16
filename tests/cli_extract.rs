//! Happy-path extract ITs.
//! Covers IT-004, IT-020.

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{extract_module, extract_sample_doc, quire};

#[test]
fn it_004_extract_emits_envelope() {
    let out = quire()
        .arg("extract")
        .arg(extract_sample_doc())
        .arg("--module")
        .arg(extract_module())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let body = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).expect("valid JSON");
    assert!(v.get("extraction").is_some(), "missing extraction key");
    assert!(v.get("edges").is_some(), "missing edges key");
    assert!(v["edges"].is_array());
}

#[test]
fn it_020_extract_is_deterministic() {
    let one = quire()
        .arg("extract")
        .arg(extract_sample_doc())
        .arg("--module")
        .arg(extract_module())
        .output()
        .unwrap();
    let two = quire()
        .arg("extract")
        .arg(extract_sample_doc())
        .arg("--module")
        .arg(extract_module())
        .output()
        .unwrap();
    assert!(one.status.success());
    assert!(two.status.success());
    assert_eq!(one.stdout, two.stdout, "extract output not deterministic");
}

#[test]
fn extract_no_dsl_archetype_errors_cleanly() {
    // The ISO module has no object_types; the FR type isn't an
    // object_type either, so extract MUST exit 1 with a stderr message
    // — not crash.
    let doc = std::env::temp_dir().join(format!("quire-cli-extract-err-{}.md", std::process::id()));
    std::fs::write(&doc, "---\nid: FR-001\ntype: FR\n---\n# [FR-001] Hello\n").unwrap();
    quire()
        .arg("extract")
        .arg(&doc)
        .arg("--module")
        .arg(common::iso_module())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("FR"));
}
