//! The stdout/stderr contract (FR-006-AC-5 / FR-018-AC-10, CR-012) —
//! `agent-ix/quire-cli#59`, `agent-ix/quire-cli#60`.
//!
//! **Results on stdout, diagnostics on stderr.** Before CR-012 every human
//! surface went through `write_diagnostic_human`, which is `eprintln!` wrapped
//! in `RED`. Measured over `agent-ix/filament-ide-rs`:
//!
//! ```text
//! quire coverage --scope . > out.txt                 # 0 bytes on stdout
//! quire validate --scope . "spec/**/*.md" > out.txt  # 0 bytes on stdout
//! quire properties --scope . 'spec/**/*.md' > out.txt # 0 bytes
//! ```
//!
//! So the obvious command produced an empty file, and
//! `Coverage: N/M rows backed (P%)` — a census — rendered in the same
//! red as every finding.
//!
//! These tests assert the split by **stream**, not by byte-for-byte snapshot of
//! the whole surface: a golden of every line would fail on any wording change
//! and teach nobody which side of the contract broke. What is pinned is the
//! property the defect violated.

mod common;

use ix_trace_rs::trace;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use tempfile::TempDir;

fn quire() -> Command {
    Command::new(env!("CARGO_BIN_EXE_quire"))
}

/// A bundle with one FR, a matrix row nothing backs, and a source tree — so
/// coverage has both a census to report and a finding to report.
fn bundle(dir: &Path) -> (String, String) {
    let module = dir.join("m");
    let spec = dir.join("spec");
    fs::create_dir_all(&module).expect("mkdir");
    fs::create_dir_all(&spec).expect("mkdir");
    fs::write(module.join("manifest.yaml"), MANIFEST).expect("write manifest");
    fs::write(
        spec.join("FR-001.md"),
        "---\nid: FR-001\ntype: FR\n---\n\n## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | Every finding shall default to warning. | Test (TC-001) |\n\
         | FR-001-AC-2 | Every finding whose key is absent from the merged map never defaults to warning. | Test (TC-002) |\n\
         | FR-001-AC-3 | It works. | Test (TC-003) |\n",
    )
    .expect("write fr");
    fs::write(
        spec.join("tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n\n## Test Cases\n\n\
         | ID | Traces To | Status |\n|----|-----------|--------|\n\
         | TC-001 | FR-001-AC-1 | 🚧 |\n\
         | TC-003 | FR-001-AC-3 | 🚧 |\n",
    )
    .expect("write matrix");
    (
        dir.to_string_lossy().into_owned(),
        module.to_string_lossy().into_owned(),
    )
}

const MANIFEST: &str = r#"
name: m
manifest_version: 1.0.0
version: 0.0.1
artifact_types:
- name: FR
  grammar_ref: iso-spec-core
- name: TestMatrix
traceability:
  trace_targets:
  - name: acceptance-criterion
    archetype: FR
    section: Acceptance Criteria
    id_column: ID
  document_references:
  - name: traces-to
    archetype: TestMatrix
    section: Test Cases
    row_id_column: ID
    column: Traces To
    targets: [acceptance-criterion]
    pattern: '([A-Z]{2,4}-\d+(?:-[A-Z]{2,4}-\d+)?)'
  status:
    column: Status
    complete: ["\u2705"]
    pending: ["\U0001F6A7"]
  trace_tags:
    markers:
    - name: rust-trace-attribute
      language: rust
      pattern: '#\[trace\(([^)]*)\)\]'
      template: '#[trace({ids})]'
"#;

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

/// IT-117 (CR-012): `coverage`'s census reaches stdout and its findings do not.
#[trace("FR-006-AC-5", "FR-017-AC-1")]
#[test]
fn it_117_coverage_puts_the_census_on_stdout_and_findings_on_stderr() {
    let dir = TempDir::new().expect("tempdir");
    let (scope, module) = bundle(dir.path());

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &module])
        .output()
        .expect("run");
    assert!(out.status.success(), "{}", stderr(&out));

    // The thing a caller redirecting with `>` came for.
    let o = stdout(&out);
    assert!(
        o.contains("rows backed"),
        "the census must reach stdout; stdout was {o:?}"
    );
    assert!(
        o.contains("FR-001.md"),
        "per-document count is present: {o:?}"
    );
    assert!(
        !o.is_empty(),
        "`quire coverage > out.txt` produced a 0-byte file before CR-012"
    );

    // …and a finding is a diagnostic, so it does not.
    let e = stderr(&out);
    assert!(
        !o.contains("has no backing symbol"),
        "a finding is a diagnostic, not a result: {o:?}"
    );
    assert!(
        e.contains("has no backing symbol"),
        "the unbacked row must still be reported, on stderr: {e:?}"
    );

    // The census carries no ANSI escape: a number is not a severity, and the
    // defect was a census rendered in error red.
    assert!(
        !o.contains('\u{1b}'),
        "the result channel is never colorized: {o:?}"
    );
}

