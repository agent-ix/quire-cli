//! FR-020: released assurance producer shapes are not interchangeable by label.

use ix_trace_rs::trace;
use quire_rs::assurance::{
    read_assurance_export, AcceptedAssurancePremises, AssuranceError, AssuranceExport,
    ASSURANCE_V1_SCHEMA,
};
use serde_json::Value;

const LEGACY_032_046: &[u8] =
    include_bytes!("fixtures/assurance/compat/quire-cli-0.32.0-quire-rs-0.46.0.json");
const LEGACY_033_047: &[u8] =
    include_bytes!("fixtures/assurance/compat/quire-cli-0.33.0-quire-rs-0.47.1.json");
const CURRENT_052_062: &[u8] = include_bytes!("fixtures/assurance/v1.json");

fn parse(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("released payload is valid JSON")
}

fn ids(value: &Value, collection: &str) -> Vec<String> {
    value[collection]
        .as_array()
        .expect("record collection")
        .iter()
        .map(|record| record["id"].as_str().expect("record identity").to_string())
        .collect()
}

fn relation_identities(value: &Value) -> Vec<String> {
    value["relations"]
        .as_array()
        .expect("relations")
        .iter()
        .map(|relation| {
            ["kind", "source", "target", "edge_type", "resolution"]
                .into_iter()
                .map(|field| relation[field].as_str().unwrap_or("-"))
                .collect::<Vec<_>>()
                .join("|")
        })
        .collect()
}

fn observation_identities(value: &Value) -> Vec<String> {
    value["relation_observations"]
        .as_array()
        .expect("relation observations")
        .iter()
        .map(|observation| {
            [
                "declaration",
                "subject",
                "availability",
                "freshness",
                "reason",
            ]
            .into_iter()
            .map(|field| observation[field].as_str().unwrap_or("-"))
            .collect::<Vec<_>>()
            .join("|")
        })
        .collect()
}

fn symbol_identities(value: &Value) -> Vec<String> {
    value["symbols"]
        .as_array()
        .expect("symbols")
        .iter()
        .map(|symbol| {
            format!(
                "{}|{}|{}|{}|{}|{}|{}",
                symbol["language"].as_str().expect("language"),
                symbol["kind"].as_str().expect("kind"),
                symbol["qualified_name"].as_str().expect("qualified name"),
                symbol["container"].as_str().unwrap_or("-"),
                symbol["capabilities"][0].as_str().expect("capability"),
                symbol["locator"]["path"].as_str().expect("symbol path"),
                symbol["locator"]["line"].as_u64().expect("symbol line")
            )
        })
        .collect()
}

fn assert_record_population(value: &Value) {
    assert_eq!(value["format"], "quire-assurance");
    assert_eq!(value["format_version"], 1);
    assert_eq!(
        value["modules"][0]["name"], "assurance-fixture",
        "the module identity is part of the compatibility oracle"
    );
    assert_eq!(value["modules"][0]["version"], "1.2.3");
    assert_eq!(
        value["modules"][0]["schemas"]
            .as_array()
            .expect("schemas")
            .iter()
            .map(|schema| schema["archetype"].as_str().expect("archetype"))
            .collect::<Vec<_>>(),
        ["FR", "NFR", "StR"]
    );
    assert_eq!(ids(value, "artifacts"), ["FR-001", "FR-002", "StR-001"]);
    assert_eq!(
        value["artifacts"]
            .as_array()
            .expect("artifacts")
            .iter()
            .map(|artifact| format!(
                "{}|{}|{}|{}",
                artifact["id"].as_str().expect("artifact id"),
                artifact["artifact_type"].as_str().expect("artifact type"),
                artifact["locator"]["path"].as_str().expect("artifact path"),
                artifact["locator"]["line"].as_u64().expect("artifact line")
            ))
            .collect::<Vec<_>>(),
        [
            "FR-001|FR|FR-001.md|1",
            "FR-002|FR|FR-002.md|1",
            "StR-001|StR|StR-001.md|1"
        ]
    );
    assert_eq!(ids(value, "obligations"), ["FR-001-AC-1", "FR-002-AC-1"]);
    assert_eq!(
        value["obligations"]
            .as_array()
            .expect("obligations")
            .iter()
            .map(|obligation| format!(
                "{}|{}|{}|{}|{}|{}|{}",
                obligation["id"].as_str().expect("obligation id"),
                obligation["source"].as_str().expect("obligation source"),
                obligation["document"].as_str().expect("obligation document"),
                obligation["statement"].as_str().expect("statement"),
                obligation["method"].as_str().expect("method"),
                obligation["target_ids"][0].as_str().unwrap_or("-"),
                obligation["locator"]["path"].as_str().expect("obligation path")
            ))
            .collect::<Vec<_>>(),
        [
            "FR-001-AC-1|acceptance-criterion|FR-001.md|The assurance export preserves the linked requirement.|Test|TC-136|FR-001.md",
            "FR-002-AC-1|acceptance-criterion|FR-002.md|The assurance export preserves the missing relation state.|Inspection|-|FR-002.md"
        ]
    );
    assert_eq!(
        value["symbols"]
            .as_array()
            .expect("symbols")
            .iter()
            .map(|symbol| symbol["qualified_name"].as_str().expect("symbol name"))
            .collect::<Vec<_>>(),
        ["evidence_binding", "production_binding"]
    );
    assert_eq!(
        symbol_identities(value),
        [
            "rust|test_function|evidence_binding|-|verifies|src/lib.rs|6",
            "rust|function|production_binding|-|implements|src/lib.rs|2"
        ]
    );
    assert_eq!(
        relation_identities(value),
        [
            "corpus|FR-001|StR-001|references|resolved",
            "corpus|FR-001|StR-001|satisfies|resolved",
            "corpus|FR-001|StR-999|references|dangling",
            "verifies|0ebc7940c7d308a4e705773d389b3e9a386b173132faa2ae19c4248dc7f9ba76|FR-001-AC-1|-|-",
            "verifies|0ebc7940c7d308a4e705773d389b3e9a386b173132faa2ae19c4248dc7f9ba76|TC-136|-|-",
            "implements|a6912a41c300742ae2b687af3ad18dd589940ee5a6a5e4fb7395090ae4c805e4|FR-001|-|-"
        ]
    );
    assert_eq!(
        observation_identities(value),
        [
            "fr-has-need|FR-001|available|not_applicable|-",
            "fr-has-need|FR-002|missing|not_applicable|-",
            "fr-has-need|unreadable.md|unknown|unknown|stream did not contain valid UTF-8",
            "nfr-has-owner|-|not_applicable|not_applicable|-",
            "nfr-has-owner|unreadable.md|unknown|unknown|stream did not contain valid UTF-8"
        ]
    );
}

