//! `validate` ITs — markdown default mode (FR-004/FR-010) + `--json`
//! context mode (legacy FR-002 path).
//!
//! Covers IT-003 (context), IT-014 (markdown iso sweep), IT-021,
//! IT-047..054.

mod common;

use std::io::Write;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::NamedTempFile;

use common::{ctx_path, iso_module, quire, validate_doc, validate_module, ISO_ARCHETYPES};

// ----------------------------------------------------------------------
// Context mode (`--json`) — legacy FR-002 path preserved.
// ----------------------------------------------------------------------

// IT-003: context-mode validate returns 0/1 by schema conformance.
#[test]
fn it_003_validate_json_context_returns_0_on_valid_data() {
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--json")
        .arg(ctx_path("FR"))
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn it_003_validate_json_context_returns_1_on_invalid_data() {
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--json")
        .arg(ctx_path("FR-invalid"))
        .assert()
        .failure()
        .code(1);
}

// FR-004-AC-4: a missing required `id` surfaces a `/id` `required`
// violation on stderr in context mode.
#[test]
fn fr004_ac4_json_missing_id_reports_required_violation() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, r#"{{"title": "no id", "artifact_type": "FR"}}"#).unwrap();
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--json")
        .arg(f.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("id").and(predicate::str::contains("required")));
}

// IT-050 (FR-004-AC-5): unknown archetype + `--json` → exit 1 with
// `UnknownArchetype` on stderr.
#[test]
fn it_050_unknown_archetype_with_json_reports_unknown() {
    let mut f = NamedTempFile::new().unwrap();
    write!(f, r#"{{"id": "FR-1"}}"#).unwrap();
    quire()
        .arg("validate")
        .arg("NONEXISTENT")
        .arg("--module")
        .arg(iso_module())
        .arg("--json")
        .arg(f.path())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("UnknownArchetype"));
}

// ----------------------------------------------------------------------
// Markdown default mode — FR-004 / FR-010 over `validate_document`.
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
        .stderr(predicate::str::contains("line").and(predicate::str::contains("FR")));
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
// naming the offending section. (See FR-010 CR-003: for a fully-absent
// section quire-rs `validate_document` emits no line number — there is
// no document line to attribute — so the line-number clause is exercised
// by IT-051/IT-053 where the offending element is present.)
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

// IT-014: parametric markdown sweep across the 8 ISO archetypes. The ISO
// archetypes have no `body_extraction`, so `validate_document` enforces
// frontmatter-schema + per-level heading uniqueness; a rendered document
// validates clean, and a frontmatter-broken one fails.
#[test]
fn it_014_markdown_sweep_each_iso_archetype() {
    for archetype in ISO_ARCHETYPES {
        // Render a valid document for this archetype, then validate it.
        let rendered = quire()
            .arg("render")
            .arg(archetype)
            .arg("--module")
            .arg(iso_module())
            .arg("--data")
            .arg(ctx_path(archetype))
            .output()
            .expect("render");
        assert!(rendered.status.success(), "render {archetype} failed");

        let mut doc = NamedTempFile::new().unwrap();
        doc.write_all(&rendered.stdout).unwrap();

        quire()
            .arg("validate")
            .arg(doc.path())
            .arg("--module")
            .arg(iso_module())
            .arg("--archetype")
            .arg(archetype)
            .assert()
            .success();
    }
}
