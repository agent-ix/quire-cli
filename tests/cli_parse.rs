//! Happy-path parse ITs.
//! Covers IT-002, IT-013, IT-019.

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::quire;

const SIMPLE_DOC: &str = "---\nid: FR-001\nartifact_type: FR\n---\n# [FR-001] Hello\n\nbody\n";

fn write_tmp(contents: &str, suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-parse-{}-{suffix}", std::process::id()));
    std::fs::write(&p, contents).unwrap();
    p
}

#[test]
fn it_002_parse_emits_quire_document_json() {
    let doc = write_tmp(SIMPLE_DOC, "it-002.md");
    quire()
        .arg("parse")
        .arg(&doc)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"frontmatter\""))
        .stdout(predicate::str::contains("FR-001"));
}

#[test]
fn it_013_empty_doc_parses_to_empty_json() {
    let doc = write_tmp("", "it-013.md");
    let out = quire().arg("parse").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let body = String::from_utf8(out.stdout).unwrap();
    // Should be a valid JSON document (no panic, parses as Value).
    let parsed: serde_json::Value =
        serde_json::from_str(body.trim()).expect("parse output is valid JSON");
    assert!(parsed.is_object(), "expected object envelope, got: {body}");
}

#[test]
fn it_019_parse_output_is_valid_json_roundtrip() {
    let doc = write_tmp(SIMPLE_DOC, "it-019.md");
    let out = quire().arg("parse").arg(&doc).output().unwrap();
    assert!(out.status.success());
    let body = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).expect("valid JSON");
    // The serialized form must round-trip back to the same JSON shape.
    let re_encoded = serde_json::to_string(&v).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&re_encoded).unwrap();
    assert_eq!(v, v2);
}
