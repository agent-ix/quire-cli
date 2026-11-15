//! FR-030-AC-26 — `quire relations --json` passes the engine's selected FR-084
//! `record` data through unchanged, on connection and linked entries and on the
//! reverse relation's rows. Kinds, verbs and keys are authored by the fixture
//! module below; the CLI knows none of them.
mod common;

use std::fs;
use std::path::Path;
use std::process::Output;

use ix_trace_rs::trace;
use serde_json::{json, Value};
use tempfile::TempDir;

const MANIFEST: &str = "\
name: record-fixture
version: 0.1.0
artifact_types:
- name: Connection
- name: Classification
  data_schema:
    schema: schemas/Classification.json
semantic:
  contract_version: 1.0.0
  package: agent-ix/record-fixture
  exports: [Classification]
  targets: [json-schema, markdown]
traceability:
  required_relations:
  - name: conn-classified
    from: Connection
    edges: [classified-as]
    to: [Classification]
    direction: outgoing
    check: conn-unclassified
    record_keys: [cats]
  - name: cls-used
    from: Classification
    edges: [classified-as]
    to: [Connection]
    direction: incoming
    check: cls-unused
    record_keys: [cats]
";

const SCHEMA: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "cats": { "type": "array", "items": { "enum": ["a", "b", "c"] } },
    "lvl": { "enum": ["hi", "lo"] }
  },
  "additionalProperties": false
}"#;

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn connection(id: &str, targets: &[&str]) -> String {
    let mut rels = String::new();
    if !targets.is_empty() {
        rels.push_str("relationships:\n");
        for target in targets {
            rels.push_str(&format!(
                "  - target: \"{target}\"\n    type: \"classified-as\"\n"
            ));
        }
    }
    format!("---\nid: {id}\ntype: Connection\ntitle: {id}\n{rels}---\n\n# {id}\n")
}

fn classification(id: &str, record: &str) -> String {
    format!("---\nid: {id}\ntype: Classification\ntitle: {id}\n{record}---\n\n# {id}\n")
}

/// CLS-001 states `cats` and `lvl`; CLS-002 states only `lvl`, so it has no
/// selected record. CONN-001 points at both, CONN-002 at CLS-001.
fn world() -> TempDir {
    let dir = TempDir::new().unwrap();
    let module = dir.path().join("m");
    write(&module, "manifest.yaml", MANIFEST);
    write(&module, "schemas/Classification.json", SCHEMA);
    let repo = dir.path().join("repo");
    write(
        &repo,
        "spec/CONN-001.md",
        &connection("CONN-001", &["CLS-001", "CLS-002"]),
    );
    write(
        &repo,
        "spec/CONN-002.md",
        &connection("CONN-002", &["CLS-001"]),
    );
    write(
        &repo,
        "spec/CLS-001.md",
        &classification("CLS-001", "cats: [a, b]\nlvl: hi\n"),
    );
    write(
        &repo,
        "spec/CLS-002.md",
        &classification("CLS-002", "lvl: lo\n"),
    );
    dir
}

fn relations(dir: &TempDir) -> Output {
    common::quire()
        .arg("relations")
        .arg("--scope")
        .arg(dir.path().join("repo"))
        .arg("--module")
        .arg(dir.path().join("m"))
        .arg("--json")
        .output()
        .unwrap()
}

fn report(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn rows<'v>(report: &'v Value, relation: &str) -> &'v Vec<Value> {
    report["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == relation)
        .unwrap_or_else(|| panic!("no relation {relation}"))["rows"]
        .as_array()
        .unwrap()
}

fn row<'v>(report: &'v Value, relation: &str, id: &str) -> &'v Value {
    rows(report, relation)
        .iter()
        .find(|r| r["document"]["id"] == id)
        .unwrap_or_else(|| panic!("no row {id} in {relation}"))
}

fn linked<'v>(row: &'v Value, id: &str) -> &'v Value {
    row["linked"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == id)
        .unwrap_or_else(|| panic!("no linked {id}"))
}

#[trace("FR-030-AC-26")]
#[test]
fn a_connection_reports_its_classifications_selected_record() {
    let dir = world();
    let report = report(&relations(&dir));
    let conn = row(&report, "conn-classified", "CONN-001");
    // The selected key is carried; the unselected `lvl` is not.
    assert_eq!(
        linked(conn, "CLS-001")["record"],
        json!({"cats": ["a", "b"]})
    );
    // A classification stating no selected key has no `record` key at all.
    assert!(linked(conn, "CLS-002")
        .as_object()
        .unwrap()
        .get("record")
        .is_none());
}

#[trace("FR-030-AC-26")]
#[test]
fn the_reverse_relation_lists_each_classifications_connections() {
    let dir = world();
    let report = report(&relations(&dir));
    let cls = row(&report, "cls-used", "CLS-001");
    assert_eq!(cls["document"]["record"], json!({"cats": ["a", "b"]}));
    let ids: Vec<&str> = cls["linked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["CONN-001", "CONN-002"]);
    // Connections declare no `cats`, so their entries carry no record.
    assert!(linked(cls, "CONN-001")
        .as_object()
        .unwrap()
        .get("record")
        .is_none());
}

#[trace("FR-030-AC-26")]
#[test]
fn level_and_unselected_keys_are_omitted_and_runs_are_byte_identical() {
    let dir = world();
    let first = relations(&dir);
    let second = relations(&dir);
    assert_eq!(first.stdout, second.stdout);
    let text = String::from_utf8(first.stdout).unwrap();
    assert!(!text.contains("\"lvl\""), "unselected key leaked: {text}");
    assert!(!text.contains("\"hi\""), "unselected value leaked: {text}");
}
