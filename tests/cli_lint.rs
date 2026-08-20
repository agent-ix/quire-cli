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

// IT-064 (FR-013-AC-1): a clean document exits 0, silent on both streams.
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

// IT-065 (FR-013-AC-2): a warning-severity finding exits 0 with a
// `warning: <rule-id>:` line plus the offending value on stderr, empty stdout.
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

// IT-066 (FR-013-AC-3): an error-severity finding exits 1 with an
// `error: <rule-id>:` line on stderr.
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

// IT-067 (FR-013-AC-4): a rule scoped `archetypes: [FR]` does not fire when
// `--archetype` overrides scoping to a non-matching name (FR-036-AC-3).
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

// IT-068 (FR-013-AC-5, FR-004): eager module-load failure — a `--module` path
// without manifest.yaml reports the REAL problem instead of a downstream
// UnknownArchetype. This is the `validate` half of the shared eager loader.
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

// IT-068 (FR-013-AC-5): and the `lint` half fails fast on the same missing
// manifest.
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
