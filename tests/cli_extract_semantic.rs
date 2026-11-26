//! Opt-in semantic extraction through the CLI process boundary.

mod common;

use std::path::PathBuf;
use std::process::Output;

use ix_trace_rs::trace;
use serde_json::{json, Value};

struct Fixture {
    _temp: tempfile::TempDir,
    source: PathBuf,
    roles: PathBuf,
    modules: Vec<PathBuf>,
    document: PathBuf,
}

impl Fixture {
    fn new(receiver: &str) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        let roles = temp.path().join("roles");
        let module = temp.path().join("workflow-module");
        let provider = temp.path().join("role-module");
        for dir in [
            source.join("spec"),
            roles.join("spec"),
            module.join("schemas"),
            provider.clone(),
        ] {
            std::fs::create_dir_all(dir).unwrap();
        }
        std::fs::write(
            module.join("manifest.yaml"),
            r#"manifest_version: 1.0.0
name: workflow-module
version: 0.1.0
semantic:
  contract_version: 1.0.0
  package: example/module-owner
  exports: [process]
  imports: {example/roles: 0.1.0}
  import_types: {example/roles: [RoleSpec, principal]}
  mappings: [record-sections]
  targets: [json-schema, markdown]
edge_types:
  receiver: {description: Receiving role, category: behavioral}
object_types:
- name: process
  allowed_links:
    receiver: [RoleSpec]
  data_schema:
    schema: schemas/Process.json
  body_extraction:
    yield_pattern:
      match:
        steps:
          from: table_row
          under_section: Steps
          required: false
          assert:
            columns: [Step, Kind, Receiver, Description]
            optional_columns: [Receiver, Description]
"#,
        )
        .unwrap();
        std::fs::write(
            provider.join("manifest.yaml"),
            r#"manifest_version: 1.0.0
name: role-module
version: 0.1.0
semantic:
  contract_version: 1.0.0
  package: example/roles
  exports: [RoleSpec, principal]
  mappings: []
  targets: [json-schema, markdown]
artifact_types:
- name: RoleSpec
  body_extraction:
    yield_pattern:
      match:
        id: {from: frontmatter_field, path: [id]}
object_types:
- name: principal
  body_extraction:
    yield_pattern:
      match:
        id: {from: frontmatter_field, path: [id]}
"#,
        )
        .unwrap();
        std::fs::write(module.join("schemas/Process.json"), json!({
            "$schema":"https://json-schema.org/draft/2020-12/schema",
            "$id":"https://example.test/Process.json", "type":"object",
            "properties":{"steps":{"type":"array","items":{"type":"object",
                "properties":{
                    "name":{"type":"string"}, "kind":{"type":"string"},
                    "doc":{"type":"string"},
                    "receiver":{"$ref":"https://schemas.agent-ix.org/semantic-core/0.3.3/SemanticId.json"}
                }
            }}}
        }).to_string()).unwrap();
        std::fs::write(
            roles.join("spec/ROLE.md"),
            "---\nid: ROLE\ntype: RoleSpec\n---\n# Receiving role\n",
        )
        .unwrap();
        std::fs::write(
            roles.join("spec/PRINCIPAL.md"),
            "---\nid: PRINCIPAL\ntype: principal\nobject: principal\n---\n# Principal\n",
        )
        .unwrap();
        let document = source.join("spec/HANDOFF.md");
        std::fs::write(&document, format!("---\nid: HANDOFF\ntype: process\nobject: process\n---\n# Handoff\n\n## Steps\n\n| Step | Kind | Receiver | Description |\n|---|---|---|---|\n| accept_request | command | {receiver} | Receive the handoff |\n")).unwrap();
        Self {
            _temp: temp,
            source,
            roles,
            modules: vec![module, provider],
            document,
        }
    }

    fn command(&self, semantic: bool) -> std::process::Command {
        let mut command = common::quire();
        command.arg("extract").arg(&self.document);
        for module in &self.modules {
            command.arg("--module").arg(module);
        }
        if semantic {
            command
                .arg("--semantic")
                .arg("--repo")
                .arg(format!("example/workflows={}", self.source.display()))
                .arg("--repo")
                .arg(format!("example/roles={}", self.roles.display()));
        }
        command
    }

    fn output(&self) -> Output {
        self.command(true).output().unwrap()
    }
}

