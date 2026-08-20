//! `quire properties` ITs — acceptance-criteria property classification
//! (FR-018, upstream quire-rs FR-052).
//!
//! The command shipped with **no owning requirement in this repo and no test
//! file at all** (agent-ix/quire-cli#31), while its `--json` payload is the
//! interface the downstream `spec-correctness` work keys generated property
//! tests on. These cover the contract FR-018 states.

mod common;

use std::fs;

use tempfile::TempDir;

use common::quire;

/// A module with one `FR` archetype bound to the `ac` grammar, which is what
/// makes a document's Acceptance Criteria table bind criteria at all.
fn module(dir: &TempDir) -> String {
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n\
         - name: Note\n",
    )
    .expect("write manifest");
    m.to_string_lossy().into_owned()
}

fn write_doc(dir: &TempDir, name: &str, body: &str) -> String {
    let p = dir.path().join(name);
    fs::write(&p, body).expect("write doc");
    p.to_string_lossy().into_owned()
}

/// A universally quantified criterion (property-shaped) beside a bare example
/// one, so the classification has something to distinguish.
const FR_DOC: &str = "---\nid: FR-001\ntype: FR\n---\n# FR-001\n\n\
     ## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n\
     | FR-001-AC-1 | Every parsed document serializes back to its input byte-for-byte. |\n\
     | FR-001-AC-2 | Parsing the sample file yields two sections. |\n";

