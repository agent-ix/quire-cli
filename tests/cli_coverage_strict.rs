//! IR51-02: a strict verdict cannot certify an unread measurement.
mod common;

use std::fs;
use std::process::Output;

use serde_json::Value;
use tempfile::TempDir;

fn fixture(references: bool) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("m")).unwrap();
    fs::create_dir_all(dir.path().join("spec")).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    let refs = if references {
        "  document_references:\n  - name: traces-to\n    archetype: TestMatrix\n    section: Cases\n    column: Traces To\n    row_id_column: ID\n    pattern: '(TC-\\d+)'\n    targets: [case]\n  status:\n    column: Status\n    complete: [done]\n"
    } else {
        ""
    };
    fs::write(dir.path().join("m/manifest.yaml"), format!(
        "name: m\nartifact_types:\n- name: TestMatrix\ntraceability:\n  trace_targets:\n  - name: case\n    archetype: TestMatrix\n    section: Cases\n    id_column: ID\n{refs}  trace_tags:\n    legacy:\n    - name: comment\n      pattern: 'TRACE: (TC-\\d+)'\n"
    )).unwrap();
    matrix(&dir, "Status");
    source(&dir, true);
    dir
}

fn matrix(dir: &TempDir, status: &str) {
    fs::write(dir.path().join("spec/tests.md"), format!(
        "---\nid: TM-001\ntype: TestMatrix\n---\n## Cases\n\n| ID | Traces To | {status} |\n|----|-----------|--------|\n| TC-001 | TC-001 | done |\n"
    )).unwrap();
}

fn source(dir: &TempDir, readable: bool) {
    let marker = if readable { "TRACE" } else { "UNDECLARED" };
    fs::write(
        dir.path().join("src/lib.rs"),
        format!("// {marker}: TC-001\n#[test]\nfn real_evidence() {{}}\n"),
    )
    .unwrap();
}

fn run(dir: &TempDir, strict: bool, projected: bool) -> (Output, Value) {
    let mut command = common::quire();
    command
        .arg("coverage")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(dir.path().join("m"))
        .arg("--json");
    if strict {
        command.arg("--strict");
    }
    if projected {
        for check in [
            "unbacked-row",
            "status-lie",
            "untracked-symbol",
            "undeclared-status",
        ] {
            command
                .arg("--severity")
                .arg(format!("coverage:{check}=off"));
        }
    }
    let output = command.output().unwrap();
    let report = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "no structured report: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output, report)
}

fn has_reason(report: &Value, reason: &str) -> bool {
    report["diagnostics"]
        .as_array()
        .is_some_and(|diagnostics| diagnostics.iter().any(|d| d["reason"] == reason))
}

fn assert_rejected(dir: &TempDir, reason: &str) {
    let (plain, report) = run(dir, false, false);
    assert!(plain.status.success());
    assert_eq!(report["totals"]["total"], 1);
    assert!(report["unbacked_rows"].as_array().unwrap().is_empty());
    assert!(report["status_lies"].as_array().unwrap().is_empty());
    assert!(has_reason(&report, reason), "{report}");
    for projected in [false, true] {
        let (strict, report) = run(dir, true, projected);
        assert_eq!(strict.status.code(), Some(1), "strict certified {reason}");
        assert!(has_reason(&report, reason));
        if projected {
            assert!(report["untracked_symbols"].as_array().unwrap().is_empty());
        }
        assert!(String::from_utf8_lossy(&strict.stderr).contains(reason));
    }
}

// Trace: IT-150, FR-017-AC-22
#[test]
fn it150_strict_rejects_unread_status_and_preserves_the_report() {
    let dir = fixture(true);
    fs::write(
        dir.path().join("src/extra.rs"),
        "// TRACE: TC-999\n#[test]\nfn untracked_evidence() {}\n",
    )
    .unwrap();
    matrix(&dir, "Coverage Status");
    assert_eq!(
        run(&dir, false, false).1["untracked_symbols"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_rejected(&dir, "status-column-matches-nothing");
    matrix(&dir, "Status");
    let (healthy, report) = run(&dir, true, true);
    assert!(healthy.status.success(), "{report}");
    assert!(!has_reason(&report, "status-column-matches-nothing"));
}

// Trace: IT-151, FR-017-AC-22
#[test]
fn it151_strict_rejects_a_hollow_measurement_without_reference_row_failures() {
    let dir = fixture(false);
    source(&dir, false);
    assert_rejected(&dir, "hollow-denominator");
    source(&dir, true);
    let (healthy, report) = run(&dir, true, true);
    assert!(healthy.status.success(), "{report}");
    assert!(!has_reason(&report, "hollow-denominator"));
}
