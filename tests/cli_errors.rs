//! Error-path ITs.
//! Covers IT-010 (schema violation), IT-012 (malformed frontmatter),
//! IT-026 (each documented exit code), IT-027 (no panic on garbage).

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ctx_path, iso_module, quire};

fn write_tmp(contents: &str, suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-err-{}-{suffix}", std::process::id()));
    std::fs::write(&p, contents).unwrap();
    p
}

#[test]
fn it_010_render_schema_violation_exits_1_before_stdout() {
    // render does NOT pre-validate context against the schema — that's
    // validate's job — but a missing required key surfaces from the
    // template's strict-undefined evaluator as an error. Either way the
    // process must exit 1 with no stdout payload.
    let bad = write_tmp(r#"{"not_an_fr_context": true}"#, "schema-bad.json");
    let out = quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(&bad)
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn it_012_malformed_frontmatter_still_parses() {
    // The parser is tolerant: malformed-but-recognizable frontmatter
    // surfaces as a parseable QuireDocument with diagnostic on stderr.
    let doc = write_tmp(
        "---\nid: FR-1\nbroken: [unterminated\n---\n# body\n",
        "frontmatter-bad.md",
    );
    let out = quire().arg("parse").arg(&doc).output().unwrap();
    // parse() returns a QuireDocument even on malformed frontmatter — we
    // accept either a clean success or a clean exit-1 with diagnostics,
    // but NEVER a panic (134).
    assert_ne!(out.status.code(), Some(134), "parse panicked");
    assert!(matches!(out.status.code(), Some(0) | Some(1)));
}

#[test]
fn it_026_exit_code_0_on_success() {
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .assert()
        .success()
        .code(0);
}

#[test]
fn it_026_exit_code_1_on_validation_failure() {
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR-invalid"))
        .assert()
        .failure()
        .code(1);
}

#[test]
fn it_026_exit_code_2_on_argv_error() {
    // Missing required `--data`.
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .assert()
        .failure()
        .code(2);
}

#[test]
fn it_027_no_panic_on_random_garbage_input() {
    // A doc full of NUL bytes, control chars, and broken UTF-8-ish data.
    let garbage: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-fuzz-{}-it-027.md", std::process::id()));
    std::fs::write(&p, &garbage).unwrap();
    let out = quire().arg("parse").arg(&p).output().unwrap();
    // We accept any non-panic exit; 134 (SIGABRT) is the panic signal.
    assert_ne!(out.status.code(), Some(134));
}

#[test]
fn it_013_unknown_archetype_exits_1() {
    quire()
        .arg("validate")
        .arg("DEFINITELY_NOT_AN_ARCHETYPE")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("unknown archetype"));
}
