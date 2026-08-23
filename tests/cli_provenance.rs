//! FR-008-AC-6 (CR-104, #68) — every payload says which build produced it, and
//! `--version` names both versions distinctly.
//!
//! Driven through the **process boundary** rather than the library, because the
//! defect was never in a function: it was that a binary somebody installed
//! months ago printed a confident number and nothing about it said which engine
//! computed that number. The assertions below are the ones a reader picking up
//! a saved payload needs to be able to make.

mod common;

use common::{extract_module, extract_sample_doc, quire};
use serde_json::Value;

/// The provenance block out of a payload, with every member checked.
///
/// A helper rather than three copies, so a surface that grows a payload gets
/// the same assertions for one line — the way `coverage` and `properties`
/// diverged on what they rendered before #66.
fn assert_provenance(payload: &Value, surface: &str) {
    let engine = payload
        .get("engine")
        .unwrap_or_else(|| panic!("{surface}: no `engine` block: {payload}"));

    assert_eq!(
        engine["cli"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "{surface}: `engine.cli` must be this crate's version",
    );

    let version = engine["engine"]
        .as_str()
        .unwrap_or_else(|| panic!("{surface}: `engine.engine` is not a string: {engine}"));
    assert!(
        !version.is_empty() && version != "unknown",
        "{surface}: the engine version must resolve, got `{version}`",
    );
    // The whole point of reading the lockfile: the engine is a separate crate
    // on its own release cadence. Reporting this crate's version as the
    // engine's is the defect (#52, #68), not a harmless coincidence.
    assert_ne!(
        version,
        env!("CARGO_PKG_VERSION"),
        "{surface}: the CLI version is being reported as the engine version",
    );

    let capabilities = engine["capabilities"]
        .as_array()
        .unwrap_or_else(|| panic!("{surface}: `capabilities` is not an array: {engine}"));
    assert!(
        capabilities.iter().any(|t| t == "binding_census"),
        "{surface}: `binding_census` is the token this whole ticket is about: {engine}",
    );
    assert!(
        capabilities.iter().all(|t| t.is_string()),
        "{surface}: every capability must be a string token: {engine}",
    );

    // Closed envelope: three members and no more. A fourth would be a field
    // the published schemas reject, and the run that discovers that should be
    // this one rather than a consumer's.
    let members: Vec<&String> = engine.as_object().expect("object").keys().collect();
    assert_eq!(
        members.len(),
        3,
        "{surface}: unexpected members: {members:?}"
    );
}

// IT-123, FR-008-AC-6: `quire --version` reports the CLI and the engine
// version distinctly.
//
// `--version` reported `CARGO_PKG_VERSION` alone, so a binary linking a stale
// engine was indistinguishable from one linking a current engine. Asserting
// both strings appear is not enough on its own — if they happened to be equal
// the test would pass over the exact failure — so the label is asserted too.
#[test]
fn it_123_version_reports_the_cli_and_the_engine() {
    let out = quire().arg("--version").output().expect("--version runs");
    assert!(out.status.success(), "--version should exit 0");
    let line = String::from_utf8(out.stdout).expect("UTF-8");

    assert!(
        line.contains(env!("CARGO_PKG_VERSION")),
        "the CLI version is missing: {line}",
    );
    assert!(
        line.contains("engine"),
        "the engine number must be labelled or the two are indistinguishable: {line}",
    );

    // One line: `quire --version` is scraped, and growing a second line breaks
    // every caller doing so.
    assert_eq!(line.trim().lines().count(), 1, "{line}");
}

// IT-124, FR-008-AC-6: the `extract` payload carries provenance.
#[test]
fn it_124_extract_payload_carries_provenance() {
    let out = quire()
        .arg("extract")
        .arg(extract_sample_doc())
        .arg("--module")
        .arg(extract_module())
        .output()
        .expect("extract runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload: Value =
        serde_json::from_slice(&out.stdout).expect("the emitted payload is valid JSON");
    assert_provenance(&payload, "extract");

    // The engine's own values are untouched — provenance rides the envelope
    // (FR-008 behaviour rule 5), it does not annotate records.
    assert!(payload["extraction"].is_object(), "{payload}");
    assert!(payload["edges"].is_array(), "{payload}");
}

// IT-125, FR-008-AC-6: the `properties` payload carries provenance.
//
// This is the payload quoin's `spec-correctness` skill reads and a generated
// property test is derived from. A classification whose classifier version is
// unknowable is a classification nobody can re-derive.
#[test]
fn it_125_properties_payload_carries_provenance() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let module = dir.path().join("m");
    std::fs::create_dir_all(&module).expect("mkdir");
    std::fs::write(
        module.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n",
    )
    .expect("write manifest");
    let doc = dir.path().join("FR-001.md");
    std::fs::write(
        &doc,
        "---\nid: FR-001\ntype: FR\n---\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | Every finding absent from the merged map defaults to warning. | Test |\n",
    )
    .expect("write doc");

    let out = quire()
        .args([
            "properties",
            &doc.to_string_lossy(),
            "--module",
            &module.to_string_lossy(),
            "--json",
        ])
        .output()
        .expect("properties runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload: Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_provenance(&payload, "properties");

    // Non-vacuous: the fixture must actually have classified something, or the
    // provenance assertions ride on an empty payload.
    assert_eq!(
        payload["documents"][0]["criteria"]
            .as_array()
            .expect("criteria")
            .len(),
        1,
        "{payload}",
    );
}

// IT-126, FR-008-AC-6: the `coverage` payload carries provenance.
//
// The surface that matters most: every ecosystem figure in four battletest
// passes came from this payload, and none of them recorded which engine
// produced it.
#[test]
fn it_126_coverage_payload_carries_provenance() {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let scope = dir.path();
    std::fs::create_dir_all(scope.join("spec")).expect("mkdir spec");
    std::fs::write(
        scope.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n    id_column: ID\n",
    )
    .expect("write manifest");
    std::fs::write(
        scope.join("spec/FR-001.md"),
        "---\nid: FR-001\ntype: FR\n---\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | It does the thing. | Test |\n",
    )
    .expect("write doc");

    let out = quire()
        .args(["coverage", "--scope", &scope.to_string_lossy(), "--json"])
        .output()
        .expect("coverage runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload: Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    assert_provenance(&payload, "coverage");

    // Non-vacuous: the model must have matched a row, or `totals` is the
    // 0/0 state and this asserts provenance over a payload about nothing.
    assert_eq!(payload["totals"]["total"], 1, "{payload}");
}

// IT-127, FR-008-AC-5 (as narrowed by CR-104): the bare-version ban still
// holds on every payload.
//
// The half of AC-5 that survives. A loose `version` lets a payload assert its
// own contract revision, which puts the contract in two places; provenance
// under a named `engine` object is a different claim.
#[test]
fn it_127_no_payload_carries_a_bare_version_key() {
    let out = quire()
        .arg("extract")
        .arg(extract_sample_doc())
        .arg("--module")
        .arg(extract_module())
        .output()
        .expect("extract runs");
    let body = String::from_utf8(out.stdout).expect("UTF-8");
    for banned in ["\"version\"", "\"schema_version\"", "\"$schema\""] {
        assert!(
            !body.contains(banned),
            "the payload carries a bare {banned} key: {body}",
        );
    }
}
