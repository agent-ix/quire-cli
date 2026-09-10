use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("quire-cli repository root")
        .to_path_buf()
}

fn rust_source_paths(directory: &Path, paths: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(directory)
        .expect("read Rust source directory")
        .map(Result::unwrap)
        .collect();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if entry.file_type().expect("source file type").is_dir() {
            rust_source_paths(&path, paths);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            paths.push(path);
        }
    }
}

fn audit_launcher(source: &str) -> Vec<String> {
    let mut findings = Vec::new();
    for forbidden in [
        "const SUPPORTED",
        "parse_document",
        "extract_tree",
        "clause_set",
        "temporal",
        "protocol",
        "child_process.exec",
        "eval(",
    ] {
        if source.contains(forbidden) {
            findings.push(format!("launcher contains forbidden token {forbidden:?}"));
        }
    }
    for required in [
        "require(\"node:child_process\")",
        "require(\"node:fs\")",
        "require(\"../package.json\")",
        "Object.prototype.hasOwnProperty.call",
        "spawnSync(bin, process.argv.slice(2), { stdio: \"inherit\" })",
        "process.kill(process.pid, result.signal)",
    ] {
        if !source.contains(required) {
            findings.push(format!(
                "launcher is missing required boundary {required:?}"
            ));
        }
    }
    findings
}

fn audit_release_workflow(workflow: &str) -> Vec<String> {
    let mut findings = Vec::new();
    let trigger = workflow
        .split_once("concurrency:")
        .map_or(workflow, |(prefix, _)| prefix);
    if !trigger.contains("workflow_dispatch:")
        || trigger.contains("\n  push:")
        || trigger.contains("\n  pull_request:")
    {
        findings.push("release trigger is not manual-dispatch-only".to_owned());
    }
    for required in [
        "node-version: \"22.15.0\"",
        "npm install -g npm@11.6.2",
        "cargo run --locked -p quire-dist -- verify-binary",
        "cargo run --locked -p quire-dist -- verify-release",
        "cargo run --locked -p quire-dist -- package-npm",
    ] {
        if !workflow.contains(required) {
            findings.push(format!("release workflow is missing {required:?}"));
        }
    }
    for forbidden in [
        "node npm/build-packages.mjs",
        "node -e",
        "perl -",
        "set_version.sh",
        "npm install -g npm@latest",
    ] {
        if workflow.contains(forbidden) {
            findings.push(format!("release workflow contains {forbidden:?}"));
        }
    }
    if workflow.matches("if: inputs.publish").count() != 2 {
        findings.push("both publication steps must be guarded by inputs.publish".to_owned());
    }
    findings
}

fn javascript_paths(root: &Path) -> Vec<PathBuf> {
    fn visit(root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) {
        let mut entries: Vec<_> = fs::read_dir(directory)
            .expect("read npm tree")
            .map(Result::unwrap)
            .collect();
        entries.sort_by_key(|entry| entry.path());
        for entry in entries {
            let path = entry.path();
            if entry.file_type().expect("npm file type").is_dir() {
                visit(root, &path, paths);
            } else if matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("js" | "mjs" | "cjs")
            ) {
                paths.push(
                    path.strip_prefix(root)
                        .expect("relative npm path")
                        .to_path_buf(),
                );
            }
        }
    }

    let mut paths = Vec::new();
    visit(root, &root.join("npm"), &mut paths);
    paths
}

#[trace("TC-815", "FR-022-AC-7", "NFR-008-AC-1")]
#[test]
fn tc815_launcher_is_the_single_bounded_javascript_host() {
    let root = repo_root();
    assert_eq!(
        javascript_paths(&root),
        vec![PathBuf::from("npm/quire-cli/bin/quire.js")]
    );
    let launcher =
        fs::read_to_string(root.join("npm/quire-cli/bin/quire.js")).expect("read launcher");
    assert_eq!(audit_launcher(&launcher), Vec::<String>::new());

    let mutant = format!("const SUPPORTED = [];\n{launcher}");
    assert!(
        audit_launcher(&mutant)
            .iter()
            .any(|finding| finding.contains("SUPPORTED")),
        "a second launcher catalog escaped"
    );
}

