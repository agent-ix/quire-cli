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

// `type` stays required under OKF: an untyped document is a hard error.
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

// Unknown type + dangling `ix://` link are tolerated as warnings (exit 0).
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

// An index.md missing a sibling artifact warns under OKF (exit 0).
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

// With no positional, --okf validates the --scope directory.
#[test]
fn okf_defaults_to_scope_directory() {
    let dir = bundle(&[("X-1.md", "---\nid: X-1\ntype: weird\n---\n# x\nbody\n")]);
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
