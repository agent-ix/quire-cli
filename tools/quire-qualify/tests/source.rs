use std::fs;
use std::path::Path;

use ix_trace_rs::trace;

fn write(path: &Path, value: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, value).unwrap();
}

fn minimum_repo(root: &Path) {
    write(&root.join("src/main.rs"), "fn main() {}\n");
    write(&root.join("src/lib.rs"), "pub fn clean() {}\n");
    write(&root.join("src/self_update/mod.rs"), "pub fn update() {}\n");
    write(&root.join("src/commands/update.rs"), "pub fn run() {}\n");
    write(
        &root.join("src/commands/assurance.rs"),
        r#"
use quire_rs::{build_assurance_export, read_assurance_export, Spec};
fn run(value: Value) {
    Spec::from_path(".");
    quire_rs::symbols::extract_tree_scoped();
    quire_rs::symbols::trace::bind();
    build_assurance_export();
    read_assurance_export();
    value.to_json_bytes();
}
"#,
    );
    write(
        &root.join("CONTRIBUTING.md"),
        "- Does any new logic belong upstream in `quire-rs`?\n",
    );
}

#[trace("TC-835", "FR-025-AC-4", "FR-025-CON-1")]
#[test]
fn rust_ast_catches_direct_aliased_and_nested_group_references() {
    let dir = tempfile::tempdir().unwrap();
    minimum_repo(dir.path());
    quire_qualify::source::check_thin_boundary(dir.path()).expect("clean fixture");

    for mutation in [
        "fn f() { quire_rs::parse_document(); }\n",
        "use quire_rs::parse_document as parse; fn f() { parse(); }\n",
        "use quire_rs::{render::{self as renderer}}; fn f() { renderer(); }\n",
        "use quire_rs::{self as qr}; fn f() { qr::validate_all(); }\n",
        "extern crate quire_rs as qr; fn f() { qr::extract(); }\n",
        "fn f() { wrapper!(quire_rs::harvest_edges()); }\n",
    ] {
        write(&dir.path().join("src/lib.rs"), mutation);
        let findings = quire_qualify::source::thin_boundary_findings(dir.path()).unwrap();
        assert!(
            findings.iter().any(|finding| {
                finding.path == Path::new("src/lib.rs")
                    && finding.line > 0
                    && finding.reason.contains("forbidden engine reference")
            }),
            "mutation escaped AST audit: {mutation}\n{findings:#?}"
        );
    }
}

#[trace("TC-836", "FR-025-AC-4")]
#[test]
fn admitted_dispatch_passes_and_assurance_boundary_mutations_name_loci() {
    let dir = tempfile::tempdir().unwrap();
    minimum_repo(dir.path());
    write(
        &dir.path().join("src/commands/parse.rs"),
        "pub fn run() { quire_rs::parse_document(); }\n",
    );
    quire_qualify::source::check_thin_boundary(dir.path()).expect("admitted dispatch");

    let assurance = fs::read_to_string(dir.path().join("src/commands/assurance.rs")).unwrap();
    write(
        &dir.path().join("src/commands/assurance.rs"),
        &format!("struct AssuranceExport;\n{assurance}"),
    );
    let findings = quire_qualify::source::thin_boundary_findings(dir.path()).unwrap();
    assert!(findings.iter().any(|finding| {
        finding.path == Path::new("src/commands/assurance.rs")
            && finding.line == 1
            && finding.reason.contains("CLI-owned assurance model")
    }));

    write(
        &dir.path().join("src/commands/assurance.rs"),
        &format!("use std::{{process::Command as Runner}};\n{assurance}\nfn spawn() {{ Runner::new(\"x\"); }}"),
    );
    let findings = quire_qualify::source::thin_boundary_findings(dir.path()).unwrap();
    assert!(findings.iter().any(|finding| {
        finding.path == Path::new("src/commands/assurance.rs")
            && finding.line > 0
            && finding.reason.contains("std::process::Command")
    }));
}

#[cfg(unix)]
#[trace("TC-836", "FR-025-AC-4")]
#[test]
fn symlinked_rust_source_is_not_an_audit_escape() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().unwrap();
    minimum_repo(dir.path());
    let outside = tempfile::NamedTempFile::new().unwrap();
    fs::write(
        outside.path(),
        "fn hidden() { quire_rs::validate_all(); }\n",
    )
    .unwrap();
    symlink(outside.path(), dir.path().join("src/hidden.rs")).unwrap();
    assert!(matches!(
        quire_qualify::source::thin_boundary_findings(dir.path()),
        Err(quire_qualify::Error::SymlinkSource { .. })
    ));
}
