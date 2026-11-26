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

#[trace("FR-025-AC-6", "FR-025-CON-2", "FR-025-CON-3")]
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

#[trace("NFR-009-M-2")]
#[test]
fn native_toolchain_trace_and_no_shell_invariants_are_explicit() {
    let root = repo_root();
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
}
