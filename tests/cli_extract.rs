//! Happy-path extract ITs.
//!
//! Trace ids sit on the tests, not in this header — a `//!` block attaches to
//! the file and binds to no symbol (agent-ix/quire-cli#43).

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{extract_module, extract_sample_doc, quire};

// IT-004, FR-003-AC-1, US-004-AC-1: `extract` emits the {extraction, edges}
// envelope.
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

    // The fixture declares a real `implements` relationship, so asserting
    // only "edges is an array" passes on an empty harvest — and the
    // determinism test compares two empty arrays quite happily
    // (agent-ix/quire-cli#31). Assert what was harvested.
    let edges = v["edges"].as_array().unwrap();
    assert!(
        edges
            .iter()
            .any(|e| e["target"] == "StR-001" && e["type"] == "implements"),
        "the declared frontmatter relationship must be harvested: {edges:?}"
    );
}

// IT-100, FR-008-AC-2, FR-008-AC-5: the `extract` payload deserializes into the
// declared envelope — `extraction` an object, `edges` an array whose every
// element carries string `target` and `type` — and carries no CLI version
// string. `.get("edges").is_some()` (IT-004) passes on an array of nulls, and
// nothing else asserts the no-version rule at all.
#[test]
fn it_100_extract_payload_matches_the_declared_envelope_and_omits_version() {
    let out = quire()
        .arg("extract")
        .arg(extract_sample_doc())
        .arg("--module")
        .arg(extract_module())
        .output()
        .unwrap();
    assert!(out.status.success());
    let body = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(body.trim()).expect("valid JSON");

    assert!(
        v["extraction"].is_object(),
        "extraction is not an object: {v}"
    );
    let edges = v["edges"].as_array().expect("edges is an array");
    assert!(!edges.is_empty(), "the fixture declares an edge to harvest");
    for e in edges {
        assert!(e["target"].is_string(), "edge target is not a string: {e}");
        assert!(e["type"].is_string(), "edge type is not a string: {e}");
    }

    // FR-008-AC-5: no CLI version string in JSON output.
    let version = env!("CARGO_PKG_VERSION");
    assert!(
        !body.contains(version) && !body.contains("\"version\""),
        "the payload leaks a version string ({version}): {body}"
    );
}

// IT-020, FR-003-AC-4: an `extract` rerun produces byte-identical stdout.
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

// IT-015, US-004-AC-2: edges are deduped by (source, type, target). The same
// relationship declared twice in frontmatter, and the same `ix://` target
// linked twice in the body, each collapse to ONE edge — asserted by counting
// occurrences, not by "an edge exists", which a duplicating harvest also
// satisfies.
#[test]
fn it_015_edges_are_deduped_by_source_type_target() {
    let doc = std::env::temp_dir().join(format!("quire-cli-it-015-{}.md", std::process::id()));
    std::fs::write(
        &doc,
        "---\nid: EX-015\ntype: ExtractSample\nrelationships:\n  \
         - target: \"ix://agent-ix/quire-cli/spec/stakeholder/StR-001\"\n    type: implements\n  \
         - target: \"ix://agent-ix/quire-cli/spec/stakeholder/StR-001\"\n    type: implements\n\
         ---\n# EX-015\n\n## Purpose\n\n\
         see ix://agent-ix/quire-cli/spec/usecase/US-004 and again \
         ix://agent-ix/quire-cli/spec/usecase/US-004\n",
    )
    .unwrap();
    let out = quire()
        .arg("extract")
        .arg(&doc)
        .arg("--module")
        .arg(extract_module())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value =
        serde_json::from_str(String::from_utf8(out.stdout).unwrap().trim()).expect("valid JSON");
    let edges = v["edges"].as_array().expect("edges array");

    let implements = edges
        .iter()
        .filter(|e| e["target"] == "StR-001" && e["type"] == "implements")
        .count();
    assert_eq!(
        implements, 1,
        "the twice-declared frontmatter relationship must be harvested once: {edges:?}"
    );
    let references = edges
        .iter()
        .filter(|e| e["target"] == "US-004" && e["type"] == "references")
        .count();
    assert_eq!(
        references, 1,
        "the twice-linked body target must be harvested once: {edges:?}"
    );
}

// IT-099, FR-003-AC-2, US-004-AC-3: a document whose `type` resolves to no archetype
// carrying a DSL exits 1 with a diagnostic naming it — never a crash, never a
// partial extraction on stdout.
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
