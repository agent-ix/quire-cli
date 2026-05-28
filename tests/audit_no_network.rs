//! IT-008 / NFR-004-AC-2: under `strace -fe network`, none of the four
//! subcommands opens an AF_INET / AF_INET6 socket on a happy-path run.
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

fn fr_ctx() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/contexts/FR.json")
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
fn render_does_not_open_inet_socket() {
    if !strace_available() {
        eprintln!("skipping: strace not on PATH");
        return;
    }
    let module = iso_module();
    let data = fr_ctx();
    let trace = run_under_strace(&[
        "render",
        "FR",
        "--module",
        module.to_str().unwrap(),
        "--data",
        data.to_str().unwrap(),
    ]);
    assert_no_inet_socket(&trace, "render");
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
    let data = fr_ctx();
    let trace = run_under_strace(&[
        "validate",
        "FR",
        "--module",
        module.to_str().unwrap(),
        "--data",
        data.to_str().unwrap(),
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
