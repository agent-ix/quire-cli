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
  data_schema:
    schema: schemas/Record.json
- name: Classification
  data_schema:
    schema: schemas/Record.json
semantic:
  contract_version: 1.0.0
  package: agent-ix/record-fixture
  exports: [Connection, Classification]
  targets: [json-schema, markdown]
traceability:
  required_relations:
  - name: conn-classified
    from: Connection
    edges: [classified-as]
    to: [Classification]
    direction: outgoing
    check: conn-unclassified
    record_keys: [regulatedCategories]
  - name: cls-used
    from: Classification
    edges: [classified-as]
    to: [Connection]
    direction: incoming
    check: cls-unused
    record_keys: [regulatedCategories]
";

const SCHEMA: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "regulatedCategories": { "type": "array", "items": { "enum": ["alpha", "beta"] } },
    "level": { "enum": ["high", "low"] }
  },
  "additionalProperties": false
}"#;

fn write(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn doc(id: &str, ty: &str, record: &str, targets: &[&str]) -> String {
    let mut rels = String::new();
    if !targets.is_empty() {
        rels.push_str("relationships:\n");
        for target in targets {
            rels.push_str(&format!(
                "  - target: \"{target}\"\n    type: \"classified-as\"\n"
            ));
        }
    }
    format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n{record}{rels}---\n\n# {id}\n")
}

/// Every document states `level`, which no relation selects. CONN-001 and
/// CLS-001 also state `regulatedCategories`; CONN-002 and CLS-002 state no
/// selected key at all. CONN-001 points at both classifications, CONN-002 at
/// CLS-001.
fn world() -> TempDir {
    let dir = TempDir::new().unwrap();
    let module = dir.path().join("m");
    write(&module, "manifest.yaml", MANIFEST);
    write(&module, "schemas/Record.json", SCHEMA);
    let repo = dir.path().join("repo");
    let docs = [
        (
            "CONN-001",
            doc(
                "CONN-001",
                "Connection",
                "regulatedCategories: [alpha, beta]\nlevel: high\n",
                &["CLS-001", "CLS-002"],
            ),
        ),
        (
            "CONN-002",
            doc("CONN-002", "Connection", "level: low\n", &["CLS-001"]),
        ),
        (
            "CLS-001",
            doc(
                "CLS-001",
                "Classification",
                "regulatedCategories: [alpha]\nlevel: high\n",
                &[],
            ),
        ),
        (
            "CLS-002",
            doc("CLS-002", "Classification", "level: low\n", &[]),
        ),
    ];
    for (id, text) in docs {
        write(&repo, &format!("spec/{id}.md"), &text);
    }
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

fn row<'v>(report: &'v Value, relation: &str, id: &str) -> &'v Value {
    report["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == relation)
        .unwrap_or_else(|| panic!("no relation {relation}"))["rows"]
        .as_array()
        .unwrap()
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

fn assert_no_record(entry: &Value) {
    assert!(
        entry.get("record").is_none(),
        "expected no record key: {entry}"
    );
}

#[trace("FR-030-AC-26")]
#[test]
fn a_connection_and_its_linked_classifications_carry_the_selected_record() {
    let dir = world();
    let report = report(&relations(&dir));
    let conn = row(&report, "conn-classified", "CONN-001");
    assert_eq!(
        conn["document"]["record"],
        json!({"regulatedCategories": ["alpha", "beta"]})
    );
    assert_eq!(
        linked(conn, "CLS-001")["record"],
        json!({"regulatedCategories": ["alpha"]})
    );
    assert_no_record(linked(conn, "CLS-002"));
    // No selected key: no `regulatedCategories`, and no `record` at all.
    assert_no_record(&row(&report, "conn-classified", "CONN-002")["document"]);
}

#[trace("FR-030-AC-26")]
#[test]
fn the_reverse_relation_lists_each_classifications_connections_with_their_record() {
    let dir = world();
    let report = report(&relations(&dir));
    let cls = row(&report, "cls-used", "CLS-001");
    assert_eq!(
        cls["document"]["record"],
        json!({"regulatedCategories": ["alpha"]})
    );
    let ids: Vec<&str> = cls["linked"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["CONN-001", "CONN-002"]);
    assert_eq!(
        linked(cls, "CONN-001")["record"],
        json!({"regulatedCategories": ["alpha", "beta"]})
    );
    assert_no_record(linked(cls, "CONN-002"));
    assert_no_record(&row(&report, "cls-used", "CLS-002")["document"]);
}

#[trace("FR-030-AC-26")]
#[test]
fn level_is_omitted_and_runs_are_byte_identical() {
    let dir = world();
    let first = relations(&dir);
    let second = relations(&dir);
    assert_eq!(first.stdout, second.stdout);
    let text = String::from_utf8(first.stdout).unwrap();
    assert!(!text.contains("\"level\""), "unselected key leaked: {text}");
    assert!(
        !text.contains("\"high\""),
        "unselected value leaked: {text}"
    );
}