#[trace("TC-819", "FR-024-AC-5", "FR-024-AC-6", "NFR-008-AC-1")]
#[test]
fn tc819_release_workflow_is_manual_native_and_mutation_sensitive() {
    let root = repo_root();
    assert!(!root.join("npm/build-packages.mjs").exists());
    assert!(!root.join("scripts/set_version.sh").exists());
    let path = root.join(".github/workflows/release.yml");
    let production = fs::read_to_string(path).expect("read release workflow");
    assert_eq!(audit_release_workflow(&production), Vec::<String>::new());

    let mutants = [
        production.replacen("workflow_dispatch:", "push:", 1),
        production.replacen(
            "cargo run --locked -p quire-dist -- verify-release",
            "true",
            1,
        ),
        production.replacen(
            "cargo run --locked -p quire-dist -- package-npm",
            "node npm/build-packages.mjs",
            1,
        ),
        production.replacen("npm install -g npm@11.6.2", "npm install -g npm@latest", 1),
        production.replacen("if: inputs.publish", "if: success()", 1),
    ];
    for mutant in mutants {
        assert!(
            !audit_release_workflow(&mutant).is_empty(),
            "release-policy mutant escaped"
        );
    }
}

#[trace("TC-820", "FR-022-AC-7", "NFR-008-AC-2", "NFR-008-AC-3")]
#[test]
fn tc820_distribution_tests_are_traced_and_semantically_contained() {
    let root = repo_root();
    let manifest = fs::read_to_string(root.join("tools/quire-dist/Cargo.toml"))
        .expect("read distribution manifest");
    assert!(!manifest.contains("quire-rs"));

    let mut sources = Vec::new();
    rust_source_paths(&root.join("tools/quire-dist/src"), &mut sources);
    for path in sources {
        let source = fs::read_to_string(&path).expect("read distribution source");
        for forbidden in [
            "quire_rs::",
            "parse_document(",
            "extract_tree(",
            "evaluate_clause",
            "parse_temporal",
            "parse_protocol",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} contains forbidden semantic dependency {forbidden:?}",
                path.display()
            );
        }
    }

    let tests = root.join("tools/quire-dist/tests");
    let mut test_sources = Vec::new();
    rust_source_paths(&tests, &mut test_sources);
    for path in test_sources {
        let source = fs::read_to_string(&path).expect("read distribution test");
        let lines: Vec<_> = source.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if line.trim() != "#[test]" {
                continue;
            }
            let previous_test = lines[..index]
                .iter()
                .rposition(|candidate| candidate.trim() == "#[test]")
                .map_or(0, |previous| previous + 1);
            assert!(
                lines[previous_test..index]
                    .iter()
                    .any(|candidate| candidate.contains("trace(")),
                "{}:{} has an untraced distribution test",
                path.display(),
                index + 1
            );
        }
    }
}

#[trace("TC-821", "NFR-008-AC-4")]
#[trace("NFR-008-AC-5", "NFR-008-AC-6")]
#[test]
fn tc821_qualification_record_is_complete_and_native() {
    let review =
        fs::read_to_string(repo_root().join("reviews/SR-063-npm-distribution-rust-review.md"))
            .expect("read qualification review");
    for required in [
        "exact 1.98.1",
        "CARGO_BUILD_JOBS=2",
        "Node v22.15.0 and npm 10.9.2",
        "Node 22.15.0 and npm 11.6.2",
        "208 dependencies scanned",
        "No GitHub workflow was dispatched",
        "No GitHub release, Cargo package, or npm",
    ] {
        assert!(
            review.contains(required),
            "qualification review is missing {required:?}"
        );
    }
}
