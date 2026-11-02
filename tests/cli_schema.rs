//! `schema` ITs — the FR-009 archetype input contract (frontmatter schema
//! plus `body_extraction` asserts) over `quire_rs::input_contract_for`
//! (upstream FR-029, recast by ADR 0004). No template-variable list.
//!
//! Covers IT-058 (path-safety), IT-060..063.

mod common;

use assert_cmd::prelude::*;
use ix_trace_rs::trace;
use predicates::prelude::*;

use common::{iso_module, quire, validate_module};

// IT-060, FR-009-AC-1: `schema FR` exits 0; JSON contains the FR
// frontmatter schema and the `body_extraction` asserts (required headings
// / columns). The validate-mod FR carries asserts (Specification section,
// Acceptance Criteria table columns).
#[test]
fn it_060_schema_fr_emits_frontmatter_schema_and_asserts() {
    quire()
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("\"frontmatter_schema\"")
                .and(predicate::str::contains("\"sections\""))
                .and(predicate::str::contains("Specification"))
                .and(predicate::str::contains("Acceptance Criteria")),
        );
}

// Current public modules declare semantic-core 0.3.3. Keep schema discovery
// working for an artifact type when that semantic block is present.
#[test]
fn it_064_schema_accepts_semantic_core_033_module() {
    quire()
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"archetype\":\"FR\""));
}

// Existing public object modules still declare semantic-core 0.3.2. Keep
// accepting that manifest while newer artifact modules declare 0.3.3.
#[trace("FR-009-AC-1")]
#[test]
fn it_065_schema_accepts_semantic_core_032_module() {
    let temp = tempfile::tempdir().expect("temporary module root");
    let source = validate_module();
    std::fs::create_dir_all(temp.path().join("schemas")).expect("schema directory");
    for relative in ["manifest.yaml", "schemas/fr-frontmatter.schema.json"] {
        std::fs::copy(source.join(relative), temp.path().join(relative))
            .expect("copy fixture file");
    }
    let manifest = temp.path().join("manifest.yaml");
    let original = std::fs::read_to_string(&manifest).expect("read fixture manifest");
    assert!(original.contains("semantic_core: 0.3.3"));
    let contents = original.replace("semantic_core: 0.3.3", "semantic_core: 0.3.2");
    std::fs::write(&manifest, contents).expect("write fixture manifest");

    quire()
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"archetype\":\"FR\""));
}

// IT-061, FR-009-AC-2: the JSON describes per-section asserts (headings /
// columns / id-patterns) — the asserts-based contract — and carries NO
// template-variable list.
#[test]
fn it_061_schema_describes_asserts_not_template_vars() {
    quire()
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stdout(
            // Asserts surface: column headers + the section kind.
            predicate::str::contains("\"columns\"")
                .and(predicate::str::contains("\"heading\""))
                .and(predicate::str::contains("table_row"))
                // No template-variable vocabulary.
                .and(predicate::str::contains("template").not())
                .and(predicate::str::contains("variable").not()),
        );
}

// IT-062, FR-009-AC-3: unknown archetype → exit 1 with `UnknownArchetype`
// on stderr, empty stdout.
#[test]
fn it_062_schema_unknown_archetype_errors() {
    quire()
        .arg("schema")
        .arg("NONEXISTENT")
        .arg("--module")
        .arg(iso_module())
        .assert()
        .failure()
        .code(3)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("UnknownArchetype"));
}

// IT-063, FR-009-AC-4: repeated `schema FR` calls produce byte-identical
// stdout.
#[test]
fn it_063_schema_output_is_byte_stable() {
    let a = quire()
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg(validate_module())
        .output()
        .expect("schema run a");
    let b = quire()
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg(validate_module())
        .output()
        .expect("schema run b");
    assert!(a.status.success() && b.status.success());
    assert_eq!(a.stdout, b.stdout, "schema stdout must be byte-stable");
}

// IT-058, FR-009-AC-5: `schema` performs module path-safety equivalent to
// `validate` — a `..` module path is rejected.
#[test]
fn it_058_schema_module_path_safety() {
    quire()
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg("../nope")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("--module"));
}
