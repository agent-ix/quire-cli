//! FR-020: the process-level assurance-v1 contract.

mod common;

use ix_trace_rs::trace;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use jsonschema::{Draft, JSONSchema};
use quire_rs::assurance::{AssuranceExport, ASSURANCE_V1_SCHEMA};
use quire_rs::{read_assurance_export, AcceptedAssurancePremises};
use serde_json::Value;
use tempfile::TempDir;

use common::quire;

const MODULE: &str = "assurance-fixture@1.2.3";
const SCHEMAS: [&str; 3] = [
    "assurance-fixture/FR",
    "assurance-fixture/NFR",
    "assurance-fixture/StR",
];
const HISTORICAL_SCHEMAS: [&str; 3] = [
    "assurance-fixture/FR@44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
    "assurance-fixture/NFR@44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
    "assurance-fixture/StR@44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a",
];
const HISTORICAL_REVISION: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const FR_020_SPEC: &str = include_str!("../spec/functional/FR-020-assurance-export-subcommand.md");

struct CompatibilityRow {
    cli_version: String,
    engine_version: String,
    evidence: String,
    result: String,
}

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
    command_with_binary(fixture, None)
}

fn command_with_binary(fixture: &Fixture, binary: Option<&Path>) -> Command {
    command_with_binary_and_generation(fixture, binary, false)
}

fn command_with_binary_and_generation(
    fixture: &Fixture,
    binary: Option<&Path>,
    historical: bool,
) -> Command {
    let mut command = match binary {
        Some(binary) => Command::new(binary),
        None => quire(),
    };
    command
        .arg("assurance")
        .arg("--scope")
        .arg(&fixture.scope)
        .arg("--module")
        .arg(&fixture.module)
        .arg("--expect-module")
        .arg(MODULE);
    if historical {
        command
            .arg("--repository")
            .arg("agent-ix/fixture")
            .arg("--revision")
            .arg(HISTORICAL_REVISION);
    }
    let schemas = if historical {
        HISTORICAL_SCHEMAS
    } else {
        SCHEMAS
    };
    for schema in schemas {
        command.arg("--expect-schema").arg(schema);
    }
    command
}

