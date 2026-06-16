//! Lookup subcommand ITs.
//! Covers FR-011 / US-005.

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::quire;

const LOOKUP_DOC: &str = "\
---
id: FR-011
type: FR
---
# Lookup Title

intro
## 1. Behavior {#blk-behavior}

behavior body
### Detail

detail body
## Acceptance

- AC-1
";

fn write_tmp(contents: &str, suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-lookup-{}-{suffix}", std::process::id()));
    std::fs::write(&p, contents).unwrap();
    p
}

#[test]
fn lookup_by_heading_and_level_returns_h1() {
    let doc = write_tmp(LOOKUP_DOC, "heading-level.md");
    let out = quire()
        .arg("lookup")
        .arg(&doc)
        .arg("--heading")
        .arg("Lookup Title")
        .arg("--level")
        .arg("1")
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["heading"], "Lookup Title");
    assert_eq!(v["level"], 1);
}

#[test]
fn lookup_by_heading_uses_number_normalization() {
    let doc = write_tmp(LOOKUP_DOC, "heading.md");
    let out = quire()
        .arg("lookup")
        .arg(&doc)
        .arg("--heading")
        .arg("Behavior")
        .arg("--level")
        .arg("2")
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["heading"], "1. Behavior");
    assert_eq!(v["block_id"], "blk-behavior");
}

#[test]
fn lookup_by_block_id_returns_stable_block() {
    let doc = write_tmp(LOOKUP_DOC, "block-id.md");
    let out = quire()
        .arg("lookup")
        .arg(&doc)
        .arg("--block-id")
        .arg("blk-behavior")
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["heading"], "1. Behavior");
    assert_eq!(v["block_id"], "blk-behavior");
}

#[test]
fn lookup_by_generated_id_returns_section() {
    let doc = write_tmp(LOOKUP_DOC, "id.md");
    let out = quire()
        .arg("lookup")
        .arg(&doc)
        .arg("--id")
        .arg("detail-L6")
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["heading"], "Detail");
}

#[test]
fn lookup_content_outputs_raw_section_content() {
    let doc = write_tmp(LOOKUP_DOC, "content.md");
    quire()
        .arg("lookup")
        .arg(&doc)
        .arg("--heading")
        .arg("Acceptance")
        .arg("--content")
        .assert()
        .success()
        .stdout("\n- AC-1\n");
}

#[test]
fn lookup_missing_selector_exits_1_without_stdout() {
    let doc = write_tmp(LOOKUP_DOC, "missing.md");
    let out = quire()
        .arg("lookup")
        .arg(&doc)
        .arg("--block-id")
        .arg("does-not-exist")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--block-id 'does-not-exist'"));
}

#[test]
fn lookup_rejects_multiple_selectors_as_argv_error() {
    let doc = write_tmp(LOOKUP_DOC, "argv.md");
    quire()
        .arg("lookup")
        .arg(&doc)
        .arg("--heading")
        .arg("Behavior")
        .arg("--block-id")
        .arg("blk-behavior")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
}
