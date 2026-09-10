use std::fs;
use std::path::{Path, PathBuf};

use ix_trace_rs::trace;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("tools/quire-qualify is two levels below the repository root")
        .to_path_buf()
}

fn recipe<'a>(makefile: &'a str, target: &str) -> Vec<&'a str> {
    let marker = format!("{target}:");
    let mut lines = makefile.lines().skip_while(|line| *line != marker);
    assert_eq!(
        lines.next(),
        Some(marker.as_str()),
        "missing make target {target}"
    );
    lines
        .take_while(|line| line.is_empty() || line.starts_with('\t'))
        .filter(|line| line.starts_with('\t'))
        .collect()
}

fn walk_files(directory: &Path, excluded: &[&str], files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        assert!(
            !kind.is_symlink(),
            "symlinked path escapes static inventory: {}",
            entry.path().display()
        );
        if kind.is_dir() {
            let name = entry.file_name();
            if excluded.iter().any(|excluded| name == *excluded) {
                continue;
            }
            walk_files(&entry.path(), excluded, files);
        } else if kind.is_file() {
            files.push(entry.path());
        }
    }
}

#[trace("TC-838", "FR-025-AC-6", "FR-025-CON-2", "FR-025-CON-3")]
#[test]
fn make_recipes_only_orchestrate_native_qualification() {
    let makefile = fs::read_to_string(repo_root().join("Makefile")).unwrap();
    let cases = [
        ("audit-unsafe", "unsafe-comments"),
        ("audit-thin-boundary", "thin-boundary"),
        ("spec", "assurance-traceability"),
        ("bench", "bench-p95"),
    ];
    for (target, native_command) in cases {
        let body = recipe(&makefile, target).join("\n");
        assert!(
            body.contains(native_command),
            "{target} does not invoke {native_command}: {body}"
        );
        for forbidden in ["python", ".py", ".sh", "grep ", "sed ", "jq ", "awk "] {
            assert!(
                !body.contains(forbidden),
                "{target} interprets content through {forbidden:?}: {body}"
            );
        }
    }
}

#[trace("TC-839", "FR-025-AC-7", "FR-025-CON-4", "FR-025-CON-5")]
#[test]
fn executable_script_inventory_is_empty_except_for_the_npm_host() {
    let root = repo_root();
    let mut files = Vec::new();
    walk_files(
        &root,
        &[
            ".git",
            ".worktrees",
            "target",
            "target-agent-c",
            "node_modules",
            "fixtures",
            "spec",
            "reviews",
        ],
        &mut files,
    );
    let prohibited: Vec<_> = files
        .iter()
        .filter(|path| {
            matches!(
                path.extension().and_then(|value| value.to_str()),
                Some("py" | "sh" | "mjs")
            )
        })
        .map(|path| path.strip_prefix(&root).unwrap().to_path_buf())
        .collect();
    assert!(
        prohibited.is_empty(),
        "non-native executable qualification paths remain: {prohibited:?}"
    );

    let javascript: Vec<_> = files
        .iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("js"))
        .map(|path| path.strip_prefix(&root).unwrap().to_path_buf())
        .collect();
    assert_eq!(javascript, [PathBuf::from("npm/quire-cli/bin/quire.js")]);
    let launcher = fs::read_to_string(root.join(&javascript[0])).unwrap();
    for forbidden in [
        "coverage",
        "benchmark",
        "bench-p95",
        "unsafe-comment",
        "thin-boundary",
        "qualification",
    ] {
        assert!(
            !launcher.contains(forbidden),
            "npm distribution host acquired qualification semantic {forbidden:?}"
        );
    }
}

#[trace("TC-840", "NFR-009")]
#[test]
fn native_toolchain_trace_and_no_shell_invariants_are_explicit() {
    let root = repo_root();
    let toolchain = fs::read_to_string(root.join("rust-toolchain.toml")).unwrap();
    assert!(toolchain.lines().any(|line| line == "channel = \"1.98.1\""));

    let manifest = fs::read_to_string(root.join("tools/quire-qualify/Cargo.toml")).unwrap();
    assert!(manifest
        .lines()
        .any(|line| line == "rust-version = \"1.98.1\""));
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    for line in makefile.lines().filter(|line| line.contains("$(CARGO)")) {
        if [" build", " clippy", " doc", " run", " test", " deny"]
            .iter()
            .any(|command| line.contains(command))
        {
            assert!(line.contains("--locked"), "unlocked Cargo command: {line}");
        }
    }

    let mut source_files = Vec::new();
    walk_files(
        &root.join("tools/quire-qualify/src"),
        &[],
        &mut source_files,
    );
    for path in &source_files {
        let source = fs::read_to_string(path).unwrap();
        assert!(
            quire_qualify::source::process_execution_lines(&source)
                .unwrap()
                .is_empty(),
            "native qualification tool can execute a process: {}",
            path.display()
        );
    }

    let mut test_files = Vec::new();
    walk_files(
        &root.join("tools/quire-qualify/tests"),
        &[],
        &mut test_files,
    );
    let tests = test_files
        .iter()
        .map(|path| fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    for id in 828..=840 {
        assert!(
            tests.contains(&format!("\"TC-{id}\"")),
            "TC-{id} lacks an ix-trace-rs marker"
        );
    }
}
