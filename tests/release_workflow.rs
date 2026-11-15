use ix_trace_rs::trace;
use yaml_serde::Value;

const MANUAL_RELEASE_WORKFLOW: &str = include_str!("../.github/workflows/release.yml");
const TAG_ATTESTATION_WORKFLOW: &str = include_str!("../.github/workflows/tag-attestation.yml");

fn parse_workflow(source: &str) -> Value {
    yaml_serde::from_str(source).expect("workflow must be valid YAML")
}

fn required<'a>(value: &'a Value, key: &str) -> &'a Value {
    value
        .get(key)
        .unwrap_or_else(|| panic!("workflow is missing `{key}`"))
}

fn required_str<'a>(value: &'a Value, key: &str) -> &'a str {
    required(value, key)
        .as_str()
        .unwrap_or_else(|| panic!("workflow field `{key}` must be a string"))
}

#[trace("FR-022-AC-5")]
#[test]
fn manual_publish_workflow_stays_independent_of_tag_attestation() {
    let manual = parse_workflow(MANUAL_RELEASE_WORKFLOW);
    let triggers = required(&manual, "on")
        .as_mapping()
        .expect("manual release triggers must be a mapping");

    assert_eq!(triggers.len(), 1, "manual release must have one trigger");
    let dispatch = triggers
        .get("workflow_dispatch")
        .expect("manual release must remain workflow_dispatch-only");
    assert!(
        triggers.get("push").is_none(),
        "manual release must not publish on tag push"
    );

    let publish_input = required(required(dispatch, "inputs"), "publish");
    assert_eq!(required_str(publish_input, "type"), "boolean");
    assert_eq!(
        required(publish_input, "default"),
        &Value::Bool(false),
        "manual dispatch must default to a dry run"
    );

    let package_steps = required(required(required(&manual, "jobs"), "package"), "steps")
        .as_sequence()
        .expect("package steps must be a sequence");
    let create_release = package_steps
        .iter()
        .find(|step| step.get("name").and_then(Value::as_str) == Some("Create GitHub Release"))
        .expect("manual workflow must create the GitHub Release");
    assert_eq!(
        required_str(create_release, "if"),
        "inputs.publish",
        "GitHub Release creation must be gated by the manual publish input"
    );

    let publish_npm = package_steps
        .iter()
        .find(|step| {
            step.get("name").and_then(Value::as_str)
                == Some("Package and publish the npm distribution")
        })
        .expect("manual workflow must invoke the shared npm publisher");
    assert!(
        required_str(publish_npm, "uses").ends_with("publish-native-npm@main"),
        "manual workflow must keep using the shared npm publisher"
    );
    assert_eq!(
        required(required(publish_npm, "with"), "publish"),
        &Value::String("${{ inputs.publish }}".to_owned()),
        "npm publication must receive the manual publish input"
    );

    let tag_attestation = parse_workflow(TAG_ATTESTATION_WORKFLOW);
    let tag_triggers = required(&tag_attestation, "on");
    assert_eq!(
        tag_triggers.as_mapping().map(|triggers| triggers.len()),
        Some(1),
        "tag-attestation must have only its independent push trigger"
    );
    let tag_push = required(tag_triggers, "push");
    let tags = required(tag_push, "tags")
        .as_sequence()
        .expect("tag-attestation push tags must be a sequence");
    assert!(
        tags.iter().any(|tag| tag.as_str() == Some("v*")),
        "tag-attestation must run on v* tag pushes"
    );
    assert!(
        required(&tag_attestation, "on")
            .get("workflow_run")
            .is_none(),
        "tag-attestation must not wait for the manual release workflow"
    );
    let attest_job = required(required(&tag_attestation, "jobs"), "attest");
    assert_eq!(
        required_str(attest_job, "uses"),
        "agent-ix/engineering-assurance/.github/workflows/release-attestation.yml@v0.9.0",
        "tag-attestation must invoke the Engineering Assurance reusable workflow"
    );
}
