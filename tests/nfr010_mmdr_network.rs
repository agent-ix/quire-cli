//! Real Mermaid renderers execute without opening network sockets, and the
//! producer executor reports completed, contained requests (NFR-010).

#![cfg(target_os = "linux")]

mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use ix_trace_rs::trace;
use serde_json::Value;
use tempfile::TempDir;

const DIAGRAMS: [(&str, &str); 4] = [
    ("flowchart", "flowchart TD\n  Start --> Stop"),
    ("sequence", "sequenceDiagram\n  Alice->>Bob: hello"),
    ("class", "classDiagram\n  class Quill"),
    ("state", "stateDiagram-v2\n  [*] --> Ready"),
];

struct Rig {
    root: TempDir,
    executor: PathBuf,
    mmdr: PathBuf,
}

impl Rig {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("temporary root");
        for path in [
            "bin",
            "repo",
            "repo/docs",
            "work",
            "log/requests",
            "log/results",
        ] {
            fs::create_dir_all(root.path().join(path)).expect("rig directory");
        }

        let executor = find_tool("engineering-assurance");
        let mmdr = find_tool("mmdr")
            .canonicalize()
            .expect("canonical mmdr path");
        let pandoc = root.path().join("bin/pandoc");
        write_executable(
            &pandoc,
            "out=\"\"\nlast=\"\"\nwhile [ $# -gt 0 ]; do\n  \
             case \"$1\" in --output) out=\"$2\"; shift ;; *) last=\"$1\" ;; esac\n  \
             shift\ndone\ncp \"$last\" \"$out\"\n",
        );

        let wrapper = root.path().join("bin/engineering-assurance");
        write_executable(
            &wrapper,
            &format!(
                concat!(
                    "REAL_EA='{}'\nLOG='{}'\n",
                    "n=$(find \"$LOG/requests\" -maxdepth 1 -type f | wc -l)\n",
                    "n=$((n + 1))\n",
                    "cat > \"$LOG/requests/$n.json\"\n",
                    "\"$REAL_EA\" producer-execute --request \"$(cat \"$LOG/requests/$n.json\")\" ",
                    "> \"$LOG/results/$n.json\" 2> \"$LOG/errors/$n.txt\"\n",
                    "status=$?\n",
                    "cat \"$LOG/results/$n.json\"\n",
                    "exit $status\n"
                ),
                executor.display(),
                root.path().join("log").display(),
            ),
        );
        fs::create_dir_all(root.path().join("log/errors")).expect("error log directory");

        git(root.path().join("repo"), &["init", "-q", "-b", "main"]);
        git(
            root.path().join("repo"),
            &["config", "user.name", "Fixture"],
        );
        git(
            root.path().join("repo"),
            &["config", "user.email", "fixture@example.invalid"],
        );

        Self {
            root,
            executor: wrapper,
            mmdr,
        }
    }

    fn add_diagram(&self, name: &str, source: &str) -> PathBuf {
        let path = self.root.path().join("repo/docs/FR-021.md");
        let document = format!(
            "---\nid: FR-021\ntitle: Diagram fixture\nversion: \"1.0\"\ntype: FR\n---\n\
             # [FR-021] Diagram fixture\n\n## Description\n\nThe system SHALL draw a diagram.\n\n\
             ## Specification\n\nThe system SHALL satisfy the described behavior.\n\n\
             ### {name}\n\n```mermaid\n{source}\n```\n\n\
             ## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n| FR-021-AC-1 | It draws the diagram. |\n"
        );
        fs::write(&path, document).expect("diagram fixture");
        git(self.root.path().join("repo"), &["add", "docs/FR-021.md"]);
        git(
            self.root.path().join("repo"),
            &["commit", "-q", "-m", "diagram fixture"],
        );
        path
    }

    fn requests(&self) -> Vec<Value> {
        sorted_json_files(self.root.path().join("log/requests"))
    }

    fn results(&self) -> Vec<Value> {
        sorted_json_files(self.root.path().join("log/results"))
    }
}

#[trace("NFR-010-AC-4")]
#[test]
fn real_mmdr_completes_without_opening_network_sockets_for_each_guaranteed_type() {
    let strace = find_tool("strace");

    for (kind, source) in DIAGRAMS {
        let rig = Rig::new();
        let input = rig.add_diagram(kind, source);
        let output = rig.root.path().join("work/out.docx");
        let trace = rig.root.path().join("work/network.trace");
        let mut command = Command::new(&strace);
        command
            .args(["-f", "-e", "trace=network", "-o"])
            .arg(&trace)
            .arg(env!("CARGO_BIN_EXE_quire"))
            .current_dir(rig.root.path().join("work"))
            .arg("export")
            .arg(&input)
            .args(["--to", "docx", "--output"])
            .arg(&output)
            .arg("--module")
            .arg(common::validate_module())
            .arg("--pandoc")
            .arg(rig.root.path().join("bin/pandoc"))
            .arg("--mmdr")
            .arg(&rig.mmdr)
            .arg("--engineering-assurance")
            .arg(&rig.executor);
        let result = command.output().expect("strace starts quire");
        assert!(
            result.status.success(),
            "{kind} export failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(output.is_file(), "{kind} export produced no DOCX");

        let network_trace = fs::read_to_string(&trace).expect("strace output");
        assert!(
            !network_trace.contains("socket("),
            "{kind} export opened a socket:\n{network_trace}"
        );

        let requests = rig.requests();
        let results = rig.results();
        assert_eq!(
            requests.len(),
            2,
            "one mmdr and one pandoc request: {requests:?}"
        );
        assert_eq!(results.len(), requests.len(), "one result per request");
        let mmdr_results: Vec<_> = requests
            .iter()
            .zip(&results)
            .filter(|(request, _)| request["producer"]["name"] == "mmdr")
            .map(|(request, result)| (request, result))
            .collect();
        assert_eq!(mmdr_results.len(), 1, "{kind} has one mmdr request");
        let (request, result) = mmdr_results[0];
        assert_eq!(
            request["producer"]["executable"],
            rig.mmdr.display().to_string()
        );
        assert_eq!(result["state"]["kind"], "completed", "{kind}: {result}");
        assert_ne!(
            result["state"]["kind"], "containment_failure",
            "{kind}: {result}"
        );
    }
}

fn find_tool(name: &str) -> PathBuf {
    let path = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .unwrap_or_else(|| panic!("this test needs `{name}` on PATH; none was found"))
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, format!("#!/bin/sh\n{body}")).expect("script");
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("script mode");
}

fn git(repository: PathBuf, args: &[&str]) {
    let result = Command::new("git")
        .current_dir(repository)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .expect("git runs");
    assert!(
        result.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
}

fn sorted_json_files(directory: PathBuf) -> Vec<Value> {
    let mut paths: Vec<_> = fs::read_dir(directory)
        .expect("read JSON directory")
        .map(|entry| entry.expect("JSON entry").path())
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| serde_json::from_slice(&fs::read(path).expect("read JSON")).expect("JSON"))
        .collect()
}
