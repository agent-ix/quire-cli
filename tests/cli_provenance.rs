//! FR-008-AC-6 (CR-104, #68) — every payload says which build produced it, and
//! `--version` names both versions distinctly.
//!
//! Driven through the **process boundary** rather than the library, because the
//! defect was never in a function: it was that a binary somebody installed
//! months ago printed a confident number and nothing about it said which engine
//! computed that number. The assertions below are the ones a reader picking up
//! a saved payload needs to be able to make.

mod common;

use common::{extract_module, extract_sample_doc, quire};
use serde_json::Value;

/// The provenance block out of a payload, with every member checked.
///
/// A helper rather than three copies, so a surface that grows a payload gets
/// the same assertions for one line — the way `coverage` and `properties`
/// diverged on what they rendered before #66.
fn assert_provenance(payload: &Value, surface: &str) {
    let engine = payload
        .get("engine")
        .unwrap_or_else(|| panic!("{surface}: no `engine` block: {payload}"));

    assert_eq!(
        engine["cli"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "{surface}: `engine.cli` must be this crate's version",
    );

    let version = engine["engine"]
        .as_str()
        .unwrap_or_else(|| panic!("{surface}: `engine.engine` is not a string: {engine}"));
    assert!(
        !version.is_empty() && version != "unknown",
        "{surface}: the engine version must resolve, got `{version}`",
    );
    // The whole point of reading the lockfile: the engine is a separate crate
    // on its own release cadence. Reporting this crate's version as the
    // engine's is the defect (#52, #68), not a harmless coincidence.
    assert_ne!(
        version,
        env!("CARGO_PKG_VERSION"),
        "{surface}: the CLI version is being reported as the engine version",
    );

    let capabilities = engine["capabilities"]
        .as_array()
        .unwrap_or_else(|| panic!("{surface}: `capabilities` is not an array: {engine}"));
    assert!(
        capabilities.iter().any(|t| t == "binding_census"),
        "{surface}: `binding_census` is the token this whole ticket is about: {engine}",
    );
    assert!(
        capabilities
            .iter()
            .any(|t| t == "binding_census.self_named"),
        "{surface}: mixed-channel self-name census capability is missing: {engine}",
    );
    assert!(
        capabilities.iter().any(|t| t == "binding_census.tagged"),
        "{surface}: tagged-versus-read census capability is missing: {engine}",
    );
    assert!(
        capabilities.iter().any(|t| t == "minted_targets"),
        "{surface}: minted-target row capability is missing: {engine}",
    );
    assert!(
        capabilities.iter().any(|t| t == "reference_only_targets"),
        "{surface}: reference-only target capability is missing: {engine}",
    );
    assert!(
        capabilities.iter().any(|t| t == "unmatched_tags"),
        "{surface}: unmatched authored-tag capability is missing: {engine}",
    );
    assert!(
        capabilities.iter().all(|t| t.is_string()),
        "{surface}: every capability must be a string token: {engine}",
    );

    // Closed envelope: three members and no more. A fourth would be a field
    // the published schemas reject, and the run that discovers that should be
    // this one rather than a consumer's.
    let members: Vec<&String> = engine.as_object().expect("object").keys().collect();
    assert_eq!(
        members.len(),
        3,
        "{surface}: unexpected members: {members:?}"
    );
}

// IT-123, FR-008-AC-6: `quire --version` reports the CLI and the engine
// version distinctly.
//
// `--version` reported `CARGO_PKG_VERSION` alone, so a binary linking a stale
// engine was indistinguishable from one linking a current engine. Asserting
// both strings appear is not enough on its own — if they happened to be equal
// the test would pass over the exact failure — so the label is asserted too.
#[test]
fn it_123_version_reports_the_cli_and_the_engine() {
    let out = quire().arg("--version").output().expect("--version runs");
    assert!(out.status.success(), "--version should exit 0");
    let line = String::from_utf8(out.stdout).expect("UTF-8");

    assert!(
        line.contains(env!("CARGO_PKG_VERSION")),
        "the CLI version is missing: {line}",
    );

    // The ENGINE version, by value. The first draft asserted only that the
    // word "engine" appeared — so mutating the format string to
    // `"(engine )"` left this test green and shipped a `--version` that named
    // no engine at all. That is the whole defect, passing its own gate.
    let engine = engine_revision_from_lockfile();
    assert!(
        line.contains(&engine[..8]),
        "`--version` must name the resolved engine revision `{engine}`: {line}",
    );
    assert!(
        line.contains("engine"),
        "the engine number must be labelled or the two are indistinguishable: {line}",
    );

    // One line: `quire --version` is scraped, and growing a second line breaks
    // every caller doing so.
    assert_eq!(line.trim().lines().count(), 1, "{line}");
}

/// The engine version this crate's lockfile resolves.
///
/// Read here, at the process boundary, rather than imported from the library:
/// the point is that the SHIPPED BINARY reports it, and a test that asked the
/// library for the expected value and then checked the library's own constant
/// would be comparing a value to itself.
fn engine_revision_from_lockfile() -> String {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
        .expect("Cargo.lock");
    quire_cli::lockfile::engine_source_revision(&lock).expect("quire-rs is a dependency")
}

fn engine_version_from_lockfile() -> String {
    let lock = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"))
        .expect("Cargo.lock");
    quire_cli::lockfile::engine_version(&lock).expect("quire-rs is a dependency")
}

/// Run `quire`, require exit 0, and parse stdout as the JSON payload.
///
/// The exit check lives here so no caller can omit it — IT-127's first draft
/// did, which made it vacuous against any failure: a crashed command produces
/// empty stdout, and "the empty string contains no banned key" is true.
fn payload_of(command: &mut std::process::Command, surface: &str) -> Value {
    raw_and_payload(command, surface).1
}

/// The emitted bytes **and** the parsed payload.
///
/// Key order can only be asserted on the bytes. `serde_json::Map` is a
/// `BTreeMap` unless the `preserve_order` feature is on, so parsing sorts every
/// object alphabetically — an assertion over a parsed `Value` measures serde's
/// map type, not what the command wrote, and passes identically whether or not
/// the emitter reorders anything.
fn raw_and_payload(command: &mut std::process::Command, surface: &str) -> (String, Value) {
    let out = command
        .output()
        .unwrap_or_else(|e| panic!("{surface}: {e}"));
    assert!(
        out.status.success(),
        "{surface} exited {}: {}",
        out.status,
        String::from_utf8_lossy(&out.stderr),
    );
    let raw = String::from_utf8(out.stdout).expect("stdout is UTF-8");
    let parsed = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("{surface}: stdout is not valid JSON: {e}\n{raw}"));
    (raw, parsed)
}

