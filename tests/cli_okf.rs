//! `validate --okf` ITs — permissive OKF bundle posture.
//!
//! `type` is required in BOTH postures, but under `--okf` unknown types,
//! broken `ix://` links, and `index.md` completeness gaps degrade to
//! warnings (exit 0) instead of errors. Bundles are built in a tempdir and
//! validated against the existing `validate-mod` module.

mod common;

use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

use common::{quire, validate_module};

fn bundle(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    for (rel, body) in files {
        let path = dir.path().join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }
    dir
}

// IT-069 (FR-014-AC-1, FR-014-AC-8, FR-003-AC-5): `type` stays required under
// OKF — an untyped document is a hard error carrying the shared
// `[frontmatter]` vocabulary.
#[test]
fn okf_untyped_document_is_error() {
    let dir = bundle(&[("NOTE-001.md", "---\nid: NOTE-001\n---\n# note\nbody\n")]);
    quire()
        .arg("validate")
        .arg("--okf")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("type").and(predicate::str::contains("[frontmatter]")));
}

// IT-070 (FR-014-AC-2, FR-014-AC-3): an unknown type and a dangling `ix://`
// link are tolerated as warnings (exit 0).
#[test]
fn okf_tolerates_unknown_type_and_broken_link() {
    let dir = bundle(&[(
        "X-1.md",
        "---\nid: X-1\ntype: weird\n---\n# x\nsee [missing](ix://o/r/MISSING)\n",
    )]);
    quire()
        .arg("validate")
        .arg("--okf")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stderr(
            predicate::str::contains("[unknown-type]")
                .and(predicate::str::contains("[dangling-reference]")),
        );
}

// IT-071 (FR-014-AC-4, FR-014-AC-5): an `index.md` missing a sibling artifact
// warns under OKF (exit 0).
#[test]
fn okf_index_incompleteness_warns() {
    let dir = bundle(&[
        ("X-1.md", "---\nid: X-1\ntype: weird\n---\n# x\nbody\n"),
        ("X-2.md", "---\nid: X-2\ntype: weird\n---\n# x\nbody\n"),
        (
            "index.md",
            "---\ntype: index\n---\n# Root\n\n## Contents\n\n* [X-1](./X-1.md)\n",
        ),
    ]);
    quire()
        .arg("validate")
        .arg("--okf")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stderr(
            predicate::str::contains("[index-incomplete]").and(predicate::str::contains("X-2")),
        );
}

// IT-072 (FR-014-AC-6): with no positional, `--okf` validates the document
// root `<scope>/spec` (CR-045) — never the scope itself, which was the
// repo-wide crawl.
#[test]
fn okf_defaults_to_scope_spec_directory() {
    let dir = bundle(&[
        ("spec/X-1.md", "---\nid: X-1\ntype: weird\n---\n# x\nbody\n"),
        // An untyped stray at the repo root would be a hard error if it
        // were visited; bounded to spec/, it never is.
        ("NOTE-1.md", "---\nid: NOTE-1\n---\n# note\nbody\n"),
    ]);
    quire()
        .arg("validate")
        .arg("--okf")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success();
}

// IT-085 (FR-014-AC-6): a scope with no `spec/` is a named error, never a
// silent fallback to walking the scope (CR-045).
#[test]
fn okf_missing_spec_root_is_a_named_error() {
    let dir = bundle(&[("X-1.md", "---\nid: X-1\ntype: weird\n---\n# x\nbody\n")]);
    // The interpolated path, not `contains("spec")`: the module-discovery
    // refusal names `spec-artifacts-process` and would satisfy the weaker
    // assertion (agent-ix/quire-cli#31).
    let expected = dir.path().join("spec").display().to_string();
    quire()
        .arg("validate")
        .arg("--okf")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
}

// IT-088 (agent-ix/quire-rs#110): a non-fatal bundle warning is emitted as a
// WARNING in the machine surface. Every `report.warnings` entry used to go
// through `emit_diagnostic`, which hardcodes `"severity": "error"` — so under
// `--diagnostics json` the severity contradicted the exit code, which was
// correctly 0. The frontmatter-less file is the CR-048 warning that made this
// visible, and the engine now distinguishes its two flavors by machine reason
// (quire-rs CR-051).
#[test]
fn it088_bundle_warnings_are_emitted_as_warnings_in_json() {
    let dir = bundle(&[
        (
            "spec/FR-001.md",
            "---\nid: FR-001\ntype: FR\n---\n# FR-001\nbody\n",
        ),
        // No frontmatter block at all.
        ("spec/draft.md", "# draft\n\nno front block.\n"),
        // A complete fence block that is not a YAML mapping.
        ("spec/listy.md", "---\n- a\n- b\n---\n# listy\nbody\n"),
    ]);

    let out = quire()
        .arg("--diagnostics-format")
        .arg("json")
        .arg("validate")
        .arg("--okf")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .output()
        .expect("run");

    // Warnings never move the exit code.
    assert!(
        out.status.success(),
        "frontmatter-less files are warnings, not errors: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let err = String::from_utf8_lossy(&out.stderr);
    let lines: Vec<serde_json::Value> = err
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    let warning_for = |needle: &str| {
        lines
            .iter()
            .find(|l| l["message"].as_str().is_some_and(|m| m.contains(needle)))
    };

    let absent = warning_for("draft.md").expect("the no-frontmatter warning");
    assert_eq!(
        absent["severity"], "warning",
        "a warning must not be emitted with error severity: {absent}"
    );
    assert_eq!(absent["kind"], "ValidationWarning");
    assert!(
        absent["message"]
            .as_str()
            .unwrap()
            .contains("[no-frontmatter]"),
        "carries the machine reason: {absent}"
    );

    // The two flavors are distinguishable in the machine surface
    // (quire-rs CR-051) — before it, both read `[no-frontmatter]`.
    let malformed = warning_for("listy.md").expect("the malformed-frontmatter warning");
    assert_eq!(malformed["severity"], "warning");
    assert!(
        malformed["message"]
            .as_str()
            .unwrap()
            .contains("[malformed-frontmatter]"),
        "the malformed flavor carries its own reason: {malformed}"
    );
}
