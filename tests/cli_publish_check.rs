//! Process-boundary tests for `quire publish-check` (FR-029).

mod common;

use ix_trace_rs::trace;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use quire_rs::clauses::{
    Clause, ClauseForce, ClauseSet, ClauseSetRights, StructureRights, TextRights,
};
use tempfile::TempDir;

use common::quire;

const SENTINEL: &str = "zephyr-sentinel-clause-body";

fn set(id: &str, text: TextRights) -> ClauseSet {
    ClauseSet {
        schema_version: "clause-set-v1".into(),
        authority: "example.invalid".into(),
        id: id.into(),
        title: "Synthetic set".into(),
        version: "1.0.0".into(),
        rights: ClauseSetRights {
            structure: StructureRights::Original,
            text,
            clearance_ref: None,
        },
        source: None,
        classification_dimensions: BTreeMap::new(),
        output_catalog: BTreeMap::new(),
        clauses: vec![Clause {
            id: "C-1".into(),
            force: ClauseForce::Mandatory,
            title: None,
            text: Some(SENTINEL.into()),
            subjects: Vec::new(),
            obligated_actors: Vec::new(),
            approval_roles: Vec::new(),
            styles: BTreeMap::new(),
            applicability: None,
            expected_outputs: Vec::new(),
        }],
        crosswalks: Vec::new(),
    }
}

fn module(dir: &TempDir, sets: &[ClauseSet]) -> String {
    let root = dir.path().join("module");
    fs::create_dir_all(root.join("clauses")).unwrap();
    let mut manifest = String::from("name: synthetic-publish-module\nclause_sets:\n");
    for set in sets {
        let rel = format!("clauses/{}.json", set.id);
        manifest.push_str(&format!("  - {rel}\n"));
        fs::write(root.join(&rel), serde_json::to_vec_pretty(set).unwrap()).unwrap();
    }
    fs::write(root.join("manifest.yaml"), manifest).unwrap();
    root.to_string_lossy().into_owned()
}

fn public_module(dir: &TempDir) -> String {
    module(
        dir,
        &[
            set("alpha", TextRights::Original),
            set("beta", TextRights::Original),
        ],
    )
}

fn restricted_module(dir: &TempDir) -> String {
    module(
        dir,
        &[
            set("alpha", TextRights::Original),
            set("gamma", TextRights::Licensed),
            set("delta", TextRights::Licensed),
        ],
    )
}

fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for entry in walk(root) {
        let rel = entry.strip_prefix(root).unwrap().display().to_string();
        out.push((rel, fs::read(&entry).unwrap()));
    }
    out.sort();
    out
}

fn walk(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files.extend(walk(&path));
        } else {
            files.push(path);
        }
    }
    files
}

