//! Process-boundary tests for `quire clauses` (FR-021).

mod common;

use ix_trace_rs::trace;
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
        rights: ClauseSetRights {
            structure: StructureRights::Original,
            text: TextRights::Original,
            clearance_ref: None,
        },
        source: None,
        classification_dimensions: BTreeMap::from([
            (
                "impact".into(),
                ClassificationDimension {
                    values: vec!["low".into(), "medium".into(), "high".into()],
                    order: vec![
                        vec!["low".into()],
                        vec!["medium".into()],
                        vec!["high".into()],
                    ],
                },
            ),
            (
                "audience".into(),
                ClassificationDimension {
                    values: vec!["internal".into(), "external".into()],
                    order: vec![vec!["internal".into()], vec!["external".into()]],
                },
            ),
        ]),
        output_catalog: BTreeMap::from([(
            "test-result".into(),
            ExpectedOutput {
                kind: "record".into(),
                description: "A synthetic test result".into(),
                producers: None,
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
            applicability: Some(ApplicabilityExpr::All {
                terms: vec![
                    ApplicabilityExpr::AtLeast {
                        dimension: "impact".into(),
                        value: "medium".into(),
                    },
                    ApplicabilityExpr::Eq {
                        dimension: "audience".into(),
                        value: "internal".into(),
                    },
                ],
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

fn module_with_cross_module_verb(dir: &TempDir) -> (String, String) {
    let owner = module(dir);
    let owner_root = PathBuf::from(&owner);
    let mut manifest = fs::read_to_string(owner_root.join("manifest.yaml")).unwrap();
    manifest.push_str("object_types:\n- name: contract\n  allowed_links: [registered-verb]\n");
    fs::write(owner_root.join("manifest.yaml"), manifest).unwrap();

    let vocabulary = dir.path().join("vocabulary-module");
    fs::create_dir_all(&vocabulary).unwrap();
    fs::write(
        vocabulary.join("manifest.yaml"),
        "name: synthetic-vocabulary-module\nedge_types:\n  registered-verb:\n    description: A registered synthetic relationship\n    category: traceability\n",
    )
    .unwrap();
    (owner, vocabulary.to_string_lossy().into_owned())
}

fn module_with_tsv_delimiter(dir: &TempDir) -> String {
    let root = dir.path().join("module");
    fs::create_dir_all(root.join("clauses")).expect("create clause directory");
    fs::write(
        root.join("manifest.yaml"),
        "name: synthetic-clause-module\nclause_sets:\n  - clauses/v1.json\n  - clauses/v2.json\n",
    )
    .expect("write manifest");

    let expected_output = "test\tresult";
    let mut first = set("1.0.0", false);
    first.output_catalog = BTreeMap::from([(
        expected_output.into(),
        ExpectedOutput {
            kind: "record".into(),
            description: "A synthetic test result".into(),
            producers: None,
        },
    )]);
    first.clauses[0].expected_outputs = vec![expected_output.into()];
    fs::write(
        root.join("clauses/v1.json"),
        serde_json::to_vec_pretty(&first).unwrap(),
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

// FR-021-AC-1..3, AC-6.
#[trace("FR-021-AC-1", "FR-021-AC-2", "FR-021-AC-3", "FR-021-AC-6")]
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
            "--context",
            "audience=internal",
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

    let not_binding = quire()
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
            "--context",
            "audience=internal",
            "--json",
        ])
        .output()
        .expect("run non-binding evaluate");
    assert!(not_binding.status.success());
    let payload: Value = serde_json::from_slice(&not_binding.stdout).expect("non-binding JSON");
    assert_eq!(payload["clauses"][0]["outcome"], "not_binding");

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

// FR-021-AC-4, AC-6.
#[trace("FR-021-AC-4", "FR-021-AC-6")]
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

// FR-021-AC-5.
#[trace("FR-021-AC-5")]
#[test]
fn it139_tsv_is_stable_and_machine_columned() {
    let dir = TempDir::new().unwrap();
    let module = module(&dir);
    let human = quire()
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
            "--context",
            "audience=internal",
        ])
        .output()
        .expect("run human output");
    assert!(human.status.success());
    let human_body = String::from_utf8(human.stdout).unwrap();
    assert!(human_body.contains("Clause set example.test/widget-assurance/1.0.0"));
    assert!(human_body.contains("W-1\tmandatory\tbinding"));

    let escaped_dir = TempDir::new().unwrap();
    let escaped_module = module_with_tsv_delimiter(&escaped_dir);
    let run = || {
        quire()
            .args([
                "clauses",
                "evaluate",
                "--module",
                &escaped_module,
                "--authority",
                "example.test",
                "--set",
                "widget-assurance",
                "--version",
                "1.0.0",
                "--context",
                "impact=low",
                "--context",
                "audience=internal",
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
    assert_eq!(
        lines.len(),
        2,
        "one clause remains one physical row: {body:?}"
    );
    assert_eq!(lines[0].split('\t').count(), 5);
    assert_eq!(lines[1].split('\t').count(), 5);
    assert!(lines[1].contains("not_binding"));
    assert!(lines[1].contains("test\\tresult"), "{body:?}");
}

// FR-021-AC-1, AC-7.
#[trace("FR-021-AC-1", "FR-021-AC-2")]
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
    assert_eq!(malformed.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&malformed.stderr).contains("KEY=VALUE"));

    let duplicate = quire()
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
            "impact=low",
            "--context",
            "impact=high",
        ])
        .output()
        .unwrap();
    assert_eq!(duplicate.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("more than once"));

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
    assert_eq!(missing.status.code(), Some(3));
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
    assert_eq!(traversal.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&traversal.stderr).contains("PathTraversal"));
}

#[trace("FR-021-AC-1", "FR-021-AC-8")]
#[test]
fn it141_clause_commands_use_only_the_explicit_closed_module_set() {
    let dir = TempDir::new().unwrap();
    let (owner, vocabulary) = module_with_cross_module_verb(&dir);
    for command in ["evaluate", "diff"] {
        let mut explicit = quire();
        explicit.args([
            "clauses",
            command,
            "--module",
            &owner,
            "--module",
            &vocabulary,
        ]);
        explicit.args(["--authority", "example.test", "--set", "widget-assurance"]);
        if command == "evaluate" {
            explicit.args(["--version", "1.0.0", "--json"]);
        } else {
            explicit.args([
                "--before-version",
                "1.0.0",
                "--after-version",
                "2.0.0",
                "--json",
            ]);
        }
        let output = explicit
            .output()
            .expect("run with complete explicit module set");
        assert!(
            output.status.success(),
            "{command} with both roots failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!output.stdout.is_empty(), "{command} emitted no result");

        let mut incomplete = quire();
        incomplete.env("IX_FILAMENT_MODULES_PATH", dir.path());
        incomplete.args(["clauses", command, "--module", &owner]);
        incomplete.args(["--authority", "example.test", "--set", "widget-assurance"]);
        if command == "evaluate" {
            incomplete.args(["--version", "1.0.0", "--json"]);
        } else {
            incomplete.args([
                "--before-version",
                "1.0.0",
                "--after-version",
                "2.0.0",
                "--json",
            ]);
        }
        let output = incomplete
            .output()
            .expect("run with vocabulary module available only through ambient discovery");
        assert_eq!(
            output.status.code(),
            Some(3),
            "{command} should refuse the incomplete set"
        );
        assert!(
            output.stdout.is_empty(),
            "{command} wrote a result after refusal"
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("UnknownEdgeType") && stderr.contains("registered-verb"),
            "{command} did not report the missing explicit vocabulary: {stderr}"
        );
    }
}

/// The CLI resolves caller inputs and renders the engine's result; clause
/// applicability, rights validation, and comparison remain upstream.
#[trace("FR-021-AC-7")]
#[test]
fn clause_command_delegates_semantics_to_the_engine() {
    let source = include_str!("../src/commands/clauses.rs");
    assert!(
        source.contains("set.evaluate(&parse_context(&args.context)?)"),
        "evaluation must use the engine's clause-set operation"
    );
    assert!(
        source.contains("diff_clause_sets(before, after)"),
        "comparison must use the engine's diff operation"
    );
    assert!(
        source.contains("Registry::load_module_set_strict(&roots)"),
        "rights and module validation must come from strict closed-set engine loading"
    );
    assert!(
        source.contains("registry.clause_set(authority, id, version)"),
        "exact set selection must be delegated to the engine registry"
    );
    assert!(
        !source.contains("ApplicabilityExpr") && !source.contains("ClauseSetRights"),
        "the CLI must not implement clause semantics or rights policy"
    );
}

fn catalog_entry(kind: &str, producers: Option<Vec<&str>>) -> ExpectedOutput {
    ExpectedOutput {
        kind: kind.into(),
        description: format!("A synthetic {kind} output"),
        producers: producers.map(|kinds| kinds.into_iter().map(String::from).collect()),
    }
}

/// A synthetic set whose clauses reference no catalog output, so each test
/// controls the catalog alone.
fn catalog_set(
    version: &str,
    changed: bool,
    catalog: BTreeMap<String, ExpectedOutput>,
) -> ClauseSet {
    let mut set = set(version, changed);
    set.output_catalog = catalog;
    for clause in &mut set.clauses {
        clause.expected_outputs.clear();
    }
    set
}

fn module_with_sets(dir: &TempDir, first: &ClauseSet, second: &ClauseSet) -> String {
    let root = dir.path().join("module");
    fs::create_dir_all(root.join("clauses")).expect("create clause directory");
    fs::write(
        root.join("manifest.yaml"),
        "name: synthetic-clause-module\nclause_sets:\n  - clauses/v1.json\n  - clauses/v2.json\n",
    )
    .expect("write manifest");
    fs::write(
        root.join("clauses/v1.json"),
        serde_json::to_vec_pretty(first).unwrap(),
    )
    .expect("write v1");
    fs::write(
        root.join("clauses/v2.json"),
        serde_json::to_vec_pretty(second).unwrap(),
    )
    .expect("write v2");
    root.to_string_lossy().into_owned()
}

fn run_diff(module: &str, format: &str) -> std::process::Output {
    quire()
        .args([
            "clauses",
            "diff",
            "--module",
            module,
            "--authority",
            "example.test",
            "--set",
            "widget-assurance",
            "--before-version",
            "1.0.0",
            "--after-version",
            "2.0.0",
            "--format",
            format,
        ])
        .output()
        .expect("run diff")
}

// FR-021-AC-4, AC-9.
#[trace("FR-021-AC-4", "FR-021-AC-9")]
#[test]
fn it142_diff_shows_output_catalog_changes_in_every_view() {
    let mut described = catalog_entry("record", None);
    let first = catalog_set(
        "1.0.0",
        false,
        BTreeMap::from([
            ("kept".into(), catalog_entry("record", Some(vec!["run"]))),
            ("dropped".into(), catalog_entry("report", None)),
            ("csv".into(), catalog_entry("record", Some(vec!["a,b"]))),
            ("desc".into(), described.clone()),
        ]),
    );
    described.description = "A reworded description".into();
    let second = catalog_set(
        "2.0.0",
        true,
        BTreeMap::from([
            (
                "kept".into(),
                catalog_entry("record", Some(vec!["run", "scan"])),
            ),
            ("fresh".into(), catalog_entry("log", Some(vec!["audit"]))),
            ("csv".into(), catalog_entry("record", Some(vec!["a", "b"]))),
            ("desc".into(), described),
        ]),
    );
    let dir = TempDir::new().unwrap();
    let module = module_with_sets(&dir, &first, &second);

    let json = run_diff(&module, "json");
    assert!(
        json.status.success(),
        "{}",
        String::from_utf8_lossy(&json.stderr)
    );
    let payload: Value = serde_json::from_slice(&json.stdout).expect("diff JSON");
    let catalog = &payload["outputCatalog"];
    assert_eq!(catalog["added"][0]["name"], "fresh");
    assert_eq!(catalog["added"][0]["producers"][0], "audit");
    assert_eq!(catalog["removed"][0]["name"], "dropped");
    assert_eq!(catalog["changed"][2]["name"], "kept");
    assert_eq!(catalog["changed"][2]["after"]["producers"][1], "scan");
    assert_conforms(&payload, "clause-diff-v1.schema.json");

    let human = run_diff(&module, "human");
    assert!(human.status.success());
    let body = String::from_utf8(human.stdout).unwrap();
    assert!(
        body.contains("Output catalog added: 1  Removed: 1  Changed: 3"),
        "{body}"
    );
    assert!(
        body.contains("output added\tfresh\tproducers: [\"audit\"]\n"),
        "{body}"
    );
    assert!(body.contains("output removed\tdropped\n"), "{body}");
    assert!(
        body.contains("output changed\tkept\tkind: record; producers: [\"run\"]; description:"),
        "{body}"
    );
    assert!(
        body.contains("-> kind: record; producers: [\"run\",\"scan\"];"),
        "{body}"
    );
    assert!(
        body.contains("output changed\tcsv\tkind: record; producers: [\"a,b\"];"),
        "{body}"
    );

    let tsv = run_diff(&module, "tsv");
    assert!(tsv.status.success());
    let body = String::from_utf8(tsv.stdout).unwrap();
    assert_eq!(
        body.lines().next(),
        Some(
            "change\tclause_id\tbefore_force\tafter_force\toutput\tbefore_producers\tafter_producers\tchanged_fields"
        ),
        "{body:?}"
    );
    for line in body.lines() {
        assert_eq!(line.split('\t').count(), 8, "{line:?}");
    }
    assert!(
        body.contains("\nadded\tW-2\t\tpermitted\t\t\t\t\n"),
        "{body:?}"
    );
    assert!(
        body.contains("\nchanged\tW-1\tmandatory\trecommended\t\t\t\t\n"),
        "{body:?}"
    );
    assert!(
        body.contains("\noutput_added\t\t\t\tfresh\t\t[\"audit\"]\t\n"),
        "{body:?}"
    );
    assert!(
        body.contains("\noutput_removed\t\t\t\tdropped\t\t\t\n"),
        "{body:?}"
    );
    assert!(
        body.contains("\noutput_changed\t\t\t\tkept\t[\"run\"]\t[\"run\",\"scan\"]\tproducers\n"),
        "{body:?}"
    );
    assert!(
        body.contains("\noutput_changed\t\t\t\tcsv\t[\"a,b\"]\t[\"a\",\"b\"]\tproducers\n"),
        "{body:?}"
    );
    assert!(
        body.contains("\noutput_changed\t\t\t\tdesc\t\t\tdescription\n"),
        "{body:?}"
    );
}

// FR-021-AC-10.
#[trace("FR-021-AC-10")]
#[test]
fn it143_invalid_catalog_producers_are_rejected_naming_the_output() {
    let invalid = [
        ("empty list", vec![]),
        ("padded kind", vec![" run"]),
        ("empty kind", vec![""]),
        ("duplicate kind", vec!["run", "run"]),
    ];
    for (label, producers) in invalid {
        let bad = catalog_set(
            "2.0.0",
            false,
            BTreeMap::from([(
                "bad-output".into(),
                catalog_entry("record", Some(producers)),
            )]),
        );
        let good = catalog_set("1.0.0", false, BTreeMap::new());
        let dir = TempDir::new().unwrap();
        let module = module_with_sets(&dir, &good, &bad);
        for command in ["evaluate", "diff"] {
            let mut run = quire();
            run.args(["clauses", command, "--module", &module]);
            run.args(["--authority", "example.test", "--set", "widget-assurance"]);
            if command == "evaluate" {
                run.args(["--version", "2.0.0", "--json"]);
            } else {
                run.args([
                    "--before-version",
                    "1.0.0",
                    "--after-version",
                    "2.0.0",
                    "--json",
                ]);
            }
            let output = run.output().expect("run clause command");
            assert_eq!(output.status.code(), Some(3), "{command} {label}");
            assert!(output.stdout.is_empty(), "{command} {label}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("bad-output"), "{command} {label}: {stderr}");
            assert!(stderr.contains("producers"), "{command} {label}: {stderr}");
        }
    }
}
