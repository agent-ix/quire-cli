//! `quire coverage` + the release-coupled `validate` surfaces (quire-cli#11).
//!
//! The `--summary` prefix, the `--severity` override and the `coverage`
//! rollup. Trace ids sit on the tests, not in this header — a `//!` block
//! attaches to the file and binds to no symbol (agent-ix/quire-cli#43).

use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn quire() -> Command {
    Command::new(env!("CARGO_BIN_EXE_quire"))
}

/// A module declaring two grammars' worth of checks via `iso-spec-core`, so a
/// single FR emits both `[ears:*]` and `[ac:*]` findings.
fn module(dir: &TempDir) -> String {
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n",
    )
    .expect("write manifest");
    m.to_string_lossy().into_owned()
}

/// An FR whose Description trips an EARS check and whose acceptance criterion
/// trips an `ac` check.
fn doc(dir: &TempDir) -> String {
    let p = dir.path().join("FR-001.md");
    fs::write(
        &p,
        "---\nid: FR-001\ntype: FR\n---\n\
         ## Description\n\nshall process the input.\n\n\
         ## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n\
         | FR-001-AC-1 | The system shall support pagination. | Test |\n",
    )
    .expect("write doc");
    p.to_string_lossy().into_owned()
}

/// TC-714, FR-004-AC-15 (upstream quire-rs FR-047-AC-8): the `--summary`
/// histogram groups by the generic
/// `[<grammar>:<check>]` prefix, so a corpus emitting both `[ears:*]` and
/// `[ac:*]` findings shows both. Before this the parser matched a hardcoded
/// `[ears:` prefix and every `ac` finding was silently absent.
#[test]
fn tc714_summary_covers_every_grammar() {
    let dir = TempDir::new().expect("tempdir");
    let out = quire()
        .args([
            "validate",
            &doc(&dir),
            "--module",
            &module(&dir),
            "--summary",
        ])
        .output()
        .expect("run");
    let err = String::from_utf8_lossy(&out.stderr);
    let summary = err
        .lines()
        .find(|l| l.contains("docs grammar-clean"))
        .unwrap_or_else(|| panic!("no summary line in:\n{err}"));

    assert!(summary.contains("ac:"), "ac findings missing: {summary}");
    assert!(
        summary.contains("ears:"),
        "ears findings missing: {summary}"
    );
    // Grammar-neutral wording: naming EARS would be wrong once `ac` fires.
    assert!(
        summary.contains("grammar finding(s)") && !summary.contains("EARS finding"),
        "summary still names one grammar: {summary}"
    );
}

/// TC-720, FR-004-AC-16 (upstream quire-rs TC-752, FR-048-AC-5/AC-9):
/// `--severity <grammar>:<check>=off` suppresses the check entirely — no
/// warning, and no row in the histogram.
#[test]
fn tc720_severity_off_suppresses_a_check() {
    let dir = TempDir::new().expect("tempdir");
    let (d, m) = (doc(&dir), module(&dir));

    let before = quire()
        .args(["validate", &d, "--module", &m, "--summary"])
        .output()
        .expect("run");
    let before = String::from_utf8_lossy(&before.stderr).to_string();
    assert!(before.contains("ac:vague-response"), "{before}");

    let after = quire()
        .args([
            "validate",
            &d,
            "--module",
            &m,
            "--summary",
            "--severity",
            "ac:vague-response=off",
        ])
        .output()
        .expect("run");
    let after = String::from_utf8_lossy(&after.stderr).to_string();
    assert!(
        !after.contains("ac:vague-response"),
        "an `off` check must not appear anywhere: {after}"
    );
    // The other grammar is untouched — `off` is per-check, not global.
    assert!(after.contains("ears:"), "{after}");
}

/// TC-721, FR-004-AC-17 (upstream quire-rs FR-048-AC-6): `--severity …=error`
/// promotes a check, and the run
/// fails on it — the per-check lever `--strict` could not express.
#[test]
fn tc721_severity_error_fails_the_run() {
    let dir = TempDir::new().expect("tempdir");
    let (d, m) = (doc(&dir), module(&dir));

    assert!(
        quire()
            .args(["validate", &d, "--module", &m])
            .status()
            .expect("run")
            .success(),
        "advisory by default"
    );
    assert!(
        !quire()
            .args([
                "validate",
                &d,
                "--module",
                &m,
                "--severity",
                "ac:vague-response=error",
            ])
            .status()
            .expect("run")
            .success(),
        "an `error` check must fail the run"
    );
}

/// TC-755, FR-004-AC-18 (upstream quire-rs FR-048-AC-10): a malformed
/// `--severity` entry is rejected before any
/// document is read, so the user gets a usage error rather than a run that
/// silently ignored the flag.
#[test]
fn tc755_malformed_severity_entry_is_rejected() {
    let dir = TempDir::new().expect("tempdir");
    let (d, m) = (doc(&dir), module(&dir));

    for bad in ["bogus", "ac:vague-response=loud", "=off", "ac:=off"] {
        let out = quire()
            .args(["validate", &d, "--module", &m, "--severity", bad])
            .output()
            .expect("run");
        assert!(!out.status.success(), "{bad} should be rejected");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("--severity"),
            "the diagnostic should name the flag for {bad}: {err}"
        );
    }
}

/// TC-740, FR-017-AC-2, FR-017-AC-6 (upstream quire-rs FR-050): `quire coverage`
/// runs the rollup and its JSON is
/// byte-identical across runs over identical input (FR-050-AC-7). A module with
/// no `traceability:` model is a clear error rather than an empty report —
/// guessing would be the agent-grep behaviour this command replaces.
#[test]
fn tc740_coverage_reports_and_is_deterministic() {
    let dir = TempDir::new().expect("tempdir");
    let m = module(&dir);
    let scope = dir.path().to_string_lossy().into_owned();
    // CR-045: documents live in `<scope>/spec`; a scope without one refuses.
    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");

    // No traceability model declared → refuse, with a message that says why.
    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("traceability"), "{err}");

    // With a model, the JSON payload is stable across runs. CR-062: a trace
    // target binds by **archetype**, never by path — the matrix is whichever
    // corpus document is typed `TestMatrix`, wherever under `spec/` it sits.
    fs::write(
        std::path::Path::new(&m).join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n- name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   id_column: Test ID\n  status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n",
    )
    .expect("rewrite manifest");

    // A matrix that actually mints, so byte-identity is asserted over a report
    // with rows in it rather than over an empty one.
    fs::write(
        dir.path().join("spec").join("tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n\
         ## Test Case Summary\n\n\
         | Test ID | Status |\n|---------|--------|\n| TC-001 | ✅ |\n",
    )
    .expect("write matrix");

    let run = || {
        let o = quire()
            .args(["coverage", "--scope", &scope, "--module", &m, "--json"])
            .output()
            .expect("run");
        String::from_utf8_lossy(&o.stdout).to_string()
    };
    let (a, b) = (run(), run());
    assert!(!a.is_empty(), "coverage emitted no JSON");
    assert_eq!(a, b, "coverage output must be byte-identical (FR-050-AC-7)");
}

