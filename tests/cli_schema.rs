//! `schema` ITs — the FR-009 archetype input contract (frontmatter schema
//! plus `body_extraction` asserts) over `quire_rs::input_contract_for`
//! (upstream FR-029, recast by ADR 0004). No template-variable list.
//!
//! Covers IT-058 (path-safety), IT-060..063.

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{iso_module, quire, validate_module};

// IT-060 (FR-009-AC-1): `schema FR` exits 0; JSON contains the FR
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

// IT-061 (FR-009-AC-2): the JSON describes per-section asserts (headings /
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

// IT-062 (FR-009-AC-3): unknown archetype → exit 1 with `UnknownArchetype`
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
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("UnknownArchetype"));
}

// IT-063 (FR-009-AC-4): repeated `schema FR` calls produce byte-identical
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

// IT-058 (FR-009-AC-5): `schema` performs module path-safety equivalent to
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
        .code(1)
        .stderr(predicate::str::contains("--module"));
}
