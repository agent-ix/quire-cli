//! Native consumer controls for quire-rs#409. The module fixture is copied
//! byte-for-byte from agent-ix/qa-corpus 7442f2770880a4ade303fb23d725804bdef454db,
//! modules/variants/reference-status-column-explicit/manifest.yaml.
mod common;

use std::fs;
use std::process::Output;

use serde_json::Value;
use tempfile::TempDir;

fn fixture(default_header: &str, alternate_header: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("spec")).unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("spec/tests.md"),
        format!(
            "---\nid: TM-409\ntype: Matrix409\n---\n\n## Default\n\n\
             | ID | Tests | {default_header} |\n|---|---|---|\n\
             | TC-001 | TC-001 | done |\n\n## Alternate\n\n\
             | ID | Tests | {alternate_header} |\n|---|---|---|\n\
             | TC-002 | TC-002 | done |\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.path().join("src/lib.rs"),
        "// Trace: TC-001\n#[test]\nfn first_evidence() { assert_eq!(1 + 1, 2); }\n\
         // Trace: TC-002\n#[test]\nfn second_evidence() { assert_eq!(1 + 1, 2); }\n",
    )
    .unwrap();
    dir
}

fn run(dir: &TempDir, strict: bool) -> (Output, Value) {
    let mut command = common::quire();
    command
        .arg("coverage")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(common::fixture_root().join("reference-status-column"))
        .arg("--json");
    if strict {
        command.arg("--strict");
    }
    let out = command.output().unwrap();
    let report = serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "JSON absent ({e}): {}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (out, report)
}

fn assert_backed(report: &Value) {
    assert_eq!(report["totals"]["total"], 2, "{report}");
    assert_eq!(report["totals"]["backed"], 2, "{report}");
    for field in ["unbacked_rows", "status_lies"] {
        assert!(
            report[field].as_array().unwrap().is_empty(),
            "{field}: {report}"
        );
    }
    // The engine omits this advisory list when empty (FR-017-AC-10).
    assert!(report.get("undeclared_statuses").is_none(), "{report}");
}

// Trace: IT-152, FR-017-AC-21
#[test]
fn it152_native_default_and_override_are_independently_readable() {
    let dir = fixture("Status", "Coverage Status");
    let (default, report) = run(&dir, false);
    assert_eq!(default.status.code(), Some(0), "{report}");
    assert_backed(&report);
    // The fully readable fixture has no diagnostics; empty lists are omitted.
    assert!(report.get("diagnostics").is_none(), "{report}");
    let (strict, strict_report) = run(&dir, true);
    assert_eq!(strict.status.code(), Some(0), "{strict_report}");
    assert_eq!(strict_report, report);
}

// Trace: IT-153, FR-017-AC-21
#[test]
fn it153_each_missing_header_reports_then_fails_strict_without_fallback() {
    for (default_header, alternate_header, declaration, setting) in [
        (
            "Status",
            "Status",
            "alternate-status",
            "traceability.document_references[].status_column",
        ),
        (
            "Coverage Status",
            "Coverage Status",
            "default-status",
            "traceability.status.column",
        ),
    ] {
        let dir = fixture(default_header, alternate_header);
        let (default, report) = run(&dir, false);
        assert_eq!(default.status.code(), Some(0), "{report}");
        assert_backed(&report);
        assert_eq!(
            report["diagnostics"].as_array().unwrap().len(),
            1,
            "{report}"
        );
        let missing = report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["reason"] == "status-column-matches-nothing")
            .collect::<Vec<_>>();
        assert_eq!(missing.len(), 1, "{report}");
        assert_eq!(missing[0]["declaration"], declaration);
        assert!(missing[0]["message"].as_str().unwrap().contains(setting));
        assert_eq!(missing[0]["path"], "spec/tests.md");
        assert!(missing[0]["line"].as_u64().unwrap() > 0);
        let (strict, strict_report) = run(&dir, true);
        assert_eq!(strict.status.code(), Some(1), "{strict_report}");
        assert_eq!(
            strict_report, report,
            "strict must retain the full JSON report"
        );
        assert!(String::from_utf8_lossy(&strict.stderr).contains("status-column-matches-nothing"));
    }
}
