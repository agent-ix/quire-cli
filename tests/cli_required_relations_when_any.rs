//! `validate --okf` over a module whose required relation is narrowed by the
//! quire-rs `when_any` selector. The CLI only loads the module and surfaces
//! the engine's findings; scope is decided by quire-rs.

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::prelude::*;
use ix_trace_rs::trace;
use predicates::prelude::*;
use tempfile::TempDir;

use common::quire;

const CHECK: &str = "[control-without-requirement]";

fn when_any_module() -> PathBuf {
    common::fixture_root().join("when-any-mod")
}

fn write_doc(root: &Path, id: &str, kind: &str, edges: &[(&str, &str)]) {
    let rows: String = edges
        .iter()
        .map(|(verb, target)| format!("- target: {target}\n  type: {verb}\n"))
        .collect();
    let relationships = if rows.is_empty() {
        String::new()
    } else {
        format!("relationships:\n{rows}")
    };
    let body = format!("---\nid: {id}\ntype: {kind}\n{relationships}---\n# {id}\nbody\n");
    fs::write(root.join(format!("{id}.md")), body).expect("write doc");
}

/// C-1 and C-4 mitigate a hazard and nothing satisfies them (in scope,
/// reported); C-2 mitigates nothing (out of scope); C-3 mitigates a hazard and
/// R-1 satisfies it (in scope, satisfied).
fn bundle() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let root = dir.path();
    write_doc(root, "HZ-1", "hazard", &[]);
    write_doc(root, "C-1", "control", &[("mitigates", "HZ-1")]);
    write_doc(root, "C-2", "control", &[]);
    write_doc(root, "C-3", "control", &[("mitigates", "HZ-1")]);
    write_doc(root, "C-4", "control", &[("mitigates", "HZ-1")]);
    write_doc(root, "R-1", "requirement", &[("satisfies", "C-3")]);
    dir
}

fn okf(dir: &TempDir, module: &Path) -> assert_cmd::assert::Assert {
    quire()
        .arg("validate")
        .arg("--okf")
        .arg(dir.path())
        .arg("--module")
        .arg(module)
        .assert()
}

#[trace("FR-014-AC-17")]
#[test]
fn when_any_reports_each_in_scope_document_lacking_the_edge() {
    let dir = bundle();
    let output = okf(&dir, &when_any_module()).success().get_output().clone();
    let stderr = String::from_utf8(output.stderr).expect("utf8 stderr");
    let reported: Vec<&str> = stderr.lines().filter(|l| l.ends_with(CHECK)).collect();
    assert_eq!(reported.len(), 2, "{stderr}");
    for (line, id) in reported.iter().zip(["C-1", "C-4"]) {
        assert!(line.starts_with("warning: "), "{stderr}");
        assert!(
            line.contains(&format!("{id}.md: nothing reaches '{id}'")),
            "{stderr}"
        );
    }
    for absent in ["'C-2'", "'C-3'"] {
        assert!(!stderr.contains(absent), "{absent} reported: {stderr}");
    }
}

#[trace("FR-014-AC-17")]
#[test]
fn a_refused_when_any_exits_3_naming_the_relation() {
    let module = TempDir::new().expect("tempdir");
    let source = when_any_module();
    fs::create_dir(module.path().join("schemas")).expect("mkdir");
    for entry in fs::read_dir(source.join("schemas")).expect("schemas") {
        let entry = entry.expect("entry");
        fs::copy(
            entry.path(),
            module.path().join("schemas").join(entry.file_name()),
        )
        .expect("copy schema");
    }
    let manifest = fs::read_to_string(source.join("manifest.yaml")).expect("manifest");
    let broken = manifest.replace("direction: outgoing, other", "direction: sideways, other");
    assert_ne!(broken, manifest, "fixture no longer carries the condition");
    fs::write(module.path().join("manifest.yaml"), broken).expect("write manifest");

    okf(&bundle(), module.path())
        .failure()
        .code(3)
        .stderr(predicate::str::contains(
            "required_relations entry 'control-has-requirement' has an invalid `when_any`",
        ));
}
