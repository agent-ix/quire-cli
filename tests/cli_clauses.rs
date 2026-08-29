//! Process-boundary tests for `quire clauses` (FR-020).

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use jsonschema::JSONSchema;
use quire_rs::clauses::{
    ApplicabilityExpr, ClassificationDimension, Clause, ClauseForce, ClauseSet, ClauseSetRights,
    ExpectedOutput, StructureRights, TextRights,
};
use serde_json::Value;
use tempfile::TempDir;

use common::quire;

fn set(version: &str, changed: bool) -> ClauseSet {
    let mut set = ClauseSet {
        schema_version: "clause-set-v1".into(),
        authority: "example.test".into(),
        id: "widget-assurance".into(),
        title: "Synthetic widget assurance rules".into(),
        version: version.into(),
        digest: String::new(),
        rights: ClauseSetRights {
            structure: StructureRights::Original,
            text: TextRights::Original,
            clearance_ref: None,
        },
        source: None,
        classification_dimensions: BTreeMap::from([(
            "impact".into(),
            ClassificationDimension {
                values: vec!["low".into(), "medium".into(), "high".into()],
                order: vec![
                    vec!["low".into()],
                    vec!["medium".into()],
                    vec!["high".into()],
                ],
            },
        )]),
        output_catalog: BTreeMap::from([(
            "test-result".into(),
            ExpectedOutput {
                kind: "record".into(),
                description: "A synthetic test result".into(),
            },
        )]),
        clauses: vec![Clause {
            id: "W-1".into(),
            force: if changed {
                ClauseForce::Recommended
            } else {
                ClauseForce::Mandatory
            },
            title: Some("Exercise material widgets".into()),
            text: Some("Exercise each material widget before release.".into()),
            subjects: vec!["widget".into()],
            obligated_actors: vec!["release-owner".into()],
            approval_roles: vec!["reviewer".into()],
            styles: BTreeMap::new(),
            applicability: Some(ApplicabilityExpr::AtLeast {
                dimension: "impact".into(),
                value: "medium".into(),
            }),
            expected_outputs: vec!["test-result".into()],
        }],
        crosswalks: Vec::new(),
    };
    if changed {
        set.clauses.push(Clause {
            id: "W-2".into(),
            force: ClauseForce::Permitted,
            title: None,
            text: None,
            subjects: Vec::new(),
            obligated_actors: Vec::new(),
            approval_roles: Vec::new(),
            styles: BTreeMap::new(),
            applicability: None,
            expected_outputs: Vec::new(),
        });
    }
    set.digest = set.computed_digest();
    set
}

fn module(dir: &TempDir) -> String {
    let root = dir.path().join("module");
    fs::create_dir_all(root.join("clauses")).expect("create clause directory");
    fs::write(
        root.join("manifest.yaml"),
        "name: synthetic-clause-module\nclause_sets:\n  - clauses/v1.json\n  - clauses/v2.json\n",
    )
    .expect("write manifest");
    fs::write(
        root.join("clauses/v1.json"),
        serde_json::to_vec_pretty(&set("1.0.0", false)).unwrap(),
    )
    .expect("write v1");
    fs::write(
        root.join("clauses/v2.json"),
        serde_json::to_vec_pretty(&set("2.0.0", true)).unwrap(),
    )
    .expect("write v2");
    root.to_string_lossy().into_owned()
}

fn published_schema(name: &str) -> Value {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata");
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("metadata JSON");
    let path = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| package["name"] == "quire-rs")
        .and_then(|package| package["manifest_path"].as_str())
        .map(PathBuf::from)
        .and_then(|path| {
            path.parent()
                .map(|root| root.join("schemas/output").join(name))
        })
        .expect("quire-rs schema path");
    serde_json::from_slice(&fs::read(path).expect("read schema")).expect("schema JSON")
}

