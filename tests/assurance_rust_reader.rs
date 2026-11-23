//! Exact assurance golden bytes preserve the Rust reader contract.

use ix_trace_rs::trace;
use std::collections::BTreeSet;

use jsonschema::{Draft, JSONSchema};
use quire_rs::assurance::ASSURANCE_V1_SCHEMA;
use quire_rs::{read_assurance_export, AcceptedAssurancePremises, AssuranceExport};
use serde_json::Value;

const GOLDEN: &[u8] = include_bytes!("fixtures/assurance/v1.json");

#[trace("FR-020-AC-8")]
#[test]
fn it_144_rust_golden_validates_and_pins_every_v1_field_and_state_token() {
    let payload: Value = serde_json::from_slice(GOLDEN).expect("golden JSON");
    let schema: Value = serde_json::from_str(ASSURANCE_V1_SCHEMA).expect("upstream schema");
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft202012)
        .compile(&schema)
        .expect("schema compiles");
    compiled
        .validate(&payload)
        .map_err(|errors| errors.map(|error| error.to_string()).collect::<Vec<_>>())
        .expect("Rust validates the exact golden");

    assert_keys(
        &schema["properties"],
        &[
            "format",
            "format_version",
            "modules",
            "artifacts",
            "obligations",
            "symbols",
            "relation_kinds",
            "relations",
            "relation_observations",
        ],
        "schema envelope properties",
    );
    let definitions = &schema["$defs"];
    assert_keys(
        definitions,
        &[
            "schemaPremise",
            "modulePremise",
            "locator",
            "artifact",
            "obligation",
            "symbol",
            "freshness",
            "relationKind",
            "corpusRelation",
            "verifiesRelation",
            "implementsRelation",
            "relationObservation",
        ],
        "schema definitions",
    );
    assert_schema_properties(definitions, "schemaPremise", &["archetype"]);
    assert_schema_properties(
        definitions,
        "modulePremise",
        &["name", "version", "schemas"],
    );
    assert_schema_properties(definitions, "locator", &["path", "line"]);
    assert_schema_properties(
        definitions,
        "artifact",
        &["id", "uuid", "artifact_type", "locator"],
    );
    assert_schema_properties(
        definitions,
        "obligation",
        &[
            "source",
            "id",
            "document",
            "statement",
            "method",
            "parameters",
            "criticality",
            "target_ids",
            "locator",
        ],
    );
    assert_schema_properties(
        definitions,
        "symbol",
        &[
            "id",
            "language",
            "kind",
            "qualified_name",
            "container",
            "capabilities",
            "locator",
        ],
    );
    assert_schema_properties(
        definitions,
        "relationKind",
        &["kind", "availability", "sources"],
    );
    assert_schema_properties(
        definitions,
        "corpusRelation",
        &[
            "kind",
            "source",
            "target",
            "edge_type",
            "resolution",
            "locator",
            "freshness",
        ],
    );
    for relation in ["verifiesRelation", "implementsRelation"] {
        assert_schema_properties(
            definitions,
            relation,
            &["kind", "source", "target", "form", "locator", "freshness"],
        );
    }
    assert_schema_properties(
        definitions,
        "relationObservation",
        &[
            "declaration",
            "subject",
            "availability",
            "freshness",
            "reason",
        ],
    );

    assert_keys(
        &payload,
        &[
            "format",
            "format_version",
            "modules",
            "artifacts",
            "obligations",
            "symbols",
            "relation_kinds",
            "relations",
            "relation_observations",
        ],
        "export",
    );
    assert_eq!(payload["format"], "quire-assurance");
    assert_eq!(payload["format_version"], 1);
    for module in payload["modules"].as_array().expect("modules") {
        assert_keys(module, &["name", "version", "schemas"], "module");
        for schema in module["schemas"].as_array().expect("module schemas") {
            assert_keys(schema, &["archetype"], "module schema");
        }
    }
    for artifact in payload["artifacts"].as_array().expect("artifacts") {
        assert_keys(artifact, &["id", "artifact_type", "locator"], "artifact");
        assert_locator_keys(&artifact["locator"]);
    }
    for obligation in payload["obligations"].as_array().expect("obligations") {
        assert_keys(
            obligation,
            &[
                "source",
                "id",
                "document",
                "statement",
                "method",
                "target_ids",
                "locator",
            ],
            "obligation",
        );
        assert_locator_keys(&obligation["locator"]);
    }
    for symbol in payload["symbols"].as_array().expect("symbols") {
        assert_keys(
            symbol,
            &[
                "id",
                "language",
                "kind",
                "qualified_name",
                "container",
                "capabilities",
                "locator",
            ],
            "symbol",
        );
        assert_locator_keys(&symbol["locator"]);
    }
    for relation_kind in payload["relation_kinds"]
        .as_array()
        .expect("relation kinds")
    {
        assert_keys(
            relation_kind,
            &["kind", "availability", "sources"],
            "relation kind",
        );
    }
    for relation in payload["relations"].as_array().expect("relations") {
        match relation["kind"].as_str().expect("relation kind") {
            "corpus" => assert_keys(
                relation,
                &[
                    "kind",
                    "source",
                    "target",
                    "edge_type",
                    "resolution",
                    "locator",
                    "freshness",
                ],
                "corpus relation",
            ),
            "verifies" | "implements" => assert_keys(
                relation,
                &["kind", "source", "target", "form", "locator", "freshness"],
                "binding relation",
            ),
            other => panic!("unexpected relation kind in v1 golden: {other}"),
        }
        assert_locator_keys(&relation["locator"]);
    }
    for observation in payload["relation_observations"]
        .as_array()
        .expect("observations")
    {
        match (
            observation.get("subject").is_some(),
            observation.get("reason").is_some(),
        ) {
            (true, true) => assert_keys(
                observation,
                &[
                    "declaration",
                    "subject",
                    "availability",
                    "freshness",
                    "reason",
                ],
                "observation with reason",
            ),
            (true, false) => assert_keys(
                observation,
                &["declaration", "subject", "availability", "freshness"],
                "observation with subject",
            ),
            (false, false) => assert_keys(
                observation,
                &["declaration", "availability", "freshness"],
                "observation without subject",
            ),
            (false, true) => panic!("observation reason requires a subject"),
        }
    }
    assert_eq!(
        collect_tokens(&payload, "availability"),
        tokens(&["available", "missing", "not_applicable", "unknown"]),
        "v1 availability state-token set",
    );
    assert_eq!(
        collect_tokens(&payload, "freshness"),
        tokens(&["not_applicable", "unknown"]),
        "v1 freshness state-token set",
    );
    assert_eq!(
        collect_tokens(&payload, "resolution"),
        tokens(&["dangling", "resolved"]),
        "v1 resolution state-token set",
    );
    assert_eq!(
        enum_tokens(&schema["$defs"]["freshness"]["enum"]),
        tokens(&["current", "not_applicable", "suspect", "unknown"]),
        "v1 freshness enum",
    );
    assert_eq!(
        enum_tokens(&schema["$defs"]["relationObservation"]["properties"]["availability"]["enum"]),
        tokens(&["available", "missing", "not_applicable", "unknown"]),
        "v1 availability enum",
    );
    assert_eq!(
        enum_tokens(&schema["$defs"]["corpusRelation"]["properties"]["resolution"]["enum"]),
        tokens(&["dangling", "resolved"]),
        "v1 resolution enum",
    );
}
#[trace("FR-020-AC-8")]
#[test]
fn it_144_rust_reader_accepts_and_refuses_golden_corpus_cases() {
    let mut valid: Value = serde_json::from_slice(GOLDEN).expect("golden JSON");
    let typed: AssuranceExport = serde_json::from_value(valid.clone()).expect("typed golden");
    let accepted = AcceptedAssurancePremises::from_export(&typed);
    let mut cases = vec![reader_case(
        "valid-v1",
        serde_json::to_vec(&valid).unwrap(),
        true,
    )];

    let mut unsupported_version = valid.clone();
    unsupported_version["format_version"] = 2.into();
    cases.push(reader_case(
        "unknown-format-version",
        serde_json::to_vec(&unsupported_version).unwrap(),
        false,
    ));

    let mut duplicate_module = valid.clone();
    let repeated_module = duplicate_module["modules"][0].clone();
    duplicate_module["modules"]
        .as_array_mut()
        .unwrap()
        .push(repeated_module);
    cases.push(reader_case(
        "duplicate-module-identity",
        serde_json::to_vec(&duplicate_module).unwrap(),
        false,
    ));

    let mut duplicate_schema = valid.clone();
    let repeated_schema = duplicate_schema["modules"][0]["schemas"][0].clone();
    duplicate_schema["modules"][0]["schemas"]
        .as_array_mut()
        .unwrap()
        .push(repeated_schema);
    cases.push(reader_case(
        "duplicate-schema-identity",
        serde_json::to_vec(&duplicate_schema).unwrap(),
        false,
    ));

    let mut inconsistent_module = valid.clone();
    inconsistent_module["modules"][0]["version"] = "9.9.9".into();
    cases.push(reader_case(
        "inconsistent-module-premise",
        serde_json::to_vec(&inconsistent_module).unwrap(),
        false,
    ));

    let mut unknown_field = valid.clone();
    unknown_field["unexpected"] = true.into();
    cases.push(reader_case(
        "closed-envelope-refuses-unknown-field",
        serde_json::to_vec(&unknown_field).unwrap(),
        false,
    ));

    cases.push(reader_case("malformed-json", b"{".to_vec(), false));
    let mut trailing_garbage = GOLDEN.to_vec();
    trailing_garbage.extend_from_slice(b" garbage");
    cases.push(reader_case("trailing-garbage", trailing_garbage, false));
    let mut concatenated = GOLDEN.to_vec();
    concatenated.extend_from_slice(GOLDEN);
    cases.push(reader_case("concatenated-json", concatenated, false));

    let mut missing_required_field = valid.clone();
    missing_required_field
        .as_object_mut()
        .unwrap()
        .remove("format_version");
    cases.push(reader_case(
        "missing-required-format-version",
        serde_json::to_vec(&missing_required_field).unwrap(),
        false,
    ));

    let mut missing_nested_required_field = valid.clone();
    missing_nested_required_field["artifacts"][0]["locator"]
        .as_object_mut()
        .unwrap()
        .remove("path");
    cases.push(reader_case(
        "missing-required-locator-path",
        serde_json::to_vec(&missing_nested_required_field).unwrap(),
        false,
    ));

    let mut wrong_version_type = valid.clone();
    wrong_version_type["format_version"] = "1".into();
    cases.push(reader_case(
        "wrong-format-version-type",
        serde_json::to_vec(&wrong_version_type).unwrap(),
        false,
    ));

    let mut wrong_array_type = valid.clone();
    wrong_array_type["modules"] = serde_json::json!({});
    cases.push(reader_case(
        "wrong-modules-type",
        serde_json::to_vec(&wrong_array_type).unwrap(),
        false,
    ));

    let mut wrong_nested_type = valid.clone();
    wrong_nested_type["artifacts"][0]["locator"]["line"] = "1".into();
    cases.push(reader_case(
        "wrong-locator-line-type",
        serde_json::to_vec(&wrong_nested_type).unwrap(),
        false,
    ));

    let artifacts = valid["artifacts"].as_array_mut().expect("artifact array");
    for index in 0..512 {
        artifacts.push(serde_json::json!({
            "id": format!("bounded-artifact-{index:04}"),
            "artifact_type": "FR",
            "locator": {"path": "spec/FR-001.md", "line": 1}
        }));
    }
    cases.push(reader_case(
        "bounded-large-valid-export",
        serde_json::to_vec(&valid).unwrap(),
        true,
    ));

    let mut deep = serde_json::json!(0);
    for _ in 0..96 {
        deep = serde_json::json!([deep]);
    }
    let mut deeply_nested = serde_json::from_slice::<Value>(GOLDEN).unwrap();
    deeply_nested["unknown_nested"] = deep;
    cases.push(reader_case(
        "bounded-deep-unknown-value",
        serde_json::to_vec(&deeply_nested).unwrap(),
        false,
    ));

    for (name, bytes, expected) in &cases {
        assert_eq!(
            read_assurance_export(bytes, &accepted).is_ok(),
            *expected,
            "Rust reader outcome for {name}",
        );
    }
}

