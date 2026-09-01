//! FR-020 / IT-136..142: the process-level assurance-v1 contract.

mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use jsonschema::{Draft, JSONSchema};
use quire_rs::assurance::{AssuranceExport, ASSURANCE_V1_SCHEMA};
use serde_json::Value;
use tempfile::TempDir;

use common::quire;

const REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const MODULE: &str = "assurance-fixture@1.2.3";
const SCHEMAS: [&str; 3] = [
    "assurance-fixture/FR@44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
    "assurance-fixture/NFR@44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
    "assurance-fixture/StR@44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
];

struct Fixture {
    _temp: TempDir,
    scope: PathBuf,
    module: PathBuf,
}

fn write(path: &Path, bytes: impl AsRef<[u8]>) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixture directory");
    }
    fs::write(path, bytes).expect("write fixture");
}

fn fixture(unreadable_document: bool) -> Fixture {
    let temp = TempDir::new().expect("tempdir");
    let module = temp.path().join("module");
    let scope = temp.path().join("scope");
    write(
        &module.join("manifest.yaml"),
        include_bytes!("fixtures/assurance/module/manifest.yaml"),
    );
    for (relative, contents) in [
        (
            "spec/FR-001.md",
            include_bytes!("fixtures/assurance/scope/spec/FR-001.md").as_slice(),
        ),
        (
            "spec/FR-002.md",
            include_bytes!("fixtures/assurance/scope/spec/FR-002.md").as_slice(),
        ),
        (
            "spec/StR-001.md",
            include_bytes!("fixtures/assurance/scope/spec/StR-001.md").as_slice(),
        ),
        (
            "src/lib.rs",
            include_bytes!("fixtures/assurance/scope/src/lib.rs").as_slice(),
        ),
    ] {
        write(&scope.join(relative), contents);
    }
    if unreadable_document {
        // `read_to_string` refuses these bytes. Quire-rs preserves that bounded
        // inability as an `unknown` observation rather than dropping it or
        // converting it to `missing` (FR-068-AC-6).
        write(&scope.join("spec/unreadable.md"), [0xff, 0xfe]);
    }
    Fixture {
        _temp: temp,
        scope,
        module,
    }
}

fn command(fixture: &Fixture) -> Command {
    command_with_source(fixture, "agent-ix/fixture", REVISION)
}

fn command_with_source(fixture: &Fixture, repository: &str, revision: &str) -> Command {
    let mut command = quire();
    command
        .arg("assurance")
        .arg("--scope")
        .arg(&fixture.scope)
        .arg("--module")
        .arg(&fixture.module)
        .arg("--repository")
        .arg(repository)
        .arg("--revision")
        .arg(revision)
        .arg("--expect-module")
        .arg(MODULE);
    for schema in SCHEMAS {
        command.arg("--expect-schema").arg(schema);
    }
    command
}

fn output(fixture: &Fixture) -> Output {
    command(fixture).output().expect("run assurance")
}

fn success(fixture: &Fixture) -> (Output, Value) {
    let output = output(fixture);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value = serde_json::from_slice(&output.stdout).expect("assurance JSON");
    (output, value)
}

