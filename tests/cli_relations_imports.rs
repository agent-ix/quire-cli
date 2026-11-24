//! Explicit relation modules load their declared semantic import providers.
mod common;

use std::fs;
use ix_trace_rs::trace;
use serde_json::Value;
use tempfile::TempDir;

fn provider_manifest(name: &str, version: &str, package: &str, edge: &str) -> String {
    format!(
        "name: {name}\nversion: {version}\nartifact_types: []\nsemantic:\n  contract_version: 1.0.0\n  package: {package}\n  exports: []\n  imports: {{}}\n  targets: [json-schema, markdown]\n  mappings: []\n  compatibility_posture: additive\n  legacy_forms: warning\nedge_types:\n  {edge}: {{description: '{edge} edge', category: traceability}}\n"
    )
}

fn write_module(root: &std::path::Path, name: &str, manifest: &str) {
    let module = root.join(name);
    fs::create_dir_all(&module).expect("module directory");
    fs::write(module.join("manifest.yaml"), manifest).expect("module manifest");
}

#[trace("FR-030-AC-27")]
#[test]
fn explicit_security_module_loads_imported_edge_vocabulary_providers() {
    let temp = TempDir::new().expect("temporary directory");
    let modules = temp.path().join("modules");
    let installed = temp.path().join("installed");
    let scope = temp.path().join("repo");
    fs::create_dir_all(&installed).expect("installed module root");
    fs::create_dir_all(scope.join("spec")).expect("document root");

    let security = modules.join("security");
    fs::create_dir_all(&security).expect("security module root");
    fs::write(
        security.join("manifest.yaml"),
        r#"name: spec-objects-security
version: 0.2.0
artifact_types:
  - name: SecurityItem
semantic:
  contract_version: 1.0.0
  package: agent-ix/spec-objects-security
  exports: []
  imports:
    agent-ix/spec-artifacts-process: 0.3.0
    agent-ix/spec-objects-architecture: 0.5.0
    agent-ix/spec-artifacts-iso: 0.3.0
  targets: [json-schema, markdown]
  mappings: []
  compatibility_posture: additive
  legacy_forms: warning
traceability:
  required_relations:
    - name: uses-process-edge
      from: SecurityItem
      edges: [process_edge]
      to: [SecurityItem]
      direction: outgoing
      check: process-edge
    - name: uses-architecture-edge
      from: SecurityItem
      edges: [architecture_edge]
      to: [SecurityItem]
      direction: outgoing
      check: architecture-edge
    - name: uses-iso-edge
      from: SecurityItem
      edges: [iso_edge]
      to: [SecurityItem]
      direction: outgoing
      check: iso-edge
"#,
    )
    .expect("security manifest");

    write_module(
        &installed,
        "process",
        &provider_manifest(
            "spec-artifacts-process",
            "0.3.0",
            "agent-ix/spec-artifacts-process",
            "process_edge",
        ),
    );
    write_module(
        &installed,
        "architecture",
        &provider_manifest(
            "spec-objects-architecture",
            "0.5.0",
            "agent-ix/spec-objects-architecture",
            "architecture_edge",
        ),
    );
    write_module(
        &installed,
        "iso",
        &provider_manifest(
            "spec-artifacts-iso",
            "0.3.0",
            "agent-ix/spec-artifacts-iso",
            "iso_edge",
        ),
    );

    let mut command = common::quire();
    let output = command
        .args([
            "relations",
            "--scope",
            scope.to_str().expect("scope path"),
            "--module",
            security.to_str().expect("security path"),
            "--json",
        ])
        .env("IX_HOME", temp.path().join("ix-home"))
        .env("IX_FILAMENT_MODULES_PATH", &installed)
        .env_remove("IX_SCHEMA_PATH")
        .output()
        .expect("run quire relations");
    assert!(
        output.status.success(),
        "status: {:?}\nstderr: {}\nstdout: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("JSON relation report");
    let names: Vec<&str> = report["relations"]
        .as_array()
        .expect("relation sections")
        .iter()
        .map(|section| section["name"].as_str().expect("relation name"))
        .collect();
    assert_eq!(
        names,
        ["uses-process-edge", "uses-architecture-edge", "uses-iso-edge"]
    );
    assert!(
        !String::from_utf8_lossy(&output.stderr).contains("semantic.import-unresolved"),
        "all exact semantic import providers loaded: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