fn reader_case(
    name: &'static str,
    input: Vec<u8>,
    accepted: bool,
) -> (&'static str, Vec<u8>, bool) {
    (name, input, accepted)
}

fn assert_locator_keys(locator: &Value) {
    assert_keys(locator, &["path", "line"], "locator");
}

fn assert_keys(value: &Value, expected: &[&str], context: &str) {
    let actual: BTreeSet<&str> = value
        .as_object()
        .unwrap_or_else(|| panic!("{context} must be an object"))
        .keys()
        .map(String::as_str)
        .collect();
    let expected: BTreeSet<&str> = expected.iter().copied().collect();
    assert_eq!(actual, expected, "{context} v1 field set");
}

fn assert_schema_properties(definitions: &Value, definition: &str, expected: &[&str]) {
    assert_keys(
        &definitions[definition]["properties"],
        expected,
        &format!("{definition} schema properties"),
    );
}

fn collect_tokens(payload: &Value, field: &str) -> BTreeSet<String> {
    fn visit(value: &Value, field: &str, tokens: &mut BTreeSet<String>) {
        match value {
            Value::Object(object) => {
                if let Some(token) = object.get(field).and_then(Value::as_str) {
                    tokens.insert(token.to_owned());
                }
                for value in object.values() {
                    visit(value, field, tokens);
                }
            }
            Value::Array(values) => {
                for value in values {
                    visit(value, field, tokens);
                }
            }
            _ => {}
        }
    }

    let mut tokens = BTreeSet::new();
    visit(payload, field, &mut tokens);
    tokens
}

fn tokens(values: &[&str]) -> BTreeSet<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

fn enum_tokens(value: &Value) -> BTreeSet<String> {
    value
        .as_array()
        .expect("schema enum array")
        .iter()
        .map(|token| token.as_str().expect("schema state token").to_owned())
        .collect()
}