fn assert_refused(output: &Output, expected: &str) {
    assert_eq!(
        output.status.code(),
        Some(1),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "partial stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "stderr did not name `{expected}`: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn it_136_complete_fixture_exercises_every_assurance_record_family() {
    let fixture = fixture(true);
    let (_, value) = success(&fixture);

    assert_eq!(value["format"], "quire-assurance");
    assert_eq!(value["format_version"], 1);
    assert_eq!(value["source"]["revision"], REVISION);
    for collection in [
        "artifacts",
        "obligations",
        "symbols",
        "relation_kinds",
        "relations",
    ] {
        assert!(
            !value[collection].as_array().expect("array").is_empty(),
            "{collection} must be non-empty: {value:#?}"
        );
    }

    let relations = value["relations"].as_array().expect("relations");
    for kind in ["corpus", "verifies", "implements"] {
        assert!(relations.iter().any(|relation| relation["kind"] == kind));
    }
    assert!(relations
        .iter()
        .any(|relation| relation["resolution"] == "dangling"));

    let states: Vec<&str> = value["relation_observations"]
        .as_array()
        .expect("observations")
        .iter()
        .filter_map(|observation| observation["availability"].as_str())
        .collect();
    for state in ["available", "missing", "not_applicable", "unknown"] {
        assert!(
            states.contains(&state),
            "missing state `{state}`: {states:?}"
        );
    }
    assert!(value["relation_observations"]
        .as_array()
        .expect("observations")
        .iter()
        .filter(|observation| observation["availability"] == "unknown")
        .all(|observation| observation["reason"]
            .as_str()
            .is_some_and(|reason| !reason.is_empty())));
}

#[test]
fn it_137_output_is_the_upstream_type_and_schema() {
    let fixture = fixture(true);
    let (output, value) = success(&fixture);
    let _: AssuranceExport = serde_json::from_slice(&output.stdout).expect("upstream type");
    let schema: Value = serde_json::from_str(ASSURANCE_V1_SCHEMA).expect("schema JSON");
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .expect("schema compiles");
    if let Err(errors) = compiled.validate(&value) {
        panic!(
            "published assurance schema rejected CLI output: {:?}",
            errors.map(|error| error.to_string()).collect::<Vec<_>>()
        );
    }
    assert!(
        value.get("engine").is_none(),
        "closed upstream envelope: {value}"
    );
}

#[test]
fn it_138_compact_pretty_and_golden_bytes_are_deterministic() {
    let fixture = fixture(true);
    let first = output(&fixture);
    let second = output(&fixture);
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(
        first.stdout,
        include_bytes!("fixtures/assurance/v1.json"),
        "review the complete golden rather than regenerating it blindly:\n{}",
        String::from_utf8_lossy(&first.stdout)
    );

    let typed: AssuranceExport = serde_json::from_slice(&first.stdout).expect("typed export");
    let mut direct = typed.to_json_bytes().expect("upstream bytes");
    direct.push(b'\n');
    assert_eq!(first.stdout, direct);

    let pretty_one = command(&fixture)
        .arg("--pretty")
        .output()
        .expect("pretty run");
    let pretty_two = command(&fixture)
        .arg("--pretty")
        .output()
        .expect("pretty rerun");
    assert!(pretty_one.status.success());
    assert_eq!(pretty_one.stdout, pretty_two.stdout);
    assert_ne!(pretty_one.stdout, first.stdout);
    assert_eq!(
        serde_json::from_slice::<Value>(&pretty_one.stdout).expect("pretty JSON"),
        serde_json::from_slice::<Value>(&first.stdout).expect("compact JSON")
    );
}

#[test]
fn it_139_every_module_or_schema_premise_drift_is_refused_atomically() {
    let fixture = fixture(false);
    let cases: Vec<(&str, Vec<&str>, &str)> = vec![
        ("other@1.2.3", SCHEMAS.to_vec(), "module 'assurance-fixture'"),
        ("assurance-fixture@9.9.9", SCHEMAS.to_vec(), "version '1.2.3'"),
        (MODULE, SCHEMAS[..2].to_vec(), "schema digest"),
        (
            MODULE,
            vec![SCHEMAS[0], SCHEMAS[1], SCHEMAS[2], "assurance-fixture/US@44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"],
            "does not exactly match",
        ),
        (
            MODULE,
            vec![
                "assurance-fixture/FR@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                SCHEMAS[1],
                SCHEMAS[2],
            ],
            "schema digest",
        ),
    ];
    for (expected_module, schemas, message) in cases {
        let mut command = quire();
        command
            .arg("assurance")
            .arg("--scope")
            .arg(&fixture.scope)
            .arg("--module")
            .arg(&fixture.module)
            .arg("--repository")
            .arg("agent-ix/fixture")
            .arg("--revision")
            .arg(REVISION)
            .arg("--expect-module")
            .arg(expected_module);
        for schema in schemas {
            command.arg("--expect-schema").arg(schema);
        }
        assert_refused(&command.output().expect("refusal run"), message);
    }
}

#[test]
fn it_140_malformed_or_incomplete_premises_and_modules_fail_before_stdout() {
    let base = fixture(false);
    let malformed = quire()
        .args([
            "assurance",
            "--scope",
            base.scope.to_str().expect("scope"),
            "--module",
            base.module.to_str().expect("module"),
            "--repository",
            "agent-ix/fixture",
            "--revision",
            REVISION,
            "--expect-module",
            "no-at-sign",
        ])
        .output()
        .expect("malformed run");
    assert_eq!(malformed.status.code(), Some(2));
    assert!(malformed.stdout.is_empty());

    let invalid_revision = command_with_source(&base, "agent-ix/fixture", "main")
        .output()
        .expect("revision run");
    assert_refused(&invalid_revision, "not a full lowercase Git object id");

    for (needle, replacement, expected) in [
        ("name: assurance-fixture\n", "", "no authored name"),
        ("version: 1.2.3\n", "", "no declared version"),
        (
            "- name: FR\n",
            "- name: FR\n  frontmatter_schema_ref: missing.json\n",
            "did not load",
        ),
    ] {
        let fixture = fixture(false);
        let manifest_path = fixture.module.join("manifest.yaml");
        let manifest = fs::read_to_string(&manifest_path).expect("manifest");
        write(&manifest_path, manifest.replacen(needle, replacement, 1));
        assert_refused(&output(&fixture), expected);
    }
}

#[test]
fn it_141_empty_unknown_and_unavailable_are_three_distinct_outcomes() {
    let empty = fixture(false);
    fs::remove_dir_all(empty.scope.join("spec")).expect("remove populated spec");
    fs::create_dir_all(empty.scope.join("spec")).expect("empty spec");
    fs::remove_dir_all(empty.scope.join("src")).expect("remove populated src");
    let (_, value) = success(&empty);
    for collection in ["artifacts", "obligations", "symbols", "relations"] {
        assert!(value[collection].as_array().expect("array").is_empty());
    }

    let unknown = fixture(true);
    let (_, value) = success(&unknown);
    assert!(value["relation_observations"]
        .as_array()
        .expect("observations")
        .iter()
        .any(|observation| observation["availability"] == "unknown"));

    let missing = fixture(false);
    fs::remove_dir_all(missing.scope.join("spec")).expect("remove spec");
    assert_refused(&output(&missing), "no document root");

    let invalid_source = fixture(false);
    let refused = command_with_source(&invalid_source, "", REVISION)
        .output()
        .expect("empty repository run");
    assert_refused(&refused, "repository is empty");
}

#[test]
fn it_142_diagnostics_stay_on_stderr_in_human_and_json_modes() {
    let fixture = fixture(true);
    let human = output(&fixture);
    assert!(human.status.success());
    let stderr = String::from_utf8_lossy(&human.stderr);
    assert!(stderr.contains("MissingUuid"), "{stderr}");
    assert!(stderr.contains("DocumentUnreadable"), "{stderr}");
    assert!(stderr.contains("DanglingReference"), "{stderr}");
    let payload: Value = serde_json::from_slice(&human.stdout).expect("payload");
    assert!(payload.get("diagnostics").is_none());

    let json = command(&fixture)
        .args(["--diagnostics-format", "json"])
        .output()
        .expect("JSON diagnostics run");
    assert!(json.status.success());
    for line in String::from_utf8_lossy(&json.stderr).lines() {
        let diagnostic: Value = serde_json::from_str(line).expect("diagnostic JSON line");
        assert_eq!(diagnostic["severity"], "error");
    }
    assert_eq!(
        serde_json::from_slice::<Value>(&human.stdout).expect("human payload"),
        serde_json::from_slice::<Value>(&json.stdout).expect("JSON-mode payload")
    );
}