/// TC-797, FR-017-AC-8 (upstream quire-rs FR-050-AC-14, CR-035): a declared
/// model that matches nothing is not
/// full coverage. It used to render as `0/0 rows backed (100%)` and pass
/// `--strict` with exit 0, because the percentage fell back to 100 on an empty
/// denominator and `--strict` fires only on non-empty unbacked/status lists —
/// both of which are empty when nothing matched. Every gate wired to this
/// command passed vacuously, which is how the ecosystem-wide `trace_tags` gap
/// went unnoticed for nine days.
#[test]
fn tc797_zero_matched_rows_is_not_full_coverage() {
    let dir = TempDir::new().expect("tempdir");
    let m = module(&dir);
    let scope = dir.path().to_string_lossy().into_owned();
    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");

    // A model whose trace target names an archetype no document in this scope
    // carries: declared, valid, and matching zero rows. (CR-062: a misspelled
    // or unpopulated archetype is the surviving shape of this fault, where
    // before it was a path pointing nowhere.)
    fs::write(
        std::path::Path::new(&m).join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n- name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   id_column: Test ID\n",
    )
    .expect("rewrite manifest");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert!(out.status.success(), "a report is not a verdict");
    let err = String::from_utf8_lossy(&out.stderr);
    let headline = err
        .lines()
        .find(|l| l.contains("rows backed"))
        .unwrap_or_else(|| panic!("no coverage headline in:\n{err}"));
    assert!(
        !headline.contains("100%"),
        "an empty denominator rendered as full coverage: {headline}"
    );
    assert!(
        headline.contains("no rows matched"),
        "the empty denominator should say so: {headline}"
    );

    // The same state under --strict is a failure, and the message says which
    // failure: nothing was reconciled, not "nothing was wrong".
    let strict = quire()
        .args(["coverage", "--scope", &scope, "--module", &m, "--strict"])
        .output()
        .expect("run");
    assert!(
        !strict.status.success(),
        "--strict passed over a model that matched nothing"
    );
    let serr = String::from_utf8_lossy(&strict.stderr);
    assert!(serr.contains("matched no rows"), "{serr}");
}

/// A module whose model mints ids from `TestMatrix` documents by archetype.
fn matrix_module(dir: &TempDir) -> String {
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   id_column: Test ID\n  status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n",
    )
    .expect("write manifest");
    m.to_string_lossy().into_owned()
}

fn matrix_doc(id: &str, row: &str) -> String {
    format!(
        "---\nid: {id}\ntype: TestMatrix\n---\n# matrix\n\n\
         ## Test Case Summary\n\n\
         | Test ID | Title | Status |\n|---------|-------|--------|\n\
         | {row} | a case | 🚧 |\n"
    )
}

/// TC-810, FR-017-AC-3 (upstream quire-rs FR-050-AC-17, CR-045): the document
/// root is `<scope>/spec` — a
/// matrix at the repository root or under `plan/` mints nothing, however
/// perfectly typed, and repo-root `README.md`/`CHANGELOG.md` are never read.
/// Before the two-root split every one of these minted into the report.
#[test]
fn tc810_document_root_is_scope_spec_not_the_repo() {
    let dir = TempDir::new().expect("tempdir");
    let m = matrix_module(&dir);
    let scope = dir.path().to_string_lossy().into_owned();

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    fs::create_dir_all(dir.path().join("plan")).expect("mkdir plan");
    fs::write(
        dir.path().join("spec/tests.md"),
        matrix_doc("TM-001", "TC-001"),
    )
    .expect("write spec matrix");
    // Decoys outside the document root: typed matrices that would mint under
    // the old repo-wide walk, plus frontmatter-less strays.
    fs::write(dir.path().join("tests.md"), matrix_doc("TM-900", "TC-999")).expect("write decoy");
    fs::write(
        dir.path().join("plan/tests.md"),
        matrix_doc("TM-901", "TC-888"),
    )
    .expect("write plan decoy");
    fs::write(dir.path().join("README.md"), "# readme\nno frontmatter\n").expect("write readme");
    fs::write(dir.path().join("CHANGELOG.md"), "# changelog\n").expect("write changelog");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m, "--json"])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Asserted over the parsed payload, not pretty-printed substrings: the
    // old `"\"total\": 1"` needle depended on `to_string_pretty` spacing and
    // went vacuous the moment the default became compact (#51 heads-up, #53).
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("coverage json");
    let docs: Vec<&str> = report["groups"]
        .as_array()
        .expect("groups array")
        .iter()
        .filter_map(|g| g["document"].as_str())
        .collect();
    assert!(
        docs.contains(&"spec/tests.md"),
        "spec/ matrix must mint: {report}"
    );
    assert!(
        !docs
            .iter()
            .any(|d| *d == "tests.md" || d.starts_with("plan/")),
        "a matrix outside spec/ minted ids — the walk left the document root: {report}"
    );
    assert_eq!(
        report["totals"]["total"], 1,
        "exactly the one spec/ row should mint: {report}"
    );
    let raw = String::from_utf8_lossy(&out.stdout);
    assert!(
        !raw.contains("README") && !raw.contains("CHANGELOG") && !raw.contains("plan/tests.md"),
        "repo-root strays appeared in the report: {raw}"
    );
}

/// TC-811, FR-017-AC-5 (upstream quire-rs FR-050-AC-17, CR-045): a scope with
/// no `spec/` directory is a
/// named error — never a silent fallback to walking the scope itself, which
/// is how the repository-wide crawl survived.
#[test]
fn tc811_missing_spec_root_is_a_named_error() {
    let dir = TempDir::new().expect("tempdir");
    let m = matrix_module(&dir);
    let scope = dir.path().to_string_lossy().into_owned();

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert!(!out.status.success(), "must refuse without a document root");
    let err = String::from_utf8_lossy(&out.stderr);
    // The interpolated path, not `contains("spec")` — the unrelated
    // "install a module (e.g. spec-artifacts-process)" refusal contains
    // "spec" too, so the weaker assertion could not tell the two apart
    // (agent-ix/quire-cli#31).
    let expected = std::path::Path::new(&scope).join("spec");
    assert!(
        err.contains(&expected.display().to_string()),
        "the diagnostic must name the missing document root by path \
         (expected {}): {err}",
        expected.display()
    );
}