#[test]
#[trace("IT-258", "FR-029-AC-1")]
fn publishable_module_exits_zero_with_clean_line() {
    let dir = TempDir::new().unwrap();
    let out = quire()
        .args(["publish-check", "--module", &public_module(&dir)])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "no restricted clause sets\n"
    );
    assert!(
        out.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
#[trace("IT-259", "FR-029-AC-2", "FR-029-AC-4")]
fn restricted_module_exits_two_naming_each_set() {
    let dir = TempDir::new().unwrap();
    let out = quire()
        .args(["publish-check", "--module", &restricted_module(&dir)])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    let delta = stderr
        .lines()
        .position(|l| l == "example.invalid/delta@1.0.0");
    let gamma = stderr
        .lines()
        .position(|l| l == "example.invalid/gamma@1.0.0");
    assert!(delta.is_some() && gamma.is_some(), "stderr: {stderr}");
    assert!(delta < gamma, "engine order: {stderr}");
    assert!(!stderr.contains("example.invalid/alpha"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("no restricted clause sets"));
    assert!(!stderr.contains(SENTINEL));
    assert!(!String::from_utf8_lossy(&out.stdout).contains(SENTINEL));
}

#[test]
#[trace("IT-259", "FR-029-AC-3", "FR-029-AC-4")]
fn json_format_carries_restricted_array_and_provenance() {
    let dir = TempDir::new().unwrap();
    let restricted = quire()
        .args([
            "publish-check",
            "--format",
            "json",
            "--module",
            &restricted_module(&dir),
        ])
        .output()
        .unwrap();
    assert_eq!(restricted.status.code(), Some(2));
    let body: serde_json::Value = serde_json::from_slice(&restricted.stdout).unwrap();
    let entries = body["restricted"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["authority"], "example.invalid");
    assert_eq!(entries[0]["id"], "delta");
    assert_eq!(entries[0]["version"], "1.0.0");
    assert_eq!(entries[0]["text_clauses"], 1);
    assert!(body.get("engine").is_some(), "{body}");
    let stderr = String::from_utf8_lossy(&restricted.stderr);
    assert!(stderr.contains("example.invalid/gamma@1.0.0"));
    assert!(!stderr.contains(SENTINEL));
    assert!(!String::from_utf8_lossy(&restricted.stdout).contains(SENTINEL));

    let clean_dir = TempDir::new().unwrap();
    let clean = quire()
        .args([
            "publish-check",
            "--format",
            "json",
            "--module",
            &public_module(&clean_dir),
        ])
        .output()
        .unwrap();
    assert_eq!(clean.status.code(), Some(0));
    assert!(clean.stderr.is_empty());
    let body: serde_json::Value = serde_json::from_slice(&clean.stdout).unwrap();
    assert_eq!(body["restricted"], serde_json::json!([]));
}

#[test]
#[trace("IT-260", "FR-029-AC-5", "FR-029-AC-6")]
fn path_and_load_failures_never_print_the_clean_verdict() {
    let dir = TempDir::new().unwrap();
    let root = restricted_module(&dir);
    let public_dir = TempDir::new().unwrap();
    let public_root = public_module(&public_dir);
    let before = snapshot(Path::new(&root));
    let public_before = snapshot(Path::new(&public_root));

    // Resolves to the publishable module, so only the path check can refuse it.
    let traversal = quire()
        .args([
            "publish-check",
            "--module",
            &format!("{public_root}/../module"),
        ])
        .output()
        .unwrap();
    assert_eq!(traversal.status.code(), Some(2));

    // Full human and json runs leave the module byte-identical.
    for (module, args) in [
        (&root, &[][..]),
        (&root, &["--format", "json"][..]),
        (&public_root, &[][..]),
        (&public_root, &["--format", "json"][..]),
    ] {
        quire()
            .args(["publish-check", "--module", module])
            .args(args)
            .output()
            .unwrap();
    }

    let missing = quire()
        .args(["publish-check", "--module", &format!("{root}-missing")])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(3));

    for out in [&traversal, &missing] {
        assert!(!String::from_utf8_lossy(&out.stdout).contains("no restricted clause sets"));
    }
    assert_eq!(before, snapshot(Path::new(&root)));
    assert_eq!(public_before, snapshot(Path::new(&public_root)));
}

#[cfg(unix)]
#[test]
#[trace("IT-260", "FR-029-AC-5")]
fn outside_root_symlink_is_refused_and_bad_set_fails_load() {
    let dir = TempDir::new().unwrap();
    let outside = dir.path().join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(
        outside.join("evil.json"),
        serde_json::to_vec(&set("evil", TextRights::Original)).unwrap(),
    )
    .unwrap();
    let root = dir.path().join("module");
    fs::create_dir_all(&root).unwrap();
    std::os::unix::fs::symlink(&outside, root.join("clause_sets")).unwrap();
    fs::write(
        root.join("manifest.yaml"),
        "name: m\nclause_sets:\n  - clause_sets/evil.json\n",
    )
    .unwrap();
    let out = quire()
        .args(["publish-check", "--module", root.to_str().unwrap()])
        .output()
        .unwrap();
    // The engine reports the escape as a strict-load failure, not a typed path
    // refusal, so it exits 3 rather than the 2 FR-029-AC-5 names.
    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("escapes its module"));
    assert!(!String::from_utf8_lossy(&out.stdout).contains("no restricted clause sets"));

    let bad = TempDir::new().unwrap();
    let bad_root = bad.path().join("module");
    fs::create_dir_all(bad_root.join("clauses")).unwrap();
    fs::write(bad_root.join("clauses/x.json"), "{ not json").unwrap();
    fs::write(
        bad_root.join("manifest.yaml"),
        "name: m\nclause_sets:\n  - clauses/x.json\n",
    )
    .unwrap();
    let out = quire()
        .args(["publish-check", "--module", bad_root.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!String::from_utf8_lossy(&out.stdout).contains("no restricted clause sets"));
}

#[test]
#[trace("IT-261", "FR-029-AC-8")]
fn publish_check_source_holds_no_rights_policy() {
    let source = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/commands/publish_check.rs"),
    )
    .unwrap()
    .to_lowercase();
    for token in ["citation_only", "clearance_ref", "rights", "licensed"] {
        assert!(!source.contains(token), "source contains {token}");
    }
}
