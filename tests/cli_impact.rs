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

fn repository(with_non_rust_change: bool) -> Repo {
    repository_with_dependency_revisions(with_non_rust_change, "old-pin", "new-pin")
}

fn repository_with_dependency_revisions(
    with_non_rust_change: bool,
    old_revision: &str,
    new_revision: &str,
) -> Repo {
    let dir = TempDir::new().expect("temporary repository");
    let root = dir.path();
    write(root, "manifest.yaml", module_manifest());
    write(
        root,
        "spec/FR-900.md",
        "---\nid: FR-900\ntitle: Impact fixture\n---\n\n## Acceptance Criteria\n\n| ID | Criterion |\n| --- | --- |\n| FR-900-AC-1 | The fixture criterion is available. |\n",
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
    write(root, "Cargo.lock", &cargo_lock(old_revision));
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
    if with_non_rust_change {
        write(
            root,
            "src/Changed.java",
            "final class Changed { int value() { return 2; } }\n",
        );
    }
    write(root, "Cargo.lock", &cargo_lock(new_revision));
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "head"]);
    let head = git(root, &["rev-parse", "HEAD"]);
    Repo { dir, base, head }
}

fn cargo_lock(revision: &str) -> String {
    format!(
        "[[package]]\nname = \"quire-rs\"\nversion = \"0.63.2\"\nsource = \"git+https://github.com/agent-ix/quire-rs?rev={revision}#{revision}\"\n"
    )
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
    assert!(findings.contains("src/Changed.java"), "{findings}");
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

#[test]
#[trace("FR-049-AC-5")]
fn it_impact_tag_findings_are_opt_in_and_fail_only_when_requested() {
    let mut repo = repository(false);
    let clean = run(&repo, "json", &["--check-tags", "--fail-on-tag-findings"]);
    assert!(
        clean.status.success(),
        "ordinary requirement documents expose criterion IDs: {}",
        String::from_utf8_lossy(&clean.stderr)
    );
    let clean_report: Value = serde_json::from_slice(&clean.stdout).expect("impact JSON");
    assert!(clean_report["findings"].as_array().unwrap().is_empty());

    let root = repo.dir.path();
    write(
        root,
        "src/lib.rs",
        "pub fn target() { let _value = 3; }\n\
         pub fn missing_link() { let _value = 4; }\n\
         #[implements(\"FR-999-AC-1\")]\n\
         pub fn invalid_link() { let _value = 5; }\n\
         #[implements(\"FR-901-AC-1\")]\n\
         pub fn unavailable_link() { let _value = 6; }\n\
         #[trace(\"FR-900-AC-1\")]\n\
         pub fn nonbinding_link() { let _value = 7; }\n",
    );
    write(
        root,
        "spec/FR-901-AC-1.md",
        "This document has no frontmatter.\n",
    );
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "tag findings"]);
    repo.head = git(root, &["rev-parse", "HEAD"]);

    let default = run(&repo, "json", &[]);
    assert!(default.status.success());
    let default_report: Value = serde_json::from_slice(&default.stdout).expect("impact JSON");
    assert!(default_report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|finding| {
            !matches!(
                finding["kind"].as_str(),
                Some(
                    "unlinked_symbol"
                        | "unresolved_id"
                        | "unavailable_specification"
                        | "non_binding_marker"
                )
            )
        }));

    let checked = run(&repo, "json", &["--check-tags"]);
    assert!(checked.status.success());
    let checked_report: Value = serde_json::from_slice(&checked.stdout).expect("impact JSON");
    let findings = checked_report["findings"].to_string();
    assert!(findings.contains("unlinked_symbol"), "{findings}");
    assert!(findings.contains("unresolved_id"), "{findings}");
    assert!(findings.contains("unavailable_specification"), "{findings}");
    assert!(findings.contains("non_binding_marker"), "{findings}");

    let failed = run(&repo, "json", &["--check-tags", "--fail-on-tag-findings"]);
    assert!(!failed.status.success());
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(stderr.contains("requirement link findings"), "{stderr}");

    let invalid = run(&repo, "json", &["--fail-on-tag-findings"]);
    assert!(!invalid.status.success());
}

