//! IT-008 / NFR-004-AC-2: under `strace -fe network`, none of the
//! subcommands opens an AF_INET / AF_INET6 socket on a happy-path run
//! (registry present). IT-081 additionally proves the new scoped discovery
//! path (modules found via IX_FILAMENT_MODULES_PATH / the default root) is
//! network-free. The scoped-`validate` empty-discovery lazy-init — which
//! spawns `quoin` to bootstrap modules — is the documented NFR-004 exception
//! (ADR-0001) and is intentionally out of these happy-path traces; its
//! quoin-absent error path is covered by IT-082 in `cli_validate.rs`.
//!
//! `strace` is Linux-only. We additionally skip if strace isn't on PATH
//! so the test stays no-op on minimal containers.

#![cfg(target_os = "linux")]

use std::path::PathBuf;
use std::process::Command;

use assert_cmd::cargo::CommandCargoExt;

fn iso_module() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/iso")
}

fn validate_module() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/validate-mod")
}

fn iso_doc(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/iso-docs/{name}"))
}

fn quire_bin() -> PathBuf {
    Command::cargo_bin("quire")
        .expect("quire binary built")
        .get_program()
        .into()
}

fn strace_available() -> bool {
    Command::new("strace")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn run_under_strace(args: &[&str]) -> String {
    let bin = quire_bin();
    let mut cmd = Command::new("strace");
    cmd.arg("-f")
        .arg("-e")
        .arg("trace=network")
        .arg("-o")
        .arg("/dev/stderr");
    cmd.arg(&bin);
    for a in args {
        cmd.arg(a);
    }
    let out = cmd.output().expect("strace failed to launch");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Assert no socket(AF_INET..) syscall appears in `strace` output.
fn assert_no_inet_socket(trace: &str, subcommand: &str) {
    for line in trace.lines() {
        // A common pattern is `socket(AF_INET, SOCK_STREAM, ...) = ...`.
        // We tolerate AF_UNIX (used by /run/nscd, locale, etc.) and the
        // various `connect()`/`bind()` that follow it.
        if line.contains("socket(AF_INET") || line.contains("socket(AF_INET6") {
            panic!(
                "IT-008 violation: {subcommand} opened an inet socket:\n  {line}\nfull trace:\n{trace}"
            );
        }
    }
}

#[test]
fn schema_does_not_open_inet_socket() {
    if !strace_available() {
        eprintln!("skipping: strace not on PATH");
        return;
    }
    let module = validate_module();
    let trace = run_under_strace(&["schema", "FR", "--module", module.to_str().unwrap()]);
    assert_no_inet_socket(&trace, "schema");
}

#[test]
fn parse_does_not_open_inet_socket() {
    if !strace_available() {
        eprintln!("skipping: strace not on PATH");
        return;
    }
    let doc =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extract-mod/sample.md");
    let trace = run_under_strace(&["parse", doc.to_str().unwrap()]);
    assert_no_inet_socket(&trace, "parse");
}

#[test]
fn validate_does_not_open_inet_socket() {
    if !strace_available() {
        eprintln!("skipping: strace not on PATH");
        return;
    }
    let module = iso_module();
    let doc = iso_doc("FR-valid.md");
    let trace = run_under_strace(&[
        "validate",
        doc.to_str().unwrap(),
        "--module",
        module.to_str().unwrap(),
    ]);
    assert_no_inet_socket(&trace, "validate");
}

#[test]
fn extract_does_not_open_inet_socket() {
    if !strace_available() {
        eprintln!("skipping: strace not on PATH");
        return;
    }
    let module = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extract-mod");
    let doc =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extract-mod/sample.md");
    let trace = run_under_strace(&[
        "extract",
        doc.to_str().unwrap(),
        "--module",
        module.to_str().unwrap(),
    ]);
    assert_no_inet_socket(&trace, "extract");
}

#[test]
fn lookup_does_not_open_inet_socket() {
    if !strace_available() {
        eprintln!("skipping: strace not on PATH");
        return;
    }
    let doc =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/extract-mod/sample.md");
    let trace = run_under_strace(&["lookup", doc.to_str().unwrap(), "--heading", "Purpose"]);
    assert_no_inet_socket(&trace, "lookup");
}

// IT-081 (FR-004-AC-13, NFR-004-AC-2): scoped validation that discovers its
// module via the new `IX_FILAMENT_MODULES_PATH` root (modules present, so no
// `quoin` is spawned) validates the document AND opens no inet socket. HOME is
// pointed at an empty dir so the default install root resolves to a missing
// path, isolating the test from the host's real ~/.ix.
#[test]
fn scoped_env_discovery_validates_without_inet_socket() {
    if !strace_available() {
        eprintln!("skipping: strace not on PATH");
        return;
    }
    let base = std::env::temp_dir().join(format!("quire-cli-it081-{}", std::process::id()));
    let modroot = base.join("modroot");
    let home = base.join("home");
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&modroot).expect("mk modroot");
    std::fs::create_dir_all(&home).expect("mk home");
    // The iso module is reachable ONLY one level below the search root, via
    // IX_FILAMENT_MODULES_PATH — exercising the discovery root added in FR-004.
    std::os::unix::fs::symlink(iso_module(), modroot.join("iso")).expect("symlink iso module");

    // Scope is a doc-only dir (no modules of its own).
    let scope = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/iso-docs");
    let doc = iso_doc("FR-valid.md");
    let bin = quire_bin();

    let mut cmd = Command::new("strace");
    cmd.arg("-f")
        .arg("-e")
        .arg("trace=network")
        .arg("-o")
        .arg("/dev/stderr")
        .arg(&bin)
        .arg("validate")
        .arg(&doc)
        .arg("--scope")
        .arg(&scope)
        .env("HOME", &home)
        .env("IX_FILAMENT_MODULES_PATH", &modroot)
        .env_remove("IX_SCHEMA_PATH");
    let out = cmd.output().expect("strace failed to launch");
    let trace = String::from_utf8_lossy(&out.stderr).into_owned();
    let _ = std::fs::remove_dir_all(&base);

    assert!(
        out.status.success(),
        "scoped validate via IX_FILAMENT_MODULES_PATH should exit 0; output:\n{trace}"
    );
    assert_no_inet_socket(&trace, "validate (scoped env discovery)");
}
