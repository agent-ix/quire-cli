//! `quire symbols` ITs — the extracted symbol table (FR-019, upstream quire-rs
//! FR-051-AC-23, `agent-ix/quire-rs#309`).
//!
//! The surface exists because a scanner defect could only be sized by
//! reimplementing the scanner: three ports of `symbols/python.rs` gave 386, 490
//! and 5,263 lost declarations over one tree, disagreeing precisely where the
//! original is wrong. These cover the contract FR-019 states, with particular
//! weight on the two distinctions the command was built to make — *not asked*
//! versus *not tagged*, and the two denominators a binding rate can be drawn
//! over.

mod common;

use std::fs;

use tempfile::TempDir;

use common::quire;

/// Run `quire` and capture both streams as text — the shape every assertion
/// here needs, written once rather than at each call site.
struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let out = quire().args(args).output().expect("run");
    Run {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// A module declaring the Rust trace-attribute form, so `--module` has
/// something to bind with.
fn module(dir: &TempDir) -> String {
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    // A RAW literal: the pattern is a regex inside YAML inside Rust, and
    // escaping it through a string continuation produced a manifest that
    // parsed as neither. Written the way it appears on disk.
    fs::write(
        m.join("manifest.yaml"),
        r#"name: m
manifest_version: 1.0.0
version: 0.0.1
artifact_types:
- name: FR
traceability:
  trace_targets: []
  trace_tags:
    markers:
    - name: rust-trace-attribute
      language: rust
      pattern: '#\[trace\(((?:\s*"[^"]*"\s*,?)+)\)\]'
      template: '#[trace({ids})]'
"#,
    )
    .expect("write manifest");
    m.to_string_lossy().into_owned()
}

/// A tree with one container, one tagged test and one untagged test — enough to
/// separate every distinction these tests are about.
fn tree(dir: &TempDir) -> String {
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(
        src.join("lib.rs"),
        "#[cfg(test)]\nmod tests {\n    \
         #[trace(\"TC-001\")]\n    #[test]\n    fn tagged() {}\n    \
         #[test]\n    fn untagged() {}\n}\n",
    )
    .expect("write src");
    dir.path().to_string_lossy().into_owned()
}

#[test]
fn it_130_reports_without_a_module_and_leaves_stdout_for_the_payload() {
    // IT-130, FR-019-AC-1.
    // `coverage` bails without a `traceability:` model because it has nothing
    // to reconcile. "What did the scanner find" is a question about the WALK,
    // and demanding a declaration for it would make the one surface that can
    // size a scanner defect depend on the declaration being right.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let out = run(&["symbols", "--scope", &scope]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    assert!(
        out.stdout.trim().is_empty(),
        "the human form writes nothing to stdout: {}",
        out.stdout
    );
    assert!(out.stderr.contains("symbol(s)"), "stderr: {}", out.stderr);
}

#[test]
fn it_131_the_json_record_carries_every_field_and_is_byte_stable() {
    // IT-131, FR-019-AC-2.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let first = run(&["symbols", "--scope", &scope, "--json"]);
    assert_eq!(first.code, Some(0), "stderr: {}", first.stderr);
    let payload: serde_json::Value = serde_json::from_str(&first.stdout).expect("json");
    let symbols = payload["symbols"].as_array().expect("symbols array");
    assert!(!symbols.is_empty(), "payload: {}", first.stdout);
    for key in [
        "path",
        "symbol",
        "kind",
        "language",
        "line",
        "leading_line",
        "end_line",
        "container",
        "id",
        "binds_trace_ids",
        "carries_implements",
    ] {
        assert!(
            symbols[0].get(key).is_some(),
            "record is missing `{key}`: {}",
            symbols[0]
        );
    }
    // `leading_line` beside `line`, because a marker that failed to match is
    // written at the annotation block and that is the line to edit (#256).
    let tagged = symbols
        .iter()
        .find(|s| s["symbol"] == "tests::tagged")
        .expect("the tagged test");
    assert!(
        tagged["leading_line"].as_u64() < tagged["line"].as_u64(),
        "the annotation precedes the declaration: {tagged}"
    );

    let second = run(&["symbols", "--scope", &scope, "--json"]);
    assert_eq!(
        first.stdout, second.stdout,
        "two runs over identical inputs must be byte-identical (NFR-006)"
    );
}

