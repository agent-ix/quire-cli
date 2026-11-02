//! IR51-02: a strict verdict cannot certify an unread measurement.
mod common;

use std::fs;
use std::process::Output;

use ix_trace_rs::trace;
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

const WITHDRAWAL_REASON: &str = "Withdrawn by the structural-identity decision: the old content-authentication behavior is removed. This id is retained and never reused.";

fn retired_fixture(retirement_declared: bool, rows: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("m")).unwrap();
    fs::create_dir(dir.path().join("spec")).unwrap();
    let vocabulary = if retirement_declared {
        "  vocabularies:\n    retired: [Withdrawn]\n"
    } else {
        ""
    };
    fs::write(
        dir.path().join("m/manifest.yaml"),
        format!(
            "name: retired-coverage\nartifact_types:\n- name: FR\ntraceability:\n\
             \x20 trace_targets:\n  - name: criterion\n    archetype: FR\n\
             \x20   section: Acceptance Criteria\n    id_column: ID\n\
             \x20 document_references:\n  - name: verification\n    archetype: FR\n\
             \x20   section: Acceptance Criteria\n    column: Verification\n\
             \x20   row_id_column: ID\n    pattern: '(FR-\\d+-AC-\\d+)'\n\
             \x20   targets: [criterion]\n  status:\n    column: Status\n\
             \x20   complete: [done]\n    pending: [planned]\n{vocabulary}"
        ),
    )
    .unwrap();
    retired_document(&dir, rows);
    dir
}

fn retired_document(dir: &TempDir, rows: &str) {
    fs::write(
        dir.path().join("spec/FR-038.md"),
        format!(
            "---\nid: FR-038\ntype: FR\n---\n\n## Acceptance Criteria\n\n\
             | ID | Criteria | Verification | Status |\n|---|---|---|---|\n{rows}"
        ),
    )
    .unwrap();
}

fn tombstone_row(method: &str) -> String {
    format!("| FR-038-AC-92 | {WITHDRAWAL_REASON} | {method} | planned |\n")
}

#[trace("FR-017-AC-7", "FR-050-AC-53")]
#[test]
fn strict_retired_rows_preserve_history_without_hiding_live_gaps() {
    let rows = format!(
        "{}| FR-038-AC-93 | Live behavior. | Test | done |\n\
         | FR-038-AC-94 | Planned until a named follow-up lands. | Test | planned |\n",
        tombstone_row("Withdrawn")
    );
    let dir = retired_fixture(true, &rows);
    let (plain, report) = run(&dir, false, false);
    assert!(plain.status.success(), "{report}");
    assert_eq!(report["totals"]["total"], 2, "{report}");
    assert_eq!(report["totals"]["backed"], 0, "{report}");
    assert_eq!(report["totals"]["retired"], 1, "{report}");
    let unbacked = report["unbacked_rows"].as_array().unwrap();
    assert_eq!(unbacked.len(), 2, "{report}");
    assert_eq!(unbacked[0]["row_id"], "FR-038-AC-93");
    assert_eq!(unbacked[1]["row_id"], "FR-038-AC-94");
    let lies = report["status_lies"].as_array().unwrap();
    assert_eq!(lies.len(), 1, "{report}");
    assert_eq!(lies[0]["row_id"], "FR-038-AC-93");
    let retired = report["retired_rows"].as_array().unwrap();
    assert_eq!(retired.len(), 1, "{report}");
    assert_eq!(retired[0]["row_id"], "FR-038-AC-92");
    assert_eq!(retired[0]["document"], "spec/FR-038.md");
    assert_eq!(retired[0]["cells"]["Criteria"], WITHDRAWAL_REASON);
    assert_eq!(retired[0]["cells"]["Verification"], "Withdrawn");
    assert!(retired[0]["line"].as_u64().unwrap() > 0);
    let minted = report["minted_targets"].as_array().unwrap();
    assert_eq!(minted.len(), 3, "{report}");
    let historical = minted
        .iter()
        .find(|row| row["id"] == "FR-038-AC-92")
        .unwrap();
    assert_eq!(historical["retired"], true);
    assert_eq!(historical["backed"], false);

    let (strict, strict_report) = run(&dir, true, false);
    assert_eq!(strict.status.code(), Some(1), "{strict_report}");
    assert_eq!(strict_report, report);
    assert!(String::from_utf8_lossy(&strict.stderr)
        .contains("2 unbacked row(s) and 1 contradicted status(es)"));
}

#[trace("FR-017-AC-8", "FR-050-AC-54")]
#[test]
fn strict_all_retired_scope_is_distinct_from_an_empty_scope() {
    let dir = retired_fixture(true, &tombstone_row("Withdrawn"));
    let (strict, report) = run(&dir, true, false);
    assert!(strict.status.success(), "{report}");
    assert_eq!(report["totals"]["total"], 0, "{report}");
    assert_eq!(report["totals"]["retired"], 1, "{report}");
    assert!(report["unbacked_rows"].as_array().unwrap().is_empty());
    assert!(report["status_lies"].as_array().unwrap().is_empty());

    let human = common::quire()
        .arg("coverage")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(dir.path().join("m"))
        .arg("--strict")
        .output()
        .unwrap();
    assert!(human.status.success());
    let summary = String::from_utf8_lossy(&human.stdout);
    assert!(
        summary.contains("0/0 rows backed (no live rows)"),
        "{summary}"
    );
    assert!(summary.contains("1 retired historical row(s)"), "{summary}");
    assert!(!summary.contains("no rows matched"), "{summary}");
    assert!(!summary.contains("100%"), "{summary}");

    retired_document(&dir, "");
    let (empty, report) = run(&dir, true, false);
    assert_eq!(empty.status.code(), Some(1), "{report}");
    assert_eq!(report["totals"]["total"], 0);
    assert!(report["totals"].get("retired").is_none());
    assert!(String::from_utf8_lossy(&empty.stderr)
        .contains("the declared traceability model matched no rows"));
}

#[trace("FR-017-AC-7", "FR-050-AC-55")]
#[test]
fn strict_undeclared_and_misspelled_retirement_values_stay_live() {
    for (declared, method) in [(false, "Withdrawn"), (true, "Withdrawnn")] {
        let dir = retired_fixture(declared, &tombstone_row(method));
        let (strict, report) = run(&dir, true, false);
        assert_eq!(strict.status.code(), Some(1), "{report}");
        assert_eq!(report["totals"]["total"], 1, "{report}");
        assert!(report["totals"].get("retired").is_none(), "{report}");
        assert_eq!(report["unbacked_rows"].as_array().unwrap().len(), 1);
        assert!(report.get("retired_rows").is_none(), "{report}");
        assert_eq!(report["unbacked_rows"][0]["row_id"], "FR-038-AC-92");
    }
}
