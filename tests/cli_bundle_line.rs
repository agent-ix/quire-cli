//! `validate --okf` renders `BundleFinding.line` (PLAT-1489, FR-014-AC-12..15).
//!
//! The module and bundle are built in tempdirs; the module names nothing the
//! engine knows (archetypes Left/Right/Link, kind column `Class`).

mod common;

use ix_trace_rs::trace;
use std::fs;
use std::path::Path;

use serde_json::Value;
use tempfile::TempDir;

use common::{quire, validate_doc, validate_module};

const MANIFEST: &str = "name: bundle-line
version: 0.1.0
description: Located bundle finding fixture
artifact_types:
- name: Left
  defaults:
    id_pattern: LD-{next:03d}
- name: Right
  defaults:
    id_pattern: RD-{next:03d}
- name: Link
  defaults:
    id_pattern: LK-{next:03d}
traceability:
  trace_targets:
  - name: lefty
    archetype: Left
    section: Items
    id_column: Item
  - name: righty
    archetype: Right
    section: Items
    id_column: Item
  document_references:
  - name: links
    archetype: Link
    section: Links
    column: Target
    row_id_column: Link
    pattern: '\\b([LR]-\\d+)\\b'
    targets: [lefty, righty]
    kind_column: Class
    kind_targets:
      north: [lefty]
      south: [righty]
";

const LEFT: &str =
    "---\nid: LD-001\ntype: Left\ntitle: Left\n---\n\n## Items\n\n| Item |\n|------|\n| L-1 |\n";
const RIGHT: &str =
    "---\nid: RD-001\ntype: Right\ntitle: Right\n---\n\n## Items\n\n| Item |\n|------|\n| R-1 |\n";
const LINKS_HEAD: &str = "---\nid: LK-001\ntype: Link\ntitle: Links\n---\n\n## Links\n\n| Link | Class | Target |\n|------|-------|--------|\n";

/// 1-based line of the first line of `doc` containing `needle`.
fn line_of(doc: &str, needle: &str) -> usize {
    doc.lines()
        .position(|l| l.contains(needle))
        .expect("needle in doc")
        + 1
}

struct Fixture {
    module: TempDir,
    bundle: TempDir,
}

fn fixture(docs: &[(&str, &str)]) -> Fixture {
    let module = TempDir::new().unwrap();
    fs::write(module.path().join("manifest.yaml"), MANIFEST).unwrap();
    let bundle = TempDir::new().unwrap();
    for (name, body) in docs {
        fs::write(bundle.path().join(name), body).unwrap();
    }
    Fixture { module, bundle }
}

/// Run `validate --okf [--strict]` and return the stderr lines.
fn stderr_lines(bundle: &Path, module: &Path, strict: bool, json: bool) -> Vec<String> {
    let mut cmd = quire();
    cmd.arg("validate").arg("--okf");
    if strict {
        cmd.arg("--strict");
    }
    if json {
        cmd.args(["--diagnostics-format", "json"]);
    }
    let out = cmd
        .arg(bundle)
        .arg("--module")
        .arg(module)
        .output()
        .unwrap();
    String::from_utf8(out.stderr)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

fn records(lines: &[String]) -> Vec<Value> {
    lines
        .iter()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

/// Records whose message ends with `[<reason>]`.
fn with_reason<'a>(records: &'a [Value], reason: &str) -> Vec<&'a Value> {
    let suffix = format!("[{reason}]");
    records
        .iter()
        .filter(|r| r["message"].as_str().is_some_and(|m| m.ends_with(&suffix)))
        .collect()
}

/// Human-form lines ending with `[<reason>]`.
fn human_with_reason<'a>(lines: &'a [String], reason: &str) -> Vec<&'a String> {
    let suffix = format!("[{reason}]");
    lines.iter().filter(|l| l.ends_with(&suffix)).collect()
}