// IT-086, FR-017-AC-5 (agent-ix/quire-rs#113): the missing-root refusal
// carries a stable machine `kind` under --diagnostics json, so a consumer can
// branch on it instead of matching prose. It was a formatted `bail!` string,
// which is exactly why the assertions above could only look for a substring.
#[test]
fn it086_missing_spec_root_carries_a_typed_kind_in_json() {
    let dir = TempDir::new().expect("tempdir");
    let m = matrix_module(&dir);
    let scope = dir.path().to_string_lossy().into_owned();

    let out = quire()
        .args([
            "--diagnostics-format",
            "json",
            "coverage",
            "--scope",
            &scope,
            "--module",
            &m,
        ])
        .output()
        .expect("run");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    let line = err
        .lines()
        .find(|l| l.contains("MissingDocumentRoot"))
        .unwrap_or_else(|| panic!("no typed diagnostic in: {err}"));
    let parsed: serde_json::Value = serde_json::from_str(line).expect("json line");
    assert_eq!(parsed["kind"], "MissingDocumentRoot");
    assert_eq!(parsed["severity"], "error");
    let expected = std::path::Path::new(&scope).join("spec");
    assert!(
        parsed["message"]
            .as_str()
            .unwrap()
            .contains(&expected.display().to_string()),
        "the message names the path: {parsed}"
    );
}

// IT-087, FR-017-AC-4 (upstream quire-rs FR-050-AC-17; agent-ix/quire-cli#31):
// the *second* half of the
// two-root derivation — the code walk excludes the document root. Nothing
// asserted it, so `extract_tree_excluding` could regress to `extract_tree`
// with the whole suite green. A trace-tagged source file under `<scope>/spec`
// is the falsifier: if the exclusion lapses, its tag binds and the id shows up
// as an untracked symbol.
#[test]
fn it087_the_code_walk_excludes_the_document_root() {
    let dir = TempDir::new().expect("tempdir");
    let scope = dir.path().to_string_lossy().into_owned();

    // A module like `matrix_module`, plus the trace-tag forms — without a
    // declared marker nothing binds and the test would pass vacuously.
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   id_column: Test ID\n  status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n\
         \x20 trace_tags:\n    legacy:\n    - name: comment-id\n\
         \x20     pattern: '(?://|#)\\s*((?:TC|FR)-\\d+)'\n",
    )
    .expect("write manifest");
    let m = m.to_string_lossy().into_owned();

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    fs::write(
        dir.path().join("spec/tests.md"),
        matrix_doc("TM-001", "TC-001"),
    )
    .expect("write matrix");

    // A source file INSIDE the document root, carrying a trace tag for an id
    // the model never declares. Ingested as source, it becomes an untracked
    // symbol; excluded, it is invisible.
    fs::write(
        dir.path().join("spec/leaked.rs"),
        "// TC-909\n#[test]\nfn tc909_inside_the_document_root() {}\n",
    )
    .expect("write leaked source");
    // The same shape outside it, which must be seen — otherwise the test
    // passes on an engine that walks nothing at all.
    fs::write(
        dir.path().join("src/real.rs"),
        "// TC-908\n#[test]\nfn tc908_outside_the_document_root() {}\n",
    )
    .expect("write real source");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m, "--json"])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("coverage json");
    let untracked = report["untracked_symbols"].as_array().expect("array");
    let ids: Vec<&str> = untracked
        .iter()
        .filter_map(|s| s["trace_id"].as_str())
        .collect();

    assert!(
        ids.contains(&"TC-908"),
        "a tagged symbol outside the document root must be seen, or this test \
         proves nothing: {ids:?}"
    );
    assert!(
        !ids.contains(&"TC-909"),
        "a source file under the document root was ingested as code — the \
         exclusion lapsed: {ids:?}"
    );
}

// IT-089, FR-017-AC-1: the default human census. Everything else about this
// command was tested through `--json`, so the rendered output — what a person
// actually runs — had no coverage at all.
#[test]
fn it089_human_census_is_printed_and_exits_zero() {
    let dir = TempDir::new().expect("tempdir");
    let m = matrix_module(&dir);
    let scope = dir.path().to_string_lossy().into_owned();
    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    fs::write(
        dir.path().join("spec/tests.md"),
        matrix_doc("TM-001", "TC-001"),
    )
    .expect("write matrix");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "a report is not a verdict: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The census renders on stderr; stdout stays clean so that the `--json`
    // payload is the only thing that ever reaches it.
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.trim().is_empty(),
        "stdout must stay clean without --json: {stdout}"
    );
    assert!(
        stderr.contains("rows backed"),
        "the census must describe the rollup: {stderr}"
    );
}

// IT-092, FR-017-AC-7: `--strict` turns an unbacked row into exit 1, and the
// same repository exits 0 without it. The flag is the whole gate/report
// distinction (FR-050-CON-1) and nothing asserted both halves.
#[test]
fn it092_strict_fails_on_an_unbacked_row_and_default_does_not() {
    let dir = TempDir::new().expect("tempdir");
    let scope = dir.path().to_string_lossy().into_owned();

    // An ISO-shaped model: an FR mints acceptance-criterion ids, and the
    // matrix's `Traces To` column references them. Reference rows are what
    // `unbacked_rows` and `status_lies` are computed from — a trace target
    // alone produces neither, which is why the matrix-only fixture used
    // elsewhere cannot exercise --strict.
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n\
         - name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n\
         \x20   id_column: ID\n\
         \x20 document_references:\n  - name: traces-to\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   column: Traces To\n    row_id_column: Test ID\n\
         \x20   pattern: '(FR-\\d+-AC-\\d+)'\n\
         \x20   targets: [acceptance-criterion]\n  status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n",
    )
    .expect("write manifest");
    let m = m.to_string_lossy().into_owned();

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    fs::write(
        dir.path().join("spec/FR-001.md"),
        "---\nid: FR-001\ntype: FR\n---\n# FR-001\n\n\
         ## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n\
         | FR-001-AC-1 | it holds |\n",
    )
    .expect("write FR");
    // A row claiming ✅ that no symbol backs: one unbacked row and one lie.
    fs::write(
        dir.path().join("spec/tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n# matrix\n\n\
         ## Test Case Summary\n\n| Test ID | Traces To | Status |\n\
         |---------|-----------|--------|\n| TC-001 | FR-001-AC-1 | ✅ |\n",
    )
    .expect("write matrix");

    let plain = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert!(
        plain.status.success(),
        "without --strict the rollup is a report: {}",
        String::from_utf8_lossy(&plain.stderr)
    );

    let strict = quire()
        .args(["coverage", "--scope", &scope, "--module", &m, "--strict"])
        .output()
        .expect("run");
    assert!(
        !strict.status.success(),
        "--strict must fail on an unbacked row: {}",
        String::from_utf8_lossy(&strict.stderr)
    );
    let err = String::from_utf8_lossy(&strict.stderr);
    assert!(
        err.contains("unbacked") || err.contains("status"),
        "the failure must say what it found: {err}"
    );
}

