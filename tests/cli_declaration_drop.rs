//! A traceability declaration dropped at load is never computed over silently
//! (PLAT-1854; quire-rs, PLAT-1804, owns the behaviour). Every command that
//! computes over the merged model refuses with the Invalid exit and names each
//! dropped declaration on stderr, including when the whole `traceability:`
//! section was dropped and no model remains.

mod common;

use std::fs;

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

/// A readable model plus two flow rules naming an archetype nothing declares:
/// each rule drops itself, the rest of the model loads.
const TWO_RULES_DROPPED: &str = "traceability:
  trace_targets: []
  flow_rules:
  - name: bad
    subject: [nowhere]
    match: [{path: [], field: id, present: true}]
    check: bad-check
    rationale: Names an archetype nothing declares.
  - name: worse
    subject: [nowhere]
    match: [{path: [], field: id, present: true}]
    check: worse-check
    rationale: Names an archetype nothing declares.
";

/// A `traceability:` section that cannot be deserialized: the whole section
/// drops and the module is left with no model at all.
const WHOLE_SECTION_DROPPED: &str = "traceability: not-a-map\n";

fn assert_refuses_naming(traceability: &str, names: &[&str], label: &str, argv: &[&str]) {
    let (_dir, scope, module) = scope_with_module(traceability);
    {
        let out = quire()
            .args(argv)
            .args(["--scope", &scope, "--module", &module])
            .output()
            .expect("run");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(INVALID_EXIT),
            "{label}; stderr: {stderr}"
        );
        assert!(
            out.stdout.is_empty(),
            "{label}: no report is written: {}",
            String::from_utf8_lossy(&out.stdout)
        );
        for name in names {
            assert!(
                stderr.contains(name),
                "{label}: stderr must name {name}: {stderr}"
            );
        }
        assert!(
            !stderr.contains("declares a `traceability:` model")
                && !stderr.contains("no loaded module declares"),
            "{label}: the drop, not an absent-model refusal, is named: {stderr}"
        );
    }
}

/// One test pair per command, so each command is proven to refuse on its own.
macro_rules! refuses {
    ($name:ident, $label:literal, [$($arg:literal),*]) => {
        mod $name {
            use super::*;

            #[test]
            fn names_each_dropped_declaration() {
                assert_refuses_naming(
                    TWO_RULES_DROPPED,
                    &["module 'm'", "flow_rule 'bad'", "flow_rule 'worse'"],
                    $label,
                    &[$($arg),*],
                );
            }

            #[test]
            fn names_a_wholly_dropped_traceability_section() {
                assert_refuses_naming(
                    WHOLE_SECTION_DROPPED,
                    &["module 'm'", "traceability_section"],
                    $label,
                    &[$($arg),*],
                );
            }
        }
    };
}

refuses!(coverage, "coverage", ["coverage", "--json"]);
refuses!(trace, "trace", ["trace", "--id", "FR-001", "--json"]);
refuses!(matrix, "matrix", ["matrix", "--format", "json"]);
refuses!(relations, "relations", ["relations", "--json"]);
refuses!(
    relations_design,
    "relations --design",
    ["relations", "--design", "--json"]
);
refuses!(
    relations_flow,
    "relations --flow",
    ["relations", "--flow", "--json"]
);
refuses!(symbols, "symbols", ["symbols", "--json"]);
refuses!(
    assurance,
    "assurance",
    ["assurance", "--expect-module", "m@0.0.1"]
);