/// IT-118 (CR-012): `properties`' census reaches stdout, and leads with the
/// specific-shape split rather than the catch-all-inflated figure alone
/// (quire-rs CR-095).
#[trace("FR-006-AC-5")]
#[test]
fn it_118_properties_census_is_a_result_and_splits_the_catch_all() {
    let dir = TempDir::new().expect("tempdir");
    let (scope, module) = bundle(dir.path());
    let fr = dir.path().join("spec/FR-001.md");

    let out = quire()
        .args([
            "properties",
            &fr.to_string_lossy(),
            "--scope",
            &scope,
            "--module",
            &module,
        ])
        .output()
        .expect("run");
    assert!(out.status.success(), "{}", stderr(&out));

    let o = stdout(&out);
    assert!(o.contains("criteria extractable"), "{o:?}");
    assert!(
        o.contains("with a specific shape"),
        "the honest half of the headline travels with the other one: {o:?}"
    );
    assert!(!o.contains('\u{1b}'), "the result is not colorized: {o:?}");
}

/// IT-119 (#59 defect 2): `--criteria` renders the per-criterion fields
/// `spec-correctness` consumes, which were `--json`-only — hundreds of kilobytes on the
/// pass-2 corpus against a few-line census.
#[trace("FR-018-AC-10")]
#[test]
fn it_119_criteria_renders_the_fields_the_census_omitted() {
    let dir = TempDir::new().expect("tempdir");
    let (scope, module) = bundle(dir.path());
    let fr = dir.path().join("spec/FR-001.md");

    let run = |extra: &[&str]| {
        let mut args = vec![
            "properties".to_string(),
            fr.to_string_lossy().into_owned(),
            "--scope".into(),
            scope.clone(),
            "--module".into(),
            module.clone(),
        ];
        args.extend(extra.iter().map(|s| (*s).to_string()));
        let out = quire().args(&args).output().expect("run");
        assert!(out.status.success(), "{}", stderr(&out));
        stdout(&out)
    };

    let census_only = run(&[]);
    let with_criteria = run(&["--criteria"]);
    assert!(
        with_criteria.len() > census_only.len(),
        "`--criteria` adds the per-criterion blocks"
    );
    // AC-2 is the specifically-shaped one; AC-1 is `universal` and is not in
    // the default set, which is the point of the default.
    assert!(
        with_criteria.contains("FR-001-AC-2"),
        "each block leads with the row id: {with_criteria:?}"
    );
    assert!(
        with_criteria.contains("FR-001.md:"),
        "each block carries its document and line locus: {with_criteria:?}"
    );
    assert!(
        with_criteria.contains("invariant [extractable]"),
        "each block carries its shape and extraction state: {with_criteria:?}"
    );
    assert!(
        !with_criteria.contains("FR-001-AC-1 ("),
        "the catch-all is not in the actionable set: {with_criteria:?}"
    );
    // …and `--all` includes it.
    let everything = run(&["--criteria", "--all"]);
    assert!(everything.contains("FR-001-AC-1 ("), "{everything:?}");
    assert!(everything.contains("FR-001-AC-3 ("), "{everything:?}");
    for empty_span in ["  domain: \n", "  precondition: \n", "  oracle: \n"] {
        assert!(!everything.contains(empty_span), "{everything:?}");
    }
    let all_without_criteria = quire()
        .args([
            "properties",
            &fr.to_string_lossy(),
            "--scope",
            &scope,
            "--module",
            &module,
            "--all",
        ])
        .output()
        .expect("run invalid all-only options");
    assert_eq!(all_without_criteria.status.code(), Some(3));
    let option_error = stderr(&all_without_criteria);
    assert!(option_error.contains("--criteria"), "{option_error:?}");
    assert!(
        with_criteria.contains("domain:"),
        "and carries the spans a generator grounds on: {with_criteria:?}"
    );
    // The census is unchanged by the flag — one surface gained a section, it
    // did not become a different report.
    let first = census_only.lines().next().unwrap_or_default();
    assert!(with_criteria.starts_with(first), "{with_criteria:?}");
}