// IT-107, FR-017-AC-10 (upstream quire-rs FR-050-AC-21, CR-083): an undeclared
// status value reaches BOTH surfaces.
//
// The engine class is inert until the CLI carries it. That is the failure the
// last programme phase kept finding — a capability shipped and unreachable —
// and the tell it recommends is exactly this: invoke the surface a user
// actually invokes, and grep for the thing.
#[test]
fn it107_an_undeclared_status_reaches_both_surfaces() {
    let dir = TempDir::new().expect("tempdir");
    let scope = dir.path().to_string_lossy().into_owned();

    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   id_column: Test ID\n\
         \x20 document_references:\n  - name: traces-to\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   column: Traces To\n    row_id_column: Test ID\n\
         \x20   pattern: '((?:TC|FR)-\\d+)'\n    targets: [test-case]\n\
         \x20 status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n\
         \x20 trace_tags:\n    legacy:\n    - name: comment-id\n\
         \x20     pattern: '(?://|#)\\s*((?:TC|FR)-\\d+)'\n",
    )
    .expect("write manifest");
    let m = m.to_string_lossy().into_owned();

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    // `🟡` is classed by no list above, so `class_of` returns Unknown. The
    // classification runs per REFERENCE ROW, which is why the module above must
    // declare a `document_references` entry — a matrix with trace targets alone
    // mints ids and scans no rows.
    fs::write(
        dir.path().join("spec/tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n# matrix\n\n\
         ## Test Case Summary\n\n\
         | Test ID | Title | Traces To | Status |\n|---------|-------|-----------|--------|\n\
         | TC-001 | a case | TC-001 | 🚧 |\n\
         | TC-002 | drifted | TC-002 | 🟡 review-open |\n",
    )
    .expect("write matrix");

    // Both rows BACKED, so the undeclared status is the only finding in the
    // report. Without this the `--strict` assertion below would pass for the
    // wrong reason — on unbacked rows, which `--strict` gates on legitimately.
    fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    fs::write(
        dir.path().join("src/real.rs"),
        "// TC-001\n#[test]\nfn tc001_one() {}\n\
         // TC-002\n#[test]\nfn tc002_two() {}\n",
    )
    .expect("write source");

    // Machine surface.
    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m, "--json"])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("coverage json");
    let drifted = report["undeclared_statuses"].as_array().expect("array");
    assert_eq!(drifted.len(), 1, "report: {report}");
    assert_eq!(drifted[0]["row_id"].as_str(), Some("TC-002"));
    assert_eq!(drifted[0]["status"].as_str(), Some("🟡 review-open"));

    // Human surface — the one a person runs.
    let human = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert_eq!(
        human.status.code(),
        Some(0),
        "the human census over a report is exit 0: {}",
        String::from_utf8_lossy(&human.stderr)
    );
    let rendered = String::from_utf8_lossy(&human.stderr).into_owned()
        + &String::from_utf8_lossy(&human.stdout);
    assert!(
        rendered.contains("🟡 review-open") && rendered.contains("classes as nothing"),
        "the finding must be rendered, not only serialized: {rendered}"
    );
    // #51 / FR-017-AC-12 + AC-16: the line leads with the row's own id, keeps
    // the reference kind visible, and the locus is the clickable
    // `document:line` form — the matrix row sits on line 12 of the fixture.
    assert!(
        rendered.contains("TC-002 (spec/tests.md:12)") && rendered.contains("[traces-to]"),
        "the undeclared-status line must lead with the row id, carry the \
         document:line locus and keep the reference kind visible: {rendered}"
    );

    // FR-017-AC-7 is unchanged: `--strict` does not gate on this class. Shipping
    // it as a gate would flip repositories red on an engine bump for a condition
    // nobody has been told about. A gate deferred, never lowered.
    //
    // Both exit codes are pinned to 0 absolutely (#56): the earlier
    // `strict == plain` equality was near-tautological — it would equally
    // pass if a regression made both runs exit 1.
    let strict = quire()
        .args(["coverage", "--scope", &scope, "--module", &m, "--strict"])
        .output()
        .expect("run");
    let plain = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert_eq!(
        strict.status.code(),
        Some(0),
        "--strict must not gate on an undeclared status alone: {}",
        String::from_utf8_lossy(&strict.stderr)
    );
    assert_eq!(
        plain.status.code(),
        Some(0),
        "the default run is a report, not a verdict: {}",
        String::from_utf8_lossy(&plain.stderr)
    );
}

// IT-109, FR-017-AC-12 (#51): every human unbacked-row and status-lie line
// leads with the row's own `row_id`, keeping the reference kind visible in a
// bracketed trailer — and two rows in the same document therefore render
// distinguishable lines. Before this the renderer printed the reference KIND
// (`traces-to`) per line: on a real corpus, 1,432 identical lines a reader
// could act on only by re-running with `--json`, even though `UnbackedRow`,
// `StatusLie` and `UndeclaredStatus` all carry `row_id: Option<String>`.
#[test]
fn it109_human_finding_lines_lead_with_the_row_id() {
    let dir = TempDir::new().expect("tempdir");
    let scope = dir.path().to_string_lossy().into_owned();

    // The it092 shape — FR-minted acceptance-criterion ids referenced from a
    // matrix declaring a `row_id_column` — with TWO rows in one document, so
    // distinguishability is asserted over lines that differ only by row id.
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n\
         - name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n\
         \x20   id_column: ID\n\
         \x20 document_references:\n  - name: traces-to\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   column: Traces To\n    row_id_column: Test ID\n\
         \x20   pattern: '(FR-\\d+-AC-\\d+)'\n\
         \x20   targets: [acceptance-criterion]\n  status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n",
    )
    .expect("write manifest");
    let m = m.to_string_lossy().into_owned();

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    fs::write(
        dir.path().join("spec/FR-001.md"),
        "---\nid: FR-001\ntype: FR\n---\n# FR-001\n\n\
         ## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n\
         | FR-001-AC-1 | it holds |\n| FR-001-AC-2 | it also holds |\n",
    )
    .expect("write FR");
    // Two rows claiming ✅ that no symbol backs: two unbacked rows and two
    // status lies, all in the same document.
    fs::write(
        dir.path().join("spec/tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n# matrix\n\n\
         ## Test Case Summary\n\n| Test ID | Traces To | Status |\n\
         |---------|-----------|--------|\n| TC-001 | FR-001-AC-1 | ✅ |\n\
         | TC-002 | FR-001-AC-2 | ✅ |\n",
    )
    .expect("write matrix");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "a report is not a verdict: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);

    // Row id leading, reference kind visible, one line per row — for both
    // finding classes this fixture produces. The locus carries the engine's
    // 1-based document line (FR-017-AC-16, quire-rs v0.42.0): the two matrix
    // rows sit on lines 11 and 12 of the fixture.
    for (row, line) in [("TC-001", 11), ("TC-002", 12)] {
        assert!(
            err.contains(&format!(
                "{row} (spec/tests.md:{line}) has no backing symbol [traces-to]"
            )),
            "unbacked-row line for {row} must lead with the row id, carry the \
             document:line locus and keep the reference kind: {err}"
        );
        assert!(
            err.contains(&format!(
                "{row} (spec/tests.md:{line}) claims `✅` but is not backed [traces-to]"
            )),
            "status-lie line for {row} must lead with the row id, carry the \
             document:line locus and keep the reference kind: {err}"
        );
    }
    // The old shape — the reference kind leading for a record that HAS a row
    // id — must be gone, or the two rows above collapse into identical lines.
    assert!(
        !err.contains("traces-to (spec/tests.md) has no backing symbol"),
        "a row with a row_id must not render under its reference kind: {err}"
    );
}

