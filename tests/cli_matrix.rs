//! `quire matrix`: the computed coverage matrix rendered as markdown, JSON or
//! TSV. Trace ids sit on the tests, not in this header — a `//!` block
//! attaches to the file and binds to no symbol (agent-ix/quire-cli#43).

mod common;

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Output;

use serde_json::Value;
use tempfile::TempDir;

/// The markers the fixture module reads: a Rust attribute and a TypeScript
/// helper call, so a same-line pair of `it(...)` registrations can bind.
const TRACE_TAGS: &str = "  trace_tags:\n    markers:\n\
    \x20   - name: rust-trace-attribute\n      language: rust\n\
    \x20     pattern: '#\\[trace\\(([^)]*)\\)\\]'\n      template: '#[trace({ids})]'\n\
    \x20   - name: ts-trace-helper\n      language: typescript\n\
    \x20     pattern: '\\btrace\\(([^)]*)\\)'\n";

/// A module whose `traceability:` model mints acceptance criteria from FR
/// documents. `obligations` adds the source `coverage_matrix` is built from;
/// `traceability_extra` and `top_level_extra` splice further declarations in.
fn manifest(obligations: bool, traceability_extra: &str, top_level_extra: &str) -> String {
    let obligations = if obligations {
        "  obligations:\n  - name: acceptance-criterion\n    target: acceptance-criterion\n\
         \x20   statement_column: Criteria\n    method_column: Verification\n"
    } else {
        ""
    };
    format!(
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: FR\n- name: TestMatrix\n{top_level_extra}\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n\
         \x20   archetype: FR\n    section: Acceptance Criteria\n    id_column: ID\n\
         {traceability_extra}\
         \x20 status:\n    column: Status\n    complete: [\"✅\"]\n    pending: [\"🚧\"]\n\
         \x20   failed: [\"❌\"]\n\
         {obligations}\
         \x20 vocabularies:\n    test_type_column: Type\n    test_type: [Test, Inspection]\n\
         \x20   no_source_symbol: [Inspection]\n\
         {TRACE_TAGS}"
    )
}

/// An FR document whose acceptance-criteria rows are `(id, criteria,
/// verification)`.
fn fr_doc(id: &str, rows: &[(&str, &str, &str)]) -> String {
    let mut doc = format!(
        "---\nid: {id}\ntype: FR\n---\n## Acceptance Criteria\n\n\
         | ID | Criteria | Verification |\n|----|----------|--------------|\n"
    );
    for (ac, criteria, method) in rows {
        doc.push_str(&format!("| {ac} | {criteria} | {method} |\n"));
    }
    doc
}

/// A Rust test tagged with `id`; `ignored` adds `#[ignore]`.
fn rust_test(name: &str, id: &str, ignored: bool) -> String {
    let ignore = if ignored { "#[ignore]\n" } else { "" };
    format!("#[trace(\"{id}\")]\n{ignore}#[test]\nfn {name}() {{}}\n\n")
}

struct Fixture {
    dir: TempDir,
}

impl Fixture {
    fn new(manifest: &str) -> Self {
        let dir = TempDir::new().expect("tempdir");
        fs::create_dir_all(dir.path().join("m")).expect("mkdir m");
        fs::create_dir_all(dir.path().join("repo/spec")).expect("mkdir spec");
        fs::write(dir.path().join("m/manifest.yaml"), manifest).expect("write manifest");
        Self { dir }
    }

    /// The default module: obligations declared, nothing else.
    fn with_obligations() -> Self {
        Self::new(&manifest(true, "", ""))
    }

    fn scope(&self) -> String {
        self.dir.path().join("repo").to_string_lossy().into_owned()
    }

    fn module(&self) -> String {
        self.dir.path().join("m").to_string_lossy().into_owned()
    }

    fn write(&self, rel: &str, body: &str) {
        let path = self.dir.path().join("repo").join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
        fs::write(path, body).expect("write");
    }

    fn quire(&self, command: &str, extra: &[&str]) -> Output {
        common::quire()
            .args([
                command,
                "--scope",
                &self.scope(),
                "--module",
                &self.module(),
            ])
            .args(extra)
            .output()
            .expect("run quire")
    }

    fn matrix(&self, extra: &[&str]) -> Output {
        self.quire("matrix", extra)
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {}", stderr(out)))
}

fn code(out: &Output) -> i32 {
    out.status.code().expect("exited with a code")
}

/// Two FR documents, three criteria between them: one tagged, one untagged,
/// one tagged only by an ignored test.
fn populated() -> Fixture {
    let f = Fixture::with_obligations();
    f.write(
        "spec/FR-001.md",
        &fr_doc(
            "FR-001",
            &[
                ("FR-001-AC-1", "The first criterion.", "Test"),
                ("FR-001-AC-2", "The second criterion.", "Test"),
            ],
        ),
    );
    f.write(
        "spec/FR-002.md",
        &fr_doc("FR-002", &[("FR-002-AC-1", "The third criterion.", "Test")]),
    );
    f.write(
        "src/lib.rs",
        &format!(
            "{}{}",
            rust_test("first", "FR-001-AC-1", false),
            rust_test("third", "FR-002-AC-1", true)
        ),
    );
    f
}

/// The same scope and modules through `coverage --json`, as the reference for
/// what the engine computed.
fn engine_matrix(f: &Fixture) -> Option<Value> {
    let out = f.quire("coverage", &["--json"]);
    json(&out).get("coverage_matrix").cloned()
}

// Trace: IT-166, FR-026-AC-1
#[test]
fn it166_a_populated_scope_renders_every_criterion_in_every_format() {
    let f = populated();
    let ids = ["FR-001-AC-1", "FR-001-AC-2", "FR-002-AC-1"];

    let markdown = f.matrix(&[]);
    assert_eq!(code(&markdown), 0, "{}", stderr(&markdown));
    let tsv = f.matrix(&["--format", "tsv"]);
    assert_eq!(code(&tsv), 0, "{}", stderr(&tsv));
    let json_out = f.matrix(&["--format", "json"]);
    assert_eq!(code(&json_out), 0, "{}", stderr(&json_out));

    let (markdown, tsv, payload) = (stdout(&markdown), stdout(&tsv), json(&json_out));
    let json_ids: Vec<&str> = payload["coverage_matrix"]
        .as_array()
        .expect("coverage_matrix array")
        .iter()
        .flat_map(|r| r["criteria"].as_array().expect("criteria"))
        .map(|c| c["id"].as_str().expect("id"))
        .collect();
    assert_eq!(json_ids, ids);
    for id in ids {
        assert!(markdown.contains(&format!("| {id} |")), "{id}: {markdown}");
        assert!(tsv.contains(&format!("\t{id}\t")), "{id}: {tsv}");
    }
}

// Trace: IT-167, FR-026-AC-2
#[test]
fn it167_matrix_shares_coverage_refusals_and_module_resolution() {
    // No `spec/` under the scope: the typed MissingDocumentRoot refusal.
    let f = Fixture::with_obligations();
    fs::remove_dir(f.dir.path().join("repo/spec")).expect("rmdir spec");
    let out = common::quire()
        .args(["--diagnostics-format", "json", "matrix", "--scope"])
        .args([f.scope(), "--module".to_string(), f.module()])
        .output()
        .expect("run");
    assert_eq!(code(&out), 1);
    assert!(out.stdout.is_empty(), "a refusal renders nothing");
    let line = stderr(&out)
        .lines()
        .find(|l| l.contains("MissingDocumentRoot"))
        .map(str::to_string)
        .unwrap_or_else(|| panic!("no typed refusal: {}", stderr(&out)));
    let parsed: Value = serde_json::from_str(&line).expect("json diagnostic");
    assert_eq!(parsed["kind"], "MissingDocumentRoot");

    // No `traceability:` model in scope.
    let bare = Fixture::new(
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n- name: FR\n",
    );
    let out = bare.matrix(&[]);
    assert_eq!(code(&out), 1);
    assert!(
        stderr(&out).contains("`traceability:` model"),
        "{}",
        stderr(&out)
    );

    // `--module` is closed: a populated module reachable only through the
    // ambient path is not consulted, so the named module's empty matrix wins.
    let named = Fixture::new(&manifest(false, "", ""));
    named.write(
        "spec/FR-001.md",
        &fr_doc("FR-001", &[("FR-001-AC-1", "A criterion.", "Test")]),
    );
    let ambient = named.dir.path().join("ambient/m2");
    fs::create_dir_all(&ambient).expect("mkdir ambient");
    fs::write(
        ambient.join("manifest.yaml"),
        manifest(true, "", "").replace("name: m\n", "name: m2\n"),
    )
    .expect("write ambient");
    let out = common::quire()
        .args([
            "matrix",
            "--scope",
            &named.scope(),
            "--module",
            &named.module(),
        ])
        .env("IX_FILAMENT_MODULES_PATH", named.dir.path().join("ambient"))
        .output()
        .expect("run");
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(stdout(&out), "No obligations matched this scope.\n");

    // `--module` is repeatable: the second root joins the set.
    let other = named.dir.path().join("other");
    fs::create_dir_all(&other).expect("mkdir other");
    fs::write(
        other.join("manifest.yaml"),
        "name: other\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n- name: US\n",
    )
    .expect("write other");
    let populated = Fixture::with_obligations();
    populated.write(
        "spec/FR-001.md",
        &fr_doc("FR-001", &[("FR-001-AC-1", "A criterion.", "Test")]),
    );
    let out = common::quire()
        .args(["matrix", "--scope", &populated.scope()])
        .args(["--module", &populated.module(), "--module"])
        .arg(&other)
        .output()
        .expect("run");
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("| FR-001-AC-1 |"), "{}", stdout(&out));

    // `--help` states the resolution order, as `coverage --help` does.
    let help = common::quire()
        .args(["matrix", "--help"])
        .output()
        .expect("run --help");
    let help = stdout(&help);
    assert!(help.contains("--module"), "{help}");
    assert!(help.contains("REPLACE ambient discovery"), "{help}");
    assert!(help.contains("used in the order given"), "{help}");
}

// Trace: IT-168, FR-026-AC-3, FR-026-AC-4
#[test]
fn it168_markdown_renders_one_table_per_requirement_in_engine_order() {
    // Documents written in reverse order and criteria out of id order: the
    // rendering follows the engine, never a sort of its own.
    let f = Fixture::with_obligations();
    f.write(
        "spec/FR-002.md",
        &fr_doc("FR-002", &[("FR-002-AC-1", "Second document.", "Test")]),
    );
    f.write(
        "spec/FR-001.md",
        &fr_doc(
            "FR-001",
            &[
                ("FR-001-AC-2", "Listed first.", "Test"),
                ("FR-001-AC-1", "Listed second.", "Test"),
            ],
        ),
    );
    let out = f.matrix(&[]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let engine = engine_matrix(&f).expect("engine emits coverage_matrix");
    let mut expected = String::new();
    for (i, requirement) in engine.as_array().expect("array").iter().enumerate() {
        if i > 0 {
            expected.push('\n');
        }
        expected.push_str(&format!(
            "## {}\n\n| Criterion | Statement | Binders | Status |\n|---|---|---|---|\n",
            requirement["document"].as_str().expect("document")
        ));
        for c in requirement["criteria"].as_array().expect("criteria") {
            expected.push_str(&format!(
                "| {} | {} | (none) | untagged |\n",
                c["id"].as_str().expect("id"),
                c["statement"].as_str().expect("statement")
            ));
        }
    }
    assert_eq!(stdout(&out), expected);
    let headings: Vec<&str> = expected.lines().filter(|l| l.starts_with("## ")).collect();
    assert_eq!(headings, ["## spec/FR-001.md", "## spec/FR-002.md"]);

    // The pinned worked example, byte for byte.
    let worked = Fixture::with_obligations();
    worked.write(
        "spec/functional/FR-009-debug-headers.md",
        &fr_doc(
            "FR-009",
            &[(
                "FR-009-AC-3",
                "Requests carrying X-Debug\\|X-Trace headers are logged verbatim",
                "Test",
            )],
        ),
    );
    // The test declaration opens on line 41, column 9.
    let mut source: String = (1..=36).map(|i| format!("// filler {i}\n")).collect();
    source.push_str(
        "mod tests {\n    mod inner {\n        #[trace(\"FR-009-AC-3\")]\n        \
         #[test]\n        fn logs() {}\n    }\n}\n",
    );
    worked.write("src/debug.rs", &source);
    let out = worked.matrix(&[]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        include_str!("snapshots/matrix-worked-example.md")
    );
}

// Trace: IT-169, FR-026-AC-5
#[test]
fn it169_binders_render_path_line_column_with_ignored_and_none_markers() {
    let f = Fixture::with_obligations();
    f.write(
        "spec/FR-001.md",
        &fr_doc(
            "FR-001",
            &[
                ("FR-001-AC-1", "Two binders on one line.", "Test"),
                ("FR-001-AC-2", "One running and one ignored binder.", "Test"),
                ("FR-001-AC-3", "No binder.", "Test"),
            ],
        ),
    );
    f.write(
        "src/service.test.ts",
        "it('a', () => { trace('FR-001-AC-1'); }); it('b', () => { trace('FR-001-AC-1'); });\n",
    );
    f.write(
        "src/lib.rs",
        &format!(
            "{}{}",
            rust_test("running", "FR-001-AC-2", false),
            rust_test("skipped", "FR-001-AC-2", true)
        ),
    );
    let out = f.matrix(&[]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let markdown = stdout(&out);
    let row = |id: &str| -> Vec<String> {
        let line = markdown
            .lines()
            .find(|l| l.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("no row {id}: {markdown}"));
        line.trim_matches('|')
            .split(" | ")
            .map(|c| c.trim().to_string())
            .collect()
    };

    let engine = engine_matrix(&f).expect("coverage_matrix");
    let binders = |id: &str| -> Vec<(String, u64, u64, bool)> {
        engine[0]["criteria"]
            .as_array()
            .expect("criteria")
            .iter()
            .find(|c| c["id"] == id)
            .expect("criterion")["binders"]
            .as_array()
            .expect("binders")
            .iter()
            .map(|b| {
                (
                    b["path"].as_str().expect("path").to_string(),
                    b["line"].as_u64().expect("line"),
                    b["column"].as_u64().expect("column"),
                    b["ignored"].as_bool().unwrap_or(false),
                )
            })
            .collect()
    };

    let same_line = binders("FR-001-AC-1");
    assert_eq!(same_line.len(), 2, "{same_line:?}");
    assert_eq!(same_line[0].1, same_line[1].1, "one line");
    assert_ne!(same_line[0].2, same_line[1].2, "two columns");
    assert_eq!(
        row("FR-001-AC-1")[2],
        format!(
            "src/service.test.ts:1:{}, src/service.test.ts:1:{}",
            same_line[0].2, same_line[1].2
        )
    );

    let mixed = binders("FR-001-AC-2");
    let expected: Vec<String> = mixed
        .iter()
        .map(|(path, line, column, ignored)| {
            let marker = if *ignored { " (ignored)" } else { "" };
            format!("{path}:{line}:{column}{marker}")
        })
        .collect();
    assert_eq!(
        mixed.iter().filter(|b| b.3).count(),
        1,
        "exactly one binder is ignored: {mixed:?}"
    );
    assert_eq!(row("FR-001-AC-2")[2], expected.join(", "));
    assert!(row("FR-001-AC-2")[2].contains(" (ignored)"));
    assert_eq!(row("FR-001-AC-2")[3], "tagged");

    assert_eq!(row("FR-001-AC-3")[2], "(none)");
    assert_eq!(row("FR-001-AC-3")[3], "untagged");
}

// Trace: IT-170, FR-026-AC-6
#[test]
fn it170_statements_truncate_to_eighty_scalar_values_before_escaping() {
    let short = "Short and whole.";
    // Exactly 80 scalar values, multi-byte: must render whole.
    let exact: String = "é".repeat(80);
    // 76 letters, a pipe at scalar 77, then more: the cut keeps the pipe, and
    // only then is it escaped.
    let piped = format!("{}|{}", "a".repeat(76), "tail ".repeat(10).trim_end());
    let f = Fixture::with_obligations();
    f.write(
        "spec/FR-001.md",
        &fr_doc(
            "FR-001",
            &[
                ("FR-001-AC-1", short, "Test"),
                ("FR-001-AC-2", &exact, "Test"),
                ("FR-001-AC-3", &piped.replace('|', "\\|"), "Test"),
            ],
        ),
    );
    let engine = engine_matrix(&f).expect("coverage_matrix");
    let statements: Vec<String> = engine[0]["criteria"]
        .as_array()
        .expect("criteria")
        .iter()
        .map(|c| c["statement"].as_str().expect("statement").to_string())
        .collect();
    assert_eq!(statements, [short, exact.as_str(), piped.as_str()]);

    let markdown = stdout(&f.matrix(&[]));
    let cell = |id: &str| -> String {
        let line = markdown
            .lines()
            .find(|l| l.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("no row {id}: {markdown}"));
        let rest = line.strip_prefix(&format!("| {id} | ")).expect("prefix");
        rest[..rest.find(" | ").expect("cell end")].to_string()
    };
    assert_eq!(cell("FR-001-AC-1"), short);
    assert_eq!(cell("FR-001-AC-2"), exact);
    let cut = cell("FR-001-AC-3");
    assert_eq!(cut, format!("{}\\|...", "a".repeat(76)));
    assert_eq!(
        cut.replace("\\|", "|").chars().count(),
        80,
        "the escape is not counted toward the budget"
    );

    let payload = json(&f.matrix(&["--format", "json"]));
    assert_eq!(
        payload["coverage_matrix"][0]["criteria"][2]["statement"],
        piped.as_str()
    );
    let tsv = stdout(&f.matrix(&["--format", "tsv"]));
    assert!(
        tsv.lines().any(|l| l.ends_with(&format!("\t{piped}"))),
        "TSV carries the untruncated, unescaped statement: {tsv}"
    );
}

// Trace: IT-171, FR-026-AC-7
#[test]
fn it171_json_wraps_the_engine_value_verbatim_with_provenance() {
    let f = populated();
    let first = f.matrix(&["--format", "json"]);
    assert_eq!(code(&first), 0, "{}", stderr(&first));
    let second = f.matrix(&["--format", "json"]);
    assert_eq!(first.stdout, second.stdout, "two runs are byte-identical");

    let payload = json(&first);
    let keys: Vec<&String> = payload.as_object().expect("object").keys().collect();
    assert_eq!(keys, ["coverage_matrix", "engine"]);
    assert_eq!(
        Some(payload["coverage_matrix"].clone()),
        engine_matrix(&f),
        "the engine's own value, unmodified"
    );
    assert!(
        payload["coverage_matrix"].is_array(),
        "a bare array of requirement entries: {payload}"
    );
    for key in ["cli", "engine", "capabilities"] {
        assert!(
            payload["engine"].get(key).is_some(),
            "engine.{key}: {payload}"
        );
    }

    // The engine omits the field when no `obligations:` source is declared,
    // and so does this surface.
    let none = Fixture::new(&manifest(false, "", ""));
    none.write(
        "spec/FR-001.md",
        &fr_doc("FR-001", &[("FR-001-AC-1", "A criterion.", "Test")]),
    );
    assert_eq!(engine_matrix(&none), None);
    let payload = json(&none.matrix(&["--format", "json"]));
    let keys: Vec<&String> = payload.as_object().expect("object").keys().collect();
    assert_eq!(keys, ["engine"]);
}

// Trace: IT-172, FR-026-AC-8
#[test]
fn it172_tsv_emits_one_untruncated_record_per_criterion() {
    let f = populated();
    let long = format!("{} | with a pipe", "x".repeat(90));
    f.write(
        "spec/FR-002.md",
        &fr_doc(
            "FR-002",
            &[("FR-002-AC-1", &long.replace('|', "\\|"), "Test")],
        ),
    );
    f.write(
        "src/lib.rs",
        &format!(
            "{}{}{}",
            rust_test("first", "FR-001-AC-1", false),
            rust_test("first_again", "FR-001-AC-1", false),
            rust_test("third", "FR-002-AC-1", true)
        ),
    );
    let out = f.matrix(&["--format", "tsv"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let engine = engine_matrix(&f).expect("coverage_matrix");
    let mut expected = String::from("document\tcriterion\tstatus\tbinders\tstatement\n");
    for requirement in engine.as_array().expect("array") {
        for c in requirement["criteria"].as_array().expect("criteria") {
            let binders: Vec<String> = c["binders"]
                .as_array()
                .expect("binders")
                .iter()
                .map(|b| {
                    let marker = if b["ignored"] == true {
                        " (ignored)"
                    } else {
                        ""
                    };
                    format!(
                        "{}:{}:{}{marker}",
                        b["path"].as_str().unwrap(),
                        b["line"],
                        b["column"]
                    )
                })
                .collect();
            expected.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\n",
                requirement["document"].as_str().unwrap(),
                c["id"].as_str().unwrap(),
                c["status"].as_str().unwrap(),
                binders.join(","),
                c["statement"].as_str().unwrap()
            ));
        }
    }
    let tsv = stdout(&out);
    assert_eq!(tsv, expected);
    assert!(tsv.contains(&format!("\t{long}\n")), "{tsv}");
    assert!(tsv.contains("(ignored)"), "{tsv}");
    assert!(
        tsv.lines()
            .any(|l| l.contains("\tFR-001-AC-1\ttagged\tsrc/lib.rs:")
                && l.matches(",src/lib.rs:").count() == 1),
        "two binders, comma-separated: {tsv}"
    );
    assert!(
        tsv.contains("\tFR-001-AC-2\tuntagged\t\tThe second criterion.\n"),
        "zero binders render empty, not (none): {tsv}"
    );

    let empty = Fixture::with_obligations();
    let out = empty.matrix(&["--format", "tsv"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "document\tcriterion\tstatus\tbinders\tstatement\n"
    );
}

/// The two zero-population scopes: a module with no `obligations:` source,
/// and one whose source derives nothing (no FR document in scope).
fn zero_populations() -> [Fixture; 2] {
    let undeclared = Fixture::new(&manifest(false, "", ""));
    undeclared.write(
        "spec/FR-001.md",
        &fr_doc("FR-001", &[("FR-001-AC-1", "A criterion.", "Test")]),
    );
    undeclared.write("src/lib.rs", &rust_test("first", "FR-001-AC-1", false));
    let derives_nothing = Fixture::with_obligations();
    derives_nothing.write("spec/README.md", "# nothing typed here\n");
    [undeclared, derives_nothing]
}

// Trace: IT-173, FR-026-AC-9
#[test]
fn it173_both_zero_population_shapes_render_identically() {
    for f in zero_populations() {
        let markdown = f.matrix(&[]);
        assert_eq!(code(&markdown), 0, "{}", stderr(&markdown));
        assert_eq!(stdout(&markdown), "No obligations matched this scope.\n");

        let tsv = f.matrix(&["--format", "tsv"]);
        assert_eq!(
            stdout(&tsv),
            "document\tcriterion\tstatus\tbinders\tstatement\n"
        );

        // Zero population has one shape: the engine omits the key, and so
        // does this surface.
        assert_eq!(engine_matrix(&f), None);
        let payload = json(&f.matrix(&["--format", "json"]));
        assert_eq!(payload.get("coverage_matrix"), None, "{payload}");
    }
}

// Trace: IT-174, FR-026-AC-10, FR-026-AC-11
#[test]
fn it174_strict_requires_at_least_one_criterion_all_backed() {
    for f in zero_populations() {
        let out = f.matrix(&["--strict"]);
        assert_eq!(code(&out), 1, "zero criteria must not pass --strict");
        assert!(
            stderr(&out).contains("no obligations matched"),
            "{}",
            stderr(&out)
        );
    }

    let f = Fixture::with_obligations();
    f.write(
        "spec/FR-001.md",
        &fr_doc(
            "FR-001",
            &[
                ("FR-001-AC-1", "Tagged.", "Test"),
                ("FR-001-AC-2", "Exempt by method.", "Inspection"),
            ],
        ),
    );
    f.write("src/lib.rs", &rust_test("first", "FR-001-AC-1", false));
    let out = f.matrix(&["--strict"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // An untagged criterion fails the gate.
    f.write(
        "spec/FR-002.md",
        &fr_doc("FR-002", &[("FR-002-AC-1", "Untagged.", "Test")]),
    );
    let out = f.matrix(&["--strict"]);
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("1 untagged"), "{}", stderr(&out));
    assert_eq!(code(&f.matrix(&[])), 0, "without --strict the gate is off");

    // So does a criterion tagged only by an ignored test.
    f.write(
        "src/lib.rs",
        &format!(
            "{}{}",
            rust_test("first", "FR-001-AC-1", false),
            rust_test("skipped", "FR-002-AC-1", true)
        ),
    );
    let out = f.matrix(&["--strict"]);
    assert_eq!(code(&out), 1);
    assert!(
        stderr(&out).contains("1 criterion(s) tagged only by ignored"),
        "{}",
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("| tagged-by-ignored-test |"),
        "{}",
        stdout(&out)
    );
}

// Trace: IT-175, FR-026-AC-11
#[test]
fn it175_method_without_symbol_never_fails_strict() {
    let f = Fixture::with_obligations();
    f.write(
        "spec/FR-001.md",
        &fr_doc(
            "FR-001",
            &[
                ("FR-001-AC-1", "Inspected, never tagged.", "Inspection"),
                ("FR-001-AC-2", "Inspected, and tagged anyway.", "Inspection"),
            ],
        ),
    );
    f.write("src/lib.rs", &rust_test("tagged", "FR-001-AC-2", false));
    let out = f.matrix(&["--strict"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let markdown = stdout(&out);
    assert!(
        markdown.contains(
            "| FR-001-AC-1 | Inspected, never tagged. | (none) | method-without-symbol |"
        ),
        "{markdown}"
    );
    assert!(
        markdown.lines().any(|l| l.starts_with("| FR-001-AC-2 |")
            && l.contains("src/lib.rs:")
            && l.ends_with("| method-without-symbol |")),
        "{markdown}"
    );
    let tsv = stdout(&f.matrix(&["--format", "tsv"]));
    assert_eq!(tsv.matches("\tmethod-without-symbol\t").count(), 2, "{tsv}");
    let payload = json(&f.matrix(&["--format", "json"]));
    for c in payload["coverage_matrix"][0]["criteria"]
        .as_array()
        .expect("criteria")
    {
        assert_eq!(c["status"], "method-without-symbol");
    }
}

// Trace: IT-176, FR-026-AC-12
#[test]
fn it176_no_severity_pack_and_no_unread_measurement_gate() {
    let f = populated();
    let out = f.matrix(&["--severity", "coverage:unbacked-row=error"]);
    assert_eq!(code(&out), 2, "--severity is not a matrix flag");
    assert!(stderr(&out).contains("--severity"), "{}", stderr(&out));

    // A module promoting a coverage check to `error`, over a scope that trips
    // it, plus a document reference whose status column reads nothing.
    let references = "  document_references:\n  - name: traces-to\n\
        \x20   archetype: TestMatrix\n    section: Cases\n    column: Traces To\n\
        \x20   row_id_column: ID\n    pattern: '(FR-\\d+-AC-\\d+)'\n\
        \x20   targets: [acceptance-criterion]\n";
    let promoted = "grammar_severity:\n  coverage:untracked-symbol: error\n";
    let plain = Fixture::new(&manifest(true, references, ""));
    let strict = Fixture::new(&manifest(true, references, promoted));
    for f in [&plain, &strict] {
        f.write(
            "spec/FR-001.md",
            &fr_doc("FR-001", &[("FR-001-AC-1", "Tagged.", "Test")]),
        );
        f.write(
            "spec/tests.md",
            "---\nid: TM-001\ntype: TestMatrix\n---\n## Cases\n\n\
             | ID | Traces To | Coverage Status |\n|----|-----------|-----------------|\n\
             | TC-001 | FR-001-AC-1 | ✅ |\n",
        );
        f.write(
            "src/lib.rs",
            &format!(
                "{}{}",
                rust_test("first", "FR-001-AC-1", false),
                rust_test("stray", "FR-999-AC-1", false)
            ),
        );
    }

    // The fixture is live: `coverage` fails on both the promotion and the
    // unread status column.
    let coverage = strict.quire("coverage", &["--json"]);
    assert_eq!(code(&coverage), 1, "the promotion is in force");
    assert!(
        stderr(&coverage).contains("coverage:untracked-symbol"),
        "{}",
        stderr(&coverage)
    );
    let report = json(&coverage);
    assert!(
        report["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|d| d["reason"] == "status-column-matches-nothing"),
        "{report}"
    );
    assert_eq!(code(&plain.quire("coverage", &["--strict"])), 1);

    // `matrix` is untouched by either.
    for format in ["markdown", "json", "tsv"] {
        let a = plain.matrix(&["--format", format, "--strict"]);
        let b = strict.matrix(&["--format", format, "--strict"]);
        assert_eq!(code(&a), 0, "{}", stderr(&a));
        assert_eq!(code(&b), 0, "{}", stderr(&b));
        assert_eq!(a.stdout, b.stdout, "{format}");
    }
}

/// Every file under `root`, with its bytes.
fn snapshot(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path.strip_prefix(root).expect("under root");
                files.insert(rel.display().to_string(), fs::read(&path).expect("read"));
            }
        }
    }
    files
}

// Trace: IT-177, FR-026-AC-14
#[test]
fn it177_matrix_writes_nothing_under_the_scope() {
    let f = populated();
    f.write("README.md", "# repo\n");
    f.write("docs/notes.md", "notes\n");
    let before = snapshot(Path::new(&f.scope()));
    for format in ["markdown", "json", "tsv"] {
        for strict in [false, true] {
            let mut args = vec!["--format", format];
            if strict {
                args.push("--strict");
            }
            f.matrix(&args);
        }
    }
    assert_eq!(snapshot(Path::new(&f.scope())), before);

    let help = stdout(
        &common::quire()
            .args(["matrix", "--help"])
            .output()
            .expect("help"),
    );
    for flag in ["--write", "--out", "--output"] {
        assert!(!help.contains(flag), "no flag names an output path: {help}");
    }
}

// Trace: TC-841, FR-026-AC-13
#[test]
fn tc841_every_matrix_exit_code_is_in_the_fr007_taxonomy() {
    let populated = populated();
    let [empty, _] = zero_populations();
    let no_spec = Fixture::with_obligations();
    fs::remove_dir(no_spec.dir.path().join("repo/spec")).expect("rmdir spec");

    let mut seen = std::collections::BTreeSet::new();
    for f in [&populated, &empty, &no_spec] {
        for args in [
            &[][..],
            &["--strict"][..],
            &["--format", "json"][..],
            &["--format", "tsv", "--strict"][..],
            &["--severity", "coverage:unbacked-row=error"][..],
            &["--format", "yaml"][..],
        ] {
            seen.insert(code(&f.matrix(args)));
        }
    }
    assert_eq!(seen.into_iter().collect::<Vec<_>>(), [0, 1, 2]);
}
