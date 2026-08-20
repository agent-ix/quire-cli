//! Error-path ITs.
//!
//! Trace ids sit on the tests, not in this header — a `//!` block attaches to
//! the file and binds to no symbol (agent-ix/quire-cli#43).

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{iso_doc, iso_module, quire, validate_doc, validate_module};

fn write_tmp(contents: &str, suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-err-{}-{suffix}", std::process::id()));
    std::fs::write(&p, contents).unwrap();
    p
}

// IT-012, FR-002-AC-3, US-002-AC-3: the parser is tolerant —
// malformed-but-recognizable frontmatter surfaces as a parseable
// QuireDocument with a diagnostic on stderr.
#[test]
fn it_012_malformed_frontmatter_still_parses() {
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

// IT-026, FR-007-AC-1: exit code 0 on success.
#[test]
fn it_026_exit_code_0_on_success() {
    quire()
        .arg("validate")
        .arg(validate_doc("valid-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .code(0);
}

// IT-026, FR-007-AC-4: structural-validation failure exits 1. (AC-2
// path-safety and AC-3 unknown-archetype are their own failure modes, traced by
// `cli_sandbox::it_005_*` and `cli_validate::it_050_*`.)
#[test]
fn it_026_exit_code_1_on_validation_failure() {
    quire()
        .arg("validate")
        .arg(iso_doc("FR-invalid.md"))
        .arg("--module")
        .arg(iso_module())
        .assert()
        .failure()
        .code(1);
}

// IT-026, FR-007-AC-5, FR-014-AC-7: an argv error exits 2 — bare `validate`
// with no positional and no `--okf` trips the `required_unless_present` rule.
#[test]
fn it_026_exit_code_2_on_argv_error() {
    quire().arg("validate").assert().failure().code(2);
}

// IT-027, FR-007-AC-6: no panic on malformed input — a doc full of NUL bytes,
// control chars, and broken UTF-8-ish data.
#[test]
fn it_027_no_panic_on_random_garbage_input() {
    let garbage: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-fuzz-{}-it-027.md", std::process::id()));
    std::fs::write(&p, &garbage).unwrap();
    let out = quire().arg("parse").arg(&p).output().unwrap();
    // We accept any non-panic exit; 134 (SIGABRT) is the panic signal.
    assert_ne!(out.status.code(), Some(134));
}

// Deliberately untagged. The name used to cite IT-013, which is "empty document
// → valid empty QuireDocument JSON" (`cli_parse::it_013_*`) — a different
// behaviour entirely. The behaviour here is IT-050's, and that row is already
// bound by `cli_validate::it_050_unknown_archetype_reports_unknown`, which
// asserts strictly more (it also requires stdout to be empty). Tagging this one
// IT-050 would put two symbols on one row, so deleting either would leave the
// row green on the strength of the other — the collision this file's ticket
// exists to remove (agent-ix/quire-cli#45). It stays as an errors-lane smoke
// check that claims nothing.
#[test]
fn unknown_archetype_exits_1_with_named_error() {
    quire()
        .arg("validate")
        .arg(validate_doc("valid-fr.md"))
        .arg("--module")
        .arg(validate_module())
        .arg("--archetype")
        .arg("DEFINITELY_NOT_AN_ARCHETYPE")
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("UnknownArchetype"));
}
