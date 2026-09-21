//! `quire trace` ITs — structural forward/inverse lookup over the trace-search
//! index (PLAT-879, upstream quire-rs FR-077/PLAT-844,
//! `agent-ix/quire-rs#477`).
//!
//! Claims and citations must never merge into one list distinguished by a
//! flag — that split, and cross-language honesty derived per-record from the
//! engine rather than a hardcoded table, are the two properties these tests
//! weight most heavily.

mod common;

use std::fs;

use tempfile::TempDir;

use common::quire;

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let out = quire().args(args).output().expect("run");
    Run {
        code: out.status.code(),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// A module declaring a Rust `verifies` marker AND an `implements` marker
/// (trace_tags.implements is a separate declared list, FR-062) — trace needs
/// both channels to exercise the claims/citations split for real.
fn module(dir: &TempDir) -> String {
    let m = dir.path().join("m");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        r#"name: m
manifest_version: 1.0.0
version: 0.0.1
artifact_types:
- name: FR
traceability:
  trace_targets: []
  trace_tags:
    markers:
    - name: rust-trace-attribute
      language: rust
      pattern: '#\[trace\(((?:\s*"[^"]*"\s*,?)+)\)\]'
      template: '#[trace({ids})]'
    implements:
    - name: rust-implements-line
      language: rust
      pattern: '(?m)^\s*//\s*Implements:\s*(.+)$'
"#,
    )
    .expect("write manifest");
    m.to_string_lossy().into_owned()
}

/// A module declaring no `traceability:` model at all, for the
/// `ModelUndeclared`-shaped failure path.
fn module_without_traceability(dir: &TempDir) -> String {
    let m = dir.path().join("no-model");
    fs::create_dir_all(&m).expect("mkdir");
    fs::write(
        m.join("manifest.yaml"),
        "name: no-model\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n- name: FR\n",
    )
    .expect("write manifest");
    m.to_string_lossy().into_owned()
}

/// One file carrying, of the same id `FR-900`: a `verifies` claim (a tagged
/// test), an `implements` claim (a marked production function), and a plain
/// citation (a comment on a third function matching neither declared form) —
/// the exact shape FR-077-AC-1 exists to keep structurally separate. Also
/// carries a small `FR-047`/`FR-047-AC-1`/`FR-0470` family for the
/// `--prefix` separator-boundary property, and a Python file with a plain
/// citation (no declared Python form at all) for cross-language honesty.
fn tree(dir: &TempDir) -> String {
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(
        src.join("lib.rs"),
        "#[trace(\"FR-900\")]\n#[test]\nfn tc001_verifies() {\n    assert!(true);\n}\n\n\
         // Implements: FR-900\n\
         fn implementing_fn() {}\n\n\
         fn citing_helper() {\n    // See FR-900 for background, not a tag.\n}\n\n\
         #[trace(\"FR-047\")]\n#[test]\nfn tc002_parent() {\n    assert!(true);\n}\n\n\
         #[trace(\"FR-047-AC-1\")]\n#[test]\nfn tc003_child() {\n    assert!(true);\n}\n\n\
         #[trace(\"FR-0470\")]\n#[test]\nfn tc004_decoy() {\n    assert!(true);\n}\n",
    )
    .expect("write lib.rs");

    let tests_dir = dir.path().join("tests");
    fs::create_dir_all(&tests_dir).expect("mkdir tests");
    fs::write(
        tests_dir.join("test_thing.py"),
        "class TestThing:\n    def test_thing(self):\n        \
         # See FR-901 for background, not a tag.\n        assert True\n",
    )
    .expect("write test_thing.py");

    dir.path().to_string_lossy().into_owned()
}

/// Two files, each tagging a nested, identically-bare-named `helper` symbol
/// with a DIFFERENT id — the ambiguity fixture FR-077-AC-4 exists for.
fn ambiguous_tree(dir: &TempDir) -> String {
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(
        src.join("a.rs"),
        "mod tests {\n    fn outer() {\n        #[trace(\"FR-910\")]\n        #[test]\n        \
         fn helper() {\n            assert!(true);\n        }\n    }\n}\n",
    )
    .expect("write a.rs");
    fs::write(
        src.join("b.rs"),
        "mod tests {\n    fn outer() {\n        #[trace(\"FR-911\")]\n        #[test]\n        \
         fn helper() {\n            assert!(true);\n        }\n    }\n}\n",
    )
    .expect("write b.rs");
    dir.path().to_string_lossy().into_owned()
}