#[trace("FR-020-AC-9")]
#[test]
fn it_145_released_payloads_preserve_their_producer_identity_but_current_reader_fails_closed() {
    let legacy_032 = parse(LEGACY_032_046);
    let legacy_033 = parse(LEGACY_033_047);
    let current = parse(CURRENT_052_062);

    // The historical releases had the same schema and fixture inputs, and
    // their separately captured payloads therefore have identical bytes.
    assert_eq!(LEGACY_032_046, LEGACY_033_047);
    assert_record_population(&legacy_032);
    assert_record_population(&legacy_033);
    assert_record_population(&current);

    // Check representative identities and provenance fields whose meanings
    // changed while the outer format_version stayed at 1.
    assert_eq!(
        legacy_032["source"],
        serde_json::json!({
            "repository": "agent-ix/fixture",
            "revision": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        })
    );
    assert_eq!(
        legacy_032["modules"][0]["schemas"][0]["schema_digest"],
        "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
    );
    assert!(legacy_032["artifacts"][0]["locator"]
        .get("digest")
        .is_some());
    assert!(legacy_032["obligations"][0].get("statement_hash").is_some());
    assert_eq!(
        legacy_032["symbols"][0]["id"],
        "0ebc7940c7d308a4e705773d389b3e9a386b173132faa2ae19c4248dc7f9ba76"
    );
    assert_eq!(
        legacy_032["symbols"][1]["id"],
        "a6912a41c300742ae2b687af3ad18dd589940ee5a6a5e4fb7395090ae4c805e4"
    );
    assert!(legacy_032["relations"]
        .as_array()
        .expect("relations")
        .iter()
        .any(|relation| relation["kind"] == "verifies" && relation["provenance"] == "canonical"));

    assert!(current.get("source").is_none());
    assert!(current["modules"][0]["schemas"][0]
        .get("schema_digest")
        .is_none());
    assert!(current["artifacts"][0]["locator"].get("digest").is_none());
    assert!(current["obligations"][0].get("statement_hash").is_none());
    assert_eq!(
        current["symbols"][0]["id"],
        "[\"rust\",\"src/lib.rs\",\"evidence_binding\",\"test_function\"]"
    );
    assert_eq!(
        current["symbols"][1]["id"],
        "[\"rust\",\"src/lib.rs\",\"production_binding\",\"function\"]"
    );

    let current_export: AssuranceExport =
        serde_json::from_slice(CURRENT_052_062).expect("current producer type");
    let accepted = AcceptedAssurancePremises::from_export(&current_export);
    assert_eq!(
        read_assurance_export(CURRENT_052_062, &accepted).expect("current shape accepted"),
        current_export
    );

    // Both historical producers advertise the same format/version and the
    // same broad capability as the current pair. The current reader still
    // refuses their incompatible schema before it returns typed records.
    assert!(quire_cli::engine::CAPABILITIES.contains(&"assurance_export.v1"));
    for legacy in [LEGACY_032_046, LEGACY_033_047] {
        let value = parse(legacy);
        assert_eq!(value["format_version"], current["format_version"]);
        assert!(matches!(
            read_assurance_export(legacy, &accepted),
            Err(AssuranceError::SchemaViolation { .. })
        ));
    }

    let current_schema: Value = serde_json::from_str(ASSURANCE_V1_SCHEMA).expect("schema JSON");
    assert_eq!(
        current_schema["$id"],
        "https://agent-ix.github.io/quire-rs/schemas/output/assurance-v1.schema.json"
    );
}