/// The it109 shape as a reusable fixture: an FR minting two acceptance-
/// criterion ids and a matrix whose two rows reference them, both claiming ✅
/// with nothing backing them — two unbacked rows and two status lies in one
/// document. Returns `(scope, module)`.
fn lying_fixture(dir: &TempDir) -> (String, String) {
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n\
         - name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n\
         \x20   id_column: ID\n\
         \x20 document_references:\n  - name: traces-to\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   column: Traces To\n    row_id_column: Test ID\n\
         \x20   pattern: '(FR-\\d+-AC-\\d+)'\n\
         \x20   targets: [acceptance-criterion]\n  status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n",
    )
    .expect("write manifest");

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    fs::write(
        dir.path().join("spec/FR-001.md"),
        "---\nid: FR-001\ntype: FR\n---\n# FR-001\n\n\
         ## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n\
         | FR-001-AC-1 | it holds |\n| FR-001-AC-2 | it also holds |\n",
    )
    .expect("write FR");
    fs::write(
        dir.path().join("spec/tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n# matrix\n\n\
         ## Test Case Summary\n\n| Test ID | Traces To | Status |\n\
         |---------|-----------|--------|\n| TC-001 | FR-001-AC-1 | ✅ |\n\
         | TC-002 | FR-001-AC-2 | ✅ |\n",
    )
    .expect("write matrix");

    (
        dir.path().to_string_lossy().into_owned(),
        m.to_string_lossy().into_owned(),
    )
}

