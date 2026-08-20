//! The consumer half of quire-rs FR-055: the `properties --json` **envelope**
//! is assembled here, so its conformance test lives here.
//!
//! quire-rs publishes both output schemas and gates the parts it emits — the
//! coverage payload and the individual criterion records. It cannot gate this
//! envelope, because it never constructs one. Without this test the published
//! `properties-v1.schema.json` would describe a shape nothing checked, which is
//! the exact gap FR-055 exists to close, moved one repository along.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use jsonschema::JSONSchema;
use serde_json::Value;
use tempfile::TempDir;

fn quire() -> Command {
    Command::new(env!("CARGO_BIN_EXE_quire"))
}

/// The published schema, read out of the pinned quire-rs checkout that Cargo
/// resolved. Reading it from the dependency source rather than vendoring a copy
/// is the point: a copy is a second artifact that drifts.
fn properties_schema() -> Value {
    let out = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata");
    let meta: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "cargo metadata: {e}\nstderr: {}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    let path = meta["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .find(|p| p["name"] == "quire-rs")
        .and_then(|p| p["manifest_path"].as_str())
        .map(|m| {
            PathBuf::from(m)
                .parent()
                .expect("crate root")
                .join("schemas/output/properties-v1.schema.json")
        })
        .expect("quire-rs in the dependency graph");
    serde_json::from_str(
        &fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("reading published schema {}: {e}", path.display())),
    )
    .expect("schema is valid JSON")
}

fn compile(schema: &Value) -> JSONSchema {
    JSONSchema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .compile(schema)
        .expect("published schema compiles")
}

fn module(dir: &TempDir) -> String {
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n    id_column: ID\n\
         \x20 obligations:\n  - name: acceptance-criterion\n\
         \x20   target: acceptance-criterion\n    statement_column: Criteria\n\
         \x20   method_column: Verification\n",
    )
    .expect("write manifest");
    m.to_string_lossy().into_owned()
}

fn doc(dir: &TempDir) -> String {
    let p = dir.path().join("FR-001.md");
    fs::write(
        &p,
        "---\nid: FR-001\ntype: FR\n---\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | Every finding absent from the merged map defaults to warning. | Test (TC-001) |\n\
         | FR-001-AC-2 | Logging in with a magic link lands on the dashboard. | Demonstration |\n",
    )
    .expect("write doc");
    p.to_string_lossy().into_owned()
}

/// IT-103, FR-018-AC-8: the emitted envelope conforms to the published
/// contract — including the FR-053 obligation nested on each record. Upstream
/// this is quire-rs FR-055-AC-4, cited in prose because quire-cli's matrix does
/// not declare quire-rs criteria; binding it here would mint a dead tag.
#[test]
fn it_103_properties_envelope_conforms_to_the_published_schema() {
    let dir = TempDir::new().expect("tempdir");
    let (d, m) = (doc(&dir), module(&dir));

    let out = quire()
        .args(["properties", &d, "--module", &m, "--json"])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let payload: Value =
        serde_json::from_slice(&out.stdout).expect("the emitted payload is valid JSON");
    let schema_doc = properties_schema();
    let schema = compile(&schema_doc);

    if let Err(errors) = schema.validate(&payload) {
        let listed: Vec<String> = errors
            .map(|e| format!("{}: {e}", e.instance_path))
            .collect();
        panic!(
            "the emitted `properties --json` envelope violates the published contract:\n{listed:#?}\n\
             payload:\n{}",
            serde_json::to_string_pretty(&payload).unwrap_or_default()
        );
    }

    // Non-vacuous: the fixture must actually have exercised the obligation
    // branch, or this test would pass over a payload with nothing in it.
    let criteria = payload["documents"][0]["criteria"]
        .as_array()
        .expect("criteria array");
    assert_eq!(criteria.len(), 2, "{criteria:#?}");
    assert!(
        criteria
            .iter()
            .any(|c| c["obligation"]["method"].as_str() == Some("Test")),
        "the fixture did not exercise the obligation branch: {criteria:#?}",
    );
}

/// IT-104, FR-018-AC-9: a module declaring no obligation source emits
/// `obligation: null` and still conforms — the shape a corpus that has not
/// adopted obligations sees.
#[test]
fn it_104_absent_obligation_is_null_and_still_conforms() {
    let dir = TempDir::new().expect("tempdir");
    let m = dir.path().join("plain");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: plain\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n",
    )
    .expect("write manifest");
    let d = doc(&dir);

    let out = quire()
        .args(["properties", &d, "--module", &m.to_string_lossy(), "--json"])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );

    let payload: Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
    let schema_doc = properties_schema();
    compile(&schema_doc)
        .validate(&payload)
        .map_err(|e| e.map(|x| x.to_string()).collect::<Vec<_>>())
        .expect("conforms");

    // Every record, not just the first: the fixture carries two criteria whose
    // `Verification` values differ (`Test` and `Demonstration`), so checking
    // only `[0]` would pass while the other silently carried an obligation.
    let criteria = payload["documents"][0]["criteria"]
        .as_array()
        .expect("criteria array");
    assert_eq!(
        criteria.len(),
        2,
        "the fixture must carry more than one criterion, or `every record` is \
         vacuous: {criteria:#?}",
    );
    assert!(
        criteria.iter().all(|c| c["obligation"].is_null()),
        "a module declaring no obligation source must emit null on every \
         record: {payload:#?}",
    );
}
