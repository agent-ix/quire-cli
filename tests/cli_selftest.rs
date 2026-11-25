//! Process-boundary tests for the embedded quire self-test suite.

mod common;

use std::fs;

use common::quire;
use ix_trace_rs::trace;
use serde_json::Value;

#[trace("FR-047-AC-1", "FR-047-AC-10")]
#[test]
fn list_includes_quire_cli_and_linked_quire_rs_cases() {
    let output = quire()
        .args(["selftest", "--list"])
        .output()
        .expect("selftest list starts");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let list = String::from_utf8(output.stdout).expect("case list is UTF-8");
    assert!(list.contains("parse-empty-input\tquire-cli"), "{list}");
    assert!(
        list.contains("fr-032-placeholder-detection\tquire-rs"),
        "{list}"
    );
}

#[trace("FR-047-AC-10")]
#[test]
fn unknown_case_is_rejected_before_a_result_file_is_created() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let result_path = dir.path().join("run.json");
    let output = quire()
        .args(["selftest", "--run", "--case", "missing-case-id", "--out"])
        .arg(&result_path)
        .output()
        .expect("selftest process starts");
    assert_eq!(output.status.code(), Some(3));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing-case-id"),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !result_path.exists(),
        "the runner must reject unknown ids before creating an output file"
    );
}

#[trace("FR-047-AC-2", "FR-047-AC-4")]
#[test]
fn explicit_output_is_the_only_run_record_stream_with_empty_home() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let empty_home = dir.path().join("home");
    fs::create_dir(&empty_home).expect("empty HOME");
    let result_path = dir.path().join("run.json");
    let output = quire()
        .current_dir(&dir)
        .env("HOME", &empty_home)
        .args(["selftest", "--run", "--out"])
        .arg(&result_path)
        .output()
        .expect("selftest process starts");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty(), "run result leaked to stdout");
    let record_bytes = fs::read(&result_path).expect("run result is retained at --out");
    let record: Value = serde_json::from_slice(&record_bytes).expect("valid JSON record");
    assert_eq!(record["schemaVersion"], 1);
    assert_eq!(record["tool"], "quire");
    assert_eq!(record["suite"], "SELFTEST");
    assert!(record["toolVersion"]
        .as_str()
        .is_some_and(|s| !s.is_empty()));
    let timestamp = record["timestamp"].as_str().expect("timestamp is a string");
    assert!(
        time::OffsetDateTime::parse(timestamp, &time::format_description::well_known::Rfc3339)
            .is_ok()
    );
    assert!(record["retainedLocation"]
        .as_str()
        .is_some_and(|location| std::path::Path::new(location).is_absolute()));
    assert_eq!(record["results"].as_array().map(Vec::len), Some(3));
    assert!(record["results"]
        .as_array()
        .is_some_and(|cases| { cases.iter().all(|case| case["status"] == "pass") }));
    assert!(record["results"]
        .as_array()
        .is_some_and(|cases| { cases.iter().all(|case| case["stderr"].is_string()) }));
}

#[trace("FR-047-AC-11")]
#[test]
fn retained_anomaly_set_joins_only_the_two_owning_repositories() {
    let dir = tempfile::tempdir().expect("temporary directory");
    let output_path = dir.path().join("run.json");
    let output = quire()
        .current_dir(&dir)
        .args(["selftest", "--run", "--out"])
        .arg(&output_path)
        .output()
        .expect("selftest process starts");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let record: Value = serde_json::from_slice(&fs::read(output_path).expect("run record"))
        .expect("valid run record");
    let set = &record["anomalySet"];
    assert_eq!(set["provided"], true);
    assert_eq!(set["producerVersion"], record["toolVersion"]);
    let documents = set["documents"].as_array().expect("document array");
    assert!(!documents.is_empty());
    assert!(documents.iter().all(|document| {
        document["raw"].is_string()
            && (document["path"]
                .as_str()
                .is_some_and(|path| path.starts_with("quire-cli/spec/anomalies/"))
                || document["path"]
                    .as_str()
                    .is_some_and(|path| path.starts_with("quire-rs/spec/anomalies/")))
    }));
    assert!(documents.iter().all(|document| {
        !document["path"]
            .as_str()
            .is_some_and(|path| path.contains("quoin"))
    }));
    assert!(documents.iter().any(|document| {
        document["path"] == "quire-cli/spec/anomalies/AN-001-dispatch-only-verification.md"
            && document["raw"]
                == include_str!("../spec/anomalies/AN-001-dispatch-only-verification.md")
    }));
    for source in quire_rs::selftest::embedded_anomaly_documents() {
        let path = format!("quire-rs/{}", source.path);
        assert!(
            documents
                .iter()
                .any(|document| { document["path"] == path && document["raw"] == source.content }),
            "missing source record {path}"
        );
    }
}