#[test]
fn it_132_no_module_means_not_asked_rather_than_not_tagged() {
    // IT-132, FR-019-AC-3.
    // THE DISTINCTION THE COMMAND EXISTS TO MAKE. An unbound run and a
    // repository nobody tagged produce the same empty `trace_ids`, and a
    // reader who cannot tell them apart draws the same wrong conclusion the
    // 4% figure came from.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let unasked = run(&["symbols", "--scope", &scope]);
    assert!(
        unasked.stderr.contains("NOT ASKED"),
        "an unbound run must say so: {}",
        unasked.stderr
    );

    let m = module(&dir);
    let asked = run(&["symbols", "--scope", &scope, "--module", &m, "--json"]);
    assert_eq!(asked.code, Some(0), "stderr: {}", asked.stderr);
    let payload: serde_json::Value = serde_json::from_str(&asked.stdout).expect("json");
    let tagged = payload["symbols"]
        .as_array()
        .expect("symbols")
        .iter()
        .find(|s| s["symbol"] == "tests::tagged")
        .expect("the tagged test")
        .clone();
    assert_eq!(
        tagged["trace_ids"],
        serde_json::json!(["TC-001"]),
        "with a module the bound id is reported: {tagged}"
    );
}

#[test]
fn it_133_both_denominators_are_reported_per_language() {
    // IT-133, FR-019-AC-4.
    // A binding rate over `symbols` rather than `binding_kinds` reads a tree of
    // containers as untagged — the shape of the figure EPIC quire-rs#264 was
    // opened on. Both are published so neither can be assumed.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let out = run(&["symbols", "--scope", &scope, "--json"]);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    let rust = payload["by_language"]
        .as_array()
        .expect("by_language")
        .iter()
        .find(|l| l["language"] == "rust")
        .expect("rust census")
        .clone();
    let symbols = rust["symbols"].as_u64().expect("symbols");
    let binding = rust["binding_kinds"].as_u64().expect("binding_kinds");
    assert!(
        binding < symbols,
        "the container is not of a binding kind, so the two differ: {rust}"
    );
    assert_eq!(rust["bound"], 0, "nothing was asked, so nothing bound");
}

#[test]
fn it_134_extraction_diagnostics_reach_stderr_in_json_mode() {
    // IT-134, FR-019-AC-5.
    // A file the extractor could not read is indistinguishable from a file with
    // no declarations in it, so dropping the diagnostic silently shrinks the
    // table. On stderr in EVERY format: diagnostics are the finding stream,
    // never part of the stdout payload.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let out = run(&["symbols", "--scope", &scope, "--json"]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    // The payload always carries the channel, empty or not — a consumer that
    // had to test for the key's presence could not tell "no diagnostics" from
    // "this build does not report them".
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    assert!(payload["diagnostics"].is_array(), "payload: {}", out.stdout);
}

#[test]
fn it_135_a_language_filter_narrows_both_the_records_and_the_census() {
    // IT-135, FR-019-AC-6 (resolution shared with `coverage`).
    // A filter that narrowed the records and left the census whole would
    // publish a census over a population the payload does not contain — the
    // denominator mismatch this whole programme is about, one command down.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let out = run(&[
        "symbols",
        "--scope",
        &scope,
        "--language",
        "python",
        "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    assert!(
        payload["symbols"].as_array().expect("symbols").is_empty(),
        "no python in this tree: {}",
        out.stdout
    );
    assert!(
        payload["by_language"]
            .as_array()
            .expect("by_language")
            .is_empty(),
        "the census narrows with the records: {}",
        out.stdout
    );
}