/// Two symbols tagging the SAME requirement under two DIFFERENT raw
/// spellings — `FR-950` and `FR_950`, which the engine's own
/// `normalized_trace_id` fold treats as identical (strips all punctuation,
/// uppercases) — for the H1 (case/separator folding) and H2 (double-count)
/// regression fixtures.
fn prefix_fold_tree(dir: &TempDir) -> String {
    let src = dir.path().join("src");
    fs::create_dir_all(&src).expect("mkdir src");
    fs::write(
        src.join("fold.rs"),
        "#[trace(\"FR-950\")]\n#[test]\nfn tc_dash_spelling() {\n    assert!(true);\n}\n\n\
         #[trace(\"FR_950\")]\n#[test]\nfn tc_underscore_spelling() {\n    assert!(true);\n}\n",
    )
    .expect("write fold.rs");
    dir.path().to_string_lossy().into_owned()
}

#[test]
fn claims_and_citations_are_structurally_separate_json_subtrees() {
    // FR-077-AC-1: the whole reason this tool exists. `.claims` and
    // `.citations` must be separate keys, and the same trace id's claim and
    // citation records must never collapse into one homogeneous list.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "FR-900", "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    assert_eq!(payload["resolved"], true, "{payload}");

    let verifies = payload["claims"]["verifies"].as_array().expect("verifies");
    assert_eq!(verifies.len(), 1, "{payload}");
    assert_eq!(verifies[0]["symbol"], "tc001_verifies", "{payload}");
    // Rust is structural (PLAT-843) — derived from the engine's own
    // `language_confidence`, never a literal here.
    assert_eq!(verifies[0]["language"], "rust", "{payload}");
    assert_eq!(verifies[0]["confidence"], "structural", "{payload}");

    let implements = payload["claims"]["implements"]
        .as_array()
        .expect("implements");
    assert_eq!(implements.len(), 1, "{payload}");
    assert_eq!(implements[0]["symbol"], "implementing_fn", "{payload}");

    let citations = payload["citations"].as_array().expect("citations");
    assert!(
        citations
            .iter()
            .any(|c| c["symbol"] == "citing_helper" && c["trace_id"] == "FR-900"),
        "{payload}"
    );
    // The structural guarantee itself: the citation's symbol must never also
    // appear as a claim, and vice versa.
    assert!(
        !verifies.iter().any(|v| v["symbol"] == "citing_helper"),
        "a citation must never appear among claims: {payload}"
    );
    assert!(
        !citations
            .iter()
            .any(|c| c["symbol"] == "tc001_verifies" || c["symbol"] == "implementing_fn"),
        "a bound claim must never appear among citations: {payload}"
    );
}

