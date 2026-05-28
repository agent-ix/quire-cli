//! Process-boundary integration tests. One smoke test per subcommand
//! against the real `spec-artifacts-iso` module.

use std::path::PathBuf;
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

fn iso_module() -> PathBuf {
    PathBuf::from("/home/peter/dev/spec-artifacts-iso/spec_artifacts_iso")
}

fn quire() -> Command {
    Command::cargo_bin("quire").expect("quire binary built")
}

fn write_tmp(contents: &str, suffix: &str) -> PathBuf {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-it-{}-{suffix}", std::process::id()));
    std::fs::write(&p, contents).unwrap();
    p
}

fn fr_context_json() -> &'static str {
    // The FR template gates several optional sections on field presence
    // (`object`, `relationships`, etc.) but the underlying MiniJinja env
    // is strict-undefined, so we provide explicit `null`/empty values
    // for every gate the template reads.
    r#"{
        "id": "FR-001",
        "title": "Render works",
        "artifact_type": "FR",
        "object": "",
        "relationships": []
    }"#
}

#[test]
fn render_smoke_fr_archetype() {
    if !iso_module().exists() {
        eprintln!("skipping: ISO module not present");
        return;
    }
    let data = write_tmp(fr_context_json(), "fr-ctx.json");
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(&data)
        .assert()
        .success()
        .stdout(predicate::str::contains("FR-001"))
        .stdout(predicate::str::contains("# [FR-001] Render works"));
}

#[test]
fn validate_smoke_fr_archetype_accepts_minimal_context() {
    if !iso_module().exists() {
        return;
    }
    let data = write_tmp(fr_context_json(), "fr-validate.json");
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(&data)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn validate_rejects_bad_data() {
    if !iso_module().exists() {
        return;
    }
    let bad = write_tmp(
        r#"{"id":"not-an-id","title":"","artifact_type":"FR"}"#,
        "bad.json",
    );
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(&bad)
        .assert()
        .failure()
        .code(1);
}

#[test]
fn parse_smoke_emits_json() {
    let doc = write_tmp(
        "---\nid: FR-001\nartifact_type: FR\n---\n# [FR-001] Hello\n\nbody\n",
        "parse-doc.md",
    );
    quire()
        .arg("parse")
        .arg(&doc)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"frontmatter\""))
        .stdout(predicate::str::contains("FR-001"));
}

#[test]
fn extract_smoke_no_dsl_archetype_errors() {
    // The ISO module has artifact_types only; without an object_type
    // carrying a body_extraction DSL, extract should fail cleanly with
    // exit 1 and a stderr message — not crash.
    if !iso_module().exists() {
        return;
    }
    let doc = write_tmp(
        "---\nid: FR-001\nartifact_type: FR\n---\n# [FR-001] Hello\n",
        "extract-doc.md",
    );
    quire()
        .arg("extract")
        .arg(&doc)
        .arg("--module")
        .arg(iso_module())
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("FR"));
}

#[test]
fn render_rejects_dotdot_in_module() {
    let data = write_tmp(fr_context_json(), "rejects-fr-ctx.json");
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg("foo/../bar")
        .arg("--data")
        .arg(&data)
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("PathTraversal"));
}
