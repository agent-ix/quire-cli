//! `quire lint` process-boundary tests (FR-013-lint) plus the eager
//! module-load failure surface shared by validate/extract/schema
//! (FR-004 CR note / upstream FR-013-AC-13).

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{fixture_root, quire};

fn lint_module() -> std::path::PathBuf {
    fixture_root().join("lint-mod")
}

fn lint_doc(name: &str) -> std::path::PathBuf {
    fixture_root().join(format!("lint-mod/docs/{name}"))
}

#[test]
fn lint_clean_doc_exits_0_silent() {
    quire()
        .arg("lint")
        .arg(lint_doc("clean.md"))
        .arg("--module")
        .arg(lint_module())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

#[test]
fn lint_warning_finding_exits_0_with_stderr() {
    quire()
        .arg("lint")
        .arg(lint_doc("warn.md"))
        .arg("--module")
        .arg(lint_module())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("warning: ac-verification-method")
                .and(predicate::str::contains("Docs audit")),
        );
}

#[test]
fn lint_error_finding_exits_1() {
    quire()
        .arg("lint")
        .arg(lint_doc("error.md"))
        .arg("--module")
        .arg(lint_module())
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("error: configuration-scope")
                .and(predicate::str::contains("vibes")),
        );
}

// Rule scoped `archetypes: [FR]` does not fire when --archetype
// overrides scoping to a non-matching name (FR-036-AC-3).
#[test]
fn lint_archetype_scoping_respects_override() {
    quire()
        .arg("lint")
        .arg(lint_doc("warn.md"))
        .arg("--module")
        .arg(lint_module())
        .arg("--archetype")
        .arg("NFR")
        .assert()
        .success()
        .stderr(predicate::str::is_empty());
}

// Eager module-load failure: a --module path without manifest.yaml
// reports the REAL problem instead of a downstream UnknownArchetype.
#[test]
fn missing_manifest_reports_real_reason_not_unknown_archetype() {
    let empty = tempfile::tempdir().expect("tmpdir");
    quire()
        .arg("validate")
        .arg(lint_doc("clean.md"))
        .arg("--module")
        .arg(empty.path())
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("manifest.yaml not found")
                .and(predicate::str::contains("UnknownArchetype").not()),
        );
}

#[test]
fn lint_missing_manifest_fails_fast_too() {
    let empty = tempfile::tempdir().expect("tmpdir");
    quire()
        .arg("lint")
        .arg(lint_doc("clean.md"))
        .arg("--module")
        .arg(empty.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("manifest.yaml not found"));
}