#[test]
fn human_output_headers_the_citations_section_as_not_evidence() {
    // The literal heading the ticket requires, word for word.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);
    let out = run(&["trace", "--scope", &scope, "--module", &m, "--id", "FR-900"]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    // The ONE required line, whole and contiguous (review finding #9: two
    // independent `contains` checks would still pass if the count and the
    // trailer text ended up on two separate lines, which is not the
    // requirement — "Citations (N) — NOT verification evidence" must be one
    // line). FR-900 in this fixture has exactly one citation (citing_helper).
    assert!(
        out.stdout
            .contains("Citations (1) — NOT verification evidence"),
        "stdout: {}",
        out.stdout
    );
    assert!(out.stdout.contains("Claims ("), "stdout: {}", out.stdout);
}

#[test]
fn zero_match_query_is_a_full_shape_not_a_tool_error() {
    // A valid query that finds nothing is not a tool error: exit 0, and
    // `resolved: false` rather than an empty `{}`.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace",
        "--scope",
        &scope,
        "--module",
        &m,
        "--id",
        "FR-999-NOWHERE",
        "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    assert_eq!(payload["resolved"], false, "{payload}");
    assert!(payload["claims"]["verifies"]
        .as_array()
        .expect("verifies")
        .is_empty());
    assert!(payload["claims"]["implements"]
        .as_array()
        .expect("implements")
        .is_empty());
    assert!(payload["citations"]
        .as_array()
        .expect("citations")
        .is_empty());
}

#[test]
fn prefix_matches_on_a_separator_boundary_not_a_digit_run() {
    // FR-077-AC-6 one level up: --prefix must union FR-047 and FR-047-AC-1
    // but never FR-0470.
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "FR-047", "--prefix", "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    let ids: Vec<String> = payload["claims"]["verifies"]
        .as_array()
        .expect("verifies")
        .iter()
        .map(|v| v["trace_id"].as_str().unwrap().to_string())
        .collect();
    assert!(ids.contains(&"FR-047".to_string()), "{payload}");
    assert!(ids.contains(&"FR-047-AC-1".to_string()), "{payload}");
    assert!(
        !ids.contains(&"FR-0470".to_string()),
        "FR-047 must not match FR-0470: {payload}"
    );

    // Without --prefix, an exact --id FR-047 must NOT pull in FR-047-AC-1.
    let exact = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "FR-047", "--json",
    ]);
    let exact_payload: serde_json::Value = serde_json::from_str(&exact.stdout).expect("json");
    let exact_ids: Vec<String> = exact_payload["claims"]["verifies"]
        .as_array()
        .expect("verifies")
        .iter()
        .map(|v| v["trace_id"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(exact_ids, vec!["FR-047".to_string()], "{exact_payload}");
}

// Review finding H1, end to end: `--prefix` must fold case and separator
// choice the same way the engine's own exact `--id` match does — reproduced
// broken (empty result) against the pre-fix `prefix_matches` before the
// `id_segments` rewrite.
#[test]
fn prefix_folds_case_and_separator_like_an_exact_id_query_does() {
    let dir = TempDir::new().expect("tempdir");
    let scope = prefix_fold_tree(&dir);
    let m = module(&dir);

    // Lowercase, hyphenated query must still find the hyphenated tag.
    let lower = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "fr-950", "--prefix", "--json",
    ]);
    assert_eq!(lower.code, Some(0), "stderr: {}", lower.stderr);
    let lower_payload: serde_json::Value = serde_json::from_str(&lower.stdout).expect("json");
    assert_eq!(lower_payload["resolved"], true, "{lower_payload}");
    let lower_symbols: Vec<String> = lower_payload["claims"]["verifies"]
        .as_array()
        .expect("verifies")
        .iter()
        .map(|v| v["symbol"].as_str().unwrap().to_string())
        .collect();
    assert!(
        lower_symbols.contains(&"tc_dash_spelling".to_string()),
        "{lower_payload}"
    );

    // Underscore query must still find the hyphenated tag (the engine folds
    // `-`/`_` as equivalent separators).
    let underscore = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "FR_950", "--prefix", "--json",
    ]);
    let underscore_payload: serde_json::Value =
        serde_json::from_str(&underscore.stdout).expect("json");
    let underscore_symbols: Vec<String> = underscore_payload["claims"]["verifies"]
        .as_array()
        .expect("verifies")
        .iter()
        .map(|v| v["symbol"].as_str().unwrap().to_string())
        .collect();
    assert!(
        underscore_symbols.contains(&"tc_dash_spelling".to_string()),
        "{underscore_payload}"
    );

    // A trailing separator on the query changes nothing.
    let trailing = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "FR-950-", "--prefix", "--json",
    ]);
    let trailing_payload: serde_json::Value = serde_json::from_str(&trailing.stdout).expect("json");
    assert_eq!(
        trailing_payload["claims"]["verifies"]
            .as_array()
            .expect("verifies")
            .len(),
        2,
        "{trailing_payload}"
    );
}