/// Assert `keys` appear in the emitted bytes in exactly this order.
///
/// Position-based, for the reason above. Each key is matched with its quotes
/// and trailing colon so a key name occurring inside a string VALUE cannot be
/// mistaken for the key.
fn assert_key_order(raw: &str, keys: &[&str], surface: &str) {
    let mut previous = 0usize;
    for key in keys {
        let needle = format!("\"{key}\":");
        let at = raw
            .find(&needle)
            .unwrap_or_else(|| panic!("{surface}: no `{key}` key in output:\n{raw}"));
        assert!(
            at >= previous,
            "{surface}: `{key}` is out of order — provenance must be APPENDED and \
             the engine's own key order left untouched:\n{raw}",
        );
        previous = at;
    }
}

fn extract_payload() -> Value {
    payload_of(
        quire()
            .arg("extract")
            .arg(extract_sample_doc())
            .arg("--module")
            .arg(extract_module()),
        "extract",
    )
}

// IT-124, FR-008-AC-6: the `extract` payload carries provenance.
#[test]
fn it_124_extract_payload_carries_provenance() {
    let (raw, payload) = raw_and_payload(
        quire()
            .arg("extract")
            .arg(extract_sample_doc())
            .arg("--module")
            .arg(extract_module()),
        "extract",
    );
    assert_provenance(&payload, "extract");

    // FR-008-AC-4: the engine's values keep their own declaration order —
    // provenance is APPENDED. The first implementation round-tripped through
    // `serde_json::Value`, whose `Map` is a `BTreeMap`, and silently sorted
    // every key at every depth; output stayed byte-identical across runs, so
    // nothing failed while every checked-in payload in the ecosystem would
    // have shown a 100%-changed diff carrying no content change.
    assert_key_order(&raw, &["extraction", "edges", "engine"], "extract");

    // The engine's own values are untouched — provenance rides the envelope
    // (FR-008 behaviour rule 5), it does not annotate records.
    assert!(payload["extraction"].is_object(), "{payload}");
    assert!(payload["edges"].is_array(), "{payload}");
}

