//! `verify-binary` accepts a matching `--version` and rejects a mismatch.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use ix_trace_rs::trace;
use quire_dist::verify_binary_version;

fn fake_binary(dir: &Path, reported: &str) -> std::path::PathBuf {
    let path = dir.join("quire");
    fs::write(&path, format!("#!/bin/sh\necho 'quire {reported}'\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[trace("TC-818", "FR-022-AC-4")]
#[test]
fn verify_binary_accepts_match_and_rejects_mismatch_or_invalid_version() {
    let dir = tempfile::tempdir().unwrap();
    let binary = fake_binary(dir.path(), "1.2.3");
    verify_binary_version("1.2.3", &binary).unwrap();
    assert!(verify_binary_version("1.2.4", &binary).is_err());
    assert!(verify_binary_version("not-semver", &binary).is_err());
}