fn assert_conforms(payload: &Value, schema_name: &str) {
    let schema = published_schema(schema_name);
    let validator = JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(&schema)
        .expect("compile schema");
    if let Err(errors) = validator.validate(payload) {
        panic!(
            "payload does not conform: {}",
            errors
                .map(|error| error.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        );
    };
}

// IT-137, FR-020-AC-1..3, AC-6.
#[test]
fn it137_evaluate_emits_three_valued_provenanced_contract() {
    let dir = TempDir::new().unwrap();
    let module = module(&dir);
    let output = quire()
        .args([
            "clauses",
            "evaluate",
            "--module",
            &module,
            "--authority",
            "example.test",
            "--set",
            "widget-assurance",
            "--version",
            "1.0.0",
            "--context",
            "impact=high",
            "--format",
            "json",
        ])
        .output()
        .expect("run evaluate");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("binding JSON");
    assert_eq!(payload["clauses"][0]["outcome"], "binding");
    assert_eq!(payload["clauseSet"]["version"], "1.0.0");
    assert!(payload["engine"]["capabilities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "clause_sets"));
    assert_conforms(&payload, "clause-binding-v1.schema.json");

    let unresolved = quire()
        .args([
            "clauses",
            "evaluate",
            "--module",
            &module,
            "--authority",
            "example.test",
            "--set",
            "widget-assurance",
            "--version",
            "1.0.0",
            "--json",
        ])
        .output()
        .expect("run unresolved evaluate");
    let payload: Value = serde_json::from_slice(&unresolved.stdout).expect("unresolved JSON");
    assert_eq!(payload["clauses"][0]["outcome"], "unresolved");
    assert_eq!(
        payload["clauses"][0]["reasons"][0]["code"],
        "missing_dimension"
    );
}

// IT-138, FR-020-AC-4, AC-6.
#[test]
fn it138_diff_emits_exact_version_schema_conformant_contract() {
    let dir = TempDir::new().unwrap();
    let module = module(&dir);
    let output = quire()
        .args([
            "clauses",
            "diff",
            "--module",
            &module,
            "--authority",
            "example.test",
            "--set",
            "widget-assurance",
            "--before-version",
            "1.0.0",
            "--after-version",
            "2.0.0",
            "--format",
            "json",
        ])
        .output()
        .expect("run diff");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: Value = serde_json::from_slice(&output.stdout).expect("diff JSON");
    assert_eq!(payload["added"][0]["id"], "W-2");
    assert_eq!(payload["changed"][0]["clauseId"], "W-1");
    assert_conforms(&payload, "clause-diff-v1.schema.json");
}

// IT-139, FR-020-AC-5.
#[test]
fn it139_tsv_is_stable_and_machine_columned() {
    let dir = TempDir::new().unwrap();
    let module = module(&dir);
    let run = || {
        quire()
            .args([
                "clauses",
                "evaluate",
                "--module",
                &module,
                "--authority",
                "example.test",
                "--set",
                "widget-assurance",
                "--version",
                "1.0.0",
                "--context",
                "impact=low",
                "--format",
                "tsv",
            ])
            .output()
            .expect("run TSV")
    };
    let (first, second) = (run(), run());
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let body = String::from_utf8(first.stdout).unwrap();
    let lines = body.lines().collect::<Vec<_>>();
    assert_eq!(lines[0].split('\t').count(), 5);
    assert_eq!(lines[1].split('\t').count(), 5);
    assert!(lines[1].contains("not_binding"));
}

// IT-140, FR-020-AC-1, AC-7.
#[test]
fn it140_invalid_context_and_unknown_exact_version_fail_closed() {
    let dir = TempDir::new().unwrap();
    let module_path = module(&dir);
    let malformed = quire()
        .args([
            "clauses",
            "evaluate",
            "--module",
            &module_path,
            "--authority",
            "example.test",
            "--set",
            "widget-assurance",
            "--version",
            "1.0.0",
            "--context",
            "impact",
        ])
        .output()
        .unwrap();
    assert_eq!(malformed.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("KEY=VALUE"));

    let missing = quire()
        .args([
            "clauses",
            "evaluate",
            "--module",
            &module_path,
            "--authority",
            "example.test",
            "--set",
            "widget-assurance",
            "--version",
            "9.0.0",
        ])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("available exact sets"));

    let traversing_module = format!("{module_path}/../module");
    let traversal = quire()
        .args([
            "clauses",
            "evaluate",
            "--module",
            &traversing_module,
            "--authority",
            "example.test",
            "--set",
            "widget-assurance",
            "--version",
            "1.0.0",
        ])
        .output()
        .unwrap();
    assert_eq!(traversal.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&traversal.stderr).contains("PathTraversal"));

    let invalid_dir = TempDir::new().unwrap();
    let invalid_module = module(&invalid_dir);
    let invalid_path = invalid_dir.path().join("module/clauses/v1.json");
    let mut invalid: Value =
        serde_json::from_slice(&fs::read(&invalid_path).unwrap()).expect("clause JSON");
    invalid["title"] = Value::String("Changed without a new digest".into());
    fs::write(&invalid_path, serde_json::to_vec_pretty(&invalid).unwrap()).unwrap();
    let rejected = quire()
        .args([
            "clauses",
            "evaluate",
            "--module",
            &invalid_module,
            "--authority",
            "example.test",
            "--set",
            "widget-assurance",
            "--version",
            "1.0.0",
        ])
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("digest"));
}