// IT-125, FR-008-AC-6: the `properties` payload carries provenance.
//
// This is the payload quoin's `spec-correctness` skill reads and a generated
// property test is derived from. A classification whose classifier version is
// unknowable is a classification nobody can re-derive.
#[test]
fn it_125_properties_payload_carries_provenance() {
    let payload = properties_payload();
    assert_provenance(&payload, "properties");

    // Non-vacuous: the fixture must actually have classified something, or the
    // provenance assertions ride on an empty payload.
    assert_eq!(
        payload["documents"][0]["criteria"]
            .as_array()
            .expect("criteria")
            .len(),
        1,
        "{payload}",
    );
}

// IT-126, FR-008-AC-6: the `coverage` payload carries provenance.
//
// The surface that matters most: every ecosystem figure in four battletest
// passes came from this payload, and none of them recorded which engine
// produced it.
#[test]
fn it_126_coverage_payload_carries_provenance() {
    let (raw, payload) = coverage_raw_and_payload();
    assert_provenance(&payload, "coverage");

    // Non-vacuous: the model must have matched a row, or `totals` is the
    // 0/0 state and this asserts provenance over a payload about nothing.
    assert_eq!(payload["totals"]["total"], 1, "{payload}");

    // FR-008-AC-4 on the payload that matters most: `CoverageReport`'s own
    // declaration order survives and `engine` lands after it. Alphabetically
    // `engine` would sit between `criteria` and `groups`, and `unbacked_rows`
    // would be last — so this ordering cannot be produced by a sorted map.
    assert_key_order(
        &raw,
        &["unbacked_rows", "status_lies", "groups", "totals", "engine"],
        "coverage",
    );
}

/// A module + document that classify one criterion, as a payload.
fn properties_payload() -> Value {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let module = dir.path().join("m");
    std::fs::create_dir_all(&module).expect("mkdir");
    std::fs::write(
        module.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n",
    )
    .expect("write manifest");
    let doc = dir.path().join("FR-001.md");
    std::fs::write(
        &doc,
        "---\nid: FR-001\ntype: FR\n---\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | Every finding absent from the merged map defaults to warning. | Test |\n",
    )
    .expect("write doc");

    let payload = payload_of(
        quire().args([
            "properties",
            &doc.to_string_lossy(),
            "--module",
            &module.to_string_lossy(),
            "--json",
        ]),
        "properties",
    );
    // Non-vacuous: the fixture must actually have classified something, or
    // every caller's assertions ride on an empty payload.
    assert_eq!(
        payload["documents"][0]["criteria"]
            .as_array()
            .expect("criteria")
            .len(),
        1,
        "{payload}",
    );
    payload
}

/// A scope whose declared model matches exactly one row, as a payload.
fn coverage_payload() -> Value {
    coverage_raw_and_payload().1
}

