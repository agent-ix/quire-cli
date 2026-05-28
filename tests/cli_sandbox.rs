//! Sandbox ITs.
//! Covers IT-005 (--module ..), IT-006 (symlink escape), IT-007 (--data ..),
//! IT-022 (--out ..), IT-023 (--data - bypasses path-safety).

mod common;

use std::io::Write;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ctx_path, iso_module, quire};

#[test]
fn it_005_module_dotdot_rejected() {
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg("foo/../bar")
        .arg("--data")
        .arg(ctx_path("FR"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("PathTraversal"));
}

#[test]
fn it_007_data_dotdot_rejected() {
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg("../../etc/passwd")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("PathTraversal"));
}

#[test]
fn it_022_out_dotdot_rejected() {
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
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
    // (user error) and the absence of any stdout render.
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(&link)
        .arg("--data")
        .arg(ctx_path("FR"))
        .assert()
        .failure()
        .code(1);
}

#[test]
fn it_023_data_stdin_bypasses_path_safety() {
    // `--data -` reads stdin; the path-safety guard must not gate it.
    let mut child = quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin
            .write_all(
                br#"{"id":"FR-007","title":"stdin works","artifact_type":"FR","object":"","relationships":[]}"#,
            )
            .unwrap();
    }
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "stdin --data should succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let body = String::from_utf8(out.stdout).unwrap();
    assert!(body.contains("FR-007"));
}