// IT-110, FR-017-AC-13 (#53): the coverage severity pack. `--severity
// coverage:<check>=off` is the projection lever — the kind's records drop
// from every output surface, with the suppression announced and the totals
// still describing the full reconciliation — and `=error` is the per-check
// gate `--strict` cannot express. Entries ride the same FR-048 machinery
// `validate --severity` uses, so parsing, precedence and reject-before-read
// come for free and cannot drift between the two commands.
#[test]
fn it110_severity_pack_projects_and_promotes() {
    let dir = TempDir::new().expect("tempdir");
    let (scope, m) = lying_fixture(&dir);

    // `off` drops the kind from the human census; the others survive, and the
    // suppression is announced so a projected report never reads as clean.
    let human = quire()
        .args([
            "coverage",
            "--scope",
            &scope,
            "--module",
            &m,
            "--severity",
            "coverage:status-lie=off",
        ])
        .output()
        .expect("run");
    assert_eq!(
        human.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    let err = String::from_utf8_lossy(&human.stderr);
    assert!(
        !err.contains("claims `✅`"),
        "an `off` kind must not render: {err}"
    );
    assert!(
        err.contains("has no backing symbol"),
        "`off` is per-check — other kinds are untouched: {err}"
    );
    assert!(
        err.contains("2 coverage:status-lie finding(s) suppressed"),
        "the suppression must be announced with its count: {err}"
    );

    // ...and from the JSON payload, with `totals` still full-computation.
    let json_out = quire()
        .args([
            "coverage",
            "--scope",
            &scope,
            "--module",
            &m,
            "--json",
            "--severity",
            "coverage:status-lie=off",
        ])
        .output()
        .expect("run");
    assert!(json_out.status.success());
    let report: serde_json::Value = serde_json::from_slice(&json_out.stdout).expect("json");
    assert_eq!(
        report["status_lies"].as_array().map(Vec::len),
        Some(0),
        "suppressed kinds drop from the payload: {report}"
    );
    assert_eq!(
        report["unbacked_rows"].as_array().map(Vec::len),
        Some(2),
        "other kinds are untouched: {report}"
    );
    assert_eq!(
        report["totals"]["total"], 2,
        "totals describe the full reconciliation, never the projection: {report}"
    );

    // Projection never lowers the gate: --strict fails on the full
    // computation even with both gated kinds rendered off.
    let strict = quire()
        .args([
            "coverage",
            "--scope",
            &scope,
            "--module",
            &m,
            "--strict",
            "--severity",
            "coverage:status-lie=off",
            "--severity",
            "coverage:unbacked-row=off",
        ])
        .output()
        .expect("run");
    assert_eq!(
        strict.status.code(),
        Some(1),
        "--strict must judge what the engine found, not what rendering shows"
    );

    // `error` promotes a kind to a failing exit without --strict.
    let promoted = quire()
        .args([
            "coverage",
            "--scope",
            &scope,
            "--module",
            &m,
            "--severity",
            "coverage:status-lie=error",
        ])
        .output()
        .expect("run");
    assert_eq!(promoted.status.code(), Some(1), "`error` must fail the run");
    let perr = String::from_utf8_lossy(&promoted.stderr);
    assert!(
        perr.contains("coverage:status-lie") && perr.contains("error"),
        "the failure names the promoted check: {perr}"
    );

    // A malformed entry is rejected before any document is read.
    let bad = quire()
        .args([
            "coverage",
            "--scope",
            &scope,
            "--module",
            &m,
            "--severity",
            "coverage:status-lie=loud",
        ])
        .output()
        .expect("run");
    assert!(!bad.status.success(), "a malformed level must be rejected");
    assert!(
        String::from_utf8_lossy(&bad.stderr).contains("--severity"),
        "the diagnostic names the flag"
    );

    // ...and so is a typo'd coverage check (#57): the pack's vocabulary is a
    // closed set of four owned by this CLI. FR-048 validates only the key's
    // shape, so without the pack-side rejection `coverage:unbaked-row=off`
    // would merge, match nothing, and silently not project — a flag that does
    // nothing must be an error, not a no-op.
    let typo = quire()
        .args([
            "coverage",
            "--scope",
            &scope,
            "--module",
            &m,
            "--severity",
            "coverage:unbaked-row=off",
        ])
        .output()
        .expect("run");
    assert_eq!(
        typo.status.code(),
        Some(1),
        "an unknown coverage check must be rejected"
    );
    let terr = String::from_utf8_lossy(&typo.stderr);
    assert!(
        terr.contains("coverage:unbaked-row") && terr.contains("unbacked-row"),
        "the rejection quotes the entry and names the valid checks: {terr}"
    );
    assert!(
        !terr.contains("rows backed"),
        "the rejection must come before any document is read: {terr}"
    );
}

// IT-111, FR-017-AC-14 (#53): `--format tsv` emits one tab-separated record
// per line on stdout — nine fixed columns, row id leading the data cells, a
// `line` column carrying the engine's 1-based line (populated since quire-rs
// v0.42.0; the column predates the data, so its arrival needed no format
// change), and byte-identical output across runs. The severity projection
// applies to it exactly as to the other surfaces.
#[test]
fn it111_tsv_emits_one_record_per_line_on_stdout() {
    let dir = TempDir::new().expect("tempdir");
    let (scope, m) = lying_fixture(&dir);

    let run = |extra: &[&str]| {
        let mut args = vec![
            "coverage", "--scope", &scope, "--module", &m, "--format", "tsv",
        ];
        args.extend_from_slice(extra);
        let out = quire().args(&args).output().expect("run");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).into_owned()
    };

    let tsv = run(&[]);
    let lines: Vec<&str> = tsv.lines().collect();
    assert_eq!(
        lines[0], "kind\tid\tdocument\treference\tstatus\tmethod\tline\ttargets\ttext",
        "the header names the nine fixed columns"
    );
    for line in &lines {
        assert_eq!(
            line.split('\t').count(),
            9,
            "every record carries every column: {line}"
        );
    }
    // Two unbacked rows and two status lies, each one line, row id leading.
    // The `line` column now carries the engine's 1-based document line
    // (FR-017-AC-14, quire-rs v0.42.0) — the arrival the empty column was
    // reserved for, with no format change: the matrix rows sit on fixture
    // lines 11 and 12.
    for (row, doc_line) in [("TC-001", "11"), ("TC-002", "12")] {
        assert!(
            lines
                .iter()
                .any(|l| l.starts_with(&format!("unbacked-row\t{row}\tspec/tests.md\ttraces-to\t"))),
            "unbacked-row record for {row} missing: {tsv}"
        );
        let lie = lines
            .iter()
            .find(|l| l.starts_with(&format!("status-lie\t{row}\t")))
            .unwrap_or_else(|| panic!("status-lie record for {row} missing: {tsv}"));
        let cells: Vec<&str> = lie.split('\t').collect();
        assert_eq!(cells[4], "✅", "the status column carries the claim: {lie}");
        assert_eq!(
            cells[6], doc_line,
            "the line column carries the record's document line: {lie}"
        );
    }

    // Deterministic across runs (FR-050-AC-7 carries over to the projection).
    assert_eq!(tsv, run(&[]), "tsv output must be byte-identical");

    // The severity projection reaches this surface too.
    let projected = run(&["--severity", "coverage:status-lie=off"]);
    assert!(
        !projected.contains("status-lie\t"),
        "a suppressed kind must not reach the tsv: {projected}"
    );
    assert!(
        projected.contains("unbacked-row\t"),
        "other kinds survive: {projected}"
    );
}

// IT-112, FR-017-AC-15 (#53): `coverage --json` honours the global
// `--pretty` — compact single-line by default (the FR-008-AC-1 posture every
// other JSON surface already has), indented with the flag, the same parsed
// value either way. `to_json()` was unconditionally pretty and the flag was
// silently ignored.
#[test]
fn it112_coverage_json_honours_the_global_pretty_flag() {
    let dir = TempDir::new().expect("tempdir");
    let (scope, m) = lying_fixture(&dir);

    let compact = quire()
        .args(["coverage", "--scope", &scope, "--module", &m, "--json"])
        .output()
        .expect("run");
    assert!(compact.status.success());
    let compact = String::from_utf8_lossy(&compact.stdout).into_owned();
    assert!(
        !compact.trim_end().contains('\n'),
        "default JSON is compact — one line (FR-008-AC-1): {compact}"
    );

    let pretty = quire()
        .args([
            "--pretty", "coverage", "--scope", &scope, "--module", &m, "--json",
        ])
        .output()
        .expect("run");
    assert!(pretty.status.success());
    let pretty = String::from_utf8_lossy(&pretty.stdout).into_owned();
    assert!(
        pretty.trim_end().contains('\n'),
        "--pretty indents: {pretty}"
    );

    let a: serde_json::Value = serde_json::from_str(&compact).expect("compact parses");
    let b: serde_json::Value = serde_json::from_str(&pretty).expect("pretty parses");
    assert_eq!(a, b, "the two shapes carry the identical report");
}

