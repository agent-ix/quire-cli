//! I/O contract ITs.
//!
//! Trace ids sit on the tests, not in this header — a `//!` block attaches to
//! the file and binds to no symbol (agent-ix/quire-cli#43).

mod common;

use std::io::Write;
use std::process::Stdio;

use common::{quire, validate_module};

const SIMPLE_DOC: &str = "---\nid: FR-001\ntype: FR\n---\n# [FR-001] Hi\n";

// IT-011 (FR-002-AC-2, US-002-AC-4): `parse -` reads stdin, and its output is
// byte-identical to `parse <file>` on the same document — the criterion is an
// equivalence, so asserting only "the id appears" would pass on a stdin path
// that dropped every section.
#[test]
fn it_011_parse_dash_reads_stdin() {
    let mut child = quire()
        .arg("parse")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(SIMPLE_DOC.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let body = String::from_utf8(out.stdout).unwrap();
    assert!(body.contains("FR-001"));

    // FR-002-AC-2: identical output to the file-path invocation.
    let path = std::env::temp_dir().join(format!("quire-cli-it-011-{}.md", std::process::id()));
    std::fs::write(&path, SIMPLE_DOC).unwrap();
    let from_file = quire().arg("parse").arg(&path).output().unwrap();
    assert!(from_file.status.success());
    assert_eq!(
        body.as_bytes(),
        from_file.stdout.as_slice(),
        "`parse -` and `parse <file>` disagree on the same document"
    );
}

// IT-024 (FR-006-AC-2): no stdout/stderr interleaving on a SUCCESS run —
// `schema` for an archetype that exists produces a single JSON payload on
// stdout and no error on stderr. The failure-side half of the contract
// (FR-006-AC-1, empty stdout + non-empty stderr) is IT-031 below, which walks
// every failure class rather than one.
#[test]
fn it_024_stdout_and_stderr_do_not_interleave() {
    let out = quire()
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg(validate_module())
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    // Sanity: nothing on stdout looks like a diagnostic header.
    assert!(!stdout.contains("PathTraversal"));
    // The contract payload is a JSON object.
    assert!(stdout.trim_start().starts_with('{'));
    // No "QuireError" prefix in stderr on a clean run.
    assert!(
        !stderr.contains("QuireError"),
        "unexpected stderr: {stderr}"
    );
}

// IT-025 (FR-006-AC-3, NFR-005-AC-2): drive a deliberate error to get a
// diagnostic. With `--diagnostics-format=json` each line on stderr should
// parse as a JSON object with a `kind`.
#[test]
fn it_025_diagnostics_format_json_produces_json_lines() {
    let out = quire()
        .arg("--diagnostics-format")
        .arg("json")
        .arg("schema")
        .arg("FR")
        .arg("--module")
        .arg("foo/../bar")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    // At least one JSON-line diagnostic with a "kind" field.
    let mut found = false;
    for line in stderr.lines() {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
            if v.get("kind").is_some() {
                found = true;
                break;
            }
        }
    }
    assert!(found, "no JSON diagnostic found in stderr: {stderr}");
}

