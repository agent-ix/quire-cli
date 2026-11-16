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

const REQUIRED_CONNECTION_SCHEMA: &str = r#"{
  "$schema":"https://json-schema.org/draft/2020-12/schema",
  "type":"object",
  "properties":{
    "sourceEnd":{"type":"object","properties":{"type":{"type":"string"}}},
    "targetEnd":{"type":"object","properties":{"type":{"type":"string"}}},
    "flowDirection":{"type":"string"}
  },
  "required":["sourceEnd","targetEnd","flowDirection"],
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

fn add_flow_rule(module: &Path) {
    let manifest = MANIFEST.replacen(
        "traceability:\n  required_relations:",
        "traceability:\n  flow_rules:\n  - name: connection-present\n    subject: [Connection]\n    match: [{path: [], field: id, present: true}]\n    check: connection-present\n    rationale: Report the authored connection.\n  required_relations:",
        1,
    );
    assert!(
        manifest.contains("flow_rules:"),
        "fixture manifest has no flow rules"
    );
    write(module, "manifest.yaml", &manifest);
}

fn add_flow_only_rule(module: &Path) {
    let declarations = MANIFEST.split("traceability:").next().unwrap();
    let manifest = format!(
        "{declarations}traceability:\n  flow_rules:\n  - name: connection-present\n    subject: [Connection]\n    match: [{{path: [], field: id, present: true}}]\n    check: connection-present\n    rationale: Report the authored connection.\n"
    );
    write(module, "manifest.yaml", &manifest);
}

fn add_flow_rule_with_remediation(module: &Path) {
    write(
        module,
        "manifest.yaml",
        "\
name: health-flow-fixture
version: 0.1.0
description: Flow remediation CLI fixture
archetypes:
- name: Connection
  frontmatter_schema_ref: schemas/Connection.json
  allowed_links:
    classified-as: [Classification]
    crosses: [TrustBoundary]
- name: Classification
  frontmatter_schema_ref: schemas/Classification.json
- name: TrustBoundary
  frontmatter_schema_ref: schemas/TrustBoundary.json
edge_types:
  classified-as: {description: connection has a classification, category: traceability}
  crosses: {description: connection crosses a trust boundary, category: traceability}
traceability:
  flow_rules:
  - name: connection-present
    subject: [Connection]
    match: [{path: [], field: id, present: true}]
    unless_any: [{path: [], field: flowDirection, any_of: [bidirectional]}]
    check: connection-present
    remediations: [set-flow-direction]
    rationale: Report the authored connection.
  remediations:
  - name: set-flow-direction
    description: Set flowDirection to bidirectional.
    sets: {field: flowDirection, value: bidirectional}
",
    );
    write(
        module,
        "schemas/Connection.json",
        r#"{"type":"object","properties":{"id":{"type":"string"},"type":{"const":"Connection"},"sourceEnd":{"type":"object"},"targetEnd":{"type":"object"},"flowDirection":{"type":"string","enum":["bidirectional"]}}}"#,
    );
    write(
        module,
        "schemas/Classification.json",
        r#"{"type":"object","properties":{"id":{"type":"string"},"type":{"const":"Classification"},"regulatedCategories":{"type":"array","items":{"type":"string"}}}}"#,
    );
    write(
        module,
        "schemas/TrustBoundary.json",
        r#"{"type":"object","properties":{"id":{"type":"string"},"type":{"const":"TrustBoundary"},"trust_level":{"type":"string"}}}"#,
    );
}