// IT-108, FR-017-AC-11 (upstream quire-rs FR-050-AC-22, CR-085): a declared
// `source_exclude` glob reaches the source walk.
//
// Same reachability argument as IT-107. The engine grew the key; without this
// line passing `model.source_exclude`, every module declaring it would be
// declaring something with no effect whatsoever.
#[test]
fn it108_a_declared_source_exclude_scopes_the_code_walk() {
    let manifest = |extra: &str| {
        format!(
            "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
             - name: TestMatrix\n\
             traceability:\n{extra}  trace_targets:\n  - name: test-case\n\
             \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
             \x20   id_column: Test ID\n  status:\n    column: Status\n\
             \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
             \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n\
             \x20 trace_tags:\n    legacy:\n    - name: comment-id\n\
             \x20     pattern: '(?://|#)\\s*((?:TC|FR)-\\d+)'\n"
        )
    };

    let build = |globs: &str| {
        let dir = TempDir::new().expect("tempdir");
        let m = dir.path().join("m");
        fs::create_dir_all(&m).expect("mkdir");
        fs::write(m.join("manifest.yaml"), manifest(globs)).expect("write manifest");
        fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
        fs::create_dir_all(dir.path().join("tests/fixtures")).expect("mkdir fixtures");
        fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
        fs::write(
            dir.path().join("spec/tests.md"),
            matrix_doc("TM-001", "TC-001"),
        )
        .expect("write matrix");
        // A fixture whose whole purpose is to hold a tag nothing declares.
        fs::write(
            dir.path().join("tests/fixtures/sample.rs"),
            "// TC-999\n#[test]\nfn tc999_covers_nothing() {}\n",
        )
        .expect("write fixture");
        // Real source, which must survive either way.
        fs::write(
            dir.path().join("src/real.rs"),
            "// TC-998\n#[test]\nfn tc998_real() {}\n",
        )
        .expect("write source");
        dir
    };

    let ids = |dir: &TempDir| -> Vec<String> {
        let scope = dir.path().to_string_lossy().into_owned();
        let m = dir.path().join("m").to_string_lossy().into_owned();
        let out = quire()
            .args(["coverage", "--scope", &scope, "--module", &m, "--json"])
            .output()
            .expect("run");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
        report["untracked_symbols"]
            .as_array()
            .expect("array")
            .iter()
            .filter_map(|s| s["trace_id"].as_str().map(str::to_string))
            .collect()
    };

    // Undeclared: the fixture's tag is reported, which is the status quo and the
    // reason #199 exists.
    let before = ids(&build(""));
    assert!(
        before.contains(&"TC-999".to_string()),
        "without the glob the fixture tag must be reported, or this test proves \
         nothing: {before:?}"
    );

    // Declared: it is gone, and real source is untouched.
    let after = ids(&build("  source_exclude: ['tests/fixtures/**']\n"));
    assert!(
        !after.contains(&"TC-999".to_string()),
        "a declared source_exclude glob did not reach the walk: {after:?}"
    );
    assert!(
        after.contains(&"TC-998".to_string()),
        "the glob removed real source: {after:?}"
    );
}

/// The it107 manifest (self-referencing TC ids, declared status vocabulary,
/// legacy comment trace tags), reusable by the #51-batch tests below.
fn self_ref_manifest() -> &'static str {
    "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
     - name: TestMatrix\n\
     traceability:\n  trace_targets:\n  - name: test-case\n\
     \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
     \x20   id_column: Test ID\n\
     \x20 document_references:\n  - name: traces-to\n\
     \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
     \x20   column: Traces To\n    row_id_column: Test ID\n\
     \x20   pattern: '((?:TC|FR)-\\d+)'\n    targets: [test-case]\n\
     \x20 status:\n    column: Status\n\
     \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
     \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n\
     \x20 trace_tags:\n    legacy:\n    - name: comment-id\n\
     \x20     pattern: '(?://|#)\\s*((?:TC|FR)-\\d+)'\n"
}

// IT-113, FR-017-AC-16 (#51 item 3, quire-rs v0.42.0 FR-050-AC-26): when a
// finding record carries the engine's 1-based line, the human locus is the
// clickable `document:line` form. The fixture puts preamble prose ABOVE the
// table so the asserted numbers can only be document lines — a renderer
// printing a table-relative row index would fail loudly here while passing
// on a fixture whose table starts at the top.
#[test]
fn it113_human_finding_locus_is_document_colon_line() {
    let dir = TempDir::new().expect("tempdir");
    let scope = dir.path().to_string_lossy().into_owned();
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(m.join("manifest.yaml"), self_ref_manifest()).expect("write manifest");
    let m = m.to_string_lossy().into_owned();

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    // Rows on document lines 13 and 14: frontmatter (1-4), title (5),
    // preamble (6-8), section (9-10), header+separator (11-12).
    fs::write(
        dir.path().join("spec/tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n# matrix\n\n\
         Preamble prose that shifts the table down the document.\n\n\
         ## Test Case Summary\n\n\
         | Test ID | Title | Traces To | Status |\n|---------|-------|-----------|--------|\n\
         | TC-001 | a case | TC-001 | ✅ |\n\
         | TC-002 | drifted | TC-002 | 🟡 |\n",
    )
    .expect("write matrix");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "a report is not a verdict: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);

    // All three pre-existing row-id kinds carry the locus.
    assert!(
        err.contains("TC-001 (spec/tests.md:13) has no backing symbol [traces-to]"),
        "unbacked-row locus must be document:line: {err}"
    );
    assert!(
        err.contains("TC-001 (spec/tests.md:13) claims `✅` but is not backed [traces-to]"),
        "status-lie locus must be document:line: {err}"
    );
    assert!(
        err.contains("TC-002 (spec/tests.md:14) has status `🟡`"),
        "undeclared-status locus must be document:line: {err}"
    );
    // And the bare-document form — the pre-line rendering — is gone for
    // records that carry a line.
    assert!(
        !err.contains("(spec/tests.md)"),
        "a record carrying a line must not render the bare document: {err}"
    );
}

// IT-114, FR-017-AC-17 (#51): `no_symbol_rows` renders in the human census
// like every other row-id-carrying kind. It was JSON/TSV-only — but the
// record explains an unbacked row the census DOES print, and an explanation
// only the machine surface carries is one nobody reads (the CR-083 argument
// that put `undeclared_statuses` on both surfaces).
#[test]
fn it114_no_symbol_rows_render_in_the_human_census() {
    let dir = TempDir::new().expect("tempdir");
    let scope = dir.path().to_string_lossy().into_owned();
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    // The self-referencing manifest plus the CR-041 vocabulary: `Inspection`
    // is declared to mint no source symbol, read from the `Type` column.
    fs::write(
        m.join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: TestMatrix\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   id_column: Test ID\n\
         \x20 document_references:\n  - name: traces-to\n\
         \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
         \x20   column: Traces To\n    row_id_column: Test ID\n\
         \x20   pattern: '((?:TC|FR)-\\d+)'\n    targets: [test-case]\n\
         \x20 status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n\
         \x20 vocabularies:\n    test_type: [Test, Inspection]\n\
         \x20   test_type_column: Type\n    no_source_symbol: [Inspection]\n",
    )
    .expect("write manifest");
    let m = m.to_string_lossy().into_owned();

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    // Rows on document lines 11 and 12; both unbacked (no source exists).
    // TC-001 is Inspection-verified — exempt, and the census must SAY so;
    // TC-002 is Test-verified and must NOT mint the exemption line.
    fs::write(
        dir.path().join("spec/tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n# matrix\n\n\
         ## Test Case Summary\n\n\
         | Test ID | Type | Traces To | Status |\n|---------|------|-----------|--------|\n\
         | TC-001 | Inspection | TC-001 | 🚧 |\n\
         | TC-002 | Test | TC-002 | 🚧 |\n",
    )
    .expect("write matrix");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "a report is not a verdict: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains(
            "TC-001 (spec/tests.md:11) is verified by `Inspection`, \
             which mints no source symbol [traces-to]"
        ),
        "the no-symbol row must render with row id, document:line locus and \
         the exempting value: {err}"
    );
    assert!(
        !err.contains("TC-002 (spec/tests.md:12) is verified by"),
        "a row whose method mints symbols must not render the exemption: {err}"
    );

    // The TSV projection of the same record carries the method and the line.
    let tsv = quire()
        .args([
            "coverage", "--scope", &scope, "--module", &m, "--format", "tsv",
        ])
        .output()
        .expect("run");
    assert!(tsv.status.success());
    let tsv = String::from_utf8_lossy(&tsv.stdout).into_owned();
    assert!(
        tsv.contains("no-symbol-row\tTC-001\tspec/tests.md\ttraces-to\t\tInspection\t11\t"),
        "the tsv no-symbol-row record must carry method and line: {tsv}"
    );
}

