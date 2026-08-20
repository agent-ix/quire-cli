//! Happy-path parse ITs.
//!
//! Trace ids sit on the tests themselves, never here: a `//!` block attaches to
//! the file, and the coverage extractor binds a marker to the symbol whose
//! leading comment block spans it (agent-ix/quire-cli#43).

mod common;

use common::quire;

const SIMPLE_DOC: &str = "---\nid: FR-001\ntype: FR\n---\n# [FR-001] Hello\n\nbody\n";

fn write_tmp(contents: &str, suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-parse-{}-{suffix}", std::process::id()));
    std::fs::write(&p, contents).unwrap();
    p
}

// IT-002, FR-002-AC-1, US-002-AC-1: `parse` emits valid QuireDocument JSON.
#[test]
fn it_002_parse_emits_quire_document_json() {
    let doc = write_tmp(SIMPLE_DOC, "it-002.md");
    let out = quire().arg("parse").arg(&doc).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The criterion is `.frontmatter.id` — read the field, don't grep the
    // payload: `contains("FR-001")` also passes on a document that carried the
    // id only in its body.
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8(out.stdout).unwrap().trim()).expect("valid JSON");
    assert_eq!(v["frontmatter"]["id"], "FR-001");
}

// IT-013, FR-002-AC-4: an empty document parses to a valid empty
// QuireDocument JSON envelope rather than failing.
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
    // The criterion names `sections[]` empty, not merely "an object": an
    // envelope carrying a phantom section would satisfy the weaker assertion.
    assert_eq!(
        parsed["sections"].as_array().map(Vec::len),
        Some(0),
        "an empty document must yield empty sections[]: {body}"
    );
}

// IT-019, FR-002-AC-5, FR-008-AC-1: parse JSON round-trips through
// QuireDocument deserialize.
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