fn assert_flow_load_failure_names_the_invalid_value(
    module_directory: &str,
    source: &str,
    invalid: &str,
    expected: &str,
) {
    let (temp, scope) = world("", "health", None, false);
    let module = temp.path().join("modules").join(module_directory);
    add_flow_rule_with_remediation(&module);

    let manifest_path = module.join("manifest.yaml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    let invalid_manifest = manifest.replace(source, invalid);
    assert_ne!(manifest, invalid_manifest, "fixture contains source text");
    fs::write(&manifest_path, invalid_manifest).unwrap();

    let output = run(&scope, &module, &["--flow", "--json"]);
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("connection-present"), "{stderr}");
    assert!(stderr.contains(expected), "{stderr}");
    assert!(
        stderr.contains(&manifest_path.to_string_lossy().to_string()),
        "expected originating manifest path {} in diagnostic: {stderr}",
        manifest_path.display()
    );
    assert!(!stderr.contains("no loaded module declares `traceability.flow_rules`"));
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout was not JSON ({error}): {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn assert_object_shape(value: &Value, encoded: &str, marker: &str, expected: &[&str]) {
    let object = value.as_object().expect("value is an object");
    let mut actual: Vec<&str> = object.keys().map(String::as_str).collect();
    actual.sort_unstable();
    let mut expected_sorted = expected.to_vec();
    expected_sorted.sort_unstable();
    assert_eq!(
        actual, expected_sorted,
        "unexpected object keys at {marker}"
    );

    let marker_start = encoded.find(marker).expect("object marker is serialized");
    let object_start = marker_start + marker.len() - 1;
    let bytes = encoded.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    let mut object_end = None;
    for (offset, byte) in bytes.iter().enumerate().skip(object_start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match *byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    object_end = Some(offset + 1);
                    break;
                }
            }
            _ => {}
        }
    }
    let object_text = &encoded[object_start..object_end.expect("object is closed")];
    let positions: Vec<usize> = expected
        .iter()
        .map(|key| {
            object_text
                .find(&format!("\"{key}\":"))
                .unwrap_or_else(|| panic!("missing key {key} from {object_text}"))
        })
        .collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "wrong key order in {object_text}"
    );
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
    assert_object_shape(
        &report,
        &encoded,
        "{",
        &["schema", "output", "flows", "gaps"],
    );
    assert_object_shape(
        &report["output"],
        &encoded,
        "\"output\":{",
        &["name", "kind", "description"],
    );
    assert_object_shape(
        &report["flows"][0],
        &encoded,
        "\"flows\":[{",
        &[
            "connection",
            "source",
            "destination",
            "classifications",
            "crossings",
        ],
    );
    assert_object_shape(
        &report["flows"][0]["connection"],
        &encoded,
        "\"connection\":{",
        &["repo", "id", "archetype", "path"],
    );
    assert_object_shape(
        &report["flows"][0]["source"],
        &encoded,
        "\"source\":{",
        &["repo", "id", "archetype", "declared_by"],
    );
    assert_object_shape(
        &report["flows"][0]["source"]["declared_by"],
        &encoded,
        "\"declared_by\":{",
        &["repo", "id", "archetype", "path"],
    );
    assert_object_shape(
        &report["flows"][0]["classifications"][0],
        &encoded,
        "\"classifications\":[{",
        &["repo", "id", "archetype", "path"],
    );
    assert_object_shape(
        &report["flows"][0]["crossings"][0],
        &encoded,
        "\"crossings\":[{",
        &["boundary", "trust_level"],
    );
    assert_object_shape(
        &report["flows"][0]["crossings"][0]["boundary"],
        &encoded,
        "\"boundary\":{",
        &["repo", "id", "archetype", "path"],
    );
    for document in [
        &report["flows"][0]["connection"],
        &report["flows"][0]["classifications"][0],
        &report["flows"][0]["crossings"][0]["boundary"],
    ] {
        assert!(document["repo"].is_null() || document["repo"].is_string());
        assert!(document["id"].is_string());
        assert!(document["archetype"].is_string());
        assert!(document["path"].is_string());
    }
    assert!(
        report["flows"][0]["source"]["repo"].is_null()
            || report["flows"][0]["source"]["repo"].is_string()
    );
    assert!(report["flows"][0]["source"]["id"].is_string());
    assert!(report["flows"][0]["source"]["archetype"].is_string());
    assert!(report["flows"][0]["source"]["declared_by"].is_object());
    assert!(report["flows"][0]["crossings"][0]["trust_level"].is_string());
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
#[trace("FR-044-AC-5", "FR-044-AC-7")]
fn flow_mode_passes_through_generic_engine_report_and_applies_strict_exit() {
    let (temp, scope) = world(
        "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        "health",
        Some("trust_level: untrusted\n"),
        true,
    );
    let module = temp.path().join("module");
    add_flow_rule(&module);
    let registry = quire_rs::Registry::load_module(&module).expect("fixture module loads");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    assert_eq!(
        registry.traceability().unwrap().flow_rules.len(),
        1,
        "fixture flow declaration is retained"
    );

    let output = run(&scope, &module, &["--flow", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = json(&output);
    assert_eq!(report["schema"], "quire.flow-report/v1");
    let repeated = run(&scope, &module, &["--flow", "--json"]);
    assert_eq!(repeated.stdout, output.stdout);
    assert_eq!(report["denials"].as_array().unwrap().len(), 1);
    assert_eq!(report["denials"][0]["check"], "connection-present");
    assert_eq!(report["denials"][0]["subject"]["id"], "CONN-1");
    assert_eq!(
        report["denials"][0]["matched"]["Connection"][0]["id"],
        "CONN-1"
    );
    assert_eq!(report["denials"][0]["remediations"], json!([]));
    assert_eq!(
        report["denials"][0]["rationale"],
        "Report the authored connection."
    );

    let strict = run(&scope, &module, &["--flow", "--strict", "--json"]);
    assert_eq!(strict.status.code(), Some(1));
    assert_eq!(json(&strict), report);

    let manifest = fs::read_to_string(module.join("manifest.yaml")).unwrap();
    write(
        &module,
        "manifest.yaml",
        &manifest.replace(
            "traceability:\n",
            "grammar_severity:\n  trace:connection-present: error\ntraceability:\n",
        ),
    );
    let promoted = run(&scope, &module, &["--flow", "--json"]);
    assert_eq!(promoted.status.code(), Some(1));
    assert_eq!(json(&promoted)["denials"][0]["severity"], "error");
    write(&module, "manifest.yaml", &manifest);

    let human = run(&scope, &module, &["--flow"]);
    assert!(human.status.success());
    let human = String::from_utf8_lossy(&human.stdout);
    assert!(human.contains("connection-present: CONN-1 (warning)"));

    let default_json = run(&scope, &module, &["--json"]);
    assert!(default_json.status.success());
    assert_eq!(json(&default_json)["flow"], report);
    let default_human = run(&scope, &module, &[]);
    assert!(default_human.status.success());
    assert!(String::from_utf8_lossy(&default_human.stdout)
        .contains("connection-present: CONN-1 (warning)"));

    let (empty_temp, empty_scope) = world(
        "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        "health",
        None,
        false,
    );
    let empty_module = empty_temp.path().join("module");
    add_flow_rule(&empty_module);
    fs::remove_file(empty_scope.join("spec/CONN-1.md")).unwrap();
    let empty = run(
        &empty_scope,
        &empty_module,
        &["--flow", "--strict", "--json"],
    );
    assert!(empty.status.success());
    assert!(json(&empty)["denials"].as_array().unwrap().is_empty());
}

#[test]
#[trace("FR-046-AC-5")]
fn flow_mode_json_preserves_remediation_details_from_the_engine() {
    let (temp, scope) = world(
        "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        "health",
        Some("trust_level: untrusted\n"),
        true,
    );
    let module = temp.path().join("module");
    add_flow_rule_with_remediation(&module);
    let registry = quire_rs::Registry::load_module(&module).expect("fixture module loads");
    assert!(registry.failures().is_empty(), "{:?}", registry.failures());
    assert_eq!(
        registry.traceability().unwrap().flow_rules.len(),
        1,
        "fixture flow declaration is retained"
    );

    let output = run(&scope, &module, &["--flow", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = json(&output);
    assert_eq!(
        report["denials"][0]["remediations"],
        json!([{
            "name": "set-flow-direction",
            "description": "Set flowDirection to bidirectional.",
            "change": {
                "sets": {
                    "field": "flowDirection",
                    "value": "bidirectional"
                }
            }
        }])
    );
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
#[trace("FR-046-AC-5")]
fn sorts_multiple_flows_classifications_crossings_and_missing_references() {
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

    let endpoints = "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n";
    let mut multiple_refs = vec![
        ("classified-as", "CLS-Z"),
        ("classified-as", "CLS-A"),
        ("crosses", "BOUNDARY-Z"),
        ("crosses", "BOUNDARY-A"),
    ];
    write(
        &scope,
        "spec/a.md",
        &document("CONN-Z", "Connection", endpoints, &multiple_refs),
    );
    multiple_refs = vec![("classified-as", "CLS-M"), ("crosses", "BOUNDARY-M")];
    write(
        &scope,
        "spec/z.md",
        &document("CONN-A", "Connection", endpoints, &multiple_refs),
    );
    multiple_refs = vec![
        ("classified-as", "CLS-G"),
        ("crosses", "BOUNDARY-Z-DANGLING"),
        ("crosses", "BOUNDARY-A-DANGLING"),
        ("crosses", "BOUNDARY-EMPTY"),
    ];
    write(
        &scope,
        "spec/a-gap.md",
        &document("CONN-GAP-Z", "Connection", endpoints, &multiple_refs),
    );
    multiple_refs = vec![
        ("classified-as", "CLS-G"),
        ("crosses", "BOUNDARY-Q-DANGLING"),
        ("crosses", "BOUNDARY-P-DANGLING"),
    ];
    write(
        &scope,
        "spec/z-gap.md",
        &document("CONN-GAP-A", "Connection", endpoints, &multiple_refs),
    );

    for id in ["CLS-Z", "CLS-A", "CLS-M", "CLS-G"] {
        write(
            &scope,
            &format!("spec/{id}.md"),
            &document(id, "Classification", "regulatedCategories: [health]\n", &[]),
        );
    }
    for id in ["BOUNDARY-Z", "BOUNDARY-A", "BOUNDARY-M"] {
        write(
            &scope,
            &format!("spec/{id}.md"),
            &document(id, "TrustBoundary", "trust_level: untrusted\n", &[]),
        );
    }
    write(
        &scope,
        "spec/BOUNDARY-EMPTY.md",
        &document("BOUNDARY-EMPTY", "TrustBoundary", "", &[]),
    );

    let output = run(&scope, &module, &["--health-data-flows", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = json(&output);
    let flows = report["flows"].as_array().unwrap();
    assert_eq!(flows.len(), 2);
    assert_eq!(flows[0]["connection"]["id"], "CONN-A");
    assert_eq!(flows[1]["connection"]["id"], "CONN-Z");
    let classification_ids: Vec<&str> = flows[1]["classifications"]
        .as_array()
        .unwrap()
        .iter()
        .map(|document| document["id"].as_str().unwrap())
        .collect();
    assert_eq!(classification_ids, ["CLS-A", "CLS-Z"]);
    let crossing_ids: Vec<&str> = flows[1]["crossings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|crossing| crossing["boundary"]["id"].as_str().unwrap())
        .collect();
    assert_eq!(crossing_ids, ["BOUNDARY-A", "BOUNDARY-Z"]);
    assert_eq!(report["gaps"].as_array().unwrap().len(), 2);
    assert_eq!(report["gaps"][0]["connection"]["id"], "CONN-GAP-A");
    assert_eq!(report["gaps"][1]["connection"]["id"], "CONN-GAP-Z");
    let missing: Vec<(&str, Option<&str>)> = report["gaps"][1]["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| {
            (
                item["field"].as_str().unwrap(),
                item["reference"]["id"].as_str(),
            )
        })
        .collect();
    assert_eq!(
        missing,
        [
            ("crossing", Some("BOUNDARY-A-DANGLING")),
            ("crossing", Some("BOUNDARY-Z-DANGLING")),
            ("trust_level", Some("BOUNDARY-EMPTY")),
        ]
    );
    let second = run(&scope, &module, &["--health-data-flows", "--json"]);
    assert_eq!(output.stdout, second.stdout);
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
    assert!(!missing.iter().any(|item| item["field"] == "destination"));
    assert!(missing
        .iter()
        .any(|item| item["field"] == "crossing" && item["reference"]["id"] == "BOUNDARY-1"));

    let (temp, scope) = world(
        "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\n",
        "health",
        None,
        false,
    );
    let output = run(
        &scope,
        &temp.path().join("module"),
        &["--health-data-flows", "--json"],
    );
    let report = json(&output);
    let missing = report["gaps"][0]["missing"].as_array().unwrap();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0]["field"], "destination");
    assert!(missing[0]["reference"].is_null());

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
    let encoded = String::from_utf8_lossy(&output.stdout);
    let missing_item = report["gaps"][0]["missing"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["field"] == "trust_level")
        .expect("missing trust level is represented");
    assert_object_shape(
        &report["gaps"][0],
        &encoded,
        "\"gaps\":[{",
        &["connection", "missing"],
    );
    assert_object_shape(
        &missing_item["reference"],
        &encoded,
        "\"reference\":{",
        &["repo", "id", "archetype", "declared_by"],
    );
    assert_object_shape(
        &missing_item["reference"]["declared_by"],
        &encoded,
        "\"declared_by\":{",
        &["repo", "id", "archetype", "path"],
    );
}

#[test]
#[trace("FR-046-AC-3")]
fn an_invalid_connection_record_reports_unavailability_without_guessing_endpoints() {
    let complete = "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\ntargetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\nflowDirection: source-to-target\n";
    let cases = [
        (
            "PORT-TARGET",
            "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\n",
        ),
        (
            "PORT-SOURCE",
            "targetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n",
        ),
    ];
    for (endpoint_id, remaining_endpoint) in cases {
        let mut record = complete.to_owned();
        let removed = if endpoint_id == "PORT-TARGET" {
            "targetEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-TARGET\n"
        } else {
            "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\n"
        };
        record = record.replace(removed, "");
        assert!(record.contains(remaining_endpoint));
        let (temp, scope) = world(&record, "health", None, false);
        write(
            &temp.path().join("module"),
            "schemas/Connection.json",
            REQUIRED_CONNECTION_SCHEMA,
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
        let gap = &report["gaps"][0];
        assert_eq!(gap["connection"]["id"], "CONN-1");
        assert_eq!(gap["missing"].as_array().unwrap().len(), 1);
        assert_eq!(gap["missing"][0]["field"], "connection_record");
        assert!(gap["missing"][0]["reference"].is_null());
        let encoded = String::from_utf8_lossy(&output.stdout);
        assert_object_shape(gap, &encoded, "\"gaps\":[{", &["connection", "missing"]);
        assert_object_shape(
            &gap["connection"],
            &encoded,
            "\"connection\":{",
            &["repo", "id", "archetype", "path"],
        );
        assert_object_shape(
            &gap["missing"][0],
            &encoded,
            "\"missing\":[{",
            &["field", "reference"],
        );
    }

    let missing_target = "sourceEnd:\n  type: ix://agent-ix/health-flow-fixture/PORT-SOURCE\nflowDirection: source-to-target\n";
    let (temp, scope) = world(missing_target, "health", None, true);
    write(
        &temp.path().join("module"),
        "schemas/Connection.json",
        REQUIRED_CONNECTION_SCHEMA,
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
    assert_eq!(missing.len(), 2);
    assert_eq!(missing[0]["field"], "connection_record");
    assert!(missing[0]["reference"].is_null());
    assert_eq!(missing[1]["field"], "crossing");
    assert_eq!(missing[1]["reference"]["id"], "BOUNDARY-1");
    assert_eq!(missing[1]["reference"]["declared_by"]["id"], "CONN-1");
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

#[test]
#[trace("FR-044-AC-11")]
fn flow_mode_rejects_missing_rules_and_relation_mode_conflicts() {
    let (temp, scope) = world("", "health", None, false);
    let module = temp.path().join("module");
    for (flags, expected) in [
        (vec!["--flow", "--json"], "traceability.flow_rules"),
        (
            vec!["--flow", "--relation", "connection-carries-classification"],
            "--relation",
        ),
        (vec!["--flow", "--design"], "--design"),
    ] {
        let output = run(&scope, &module, &flags);
        assert_eq!(output.status.code(), Some(3));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
#[trace("FR-044-AC-9")]
fn flow_mode_reports_invalid_rule_values_in_the_module_load_failure() {
    assert_flow_load_failure_names_the_invalid_value(
        "invalid-rule",
        "any_of: [bidirectional]",
        "any_of: [medical]",
        "medical",
    );
}

#[test]
#[trace("FR-044-AC-10")]
fn flow_mode_reports_invalid_remediations_in_the_module_load_failure() {
    assert_flow_load_failure_names_the_invalid_value(
        "invalid-remediation",
        "remediations: [set-flow-direction]",
        "remediations: [encrypt-in-transt]",
        "encrypt-in-transt",
    );
}

#[test]
#[trace("FR-044-AC-16")]
fn flow_mode_does_not_require_required_relations() {
    let (temp, scope) = world("", "health", None, false);
    let module = temp.path().join("module");
    add_flow_only_rule(&module);

    let output = run(&scope, &module, &["--flow", "--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(json(&output)["denials"].as_array().unwrap().len(), 1);

    let relations = run(&scope, &module, &["--json"]);
    assert_eq!(relations.status.code(), Some(3));
    assert!(relations.stdout.is_empty());
    assert!(String::from_utf8_lossy(&relations.stderr).contains("required_relations"));
}
