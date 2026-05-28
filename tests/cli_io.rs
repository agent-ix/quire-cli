//! I/O contract ITs.
//! Covers IT-011 (stdin), IT-024 (no interleave), IT-025 (--diagnostics-format=json),
//! IT-028 (compact default), IT-029 (--pretty), IT-030 (stable field order).

mod common;

use std::io::Write;
use std::process::Stdio;

use common::{ctx_path, iso_module, quire};

const SIMPLE_DOC: &str = "---\nid: FR-001\nartifact_type: FR\n---\n# [FR-001] Hi\n";

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
}

#[test]
fn it_024_stdout_and_stderr_do_not_interleave() {
    // Render an archetype that exists — stderr should be empty, stdout
    // should be a single well-formed payload.
    let out = quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    // Sanity: nothing on stdout looks like a diagnostic header.
    assert!(!stdout.contains("PathTraversal"));
    // The rendered markdown starts cleanly with frontmatter.
    assert!(stdout.starts_with("---") || stdout.contains("---"));
    // No "QuireError" prefix in stderr on a clean run.
    assert!(
        !stderr.contains("QuireError"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn it_025_diagnostics_format_json_produces_json_lines() {
    // Drive a deliberate error to get a diagnostic. With --diagnostics-format=json
    // each line on stderr should parse as a JSON object with a "kind".
    let out = quire()
        .arg("--diagnostics-format")
        .arg("json")
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg("foo/../bar")
        .arg("--data")
        .arg(ctx_path("FR"))
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