/// Every per-criterion record in the payload, flattened across documents. The
/// envelope is `{"documents": [{document, archetype, criteria: [...]}]}`.
fn criteria_of(payload: &serde_json::Value) -> Vec<serde_json::Value> {
    payload["documents"]
        .as_array()
        .map(|docs| {
            docs.iter()
                .flat_map(|d| d["criteria"].as_array().cloned().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default()
}

// IT-093, FR-018-AC-1: the default human census renders and exits 0.
#[test]
fn it093_human_census_renders_and_exits_zero() {
    let dir = TempDir::new().expect("tempdir");
    let m = module(&dir);
    let doc = write_doc(&dir, "FR-001.md", FR_DOC);

    let out = quire()
        .args(["properties", &doc, "--module", &m])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "classification is a report, never a verdict: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !String::from_utf8_lossy(&out.stderr).trim().is_empty(),
        "the census must render something"
    );
}

// IT-094, FR-018-AC-2: `--json` emits one record per binding criterion, each
// carrying `row_id` and a shape, and repeated runs are byte-identical. This is
// the payload `spec-correctness` keys its generated tests on.
#[test]
fn it094_json_records_carry_row_ids_and_are_deterministic() {
    let dir = TempDir::new().expect("tempdir");
    let m = module(&dir);
    let doc = write_doc(&dir, "FR-001.md", FR_DOC);

    let run = || {
        let o = quire()
            .args(["properties", &doc, "--module", &m, "--json"])
            .output()
            .expect("run");
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        String::from_utf8(o.stdout).expect("utf8")
    };
    let (a, b) = (run(), run());
    assert_eq!(a, b, "the payload must be byte-identical across runs");

    let payload: serde_json::Value = serde_json::from_str(&a).expect("valid JSON");
    let records = criteria_of(&payload);

    assert!(
        records.len() >= 2,
        "both criteria must be classified: {records:?}"
    );
    let ids: Vec<&str> = records
        .iter()
        .filter_map(|r| r["row_id"].as_str())
        .collect();
    assert!(
        ids.contains(&"FR-001-AC-1") && ids.contains(&"FR-001-AC-2"),
        "every record carries the criterion id the generator keys on: {ids:?}"
    );
}

// IT-095, FR-018-AC-3: a document whose archetype binds no criteria yields an
// empty record set and still exits 0 — "nothing to classify" is not an error.
#[test]
fn it095_a_document_binding_no_criteria_is_empty_and_succeeds() {
    let dir = TempDir::new().expect("tempdir");
    let m = module(&dir);
    let doc = write_doc(
        &dir,
        "NOTE-001.md",
        "---\nid: NOTE-001\ntype: Note\n---\n# note\n\n## Body\n\nprose.\n",
    );

    let out = quire()
        .args(["properties", &doc, "--module", &m, "--json"])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "a document with no criteria is not a failure: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let body = String::from_utf8_lossy(&out.stdout);
    let payload: serde_json::Value = serde_json::from_str(body.trim()).expect("valid JSON");
    let records = criteria_of(&payload);
    assert!(records.is_empty(), "expected no records: {records:?}");
}

// IT-096, FR-018-AC-4: a criterion the classifier cannot extract from is
// reported, never failed. FR-052-CON-1 forbids the shape classification from
// being addressable by the severity registry precisely so authors are not
// steered into rewording criteria to satisfy a checker.
#[test]
fn it096_unextractable_criteria_never_change_the_exit_code() {
    let dir = TempDir::new().expect("tempdir");
    let m = module(&dir);
    let doc = write_doc(
        &dir,
        "FR-002.md",
        "---\nid: FR-002\ntype: FR\n---\n# FR-002\n\n\
         ## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n\
         | FR-002-AC-1 | It works. |\n",
    );

    let out = quire()
        .args(["properties", &doc, "--module", &m, "--json"])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "classification has no failure mode: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("FR-002-AC-1"),
        "the criterion is still reported"
    );
}

// IT-097, FR-018-AC-5: path-safety applies before any load.
#[test]
fn it097_path_traversal_is_rejected() {
    let dir = TempDir::new().expect("tempdir");
    let m = module(&dir);

    let out = quire()
        .args(["properties", "../escape.md", "--module", &m])
        .output()
        .expect("run");
    assert!(!out.status.success(), "a `..` path must be refused");
}

// IT-098, FR-018-AC-7: an obligation source's `exclude:` binds THIS surface.
// (AC-6 is the thin-boundary Inspection criterion, traced by TC-090 — not this
// test. The citation read AC-6 while it was inert prose; binding it would have
// marked a criterion backed by a test that does not verify it.)
//
// quire-rs FR-053-AC-14. The engine honoured a source's `exclude:` globs in the
// coverage rollup and could not honour them here, because this crate never
// handed it the document's path — so a criterion in an excluded fixture stated
// no obligation in `coverage --json` and stated one in `properties --json`.
// That payload is what `spec-correctness` generates property tests from, so the
// asymmetry became a generated test carrying a trace tag for an id nothing
// mints — quire-rs#72's dead-tag failure through a new door.
#[test]
fn it098_excluded_document_states_no_obligation() {
    let dir = TempDir::new().expect("tempdir");
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n\
         \x20   id_column: ID\n  obligations:\n  - name: acceptance-criterion\n\
         \x20   target: acceptance-criterion\n    exclude:\n    - \"**/fixtures/**\"\n\
         \x20   statement_column: Criteria\n",
    )
    .expect("write manifest");
    let m = m.to_string_lossy().into_owned();

    let spec = dir.path().join("spec");
    fs::create_dir_all(spec.join("fixtures")).expect("mkdir");
    fs::write(spec.join("FR-001.md"), FR_DOC).expect("write");
    fs::write(
        spec.join("fixtures").join("FR-009.md"),
        FR_DOC.replace("FR-001", "FR-009"),
    )
    .expect("write");

    let obligations = |doc: &str| -> Vec<serde_json::Value> {
        let out = quire()
            .args([
                "properties",
                doc,
                "--module",
                &m,
                "--scope",
                &dir.path().to_string_lossy(),
                "--json",
            ])
            .output()
            .expect("run");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let payload: serde_json::Value = serde_json::from_slice(&out.stdout).expect("valid JSON");
        criteria_of(&payload)
            .into_iter()
            .map(|c| c["obligation"].clone())
            .collect()
    };

    let included = obligations(&spec.join("FR-001.md").to_string_lossy());
    assert!(
        included.iter().all(|o| !o.is_null()),
        "an included document still states its obligations: {included:#?}",
    );

    let excluded = obligations(&spec.join("fixtures/FR-009.md").to_string_lossy());
    assert!(
        !excluded.is_empty(),
        "the fixture document must still classify its criteria",
    );
    assert!(
        excluded.iter().all(|o| o.is_null()),
        "an excluded document must state no obligation on this surface either: {excluded:#?}",
    );
}