// IT-262, FR-014-AC-12, FR-014-AC-13: a dangling trace reference prints and
// serializes the line of its referencing row; every id of a multi-id cell
// carries that row's line; the same id on two rows is two findings.
#[trace("IT-262", "FR-014-AC-12", "FR-014-AC-13")]
#[test]
fn it262_dangling_trace_reference_shows_its_row_line() {
    let links = format!(
        "{LINKS_HEAD}| K-0 | north | L-9 |\n| K-1 | north | L-1 |\n| K-2 | north | L-8 and L-9 |\n"
    );
    let f = fixture(&[("left.md", LEFT), ("right.md", RIGHT), ("links.md", &links)]);
    let path = f.bundle.path().join("links.md");
    let p = path.display();
    let (k0, k2) = (line_of(&links, "| K-0 |"), line_of(&links, "| K-2 |"));

    let human = stderr_lines(f.bundle.path(), f.module.path(), false, false);
    let found = human_with_reason(&human, "dangling-trace-reference");
    assert_eq!(found.len(), 3, "{human:?}");
    for (id, row, line) in [("L-9", "K-0", k0), ("L-8", "K-2", k2), ("L-9", "K-2", k2)] {
        let want = format!(
            "warning: {p}: line {line}: trace reference '{id}' (row '{row}') declared by 'links' resolves to no lefty/righty target [dangling-trace-reference]"
        );
        assert!(found.contains(&&want), "{want} not in {found:?}");
    }

    let json = records(&stderr_lines(f.bundle.path(), f.module.path(), false, true));
    let found = with_reason(&json, "dangling-trace-reference");
    assert_eq!(found.len(), 3, "{json:?}");
    let mut lines: Vec<u64> = found.iter().map(|r| r["line"].as_u64().unwrap()).collect();
    lines.sort_unstable();
    assert_eq!(lines, [k0 as u64, k2 as u64, k2 as u64]);
    assert!(
        found.iter().all(|r| r["severity"] == "warning"),
        "{found:?}"
    );
}

// IT-263, FR-014-AC-12, FR-014-AC-13: a kind mismatch carries its own row's
// line, not the table header's.
#[trace("IT-263", "FR-014-AC-12", "FR-014-AC-13")]
#[test]
fn it263_reference_kind_mismatch_shows_its_row_line() {
    let links = format!("{LINKS_HEAD}| K-1 | north | L-1 |\n| K-2 | north | R-1 |\n");
    let f = fixture(&[("left.md", LEFT), ("right.md", RIGHT), ("links.md", &links)]);
    let path = f.bundle.path().join("links.md");
    let line = line_of(&links, "| K-2 |");

    let human = stderr_lines(f.bundle.path(), f.module.path(), false, false);
    let found = human_with_reason(&human, "reference-kind-mismatch");
    assert_eq!(found.len(), 1, "{human:?}");
    let want = format!(
        "warning: {}: line {line}: trace reference 'R-1' (row 'K-2') declared by 'links' is on a row of kind 'north' which may reference only lefty targets [reference-kind-mismatch]",
        path.display()
    );
    assert_eq!(found[0], &want);

    let json = records(&stderr_lines(f.bundle.path(), f.module.path(), false, true));
    let found = with_reason(&json, "reference-kind-mismatch");
    assert_eq!(found.len(), 1, "{json:?}");
    assert_eq!(found[0]["line"].as_u64(), Some(line as u64));
    assert_eq!(found[0]["path"], path.display().to_string());
    assert_eq!(found[0]["reason"], "reference-kind-mismatch");
}

fn validate_mod_bundle(docs: &[&str]) -> TempDir {
    let dir = TempDir::new().unwrap();
    for name in docs {
        fs::copy(validate_doc(name), dir.path().join(name)).unwrap();
    }
    dir
}

