//! FR-046 — health-data-flow reports project only selected FR-084 facts.
mod common;

use std::fs;
use std::path::Path;
use std::process::Output;

use ix_trace_rs::trace;
use serde_json::{json, Value};
use tempfile::TempDir;

const MANIFEST: &str = "\
name: health-flow-fixture
manifest_version: 1.0.0
version: 0.1.0
artifact_types:
- name: Connection
  data_schema:
    schema: schemas/Connection.json
- name: Classification
  data_schema:
    schema: schemas/Classification.json
- name: TrustBoundary
  data_schema:
    schema: schemas/TrustBoundary.json
semantic:
  contract_version: 1.0.0
  package: agent-ix/health-flow-fixture
  exports: [Connection, Classification, TrustBoundary]
  targets: [json-schema, markdown]
traceability:
  required_relations:
  - name: connection-carries-classification
    from: Connection
    edges: [classified-as]
    to: [Classification]
    direction: outgoing
    check: connection-unclassified
    record_keys: [sourceEnd, targetEnd, regulatedCategories]
  - name: connection-crosses-trust-boundary
    from: Connection
    edges: [crosses]
    to: [TrustBoundary]
    direction: outgoing
    check: connection-crossing-unknown
    record_keys: [trust_level]
";

const CONNECTION_SCHEMA: &str = r#"{
  "$schema":"https://json-schema.org/draft/2020-12/schema",
  "type":"object",
  "properties":{
    "sourceEnd":{"type":"object","properties":{"type":{"type":"string"}}},
    "targetEnd":{"type":"object","properties":{"type":{"type":"string"}}}
  },
  "additionalProperties":false
}"#;

const CLASSIFICATION_SCHEMA: &str = r#"{
  "$schema":"https://json-schema.org/draft/2020-12/schema",
  "type":"object",
  "properties":{"regulatedCategories":{"type":"array","items":{"type":"string"}}},
  "additionalProperties":false
}"#;

const BOUNDARY_SCHEMA: &str = r#"{
  "$schema":"https://json-schema.org/draft/2020-12/schema",
  "type":"object",
  "properties":{"trust_level":{"type":"string"}},
  "additionalProperties":false
}"#;

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("fixture path has parent")).unwrap();
    fs::write(path, contents).unwrap();
}

fn document(id: &str, archetype: &str, record: &str, relationships: &[(&str, &str)]) -> String {
    let rels = relationships
        .iter()
        .map(|(verb, target)| format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"))
        .collect::<String>();
    let rels = if rels.is_empty() {
        String::new()
    } else {
        format!("relationships:\n{rels}")
    };
    format!("---\nid: {id}\ntype: {archetype}\ntitle: {id}\n{record}{rels}---\n\n# {id}\n")
}

fn world(
    connection_record: &str,
    category: &str,
    boundary_record: Option<&str>,
    crossing: bool,
) -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    let module = temp.path().join("module");
    write(&module, "manifest.yaml", MANIFEST);
    write(&module, "schemas/Connection.json", CONNECTION_SCHEMA);
    write(
        &module,
        "schemas/Classification.json",
        CLASSIFICATION_SCHEMA,
    );
    write(&module, "schemas/TrustBoundary.json", BOUNDARY_SCHEMA);

    let scope = temp.path().join("repo");
    write(
        &scope,
        "spec/CONN-1.md",
        &document(
            "CONN-1",
            "Connection",
            connection_record,
            &if crossing {
                vec![("classified-as", "CLS-1"), ("crosses", "BOUNDARY-1")]
            } else {
                vec![("classified-as", "CLS-1")]
            },
        ),
    );
    write(
        &scope,
        "spec/CLS-1.md",
        &document(
            "CLS-1",
            "Classification",
            &format!("regulatedCategories: [{category}]\n"),
            &[],
        ),
    );
    if let Some(record) = boundary_record {
        write(
            &scope,
            "spec/BOUNDARY-1.md",
            &document("BOUNDARY-1", "TrustBoundary", record, &[]),
        );
    }
    (temp, scope)
}

fn run(scope: &Path, module: &Path, extra: &[&str]) -> Output {
    let mut command = common::quire();
    command
        .args(["relations", "--scope"])
        .arg(scope)
        .arg("--module")
        .arg(module)
        .args(extra);
    command.output().unwrap()
}

