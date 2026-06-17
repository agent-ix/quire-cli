//! `quire fix` ITs — unlinked-reference autofix (FR-015, ADR 0007).
//!
//! `fix` scans a bundle for bare artifact-id tokens that should be internal
//! relative-path links (upstream quire-rs FR-039). Dry-run reports
//! `would-fix`/`warning` and exits 1 when auto-fixes remain; `--write`
//! applies them in place and is idempotent. No `--module` is needed: the
//! check operates over the loaded corpus alone.

mod common;

use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

use common::quire;

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

const FR008: &str = "---\nid: FR-008\ntype: FR\n---\n# FR-008\n";

// IT-076 / FR-015-AC-1: dry-run reports `would-fix` and exits 1; no write.
#[test]
fn dry_run_reports_and_exits_one() {
    let dir = bundle(&[
        (
            "functional/FR-001-foo.md",
            "---\nid: FR-001\ntype: FR\n---\nSee FR-008 here.\n",
        ),
        ("functional/FR-008-byte.md", FR008),
    ]);
    let before = fs::read_to_string(dir.path().join("functional/FR-001-foo.md")).unwrap();

    quire()
        .arg("fix")
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("would-fix")
                .and(predicate::str::contains("FR-008"))
                .and(predicate::str::contains("[FR-008](./FR-008-byte.md)")),
        );

    // Untouched in dry-run.
    let after = fs::read_to_string(dir.path().join("functional/FR-001-foo.md")).unwrap();
    assert_eq!(before, after);
}

// IT-077 / FR-015-AC-2: --write applies the link; a second run is a no-op (exit 0).
#[test]
fn write_applies_and_is_idempotent() {
    let dir = bundle(&[
        (
            "functional/FR-001-foo.md",
            "---\nid: FR-001\ntype: FR\n---\nSee FR-008 here.\n",
        ),
        ("functional/FR-008-byte.md", FR008),
    ]);

    quire()
        .arg("fix")
        .arg(dir.path())
        .arg("--write")
        .assert()
        .success();

    let rewritten = fs::read_to_string(dir.path().join("functional/FR-001-foo.md")).unwrap();
    assert!(
        rewritten.contains("See [FR-008](./FR-008-byte.md) here."),
        "got: {rewritten}"
    );

    // Idempotent: nothing left to fix.
    quire()
        .arg("fix")
        .arg(dir.path())
        .arg("--write")
        .assert()
        .success();
    quire().arg("fix").arg(dir.path()).assert().success();
}

// IT-078 / FR-015-AC-3: a warn-only (unresolved) token warns, is never
// written, and does not by itself cause a nonzero exit.
#[test]
fn warn_only_never_written_exit_zero() {
    let dir = bundle(&[(
        "functional/FR-001-foo.md",
        "---\nid: FR-001\ntype: FR\n---\nSee FR-900 cross-repo.\n",
    )]);
    let before = fs::read_to_string(dir.path().join("functional/FR-001-foo.md")).unwrap();

    quire()
        .arg("fix")
        .arg(dir.path())
        .arg("--write")
        .assert()
        .success()
        .stderr(predicate::str::contains("FR-900").and(predicate::str::contains("unresolved")));

    let after = fs::read_to_string(dir.path().join("functional/FR-001-foo.md")).unwrap();
    assert_eq!(before, after);
}

// IT-079 / FR-015-AC-4: a clean bundle exits 0 in both modes.
#[test]
fn clean_bundle_exits_zero() {
    let dir = bundle(&[(
        "functional/FR-001-foo.md",
        "---\nid: FR-001\ntype: FR\n---\nNo references here.\n",
    )]);
    quire().arg("fix").arg(dir.path()).assert().success();
    quire()
        .arg("fix")
        .arg(dir.path())
        .arg("--write")
        .assert()
        .success();
}

// Multi-token inline-code span (`FR-008/FR-009`) must NOT be converted and
// must NOT corrupt the file (quire-rs FR-039-AC-10 + CLI overlap guard).
#[test]
fn multi_token_code_span_not_corrupted() {
    let dir = bundle(&[
        (
            "functional/FR-001-foo.md",
            "---\nid: FR-001\ntype: FR\n---\nSee `FR-008/FR-009` together.\n",
        ),
        ("functional/FR-008-byte.md", FR008),
        ("functional/FR-009-baz.md", "---\nid: FR-009\ntype: FR\n---\n# FR-009\n"),
    ]);
    let before = fs::read_to_string(dir.path().join("functional/FR-001-foo.md")).unwrap();
    quire().arg("fix").arg(dir.path()).arg("--write").assert().success();
    let after = fs::read_to_string(dir.path().join("functional/FR-001-foo.md")).unwrap();
    // The code span is left intact (no conversion, no corruption).
    assert_eq!(before, after, "multi-token code span must be left untouched");
    assert!(after.contains("`FR-008/FR-009`"));
    // And no orphan link fragment was produced.
    assert!(!after.contains(".md)F") && !after.contains(".md).md"));
}

// IT-080 / FR-015-AC-5: a `..` path on the bundle root is rejected by
// path-safety before any load.
#[test]
fn path_traversal_rejected() {
    quire()
        .arg("fix")
        .arg("../etc")
        .assert()
        .failure()
        .stderr(predicate::str::contains("PathTraversal").and(predicate::str::contains("..")));
}
