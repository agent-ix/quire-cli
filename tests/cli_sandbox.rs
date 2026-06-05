//! Sandbox ITs (path-safety / FR-005).
//! Covers IT-005 (--module ..), IT-006 (symlink escape),
//! IT-022 (--out .. on edit), IT-023 (positional `-` bypasses path-safety).

mod common;

use std::io::Write;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{iso_doc, iso_module, quire};

#[test]
fn it_005_module_dotdot_rejected() {
    quire()
        .arg("validate")
        .arg(iso_doc("FR-valid.md"))
        .arg("--module")
        .arg("foo/../bar")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("PathTraversal"));
}

#[test]
fn it_022_out_dotdot_rejected() {
    // The `--out` write-target path-safety survives on `edit`. A `..` out
    // path is rejected before any write.
    quire()
        .arg("edit")
        .arg(iso_doc("FR-valid.md"))
        .arg("--heading")
        .arg("FR-001 FR sweep fixture")
        .arg("--content")
        .arg(iso_doc("FR-valid.md"))
        .arg("--out")
        .arg("../escape.md")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("PathTraversal"));
}

#[cfg(unix)]
#[test]
fn it_006_symlink_escape_refused_at_load() {
    // Construct a tempdir containing a symlink whose target is outside
    // the tempdir (specifically `/etc`). The CLI must refuse to load it
    // as a module.
    let dir = tempfile::tempdir().unwrap();
    let link = dir.path().join("escape");
    std::os::unix::fs::symlink("/etc", &link).unwrap();

    // The link resolves to a real directory, but it's not a valid module
    // root (no manifest.yaml). load_module should report failure rather
    // than fall back to a sibling directory. We assert on exit code 1
    // (user error).
    quire()
        .arg("validate")
        .arg(iso_doc("FR-valid.md"))
        .arg("--module")
        .arg(&link)
        .assert()
        .failure()
        .code(1);
}

#[test]
fn it_023_positional_stdin_bypasses_path_safety() {
    // A positional `-` reads the document from stdin; the path-safety
    // guard must not gate it. The document still validates structurally.
    let valid = std::fs::read(iso_doc("FR-valid.md")).unwrap();
    let mut child = quire()
        .arg("validate")
        .arg("-")
        .arg("--module")
        .arg(iso_module())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(&valid).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "stdin validate should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stdout.is_empty(), "no stdout on success");
}