// Review finding H2, end to end: two raw spellings of the same requirement
// (`FR-950`, `FR_950`) that fold to the same normalized id must be searched
// once between them, not once each — reproduced broken (4 rows instead of 2)
// before the normalized-dedup fix in `search_by_prefix`.
#[test]
fn prefix_does_not_double_count_ids_that_normalize_the_same() {
    let dir = TempDir::new().expect("tempdir");
    let scope = prefix_fold_tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "FR-950", "--prefix", "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    let verifies = payload["claims"]["verifies"].as_array().expect("verifies");
    // Exactly one row per symbol — never two, which is what searching once
    // per raw spelling (rather than once per normalized-equivalence class)
    // used to produce.
    assert_eq!(verifies.len(), 2, "{payload}");
    let mut symbols: Vec<&str> = verifies
        .iter()
        .map(|v| v["symbol"].as_str().unwrap())
        .collect();
    symbols.sort();
    assert_eq!(symbols, vec!["tc_dash_spelling", "tc_underscore_spelling"]);

    // `matched_ids` reports BOTH distinct raw spellings — transparency is
    // preserved even though the search itself deduplicated.
    let matched_ids = payload["query"]["matched_ids"]
        .as_array()
        .expect("matched_ids");
    let mut matched: Vec<&str> = matched_ids.iter().map(|v| v.as_str().unwrap()).collect();
    matched.sort();
    assert_eq!(matched, vec!["FR-950", "FR_950"]);

    // The human form surfaces `matched_ids` too (review finding #12).
    let human = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "FR-950", "--prefix",
    ]);
    assert!(
        human.stdout.contains("matched ids (2)"),
        "stdout: {}",
        human.stdout
    );
}

#[test]
fn ambiguous_bare_symbol_name_lists_every_candidate() {
    // FR-077-AC-4: never a silent pick.
    let dir = TempDir::new().expect("tempdir");
    let scope = ambiguous_tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace", "--scope", &scope, "--module", &m, "--symbol", "helper", "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    assert_eq!(payload["resolved"], true, "{payload}");
    let mut matches: Vec<String> = payload["ambiguous_matches"]
        .as_array()
        .expect("ambiguous_matches")
        .iter()
        .map(|m| m.as_str().unwrap().to_string())
        .collect();
    matches.sort();
    assert_eq!(
        matches,
        vec![
            "src/a.rs#tests::outer::helper".to_string(),
            "src/b.rs#tests::outer::helper".to_string(),
        ],
        "{payload}"
    );
    assert!(payload["claims"]["verifies"]
        .as_array()
        .expect("verifies")
        .is_empty());

    // The exact ref form resolves unambiguously.
    let exact = run(&[
        "trace",
        "--scope",
        &scope,
        "--module",
        &m,
        "--symbol",
        "src/a.rs#tests::outer::helper",
        "--json",
    ]);
    let exact_payload: serde_json::Value = serde_json::from_str(&exact.stdout).expect("json");
    let verifies = exact_payload["claims"]["verifies"]
        .as_array()
        .expect("verifies");
    assert_eq!(verifies.len(), 1, "{exact_payload}");
    assert_eq!(verifies[0]["trace_id"], "FR-910", "{exact_payload}");
    assert!(exact_payload["ambiguous_matches"]
        .as_array()
        .expect("ambiguous_matches")
        .is_empty());
}

#[test]
fn file_query_returns_every_claim_and_citation_in_that_file() {
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace",
        "--scope",
        &scope,
        "--module",
        &m,
        "--file",
        "src/lib.rs",
        "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    let verifies = payload["claims"]["verifies"].as_array().expect("verifies");
    // FR-900, FR-047, FR-047-AC-1, FR-0470 all in src/lib.rs.
    assert_eq!(verifies.len(), 4, "{payload}");
}