#[test]
#[trace("FR-049-AC-6")]
fn it_impact_reports_dependency_move_chain_and_unavailable_upstream() {
    let upstream = TempDir::new().expect("temporary upstream repository");
    let upstream_root = upstream.path();
    write(
        upstream_root,
        "Cargo.toml",
        "[package]\nname = \"quire-rs\"\nversion = \"0.63.2\"\nedition = \"2024\"\n",
    );
    write(
        upstream_root,
        "src/lib.rs",
        "pub fn changed_api() { let _value = 1; }\npub fn unrelated_api() { let _value = 1; }\n",
    );
    write(
        upstream_root,
        "spec/FR-940.md",
        "# Upstream API\n\nOld behavior.\n",
    );
    write(upstream_root, "schema/adapter.json", "{\"version\":1}\n");
    git(upstream_root, &["init", "-q", "-b", "main"]);
    git(upstream_root, &["config", "user.name", "Impact test"]);
    git(
        upstream_root,
        &["config", "user.email", "impact-test@example.invalid"],
    );
    git(upstream_root, &["add", "-A"]);
    git(upstream_root, &["commit", "-q", "-m", "upstream base"]);
    let old_revision = git(upstream_root, &["rev-parse", "HEAD"]);
    write(
        upstream_root,
        "src/lib.rs",
        "pub fn changed_api() { let _value = 2; }\npub fn unrelated_api() { let _value = 2; }\n",
    );
    write(
        upstream_root,
        "spec/FR-940.md",
        "# Upstream API\n\nNew behavior.\n",
    );
    write(upstream_root, "schema/adapter.json", "{\"version\":2}\n");
    git(upstream_root, &["add", "-A"]);
    git(upstream_root, &["commit", "-q", "-m", "upstream head"]);
    let new_revision = git(upstream_root, &["rev-parse", "HEAD"]);

    let repo = repository_with_dependency_revisions(false, &old_revision, &new_revision);
    let root = repo.dir.path();
    write(
        root,
        "src/lib.rs",
        "#[implements(\"FR-900-AC-1\")]\npub fn adapter() { quire_rs::changed_api(); }\n#[implements(\"FR-900-AC-1\")]\npub fn second_adapter() { quire_rs::changed_api(); }\n",
    );
    write(
        root,
        "tests/impact.rs",
        "#[trace(\"FR-900-AC-1\")]\n#[test]\nfn selected_impact_test() { adapter(); }\n",
    );
    git(root, &["add", "-A"]);
    git(root, &["commit", "-q", "-m", "adapter"]);
    let updated_head = git(root, &["rev-parse", "HEAD"]);
    let updated_repo = Repo {
        dir: repo.dir,
        base: repo.base,
        head: updated_head,
    };

    let output = run(
        &updated_repo,
        "json",
        &[
            "--dependency-move",
            upstream_root.to_str().unwrap(),
            &old_revision,
            &new_revision,
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("impact JSON");
    let movement = &report["dependency_moves"][0];
    assert_eq!(movement["old_revision"], old_revision);
    assert_eq!(movement["new_revision"], new_revision);
    assert!(movement["upstream_changes"]
        .to_string()
        .contains("changed_api"));
    assert!(movement["upstream_changes"]
        .to_string()
        .contains("spec/FR-940.md"));
    assert!(movement["upstream_changes"]
        .to_string()
        .contains("schema/adapter.json"));
    assert!(movement["local_references"].to_string().contains("adapter"));
    assert!(movement["local_references"]
        .to_string()
        .contains("second_adapter"));
    assert!(movement["selected_tests"]
        .to_string()
        .contains("selected_impact_test"));
    assert!(movement["uncovered"].to_string().contains("unrelated_api"));

    let human = run(
        &updated_repo,
        "human",
        &[
            "--dependency-move",
            upstream_root.to_str().unwrap(),
            &old_revision,
            &new_revision,
        ],
    );
    assert!(human.status.success());
    let human = String::from_utf8_lossy(&human.stdout);
    assert!(human.contains("Dependency move"), "{human}");
    assert!(human.contains("changed_api"), "{human}");
    assert!(human.contains("spec/FR-940.md"), "{human}");
    assert!(human.contains("schema/adapter.json"), "{human}");
    assert!(human.contains("upstream_path="), "{human}");
    assert!(human.contains("upstream_item="), "{human}");
    assert!(human.contains("adapter"), "{human}");
    assert!(human.contains("second_adapter"), "{human}");
    assert!(human.contains("call_path="), "{human}");
    assert!(human.contains("Uncovered upstream change"), "{human}");

    let unavailable = run(
        &updated_repo,
        "json",
        &[
            "--dependency-move",
            upstream_root.to_str().unwrap(),
            "not-a-revision",
            &new_revision,
        ],
    );
    assert!(unavailable.status.success());
    let report: Value = serde_json::from_slice(&unavailable.stdout).expect("impact JSON");
    let movement = &report["dependency_moves"][0];
    assert_eq!(movement["unresolved"], true);
    assert!(movement["unavailable_upstream"]
        .to_string()
        .contains("base"));
    assert!(movement["upstream_changes"].as_array().unwrap().is_empty());
}