/// IT-120 (#60): the diagnostics channel is actually exercised, and by
/// something other than the one class that dominated pass 2.
///
/// In that pass all 33 diagnostics were `uncatalogued-verification-method`
/// while unmatched trace symbols went unmentioned — so "diagnostics is
/// non-empty" was true and meant nothing. This asserts a **binding** finding
/// specifically, the class that was silent.
#[test]
fn it_120_the_diagnostics_channel_reports_a_binder_that_read_nothing() {
    let dir = TempDir::new().expect("tempdir");
    let (scope, module) = bundle(dir.path());
    // A test carrying a marker spelling the module never declared: real tests,
    // real tags, nothing bound.
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("mkdir");
    fs::write(
        src.join("lib.rs"),
        "//! fixture\n\n#[cfg(test)]\nmod tests {\n    #[tracks(\"TC-001\")]\n    \
         #[test]\n    fn covers() {\n        let _ = 1;\n    }\n}\n",
    )
    .expect("write src");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &module, "--json"])
        .output()
        .expect("run");
    let payload: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json on stdout");

    let census = &payload["binding_census"];
    assert!(
        census.is_array() && !census.as_array().expect("array").is_empty(),
        "the premise is carried whether or not it holds: {payload}"
    );
    let rust = census
        .as_array()
        .expect("array")
        .iter()
        .find(|c| c["language"] == "rust")
        .expect("a rust census");
    assert!(rust["candidates"].as_u64().expect("candidates") > 0);
    assert_eq!(rust["bound"], 0, "nothing bound");

    let reasons: Vec<&str> = payload["diagnostics"]
        .as_array()
        .map(|d| d.iter().filter_map(|x| x["reason"].as_str()).collect())
        .unwrap_or_default();
    assert!(
        reasons.contains(&"no-symbol-bound"),
        "the class that was silent in pass 2 now fires: {reasons:?}"
    );
}

/// This case covers the three human census surfaces together so a summary
/// cannot silently move back to stderr or render as a diagnostic.
#[trace("FR-006-AC-5")]
#[test]
fn human_census_figures_stay_on_stdout_without_color() {
    let dir = TempDir::new().expect("tempdir");
    let (scope, module) = bundle(dir.path());
    let fr = dir.path().join("spec/FR-001.md");

    let coverage = quire()
        .args(["coverage", "--scope", &scope, "--module", &module])
        .output()
        .expect("run coverage");
    assert!(coverage.status.success(), "{}", stderr(&coverage));
    let coverage_stdout = stdout(&coverage);
    assert!(
        coverage_stdout.contains("rows backed"),
        "{coverage_stdout:?}"
    );
    assert!(coverage_stdout.contains("FR-001"), "{coverage_stdout:?}");
    assert!(
        !coverage_stdout.contains('\u{1b}'),
        "coverage census is not colorized: {coverage_stdout:?}"
    );
    assert!(
        stderr(&coverage).contains("has no backing symbol"),
        "coverage findings stay on stderr: {}",
        stderr(&coverage)
    );

    let properties = quire()
        .args([
            "properties",
            &fr.to_string_lossy(),
            "--scope",
            &scope,
            "--module",
            &module,
        ])
        .output()
        .expect("run properties");
    assert!(properties.status.success(), "{}", stderr(&properties));
    let properties_stdout = stdout(&properties);
    assert!(
        properties_stdout.contains("criteria extractable"),
        "{properties_stdout:?}"
    );
    assert!(
        !properties_stdout.contains('\u{1b}'),
        "properties census is not colorized: {properties_stdout:?}"
    );

    let ears_dir = tempfile::tempdir().expect("tempdir");
    let validate_mod = common::ears_module(ears_dir.path());
    let ears_doc = common::validate_doc("ears-warn-fr.md");
    let validate = quire()
        .arg("validate")
        .arg(&ears_doc)
        .arg("--module")
        .arg(&validate_mod)
        .arg("--summary")
        .output()
        .expect("run validate");
    assert!(validate.status.success(), "{}", stderr(&validate));
    let validate_stdout = stdout(&validate);
    assert!(
        validate_stdout.contains("docs grammar-clean"),
        "{validate_stdout:?}"
    );
    assert!(
        !validate_stdout.contains('\u{1b}'),
        "validate censuses are not colorized: {validate_stdout:?}"
    );
    assert!(
        stderr(&validate).contains("[ears:vague-response]"),
        "validation warnings stay on stderr: {}",
        stderr(&validate)
    );
}