#[test]
#[trace("FR-003-AC-7")]
fn semantic_receiver_is_structured_and_keeps_the_source_member_identity() {
    let fixture = Fixture::new("ix://example/roles/ROLE");
    let output = fixture.output();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        body["semantic"]["record"]["steps"][0]["receiver"],
        "ix://example/roles/ROLE"
    );
    let relation = &body["semantic"]["recordRelations"][0];
    assert_eq!(relation["verb"], "receiver");
    assert_eq!(relation["target"], "ix://example/roles/ROLE");
    assert_eq!(relation["validationPath"], "/record/steps/0/receiver");
    assert_eq!(
        relation["sourceSpan"]["sourceIdentity"],
        "ix://example/workflows/spec"
    );
    assert_eq!(relation["sourceSpan"]["path"], "spec/HANDOFF.md");
}

#[test]
#[trace("FR-003-AC-8")]
fn semantic_receiver_wrong_kind_and_missing_target_are_engine_refusals() {
    for (target, code) in [
        ("PRINCIPAL", "semantic.reference-kind-mismatch"),
        ("MISSING", "semantic.invalid-model-cell"),
    ] {
        let fixture = Fixture::new(&format!("ix://example/roles/{target}"));
        let output = fixture.output();
        assert_eq!(
            output.status.code(),
            Some(3),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
#[trace("FR-003-AC-8")]
fn semantic_empty_optional_receiver_is_absent_without_a_relation() {
    let fixture = Fixture::new("");
    let output = fixture.output();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(body["semantic"]["record"]["steps"][0]
        .get("receiver")
        .is_none());
    assert!(body["semantic"].get("recordRelations").is_none());
}

#[test]
#[trace("FR-003-AC-10")]
fn default_extract_wire_has_no_semantic_member_and_keeps_raw_steps() {
    let fixture = Fixture::new("ix://example/roles/ROLE");
    let output = fixture.command(false).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(body.get("semantic").is_none());
    assert!(body["extraction"]["records"][0]["steps"].is_string());
    assert_eq!(body.as_object().unwrap().len(), 3);
    assert!(body.get("edges").is_some() && body.get("engine").is_some());
}

#[test]
#[trace("FR-003-AC-9")]
fn semantic_document_must_belong_to_one_declared_member() {
    let fixture = Fixture::new("ix://example/roles/ROLE");
    let outside = fixture._temp.path().join("outside.md");
    std::fs::copy(&fixture.document, &outside).unwrap();
    let mut command = common::quire();
    command
        .arg("extract")
        .arg(outside)
        .arg("--semantic")
        .arg("--module")
        .arg(&fixture.modules[0])
        .arg("--repo")
        .arg(format!("example/workflows={}", fixture.source.display()));
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("exactly one"));
}

#[test]
#[trace("FR-003-AC-9")]
fn semantic_stdin_ambiguous_members_and_repo_without_mode_are_refused() {
    let fixture = Fixture::new("ix://example/roles/ROLE");
    let mut stdin = common::quire();
    stdin
        .arg("extract")
        .arg("-")
        .arg("--semantic")
        .arg("--module")
        .arg(&fixture.modules[0])
        .arg("--repo")
        .arg(format!("example/workflows={}", fixture.source.display()));
    let output = stdin.output().unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires a file"));

    let output = fixture
        .command(true)
        .arg("--repo")
        .arg(format!("example/alias={}", fixture.source.display()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("exactly one"));

    let output = fixture
        .command(false)
        .arg("--repo")
        .arg(format!("example/workflows={}", fixture.source.display()))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--semantic"));
}