// IT-115, FR-017-AC-18 (#51, quire-rs #215): the `source_exclude`
// subtraction is observable — the census counts what the globs removed — and
// `SymbolExtraction` diagnostics reach stderr instead of being computed and
// dropped. Without the count, an over-broad glob reads exactly like tests
// that were never written; without the diagnostics, a walk that read less
// than declared said nothing.
#[test]
fn it115_excluded_count_and_extraction_diagnostics_reach_stderr() {
    let build = |declare_glob: bool| {
        let dir = TempDir::new().expect("tempdir");
        let m = dir.path().join("m");
        fs::create_dir_all(&m).expect("mkdir");
        let glob = if declare_glob {
            "  source_exclude: ['tests/fixtures/**']\n"
        } else {
            ""
        };
        fs::write(
            m.join("manifest.yaml"),
            format!(
                "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
                 - name: TestMatrix\n\
                 traceability:\n{glob}  trace_targets:\n  - name: test-case\n\
                 \x20   archetype: TestMatrix\n    section: Test Case Summary\n\
                 \x20   id_column: Test ID\n  status:\n    column: Status\n\
                 \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
                 \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n\
                 \x20 trace_tags:\n    legacy:\n    - name: comment-id\n\
                 \x20     pattern: '(?://|#)\\s*((?:TC|FR)-\\d+)'\n"
            ),
        )
        .expect("write manifest");
        fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
        fs::create_dir_all(dir.path().join("tests/fixtures")).expect("mkdir fixtures");
        fs::write(
            dir.path().join("spec/tests.md"),
            matrix_doc("TM-001", "TC-001"),
        )
        .expect("write matrix");
        fs::write(
            dir.path().join("tests/fixtures/sample.rs"),
            "// TC-999\n#[test]\nfn tc999_covers_nothing() {}\n",
        )
        .expect("write fixture");
        dir
    };

    let run = |dir: &TempDir| {
        let scope = dir.path().to_string_lossy().into_owned();
        let m = dir.path().join("m").to_string_lossy().into_owned();
        let out = quire()
            .args(["coverage", "--scope", &scope, "--module", &m])
            .output()
            .expect("run");
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stdout).trim().is_empty(),
            "the human census keeps stdout empty (FR-017-AC-1)"
        );
        String::from_utf8_lossy(&out.stderr).into_owned()
    };

    // Declared and matching one file: the census says so, exactly once.
    let with_glob = run(&build(true));
    assert!(
        with_glob.contains("1 source file(s) excluded by source_exclude"),
        "the census must count the subtraction: {with_glob}"
    );
    // Undeclared: zero is the no-op every conformant repo is in — silent.
    let without = run(&build(false));
    assert!(
        !without.contains("excluded by source_exclude"),
        "no declaration, no line: {without}"
    );

    // An unreadable source file mints a SymbolExtraction diagnostic, and the
    // CLI surfaces it on stderr rather than dropping it.
    let dir = build(false);
    fs::write(dir.path().join("src.rs"), [0xFFu8, 0xFE, 0x00, 0x01]).expect("write bytes");
    let err = run(&dir);
    assert!(
        err.contains("src.rs: unreadable"),
        "an extraction diagnostic must reach stderr, naming the file: {err}"
    );
}

// IT-116, FR-017-AC-19 (quire-rs v0.42.0, FR-050-AC-23/CR-087): the advisory
// `shared_trace_ids` list passes through `--json` — one status-carrying row
// id bound by N distinct symbols is reported, so a green row can no longer
// rot N-1 tests deep invisibly. Reachability, not re-testing the engine: the
// CLI serializes the whole `CoverageReport`, and this pins that the field
// actually arrives from the surface a user invokes.
#[test]
fn it116_shared_trace_ids_pass_through_json() {
    let dir = TempDir::new().expect("tempdir");
    let scope = dir.path().to_string_lossy().into_owned();
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(m.join("manifest.yaml"), self_ref_manifest()).expect("write manifest");
    let m = m.to_string_lossy().into_owned();

    fs::create_dir_all(dir.path().join("spec")).expect("mkdir spec");
    fs::write(
        dir.path().join("spec/tests.md"),
        "---\nid: TM-001\ntype: TestMatrix\n---\n# matrix\n\n\
         ## Test Case Summary\n\n\
         | Test ID | Title | Traces To | Status |\n|---------|-------|-----------|--------|\n\
         | TC-001 | shared | TC-001 | 🚧 |\n",
    )
    .expect("write matrix");
    // TWO distinct symbols in two files bind the one status-carrying row id.
    fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    fs::write(
        dir.path().join("src/a.rs"),
        "// TC-001\n#[test]\nfn tc001_first_binder() {}\n",
    )
    .expect("write a");
    fs::write(
        dir.path().join("src/b.rs"),
        "// TC-001\n#[test]\nfn tc001_second_binder() {}\n",
    )
    .expect("write b");

    let out = quire()
        .args(["coverage", "--scope", &scope, "--module", &m, "--json"])
        .output()
        .expect("run");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    let shared = report["shared_trace_ids"].as_array().expect("array");
    assert_eq!(shared.len(), 1, "one shared id, one record: {report}");
    assert_eq!(shared[0]["trace_id"].as_str(), Some("TC-001"));
    assert_eq!(
        shared[0]["symbols"].as_array().map(Vec::len),
        Some(2),
        "both distinct binders are listed: {report}"
    );
    // `vocabulary_coverage` rides the same wholesale serialization and is
    // absent when empty — the AC-2 byte-identity posture for every module
    // that has not adopted the declaration.
    assert!(
        report.get("vocabulary_coverage").is_none(),
        "an empty vocabulary_coverage must stay off the wire: {report}"
    );
}