// IT-264, FR-014-AC-12..14: a bridged document error under `--okf --strict`
// shows the line single-document `validate` prints for the same document.
#[trace("IT-264", "FR-014-AC-12", "FR-014-AC-13", "FR-014-AC-14")]
#[test]
fn it264_bridged_document_error_shows_its_line() {
    let bundle = validate_mod_bundle(&["broken-fr.md"]);
    let doc = bundle.path().join("broken-fr.md");

    let single = quire()
        .arg("validate")
        .arg(&doc)
        .arg("--module")
        .arg(validate_module())
        .output()
        .unwrap();
    let single = String::from_utf8(single.stderr).unwrap();
    let expected = single
        .lines()
        .find(|l| l.contains("[assert]"))
        .expect("single-document diagnostic")
        .to_string();
    assert!(expected.contains(": line "), "{expected}");

    let human = stderr_lines(bundle.path(), &validate_module(), true, false);
    assert!(human.contains(&expected), "{expected} not in {human:?}");

    let json = records(&stderr_lines(bundle.path(), &validate_module(), true, true));
    let found = with_reason(&json, "assert");
    assert_eq!(found.len(), 1, "{json:?}");
    let line = expected
        .split(": line ")
        .nth(1)
        .and_then(|rest| rest.split(':').next())
        .and_then(|n| n.parse::<u64>().ok())
        .expect("line number");
    assert_eq!(found[0]["line"].as_u64(), Some(line));
    assert_eq!(found[0]["path"], doc.display().to_string());
    assert_eq!(found[0]["reason"], "assert");
}

// IT-265, FR-014-AC-15: a finding with no line renders exactly as before, in
// human and JSON form (no `line` key).
#[trace("IT-265", "FR-014-AC-15")]
#[test]
fn it265_unlocated_finding_renders_exactly_as_before() {
    let bundle = validate_mod_bundle(&["missing-section-fr.md"]);
    let doc = bundle.path().join("missing-section-fr.md");
    let message = "[FR] required 'specification' (section_body(Specification)) is missing";

    let human = stderr_lines(bundle.path(), &validate_module(), true, false);
    let want = format!("{}: {message} [missing]", doc.display());
    assert!(human.contains(&want), "{want} not in {human:?}");

    let json = records(&stderr_lines(bundle.path(), &validate_module(), true, true));
    let found = with_reason(&json, "missing");
    assert_eq!(found.len(), 1, "{json:?}");
    assert_eq!(
        *found[0],
        serde_json::json!({
            "kind": "ValidationError",
            "message": want,
            "severity": "error",
        })
    );
}

// IT-266, FR-014-AC-16: warnings are emitted before errors, in human and JSON
// output, for a bundle that mixes both.
#[trace("IT-266", "FR-014-AC-16")]
#[test]
fn it266_warnings_precede_errors() {
    let bad_left = LEFT.replace("title: Left\n", "title: Left\ntags: notalist\n");
    // No Right document: the `righty` declaration scans nothing, which is a
    // (line-less) warning beside the frontmatter errors.
    let f = fixture(&[("left.md", &bad_left)]);

    let json = records(&stderr_lines(f.bundle.path(), f.module.path(), true, true));
    let severities: Vec<&str> = json
        .iter()
        .filter(|r| r["kind"] == "ValidationWarning" || r["kind"] == "ValidationError")
        .map(|r| r["severity"].as_str().unwrap())
        .collect();
    assert!(
        severities.contains(&"warning") && severities.contains(&"error"),
        "{json:?}"
    );
    let first_error = severities.iter().position(|s| *s == "error").unwrap();
    assert!(
        severities[first_error..].iter().all(|s| *s == "error"),
        "{severities:?}"
    );

    let human = stderr_lines(f.bundle.path(), f.module.path(), true, false);
    let tags: Vec<bool> = human
        .iter()
        .filter(|l| l.ends_with(']'))
        .map(|l| l.starts_with("warning: "))
        .collect();
    assert!(tags.contains(&true) && tags.contains(&false), "{human:?}");
    let first_error = tags.iter().position(|w| !w).unwrap();
    assert!(tags[first_error..].iter().all(|w| !w), "{human:?}");
}
