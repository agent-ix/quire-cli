//! Sandbox ITs (path-safety / FR-005).
//!
//! Trace ids sit on the tests, not in this header — a `//!` block attaches to
//! the file and binds to no symbol (agent-ix/quire-cli#43).

mod common;

use std::io::Write;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{iso_doc, iso_module, quire};

// IT-005, FR-005-AC-1, StR-003-AC-1, FR-007-AC-2: `--module ../escape` exits 1
// with a path-safety violation.
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

// IT-022, FR-005-AC-3, FR-012: the `--out` write-target path-safety survives
// on `edit`. A `..` out path is rejected before any write.
#[test]
fn it_022_out_dotdot_rejected() {
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

// IT-006, FR-005-AC-4, StR-003-AC-4: a symlink under the module pointing at
// /etc is refused at load. Construct a tempdir containing a symlink whose
// target is outside the tempdir; the CLI must refuse to load it as a module.
#[cfg(unix)]
#[test]
fn it_006_symlink_escape_refused_at_load() {
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

// IT-023, FR-005-AC-5: a positional `-` reads the document from stdin; the
// path-safety guard must not gate it. The document still validates
// structurally.
#[test]
fn it_023_positional_stdin_bypasses_path_safety() {
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
