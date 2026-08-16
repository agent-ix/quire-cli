//! `quire coverage` + the release-coupled `validate` surfaces (quire-cli#11).
//!
//! Covers quire-rs TC-714 (generic `--summary` prefix), TC-720/721/755
//! (`--severity` override) and TC-740 (`quire coverage`).

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

/// TC-714 (FR-047-AC-8): the `--summary` histogram groups by the generic
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

/// TC-720 / TC-752 (FR-048-AC-5/AC-9): `--severity <grammar>:<check>=off`
/// suppresses the check entirely — no warning, and no row in the histogram.
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

/// TC-721 (FR-048-AC-6): `--severity …=error` promotes a check, and the run
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

/// TC-755 (FR-048-AC-10): a malformed `--severity` entry is rejected before any
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

/// TC-740 (FR-050): `quire coverage` runs the rollup and its JSON is
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

    // With a model, the JSON payload is stable across runs.
    fs::write(
        std::path::Path::new(&m).join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   document: tests.md\n    section: Test Case Summary\n\
         \x20   id_column: Test ID\n  status:\n    column: Status\n\
         \x20   complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n    retired: [\"⛔\"]\n",
    )
    .expect("rewrite manifest");

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

/// TC-797 (FR-050-AC-14, CR-035): a declared model that matches nothing is not
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

    // A model whose trace target names a document this scope does not have:
    // declared, valid, and matching zero rows.
    fs::write(
        std::path::Path::new(&m).join("manifest.yaml"),
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n  grammar_ref: iso-spec-core\n\
         traceability:\n  trace_targets:\n  - name: test-case\n\
         \x20   document: nowhere/tests.md\n    section: Test Case Summary\n\
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

/// TC-810 (FR-050-AC-17, CR-045): the document root is `<scope>/spec` — a
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
    let json = String::from_utf8_lossy(&out.stdout);
    assert!(
        json.contains("spec/tests.md"),
        "spec/ matrix must mint: {json}"
    );
    assert!(
        !json.contains("plan/tests.md") && !json.contains("\"document\": \"tests.md\""),
        "a matrix outside spec/ minted ids — the walk left the document root: {json}"
    );
    assert!(
        json.contains("\"total\": 1"),
        "exactly the one spec/ row should mint: {json}"
    );
    assert!(
        !json.contains("README") && !json.contains("CHANGELOG"),
        "repo-root strays appeared in the report: {json}"
    );
}

/// TC-811 (FR-050-AC-17, CR-045): a scope with no `spec/` directory is a
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
    assert!(
        err.contains("spec"),
        "the diagnostic must name the missing document root: {err}"
    );
}