// IT-031 (NFR-005-AC-1, NFR-005-AC-2, FR-006-AC-1): EVERY known failure class
// produces empty stdout with a non-empty stderr, and renders its
// stderr as `Diagnostic` JSON under `--diagnostics-format json`. IT-025 asserts
// that *at least one* line parses for *one* class, which an implementation
// mixing a bare `anyhow` message into the stream still satisfies; this walks
// each class and requires every non-empty line to carry `kind` + `severity`.
#[test]
fn it_031_every_error_class_renders_as_diagnostic_json() {
    let doc = std::env::temp_dir().join(format!("quire-cli-it-031-{}.md", std::process::id()));
    std::fs::write(&doc, SIMPLE_DOC).unwrap();
    let empty_module = tempfile::tempdir().expect("tmpdir");

    let module = validate_module();
    let module = module.to_str().unwrap();
    let valid = common::validate_doc("valid-fr.md");
    let valid = valid.to_str().unwrap();
    let placeholder = common::validate_doc("placeholder-fr.md");
    let placeholder = placeholder.to_str().unwrap();
    let doc = doc.to_str().unwrap().to_owned();

    let classes: Vec<(&str, Vec<&str>)> = vec![
        (
            "path-safety",
            vec!["validate", valid, "--module", "foo/../bar"],
        ),
        (
            "unknown-archetype",
            vec!["validate", valid, "--module", module, "--archetype", "NOPE"],
        ),
        (
            "missing-manifest",
            vec![
                "validate",
                valid,
                "--module",
                empty_module.path().to_str().unwrap(),
            ],
        ),
        (
            "structural-validation",
            vec!["validate", placeholder, "--module", module],
        ),
        (
            "section-not-found",
            vec!["lookup", &doc, "--block-id", "definitely-not-a-block"],
        ),
    ];

    for (class, args) in classes {
        let out = quire()
            .arg("--diagnostics-format")
            .arg("json")
            .args(&args)
            .output()
            .unwrap();
        assert!(
            !out.status.success(),
            "{class} was expected to fail; it exited {:?}",
            out.status.code()
        );
        assert!(out.stdout.is_empty(), "{class} wrote to stdout");
        let stderr = String::from_utf8(out.stderr).unwrap();
        let mut lines = 0;
        for line in stderr.lines().filter(|l| !l.trim().is_empty()) {
            lines += 1;
            let v: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|e| panic!("{class}: stderr line is not JSON ({e}): {line}"));
            assert!(
                v.get("kind").and_then(|k| k.as_str()).is_some(),
                "{class}: diagnostic carries no `kind`: {line}"
            );
            assert!(
                v.get("severity").and_then(|s| s.as_str()).is_some(),
                "{class}: diagnostic carries no `severity`: {line}"
            );
        }
        assert!(lines > 0, "{class} produced no diagnostic at all");
    }
}

// IT-028 (FR-008-AC-1): default JSON output is compact (one line).
#[test]
fn it_028_parse_json_output_is_compact_by_default() {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-it-028-{}.md", std::process::id()));
    std::fs::write(&p, SIMPLE_DOC).unwrap();
    let out = quire().arg("parse").arg(&p).output().unwrap();
    assert!(out.status.success());
    let body = String::from_utf8(out.stdout).unwrap();
    // Compact: at most one trailing newline.
    let trimmed = body.trim_end_matches('\n');
    assert!(
        !trimmed.contains('\n'),
        "expected compact JSON, got:\n{body}"
    );
}

// IT-029 (FR-008-AC-3): `--pretty` produces multi-line indented JSON.
#[test]
fn it_029_pretty_flag_indents_json() {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-it-029-{}.md", std::process::id()));
    std::fs::write(&p, SIMPLE_DOC).unwrap();
    let out = quire()
        .arg("--pretty")
        .arg("parse")
        .arg(&p)
        .output()
        .unwrap();
    assert!(out.status.success());
    let body = String::from_utf8(out.stdout).unwrap();
    let internal_newlines = body.trim_end_matches('\n').matches('\n').count();
    assert!(
        internal_newlines > 0,
        "expected multi-line pretty output, got:\n{body}"
    );
}

// IT-030 (FR-008-AC-4): JSON field order matches Rust struct order, so two
// runs are byte-identical.
#[test]
fn it_030_parse_json_field_order_is_stable_across_runs() {
    let dir = std::env::temp_dir();
    let p = dir.join(format!("quire-cli-it-030-{}.md", std::process::id()));
    std::fs::write(&p, SIMPLE_DOC).unwrap();
    let one = quire().arg("parse").arg(&p).output().unwrap();
    let two = quire().arg("parse").arg(&p).output().unwrap();
    assert!(one.status.success() && two.status.success());
    // Byte-identical output -> stable field order.
    assert_eq!(one.stdout, two.stdout);
}