fn run_repos(repos: &[(&str, &Path)], module: &Path, extra: &[&str]) -> Output {
    let mut command = common::quire();
    command.arg("relations");
    for (identity, root) in repos {
        command
            .arg("--repo")
            .arg(format!("{identity}={}", root.display()));
    }
    command.arg("--module").arg(module).args(extra);
    command.output().unwrap()
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout was not JSON ({error}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

#[test]
#[trace("FR-046-AC-1", "FR-046-AC-2", "FR-046-AC-5", "FR-046-AC-6")]
fn emits_the_canonical_health_flow_envelope_from_generic_relation_facts() {
    let (temp, scope) = world(
        "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        "health",
        Some("trust_level: untrusted\n"),
        true,
    );
    let output = run(
        &scope,
        &temp.path().join("module"),
        &["--health-data-flows", "--json"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = json(&output);
    assert_eq!(report["schema"], "quire.health-data-flows/v1");
    assert_eq!(
        report["output"],
        json!({
            "name":"health-data-flows",
            "kind":"record",
            "description":"Health-data flow records; PLAT-1756 in the Data-Flow Model project owns the producer definition."
        })
    );
    assert_eq!(report["flows"].as_array().unwrap().len(), 1);
    assert!(report["gaps"].as_array().unwrap().is_empty());
    assert_eq!(report["flows"][0]["connection"]["id"], "CONN-1");
    assert_eq!(report["flows"][0]["source"]["id"], "PORT-SOURCE");
    assert_eq!(report["flows"][0]["destination"]["id"], "PORT-TARGET");
    assert_eq!(report["flows"][0]["source"]["repo"], Value::Null);
    assert_eq!(
        report["flows"][0]["source"]["declared_by"]["path"],
        "spec/CONN-1.md"
    );
    assert_eq!(report["flows"][0]["classifications"][0]["id"], "CLS-1");
    assert_eq!(
        report["flows"][0]["crossings"][0]["boundary"]["id"],
        "BOUNDARY-1"
    );
    assert_eq!(
        report["flows"][0]["crossings"][0]["trust_level"],
        "untrusted"
    );
    let encoded = String::from_utf8_lossy(&output.stdout);
    let top_level = ["\"schema\"", "\"output\"", "\"flows\"", "\"gaps\""];
    let top_positions: Vec<usize> = top_level
        .iter()
        .map(|key| encoded.find(key).expect("field is serialized"))
        .collect();
    assert!(top_positions.windows(2).all(|pair| pair[0] < pair[1]));
    let flow_fields = [
        "\"connection\"",
        "\"source\"",
        "\"destination\"",
        "\"classifications\"",
        "\"crossings\"",
    ];
    let flow_positions: Vec<usize> = flow_fields
        .iter()
        .map(|key| encoded.find(key).expect("flow field is serialized"))
        .collect();
    assert!(flow_positions.windows(2).all(|pair| pair[0] < pair[1]));
    let second = run(
        &scope,
        &temp.path().join("module"),
        &["--health-data-flows", "--json", "--strict"],
    );
    assert!(second.status.success());
    assert_eq!(output.stdout, second.stdout);
}

#[test]
#[trace("FR-046-AC-5")]
fn multi_repo_references_keep_their_declared_member_identity() {
    let temp = TempDir::new().unwrap();
    let module = temp.path().join("module");
    write(&module, "manifest.yaml", MANIFEST);
    write(&module, "schemas/Connection.json", CONNECTION_SCHEMA);
    write(
        &module,
        "schemas/Classification.json",
        CLASSIFICATION_SCHEMA,
    );
    write(&module, "schemas/TrustBoundary.json", BOUNDARY_SCHEMA);
    let source = temp.path().join("source");
    let target = temp.path().join("target");
    write(
        &source,
        "spec/CONN-1.md",
        &document(
            "CONN-1",
            "Connection",
            "sourceEnd:\n  type: ix://org/target/PORT-SOURCE\ntargetEnd:\n  type: ix://org/target/PORT-TARGET\n",
            &[
                ("classified-as", "ix://org/target/spec/CLS-1"),
                ("crosses", "ix://org/target/spec/BOUNDARY-1"),
            ],
        ),
    );
    write(
        &target,
        "spec/CLS-1.md",
        &document(
            "CLS-1",
            "Classification",
            "regulatedCategories: [health]\n",
            &[],
        ),
    );
    write(
        &target,
        "spec/BOUNDARY-1.md",
        &document(
            "BOUNDARY-1",
            "TrustBoundary",
            "trust_level: untrusted\n",
            &[],
        ),
    );
    let output = run_repos(
        &[("org/source", &source), ("org/target", &target)],
        &module,
        &["--health-data-flows", "--json"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = json(&output);
    assert_eq!(
        report["flows"][0]["connection"]["repo"], "org/source",
        "{report}"
    );
    assert_eq!(report["flows"][0]["source"]["repo"], "org/target");
    assert_eq!(report["flows"][0]["destination"]["repo"], "org/target");
    assert_eq!(
        report["flows"][0]["classifications"][0]["repo"],
        "org/target"
    );
    assert_eq!(
        report["flows"][0]["crossings"][0]["boundary"]["repo"],
        "org/target"
    );
}

#[test]
#[trace("FR-046-AC-3")]
fn reports_missing_endpoint_dangling_crossing_and_missing_trust_level() {
    let (temp, scope) = world(
        "targetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        "health",
        None,
        true,
    );
    let output = run(
        &scope,
        &temp.path().join("module"),
        &["--health-data-flows", "--json"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = json(&output);
    assert!(report["flows"].as_array().unwrap().is_empty());
    let missing = report["gaps"][0]["missing"].as_array().unwrap();
    assert!(missing
        .iter()
        .any(|item| item["field"] == "source" && item["reference"].is_null()));
    assert!(missing
        .iter()
        .any(|item| item["field"] == "crossing" && item["reference"]["id"] == "BOUNDARY-1"));

    let (temp, scope) = world(
        "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        "health",
        Some(""),
        true,
    );
    let output = run(
        &scope,
        &temp.path().join("module"),
        &["--health-data-flows", "--json"],
    );
    let report = json(&output);
    assert!(report["gaps"][0]["missing"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["field"] == "trust_level"));
}

#[test]
#[trace("FR-046-AC-4", "FR-046-AC-7")]
fn ignores_non_health_classifications_and_accepts_no_crossing() {
    let (temp, scope) = world(
        "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        "financial",
        None,
        false,
    );
    let output = run(
        &scope,
        &temp.path().join("module"),
        &["--health-data-flows", "--json"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = json(&output);
    assert!(report["flows"].as_array().unwrap().is_empty());
    assert!(report["gaps"].as_array().unwrap().is_empty());

    let (temp, scope) = world(
        "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        "health",
        None,
        false,
    );
    let output = run(
        &scope,
        &temp.path().join("module"),
        &["--health-data-flows", "--json"],
    );
    assert!(output.status.success());
    let report = json(&output);
    assert_eq!(report["flows"].as_array().unwrap().len(), 1);
    assert!(report["flows"][0]["crossings"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(report["gaps"].as_array().unwrap().is_empty());
}

#[test]
#[trace("FR-046-AC-8")]
fn strict_fails_only_after_emitting_a_complete_gapped_report() {
    let (temp, scope) = world("", "health", None, false);
    let output = run(
        &scope,
        &temp.path().join("module"),
        &["--health-data-flows", "--strict", "--json"],
    );
    assert_eq!(output.status.code(), Some(1));
    let report = json(&output);
    assert_eq!(report["gaps"].as_array().unwrap().len(), 1);
}

#[test]
#[trace("FR-046-AC-9")]
fn health_mode_conflicts_exit_three_without_a_report() {
    let (temp, scope) = world("", "health", None, false);
    for (mode, flag) in [
        ("--relation", "some-relation"),
        ("--design", ""),
        ("--flow", ""),
    ] {
        let mut args = vec!["--health-data-flows", "--json", mode];
        if !flag.is_empty() {
            args.push(flag);
        }
        let output = run(&scope, &temp.path().join("module"), &args);
        assert_eq!(
            output.status.code(),
            Some(3),
            "mode {mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty(), "mode {mode} wrote a report");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(mode) && stderr.contains("--health-data-flows"),
            "{stderr}"
        );
    }
}
