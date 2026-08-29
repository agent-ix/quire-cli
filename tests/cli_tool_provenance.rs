mod common;

use common::quire;

#[test]
fn provenance_reports_the_full_engine_revision_and_sorted_capabilities() {
    let output = quire()
        .args(["provenance", "--json"])
        .output()
        .expect("provenance runs");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let payload: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(payload["schemaVersion"], "quire-tool-provenance-v1");
    for component in ["cli", "engine"] {
        assert!(payload[component]["version"].is_string());
        let revision = payload[component]["sourceRevision"]
            .as_str()
            .expect("full source revision");
        assert_eq!(revision.len(), 40, "{component}: {payload}");
    }
    assert_eq!(payload["engine"]["sourceState"], "clean");
    let capabilities = payload["capabilities"].as_array().expect("capabilities");
    let mut sorted = capabilities.clone();
    sorted.sort_by(|left, right| left.as_str().cmp(&right.as_str()));
    assert_eq!(*capabilities, sorted);
    for required in [
        "action_guidance.structured",
        "binding_census.self_named",
        "declaration_origins",
        "property_spans.safe_refusal",
    ] {
        assert!(
            capabilities.iter().any(|value| value == required),
            "{required}"
        );
    }
}

#[test]
fn provenance_is_byte_deterministic() {
    let first = quire().args(["provenance", "--json"]).output().unwrap();
    let second = quire().args(["provenance", "--json"]).output().unwrap();
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
}
