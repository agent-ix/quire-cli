//! A traceability declaration dropped at load is never computed over silently
//! (PLAT-1854; upstream quire-rs FR-058-AC-47, PLAT-1804): `coverage` and
//! `trace` refuse with the Invalid exit and name each dropped declaration on
//! stderr, including when the whole `traceability:` section was dropped and no
//! model remains.

mod common;

use std::fs;

use ix_trace_rs::trace;
use tempfile::TempDir;

use common::quire;

/// Exit code of an invalid request (FR-007).
const INVALID_EXIT: i32 = 3;

/// A scope holding an empty `spec/` and one module whose manifest carries the
/// given `traceability:` block. Returns `(scope, module)` paths.
fn scope_with_module(traceability: &str) -> (TempDir, String, String) {
    let dir = TempDir::new().expect("tempdir");
    fs::create_dir_all(dir.path().join("spec")).expect("spec dir");
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("module dir");
    fs::write(
        m.join("manifest.yaml"),
        format!(
            "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n- name: FR\n{traceability}"
        ),
    )
    .expect("write manifest");
    let scope = dir.path().to_string_lossy().into_owned();
    let module = m.to_string_lossy().into_owned();
    (dir, scope, module)
}

/// A readable model plus one flow rule naming an archetype nothing declares:
/// the rule drops itself, the rest of the model loads.
const ONE_RULE_DROPPED: &str = "traceability:
  trace_targets: []
  flow_rules:
  - name: bad
    subject: [nowhere]
    match: [{path: [], field: id, present: true}]
    check: bad-check
    rationale: Names an archetype nothing declares.
";

/// A `traceability:` section that cannot be deserialized: the whole section
/// drops and the module is left with no model at all.
const WHOLE_SECTION_DROPPED: &str = "traceability: not-a-map\n";

fn assert_refuses_naming(args: &[&str], names: &[&str]) {
    let out = quire().args(args).output().expect("run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(INVALID_EXIT), "stderr: {stderr}");
    assert!(
        out.stdout.is_empty(),
        "no report is written: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    for name in names {
        assert!(stderr.contains(name), "stderr must name {name}: {stderr}");
    }
    assert!(
        !stderr.contains("declares a `traceability:` model"),
        "the drop, not the absent-model refusal, is named: {stderr}"
    );
}

#[trace("FR-058-AC-47")]
#[test]
fn coverage_refuses_a_dropped_declaration_and_names_it() {
    let (_dir, scope, module) = scope_with_module(ONE_RULE_DROPPED);
    assert_refuses_naming(
        &["coverage", "--scope", &scope, "--module", &module, "--json"],
        &["module 'm'", "flow_rule 'bad'"],
    );
}

#[trace("FR-058-AC-47")]
#[test]
fn trace_refuses_a_dropped_declaration_and_names_it() {
    let (_dir, scope, module) = scope_with_module(ONE_RULE_DROPPED);
    assert_refuses_naming(
        &[
            "trace", "--scope", &scope, "--module", &module, "--id", "FR-001", "--json",
        ],
        &["module 'm'", "flow_rule 'bad'"],
    );
}

#[trace("FR-058-AC-47")]
#[test]
fn coverage_names_a_wholly_dropped_traceability_section() {
    let (_dir, scope, module) = scope_with_module(WHOLE_SECTION_DROPPED);
    assert_refuses_naming(
        &["coverage", "--scope", &scope, "--module", &module, "--json"],
        &["module 'm'", "traceability_section"],
    );
}

#[trace("FR-058-AC-47")]
#[test]
fn trace_names_a_wholly_dropped_traceability_section() {
    let (_dir, scope, module) = scope_with_module(WHOLE_SECTION_DROPPED);
    assert_refuses_naming(
        &[
            "trace", "--scope", &scope, "--module", &module, "--id", "FR-001", "--json",
        ],
        &["module 'm'", "traceability_section"],
    );
}