fn coverage_raw_and_payload() -> (String, Value) {
    let dir = tempfile::TempDir::new().expect("tempdir");
    let scope = dir.path();
    std::fs::create_dir_all(scope.join("spec")).expect("mkdir spec");
    std::fs::write(
        scope.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n    id_column: ID\n",
    )
    .expect("write manifest");
    std::fs::write(
        scope.join("spec/FR-001.md"),
        "---\nid: FR-001\ntype: FR\n---\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | It does the thing. | Test |\n",
    )
    .expect("write doc");

    raw_and_payload(
        quire().args(["coverage", "--scope", &scope.to_string_lossy(), "--json"]),
        "coverage",
    )
}

// IT-127, FR-008-AC-5 (as narrowed by CR-104): the bare-version ban still
// holds on every payload.
//
// The half of AC-5 that survives. A loose `version` lets a payload assert its
// own contract revision, which puts the contract in two places; provenance
// under a named `engine` object is a different claim.
#[test]
fn it_127_no_payload_carries_a_bare_version_key() {
    // Every payload, not just `extract`. The first draft ran one command while
    // the matrix row claimed "no payload" — so a `version` key added to the
    // coverage envelope would have left this green and the matrix reporting
    // AC-5 covered.
    for payload in [extract_payload(), properties_payload(), coverage_payload()] {
        let mut found = Vec::new();
        collect_banned_keys(&payload, &mut found);
        assert!(
            found.is_empty(),
            "a payload names the contract revision it claims to conform to \
             (FR-055-CON-2): {found:?} in {payload}",
        );
    }
}

/// Every banned key name appearing anywhere in a payload, at any depth.
///
/// A key WALK, not a substring grep over the rendered bytes. The first draft
/// checked `body.contains("\"version\"")`, which both over- and under-fires: a
/// record whose extracted *value* is the text `"version"` failed a test it did
/// not violate, and a key at depth was caught only by luck of quoting.
fn collect_banned_keys(node: &Value, out: &mut Vec<String>) {
    match node {
        Value::Object(map) => {
            for (key, child) in map {
                if matches!(key.as_str(), "version" | "schema_version" | "$schema") {
                    out.push(key.clone());
                }
                collect_banned_keys(child, out);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_banned_keys(item, out);
            }
        }
        _ => {}
    }
}

// IT-129, FR-008-AC-6 (#68 AC-4, extending #60): the envelope SHAPE is pinned
// by a golden snapshot.
//
// #60 established golden snapshots for the stream contract; the envelope had
// none, so every assertion about it was a hand-written `assert!` that only
// checks what somebody thought to name. A snapshot fails on anything that
// moves — a key added, a key reordered, a key silently dropped.
//
// **The version VALUES are redacted, the structure is not.** A snapshot
// carrying `0.45.0` would have to be regenerated on every engine bump, which
// trains a reader to regenerate it without reading it — and a golden file
// nobody reads is the gate that let #52 ship four binaries reporting 0.23.0.
// What must not drift is the shape: which keys, in which order.
#[test]
fn it_129_the_envelope_shape_is_pinned_by_a_golden_snapshot() {
    let (raw, _) = raw_and_payload(
        quire()
            .arg("extract")
            .arg(extract_sample_doc())
            .arg("--module")
            .arg(extract_module())
            .arg("--pretty"),
        "extract",
    );

    let redacted = raw
        .replace(env!("CARGO_PKG_VERSION"), "<cli>")
        .replace(&engine_version_from_lockfile(), "<engine>");
    let snapshot = include_str!("snapshots/extract-envelope.json");

    assert_eq!(
        redacted.trim_end(),
        snapshot.trim_end(),
        "the extract envelope drifted from tests/snapshots/extract-envelope.json.\n\
         Read the diff before regenerating: this file exists so a key that moves, \
         appears or vanishes cannot land unnoticed.",
    );

    // Non-vacuous: the snapshot must actually carry the redaction markers, or a
    // regeneration that baked in literal versions would silently pass forever.
    assert!(
        snapshot.contains("<cli>"),
        "the snapshot lost its redaction"
    );
    assert!(
        snapshot.contains("<engine>"),
        "the snapshot lost its redaction"
    );
}
