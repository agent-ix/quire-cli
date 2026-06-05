//! `validate` ITs — markdown-only structural validation over
//! `quire_rs::validate_document` (FR-004 / FR-010). The `--json` context
//! mode was removed with the render retirement (FR-004 CR note).
//!
//! Covers IT-014 (direct-markdown ISO sweep), IT-021, IT-047..059.

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{iso_doc, iso_module, quire, validate_doc, validate_module, ISO_ARCHETYPES};

// ----------------------------------------------------------------------
// Markdown structural validation — FR-004 / FR-010 over validate_document.
// ----------------------------------------------------------------------

// IT-047 (FR-004-AC-1, FR-010-AC-4): a valid document exits 0, no output.
#[test]
fn it_047_valid_markdown_exits_0_no_output() {
    quire()
        .arg("validate")
        .arg(validate_doc("valid-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty());
}

// IT-048 (FR-004-AC-2): a broken document exits 1 with a line-numbered
// diagnostic naming the failing section/assert.
#[test]
fn it_048_broken_markdown_exits_1_with_line_numbered_diagnostic() {
    quire()
        .arg("validate")
        .arg(validate_doc("broken-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::is_match(r"line \d+")
                .unwrap()
                .and(predicate::str::contains("FR")),
        );
}

// IT-049 (FR-004-AC-3): `--archetype` overrides frontmatter resolution.
// The document has no `artifact_type`, so default resolution fails; the
// override lets it resolve to FR and validate clean.
#[test]
fn it_049_archetype_flag_overrides_frontmatter_resolution() {
    // Without override → resolution fails (no artifact_type).
    quire()
        .arg("validate")
        .arg(validate_doc("override-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .code(1);

    // With override → resolves to FR, validates clean.
    quire()
        .arg("validate")
        .arg(validate_doc("override-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .arg("--archetype")
        .arg("FR")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// IT-050 (FR-004-AC-6): an unknown `--archetype` → exit 1 with
// `UnknownArchetype` on stderr (re-pointed off the removed `--json` mode).
#[test]
fn it_050_unknown_archetype_reports_unknown() {
    quire()
        .arg("validate")
        .arg(validate_doc("valid-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .arg("--archetype")
        .arg("NONEXISTENT")
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("UnknownArchetype"));
}

// IT-051 (FR-010-AC-1): placeholder-only Specification → reason
// `placeholder`.
#[test]
fn it_051_placeholder_section_reports_placeholder() {
    quire()
        .arg("validate")
        .arg(validate_doc("placeholder-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("placeholder"));
}

// IT-052 (FR-010-AC-2): a missing required section → reason `missing`,
// naming the offending section.
#[test]
fn it_052_missing_section_reports_missing() {
    quire()
        .arg("validate")
        .arg(validate_doc("missing-section-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("missing").and(predicate::str::contains("Specification")));
}

// IT-053 (FR-010-AC-3): an Acceptance Criteria table with wrong columns
// (and zero conforming rows) → reason `assert`.
#[test]
fn it_053_bad_table_reports_assert() {
    quire()
        .arg("validate")
        .arg(validate_doc("broken-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("assert"));
}

// IT-021 + IT-054 (FR-010-AC-5): structural failure → empty stdout,
// non-empty stderr carrying the quire-rs diagnostics.
#[test]
fn it_054_structural_failure_empty_stdout_nonempty_stderr() {
    quire()
        .arg("validate")
        .arg(validate_doc("placeholder-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty().not());
}

#[test]
fn it_021_valid_markdown_writes_nothing_to_stdout() {
    quire()
        .arg("validate")
        .arg(validate_doc("valid-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// IT-014: parametric direct-markdown validate sweep across the 8 ISO
// archetypes (no render-then-validate). The ISO archetypes have no
// `body_extraction`, so `validate_document` enforces frontmatter-schema +
// per-level heading uniqueness; a well-formed document validates clean and
// a frontmatter-broken one (bad `id`) fails.
#[test]
fn it_014_markdown_sweep_each_iso_archetype() {
    for archetype in ISO_ARCHETYPES {
        quire()
            .arg("validate")
            .arg(iso_doc(&format!("{archetype}-valid.md")))
            .arg("--module")
            .arg(iso_module())
            .assert()
            .success()
            .stdout(predicate::str::is_empty());

        quire()
            .arg("validate")
            .arg(iso_doc(&format!("{archetype}-invalid.md")))
            .arg("--module")
            .arg(iso_module())
            .assert()
            .failure()
            .code(1);
    }
}

// ----------------------------------------------------------------------
// FR-004 archetype-resolution failure paths + path-safety arg label.
// ----------------------------------------------------------------------

// IT-056 (FR-004-AC-4): a document with no frontmatter and no `--archetype`
// exits 1; stderr names the missing frontmatter / `--archetype` remedy;
// empty stdout.
#[test]
fn it_056_no_frontmatter_names_archetype_remedy() {
    quire()
        .arg("validate")
        .arg(iso_doc("no-frontmatter.md"))
        .arg("--module")
        .arg(iso_module())
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("frontmatter").and(predicate::str::contains("--archetype")),
        );
}

// IT-057 (FR-004-AC-5): frontmatter present but no string `artifact_type`
// and no `--archetype` exits 1; the diagnostic names `--archetype` /
// `artifact_type`.
#[test]
fn it_057_no_artifact_type_names_archetype() {
    quire()
        .arg("validate")
        .arg(iso_doc("no-type.md"))
        .arg("--module")
        .arg(iso_module())
        .assert()
        .failure()
        .code(1)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("artifact_type").and(predicate::str::contains("--archetype")),
        );
}

// IT-055 (FR-005-AC-2 / FR-004-AC-7): a `..` document path exits 1 with a
// PathTraversal naming the positional `document` arg.
#[test]
fn it_055_dotdot_document_path_rejected() {
    quire()
        .arg("validate")
        .arg("../../etc/passwd")
        .arg("--module")
        .arg(iso_module())
        .assert()
        .failure()
        .code(1)
        .stderr(
            predicate::str::contains("PathTraversal").and(predicate::str::contains("document")),
        );
}

// IT-058 (FR-004-AC-7): the path-safety diagnostic names the offending
// arg label — `document` for the positional, `--module` for the module.
#[test]
fn it_058_path_safety_diagnostic_names_arg_label() {
    // Module arg violation names `--module`.
    quire()
        .arg("validate")
        .arg(validate_doc("valid-fr.md"))
        .arg("--module")
        .arg("../nope")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("--module"));
}

// IT-059 (FR-004-AC-8): `validate - --module $ISO` reads the document from
// stdin (path-safety-exempt) and still validates structurally.
#[test]
fn it_059_stdin_dash_is_path_safety_exempt_and_validated() {
    use std::io::Write;

    let valid = std::fs::read(iso_doc("FR-valid.md")).unwrap();
    let mut cmd = quire();
    cmd.arg("validate")
        .arg("-")
        .arg("--module")
        .arg(iso_module());
    let mut child = cmd
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&valid).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "stdin validate should succeed");
    assert!(out.stdout.is_empty(), "no stdout on success");
}
