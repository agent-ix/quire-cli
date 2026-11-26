//! Process-boundary tests for `quire impact` (FR-049).

mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use ix_trace_rs::trace;
use serde_json::Value;
use tempfile::TempDir;

use common::quire;

struct Repo {
    dir: TempDir,
    base: String,
    head: String,
}

fn repository(with_python_change: bool) -> Repo {
    let dir = TempDir::new().expect("temporary repository");
    let root = dir.path();
    write(root, "manifest.yaml", module_manifest());
    write(
        root,
        "spec/FR-900-AC-1.md",
        "---\nid: FR-900-AC-1\ntitle: Impact fixture\n---\n",
    );
    write(
        root,
        "src/lib.rs",
        "#[implements(\"FR-900-AC-1\")]\npub fn target() { let _value = 1; }\n\
         #[implements(\"FR-900-AC-1\")]\npub fn sibling() {}\n",
    );
    write(
        root,
        "tests/impact.rs",
        "#[trace(\"FR-900-AC-1\")]\n#[test]\nfn selected_impact_test() { target(); }\n",
    );
    git(root, &["init", "-q", "-b", "main"]);
    git(root, &["config", "user.name", "Impact test"]);
    git(
        root,
        &["config", "user.email", "impact-test@example.invalid"],
    );
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "base"]);
    let base = git(root, &["rev-parse", "HEAD"]);

    write(
        root,
        "src/lib.rs",
        "#[implements(\"FR-900-AC-1\")]\npub fn target() { let _value = 2; }\n\
         #[implements(\"FR-900-AC-1\")]\npub fn sibling() {}\n",
    );
    if with_python_change {
        write(root, "src/changed.py", "def changed():\n    return 2\n");
    }
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "head"]);
    let head = git(root, &["rev-parse", "HEAD"]);
    Repo { dir, base, head }
}

fn module_manifest() -> &'static str {
    r#"name: impact-fixture
manifest_version: 1.0.0
version: 0.0.1
artifact_types: []
traceability:
  trace_targets: []
  trace_tags:
    markers:
    - name: rust-trace
      language: rust
      pattern: '#\[trace\(([^)]*)\)\]'
      template: '#[trace({ids})]'
    implements:
    - name: rust-implements
      language: rust
      pattern: '#\[implements\(([^)]*)\)\]'
      template: '#[implements({ids})]'
      syntax: rust_attribute
"#
}

fn write(root: &Path, relative: &str, contents: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).expect("create fixture directory");
    fs::write(path, contents).expect("write fixture file");
}

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output is UTF-8")
        .trim()
        .to_owned()
}

fn run(repo: &Repo, format: &str, extra: &[&str]) -> std::process::Output {
    let mut command = quire();
    command
        // Local Git object operations remain available; remote protocols are
        // denied so an accidental fetch makes this process-boundary test fail.
        .env("GIT_ALLOW_PROTOCOL", "file")
        .args([
            "impact",
            "--repo",
            repo.dir.path().to_str().unwrap(),
            "--base",
            &repo.base,
            "--head",
            &repo.head,
            "--caller-depth",
            "2",
            "--format",
            format,
        ])
        .args(extra);
    command.output().expect("run quire impact")
}

#[test]
#[trace("FR-049-AC-2")]
fn it_impact_json_and_human_reports_required_fields_filters_and_reasons() {
    let repo = repository(false);
    let json_run = run(&repo, "json", &[]);
    assert!(
        json_run.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&json_run.stderr)
    );
    let report: Value = serde_json::from_slice(&json_run.stdout).expect("impact JSON");
    for field in [
        "base_revision",
        "head_revision",
        "changed_symbols",
        "unmapped_changes",
        "selected_tests",
        "related_implementations",
        "selection_complete",
        "findings",
        "caller_search",
    ] {
        assert!(report.get(field).is_some(), "missing JSON field {field}");
    }
    assert!(
        report.get("caller_searches").is_none(),
        "the public key is caller_search"
    );
    let selected = report["selected_tests"].as_array().unwrap();
    let test = selected
        .iter()
        .find(|test| test["filter"] == "selected_impact_test")
        .expect("bound test filter");
    assert!(test["reasons"].to_string().contains("implemented_id"));
    assert!(report["related_implementations"]
        .to_string()
        .contains("sibling"));

    let human_run = run(&repo, "human", &[]);
    assert!(human_run.status.success());
    let human = String::from_utf8_lossy(&human_run.stdout);
    assert!(human.contains("selected_impact_test"), "{human}");
    assert!(human.contains("reasons="), "{human}");
    assert!(human.contains("Related implementation"), "{human}");
}

#[test]
#[trace("FR-049-AC-3")]
#[trace("FR-049-AC-4")]
fn it_impact_reports_unsupported_source_and_incomplete_status() {
    let repo = repository(true);
    let output = run(&repo, "json", &[]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("impact JSON");
    assert_eq!(report["selection_complete"], false);
    let findings = report["findings"].to_string();
    assert!(findings.contains("unsupported_source"), "{findings}");
    assert!(findings.contains("src/changed.py"), "{findings}");
    assert!(!report.to_string().contains("confidence"), "{report}");

    let human = run(&repo, "human", &[]);
    assert!(human.status.success());
    let human = String::from_utf8_lossy(&human.stdout);
    assert!(human.contains("Selection complete: false"), "{human}");
    assert!(human.contains("UnsupportedSource"), "{human}");
}

#[test]
#[trace("FR-049-AC-7")]
fn it_impact_json_is_byte_identical_for_identical_inputs() {
    let repo = repository(false);
    let first = run(&repo, "json", &[]);
    let second = run(&repo, "json", &[]);
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
#[trace("FR-049-CON-4")]
fn it_impact_does_not_modify_the_analyzed_repository() {
    let repo = repository(false);
    let root = repo.dir.path();
    git(
        root,
        &[
            "remote",
            "add",
            "origin",
            "https://network-disabled.invalid/quire-impact-fixture.git",
        ],
    );
    let before_head = git(root, &["rev-parse", "HEAD"]);
    let before_status = git(root, &["status", "--porcelain"]);
    let output = run(&repo, "json", &[]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(git(root, &["rev-parse", "HEAD"]), before_head);
    assert_eq!(git(root, &["status", "--porcelain"]), before_status);
}