/// FR-020-AC-9, FR-020-AC-10: run actual released producers against the same
/// deterministic corpus and pass their bytes through the current Rust reader.
///
/// The release lane directory contains `<version>/quire` and the exact
/// `<version>/Cargo.lock` from that CLI tag. Keeping this out of the default
/// suite avoids requiring network-fetched release assets for ordinary tests;
/// the named compatibility lane must provide each row from FR-020's matrix.
#[trace("FR-020-AC-9")]
#[trace("FR-020-AC-10")]
#[test]
#[ignore = "release compatibility lane: provide QUIRE_CLI_RELEASE_DIR with binaries and tag lockfiles"]
fn released_cli_compatibility_matrix_matches_the_current_reader() {
    let release_dir = std::env::var_os("QUIRE_CLI_RELEASE_DIR")
        .map(PathBuf::from)
        .expect("QUIRE_CLI_RELEASE_DIR must provide released binaries and exact tag lockfiles");
    let fixture = fixture(true);
    let (current_output, current_value) = success(&fixture);
    let current_export: AssuranceExport =
        serde_json::from_slice(&current_output.stdout).expect("current assurance export");
    let accepted = AcceptedAssurancePremises::from_export(&current_export);
    assert!(
        read_assurance_export(&current_output.stdout, &accepted).is_ok(),
        "current producer output must be accepted by the current reader"
    );

    for row in compatibility_rows() {
        let cli_version = row.cli_version.as_str();
        let engine_version = row.engine_version.as_str();
        let supported = row.result.starts_with("Accept");
        assert!(
            supported || row.result.starts_with("Refuse"),
            "FR-020 matrix result must say Accept or Refuse: {}",
            row.result
        );
        let version_dir = release_dir.join(cli_version);
        let binary = version_dir.join("quire");
        assert!(
            binary.is_file(),
            "released quire-cli {cli_version} binary is missing: {}",
            binary.display()
        );
        let version = Command::new(&binary)
            .arg("--version")
            .output()
            .unwrap_or_else(|error| {
                panic!("run released quire-cli {cli_version} --version: {error}")
            });
        assert!(
            version.status.success(),
            "--version failed for quire-cli {cli_version}"
        );
        let version_text = String::from_utf8_lossy(&version.stdout);
        assert!(
            version_text.contains(cli_version),
            "released binary version mismatch for expected quire-cli {cli_version}: {version_text}"
        );

        let lockfile_path = version_dir.join("Cargo.lock");
        let lockfile_text = fs::read_to_string(&lockfile_path).unwrap_or_else(|error| {
            panic!(
                "missing exact v{cli_version} release lockfile at {}: {error}",
                lockfile_path.display()
            )
        });
        let lockfile: toml::Value = toml::from_str(&lockfile_text)
            .unwrap_or_else(|error| panic!("parse v{cli_version} Cargo.lock: {error}"));
        let packages = lockfile["package"]
            .as_array()
            .unwrap_or_else(|| panic!("v{cli_version} Cargo.lock has no package list"));
        let locked_engine = packages
            .iter()
            .find(|package| package["name"].as_str() == Some("quire-rs"))
            .and_then(|package| package["version"].as_str())
            .unwrap_or_else(|| panic!("v{cli_version} Cargo.lock has no quire-rs version"));
        assert_eq!(
            locked_engine, engine_version,
            "embedded engine evidence from exact v{cli_version} release lockfile; matrix evidence: {}",
            row.evidence
        );
        if supported {
            assert!(
                version_text.contains(engine_version),
                "released quire-cli {cli_version} --version omitted linked engine {engine_version}: {version_text}"
            );
        }

        let historical = !supported;
        let output = command_with_binary_and_generation(&fixture, Some(&binary), historical)
            .output()
            .unwrap_or_else(|error| {
                panic!("run released quire-cli {cli_version} assurance: {error}")
            });
        assert!(
            output.status.success(),
            "released quire-cli {cli_version} failed to generate assurance bytes: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let producer_value: Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
                panic!("quire-cli {cli_version} emitted invalid JSON: {error}")
            });
        let reader_result = read_assurance_export(&output.stdout, &accepted);
        if supported {
            assert!(
                reader_result.is_ok(),
                "current reader refused released quire-cli {cli_version} / quire-rs {engine_version} payload: {reader_result:?}"
            );
            assert_eq!(
                assurance_identity(&producer_value),
                assurance_identity(&current_value),
                "supported v{cli_version} payload changed assurance field identities"
            );
        } else {
            assert!(
                reader_result.is_err(),
                "current reader accepted historical quire-cli {cli_version} / quire-rs {engine_version} payload"
            );
        }
    }
}

fn assurance_identity(value: &Value) -> Value {
    serde_json::json!({
        "modules": value["modules"],
        "artifacts": value["artifacts"],
        "obligations": value["obligations"],
        "symbols": value["symbols"],
        "relation_kinds": value["relation_kinds"],
        "relations": value["relations"],
        "relation_observations": value["relation_observations"],
    })
}

