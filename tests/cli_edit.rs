//! Edit subcommand ITs.
//! Covers FR-012 (section/block byte-exact writeback via the CLI).

mod common;

use std::io::Write;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::quire;

const EDIT_DOC: &str = "\
---
id: FR-012
artifact_type: FR
---
# [FR-012] Title

## Description

old description

## Behavior {#blk-behavior}

old behavior

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-012-AC-1 | TODO | Integration Test |
";

fn write_tmp(contents: &str, suffix: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-edit-{}-{suffix}", std::process::id()));
    std::fs::write(&p, contents).unwrap();
    p
}

#[test]
fn edit_by_heading_replaces_body_and_leaves_rest_byte_identical() {
    let doc = write_tmp(EDIT_DOC, "heading.md");
    let content = write_tmp("\nnew description text\n", "heading-body.txt");
    let assert = quire()
        .arg("edit")
        .arg(&doc)
        .arg("--heading")
        .arg("Description")
        .arg("--content")
        .arg(&content)
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();

    // The edited body is present...
    assert!(out.contains("new description text"));
    assert!(!out.contains("old description"));
    // ...and every untouched region is byte-identical.
    assert!(out.starts_with("---\nid: FR-012\nartifact_type: FR\n---\n"));
    assert!(out.contains("## Behavior {#blk-behavior}\n\nold behavior\n"));
    assert!(out.contains("| FR-012-AC-1 | TODO | Integration Test |"));
}

#[test]
fn edit_writes_in_place_with_out_pointing_at_input() {
    let doc = write_tmp(EDIT_DOC, "inplace.md");
    let content = write_tmp(
        "\n| ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-012-AC-1 | Given X, when Y, then Z | Integration Test |\n",
        "inplace-body.txt",
    );
    quire()
        .arg("edit")
        .arg(&doc)
        .arg("--heading")
        .arg("Acceptance Criteria")
        .arg("--content")
        .arg(&content)
        .arg("--out")
        .arg(&doc)
        .assert()
        .success();
    let updated = std::fs::read_to_string(&doc).unwrap();
    assert!(updated.contains("Given X, when Y, then Z"));
    assert!(!updated.contains("| FR-012-AC-1 | TODO |"));
    // Untouched sections survive the in-place rewrite.
    assert!(updated.contains("## Description\n\nold description\n"));
}

#[test]
fn edit_by_block_id_replaces_full_block() {
    let doc = write_tmp(EDIT_DOC, "block.md");
    // update_block takes the FULL block rendering: heading line + body.
    let content = write_tmp(
        "## Behavior {#blk-behavior}\n\nnew behavior body\n",
        "block-body.txt",
    );
    let assert = quire()
        .arg("edit")
        .arg(&doc)
        .arg("--block-id")
        .arg("blk-behavior")
        .arg("--content")
        .arg(&content)
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(out.contains("new behavior body"));
    assert!(!out.contains("old behavior"));
    assert!(out.contains("## Description\n\nold description\n"));
}

#[test]
fn edit_missing_heading_exits_1_without_writing() {
    let doc = write_tmp(EDIT_DOC, "missing.md");
    let content = write_tmp("x", "missing-body.txt");
    let out = quire()
        .arg("edit")
        .arg(&doc)
        .arg("--heading")
        .arg("Nonexistent")
        .arg("--content")
        .arg(&content)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    // The input file is left untouched.
    assert_eq!(std::fs::read_to_string(&doc).unwrap(), EDIT_DOC);
}

#[test]
fn edit_rejects_both_selectors_as_argv_error() {
    let doc = write_tmp(EDIT_DOC, "argv.md");
    let content = write_tmp("x", "argv-body.txt");
    quire()
        .arg("edit")
        .arg(&doc)
        .arg("--heading")
        .arg("Description")
        .arg("--block-id")
        .arg("blk-behavior")
        .arg("--content")
        .arg(&content)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn edit_requires_a_selector() {
    let doc = write_tmp(EDIT_DOC, "noselector.md");
    let content = write_tmp("x", "noselector-body.txt");
    let out = quire()
        .arg("edit")
        .arg(&doc)
        .arg("--content")
        .arg(&content)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("exactly one of --heading or --block-id"));
}

#[test]
fn edit_rejects_doc_and_content_both_stdin() {
    let mut child = quire()
        .arg("edit")
        .arg("-")
        .arg("--heading")
        .arg("Description")
        .arg("--content")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"x").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("both <doc> and --content from stdin"));
}