// Review finding #7: this test's ORIGINAL name claimed to prove the absence
// of a hardcoded per-language table. It cannot — a hidden
// `match language { "rust" => "structural", _ => "line_heuristic" }` inside
// this crate would pass every assertion below identically, since Rust really
// is `structural` and Python really is `line_heuristic` either way. What
// actually establishes "derived, not hardcoded" is source inspection (no
// such match arm exists anywhere in `src/commands/trace.rs`; every
// `confidence` field is computed by calling the engine's own
// `language_confidence`/`symbol_language` — confirmed during review). This
// test's real job, honestly named, is a regression pin: if the engine's
// mapping for either language ever changes, or a hardcoded table gets
// introduced later with the WRONG values, this fails.
#[test]
fn cross_language_confidence_matches_the_engines_reported_values() {
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace", "--scope", &scope, "--module", &m, "--id", "FR-901", "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    let citations = payload["citations"].as_array().expect("citations");
    assert_eq!(citations.len(), 1, "{payload}");
    assert_eq!(citations[0]["language"], "python", "{payload}");
    assert_eq!(citations[0]["confidence"], "line_heuristic", "{payload}");

    // And the human caveat is present because THIS result set actually
    // contains a line_heuristic record — not because "python" is
    // special-cased anywhere in this crate.
    let human = run(&["trace", "--scope", &scope, "--module", &m, "--id", "FR-901"]);
    assert!(
        human.stderr.contains("line_heuristic"),
        "stderr: {}",
        human.stderr
    );

    // The Rust-only FR-900 query, by contrast, carries no line_heuristic
    // record and must print no caveat.
    let rust_only = run(&["trace", "--scope", &scope, "--module", &m, "--id", "FR-900"]);
    assert!(
        !rust_only.stderr.contains("line_heuristic"),
        "stderr: {}",
        rust_only.stderr
    );
}

#[test]
fn exclude_path_drops_matching_citations_and_reports_the_count() {
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace",
        "--scope",
        &scope,
        "--module",
        &m,
        "--id",
        "FR-900",
        "--exclude-path",
        "src/**",
        "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    assert!(payload["citations"]
        .as_array()
        .expect("citations")
        .is_empty());
    assert_eq!(payload["citations_excluded_by_path_filter"], 1, "{payload}");
    // Claims are NEVER touched by --exclude-path (it is scoped to citations
    // only, the exact false-positive class it exists to filter).
    assert_eq!(
        payload["claims"]["verifies"]
            .as_array()
            .expect("verifies")
            .len(),
        1,
        "{payload}"
    );
}

// Review finding #3: `resolved` must be recomputed AFTER `--exclude-path`
// filtering, not left as the engine's pre-filter verdict. FR-901 resolves to
// exactly one citation and zero claims (the Python fixture in `tree()`), so
// excluding its only path empties the result entirely — `resolved` must flip
// to `false`. Reproduced broken (stayed `true`) before the recompute fix.
#[test]
fn exclude_path_that_empties_the_result_flips_resolved_to_false() {
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);
    let out = run(&[
        "trace",
        "--scope",
        &scope,
        "--module",
        &m,
        "--id",
        "FR-901",
        "--exclude-path",
        "tests/**",
        "--json",
    ]);
    assert_eq!(out.code, Some(0), "stderr: {}", out.stderr);
    let payload: serde_json::Value = serde_json::from_str(&out.stdout).expect("json");
    assert_eq!(payload["citations_excluded_by_path_filter"], 1, "{payload}");
    assert!(payload["citations"]
        .as_array()
        .expect("citations")
        .is_empty());
    assert_eq!(
        payload["resolved"], false,
        "an exclude-path that empties the whole result must flip resolved: {payload}"
    );
}

#[test]
fn exactly_one_selector_is_required() {
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module(&dir);

    let none = run(&["trace", "--scope", &scope, "--module", &m]);
    assert_ne!(none.code, Some(0), "stdout: {}", none.stdout);
    assert!(
        none.stderr.contains("exactly one"),
        "stderr: {}",
        none.stderr
    );

    let both = run(&[
        "trace",
        "--scope",
        &scope,
        "--module",
        &m,
        "--id",
        "FR-900",
        "--file",
        "src/lib.rs",
    ]);
    assert_ne!(both.code, Some(0), "stdout: {}", both.stdout);
}

#[test]
fn missing_traceability_model_fails_with_the_coverage_shaped_error() {
    let dir = TempDir::new().expect("tempdir");
    let scope = tree(&dir);
    let m = module_without_traceability(&dir);
    let out = run(&["trace", "--scope", &scope, "--module", &m, "--id", "FR-900"]);
    assert_ne!(out.code, Some(0), "stdout: {}", out.stdout);
    assert!(out.stderr.contains("traceability"), "{}", out.stderr);
}