fn compatibility_rows() -> Vec<CompatibilityRow> {
    let mut table = FR_020_SPEC
        .lines()
        .skip_while(|line| !line.contains("| Producer `quire-cli` |"));
    assert!(table.next().is_some(), "FR-020 compatibility matrix header");

    let mut rows = Vec::new();
    for line in table {
        let line = line.trim();
        if !line.starts_with('|') {
            break;
        }
        let cells: Vec<String> = line
            .trim_matches('|')
            .split('|')
            .map(|cell| cell.trim().trim_matches('`').to_owned())
            .collect();
        if cells.first().is_some_and(|cell| cell.starts_with("---")) {
            continue;
        }
        assert_eq!(cells.len(), 4, "FR-020 compatibility matrix row: {line}");
        rows.push(CompatibilityRow {
            cli_version: cells[0].clone(),
            engine_version: cells[1].clone(),
            evidence: cells[2].clone(),
            result: cells[3].clone(),
        });
    }
    assert!(!rows.is_empty(), "FR-020 compatibility matrix has rows");
    rows
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

fn assert_refused(output: &Output, code: i32, expected: &str) {
    assert_eq!(
        output.status.code(),
        Some(code),
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

fn remove_insignificant_json_whitespace(bytes: &[u8]) -> Vec<u8> {
    let mut compact = Vec::with_capacity(bytes.len());
    let mut in_string = false;
    let mut escaped = false;
    for &byte in bytes {
        if in_string {
            compact.push(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
            compact.push(byte);
        } else if !byte.is_ascii_whitespace() {
            compact.push(byte);
        }
    }
    compact
}

#[trace("FR-020-AC-1")]
#[test]
fn it_136_complete_fixture_exercises_every_assurance_record_family() {
    let fixture = fixture(true);
    let (_, value) = success(&fixture);

    assert_eq!(value["format"], "quire-assurance");
    assert_eq!(value["format_version"], 1);
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

#[trace("FR-020-AC-1")]
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

#[trace("FR-020-AC-2")]
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
        remove_insignificant_json_whitespace(&pretty_one.stdout),
        remove_insignificant_json_whitespace(&first.stdout),
        "pretty output must preserve every non-whitespace byte and key order"
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&pretty_one.stdout).expect("pretty JSON"),
        serde_json::from_slice::<Value>(&first.stdout).expect("compact JSON")
    );
}

#[trace("FR-020-AC-3", "FR-020-CON-4")]
#[test]
fn it_139_every_module_or_schema_premise_drift_is_refused_atomically() {
    let fixture = fixture(false);
    let cases: Vec<(&str, Vec<&str>, i32, &str)> = vec![
        (
            "other@1.2.3",
            SCHEMAS.to_vec(),
            3,
            "module 'assurance-fixture'",
        ),
        (
            "assurance-fixture@9.9.9",
            SCHEMAS.to_vec(),
            2,
            "version '1.2.3'",
        ),
        (
            MODULE,
            vec![SCHEMAS[0], SCHEMAS[1], SCHEMAS[2], "assurance-fixture/US"],
            2,
            "does not exactly match",
        ),
    ];
    for (expected_module, schemas, code, message) in cases {
        let mut command = quire();
        command
            .arg("assurance")
            .arg("--scope")
            .arg(&fixture.scope)
            .arg("--module")
            .arg(&fixture.module)
            .arg("--expect-module")
            .arg(expected_module);
        for schema in schemas {
            command.arg("--expect-schema").arg(schema);
        }
        assert_refused(&command.output().expect("refusal run"), code, message);
    }
}

#[trace("FR-020-AC-3", "FR-020-CON-4")]
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
            "--expect-module",
            "no-at-sign",
        ])
        .output()
        .expect("malformed run");
    assert_eq!(malformed.status.code(), Some(3));
    assert!(malformed.stdout.is_empty());

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
        assert_refused(&output(&fixture), 3, expected);
    }
}

#[trace("FR-020-AC-4")]
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
    assert_refused(&output(&missing), 3, "no document root");

    let no_model = fixture(false);
    let manifest_path = no_model.module.join("manifest.yaml");
    let manifest = fs::read_to_string(&manifest_path).expect("manifest");
    let manifest = manifest
        .split_once("traceability:\n")
        .map(|(prefix, _)| prefix)
        .expect("fixture traceability section");
    write(&manifest_path, manifest);
    let (_, value) = success(&no_model);
    assert!(!value["artifacts"].as_array().expect("artifacts").is_empty());
    assert!(!value["symbols"].as_array().expect("symbols").is_empty());
    assert!(value["obligations"]
        .as_array()
        .expect("obligations")
        .is_empty());
    assert!(value["relations"]
        .as_array()
        .expect("relations")
        .iter()
        .all(|relation| relation["kind"] != "verifies" && relation["kind"] != "implements"));
}

#[trace("FR-020-AC-5")]
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
