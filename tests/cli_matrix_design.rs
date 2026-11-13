//! `quire matrix` design column and design gate (FR-043). Every kind, verb,
//! relation, check and criterion id below is invented and unrelated to
//! requirements or design: a `Glyph` is the obliged document, a `Lantern` the
//! thing that lights it, `kindles` / `kindled_by` the verb pair. The CLI knows
//! none of them (FR-043-AC-14).

mod common;

use ix_trace_rs::trace;
use std::fs;
use std::path::Path;
use std::process::Output;

use serde_json::Value;
use tempfile::TempDir;

const TRACE_TAGS: &str = "  trace_tags:\n    markers:\n\
    \x20   - name: rust-trace-attribute\n      language: rust\n\
    \x20     pattern: '#\\[trace\\(([^)]*)\\)\\]'\n      template: '#[trace({ids})]'\n";

/// The relation every fixture module declares, `where`-free by default.
const LIT: &str = "  - name: glyph-has-lantern
    from: Glyph
    edges: [kindles]
    to: [Lantern]
    direction: incoming
    check: glyph-dark
";

/// The same contract authored outward from the obliged document.
const OUT: &str = "  - name: glyph-out
    from: Glyph
    edges: [kindled_by]
    to: [Lantern]
    direction: outgoing
    check: glyph-dark
";

/// A second relation over the same documents, for two-block requirements.
const WARDED: &str = "  - name: glyph-has-prism
    from: Glyph
    edges: [wards]
    to: [Prism]
    direction: incoming
    check: glyph-unwarded
";

fn manifest(relations: &str, design_relations: &str) -> String {
    format!(
        "name: m\nmanifest_version: 1.0.0\nversion: 0.0.1\nartifact_types:\n\
         - name: Glyph\n- name: Lantern\n- name: Prism\n- name: Bulb\n\
         edge_types:\n  kindles: {{ description: x, category: traceability, inverse: kindled_by }}\n\
         \x20 wards: {{ description: y, category: traceability }}\n\
         traceability:\n  trace_targets:\n  - name: glyph-check\n    archetype: Glyph\n\
         \x20   section: Acceptance Criteria\n    id_column: ID\n\
         \x20 obligations:\n  - name: glyph-check\n    target: glyph-check\n\
         \x20   statement_column: Criteria\n\
         \x20 required_relations:\n{relations}{design_relations}{TRACE_TAGS}"
    )
}

/// A document of type `ty` with extra frontmatter, edges, and a criteria table.
fn doc(
    id: &str,
    ty: &str,
    extra: &str,
    edges: &[(&str, &str)],
    criteria: &[(&str, &str)],
) -> String {
    let mut text = format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n{extra}");
    if !edges.is_empty() {
        text.push_str("relationships:\n");
        for (verb, target) in edges {
            text.push_str(&format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"));
        }
    }
    text.push_str(&format!("---\n\n# {id}\n"));
    if !criteria.is_empty() {
        text.push_str("\n## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n");
        for (ac, statement) in criteria {
            text.push_str(&format!("| {ac} | {statement} |\n"));
        }
    }
    text
}

fn glyph(id: &str, edges: &[(&str, &str)], criteria: &[(&str, &str)]) -> String {
    doc(id, "Glyph", "", edges, criteria)
}

fn lantern(id: &str, edges: &[(&str, &str)]) -> String {
    doc(id, "Lantern", "", edges, &[])
}

fn rust_test(name: &str, id: &str) -> String {
    format!("#[trace(\"{id}\")]\n#[test]\nfn {name}() {{}}\n\n")
}

/// One repository on disk under `base/<name>`, documents named by their id.
struct Repo {
    root: std::path::PathBuf,
}

impl Repo {
    fn new(base: &Path, name: &str, docs: &[String], tagged: &[&str]) -> Self {
        let root = base.join(name);
        fs::create_dir_all(root.join("spec")).expect("mkdir spec");
        fs::create_dir_all(root.join("src")).expect("mkdir src");
        for text in docs {
            let id = text
                .lines()
                .find_map(|l| l.strip_prefix("id: "))
                .expect("id line");
            fs::write(root.join(format!("spec/{id}.md")), text).expect("write doc");
        }
        let tests: String = tagged
            .iter()
            .enumerate()
            .map(|(n, id)| rust_test(&format!("t{n}"), id))
            .collect();
        fs::write(root.join("src/lib.rs"), tests).expect("write tests");
        Self { root }
    }

    fn dir(&self) -> String {
        self.root.to_string_lossy().into_owned()
    }
}

/// A fixture directory with one module and any number of repositories.
struct Fixture {
    dir: TempDir,
}

impl Fixture {
    fn new(manifest: &str) -> Self {
        let dir = TempDir::new().expect("tempdir");
        fs::create_dir_all(dir.path().join("m")).expect("mkdir m");
        fs::write(dir.path().join("m/manifest.yaml"), manifest).expect("write manifest");
        Self { dir }
    }

    fn repo(&self, name: &str, docs: &[String], tagged: &[&str]) -> Repo {
        Repo::new(self.dir.path(), name, docs, tagged)
    }

    fn module(&self) -> String {
        self.dir.path().join("m").to_string_lossy().into_owned()
    }

    fn matrix(&self, repo: &Repo, extra: &[&str]) -> Output {
        common::quire()
            .args(["matrix", "--scope", &repo.dir(), "--module", &self.module()])
            .args(extra)
            .output()
            .expect("run quire")
    }

    /// `--repo <identity>=<dir>` for each `(identity, repo)`.
    fn matrix_repos(&self, repos: &[(&str, &Repo)], extra: &[&str]) -> Output {
        let mut command = common::quire();
        command.args(["matrix", "--module", &self.module()]);
        for (identity, repo) in repos {
            command.args(["--repo", &format!("{identity}={}", repo.dir())]);
        }
        command.args(extra).output().expect("run quire")
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("stdout is UTF-8")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn code(out: &Output) -> i32 {
    out.status.code().expect("exited with a code")
}

fn json(out: &Output) -> Value {
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON ({e}): {}", stderr(out)))
}

/// Every file under `root` with its bytes, sorted by path.
fn tree(root: &Path) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    fn walk(dir: &Path, out: &mut Vec<(std::path::PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).expect("read_dir") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                let bytes = fs::read(&path).expect("read");
                out.push((path, bytes));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

fn design_lines(err: &str) -> Vec<&str> {
    err.lines()
        .filter(|l| l.starts_with("design gap:"))
        .collect()
}

/// GLY-001 has two criteria, realized by LAN-001 as a whole and by LAN-002 on
/// `chk-a` alone; GLY-002 has one criterion and no lantern.
fn lit_and_dark() -> (Fixture, Repo) {
    let f = Fixture::new(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let r = f.repo(
        "r",
        &[
            glyph(
                "GLY-001",
                &[],
                &[("chk-a", "First holds."), ("chk-b", "Second holds.")],
            ),
            glyph("GLY-002", &[], &[("chk-c", "Third holds.")]),
            lantern("LAN-001", &[("kindles", "GLY-001")]),
            lantern("LAN-002", &[("kindles", "chk-a")]),
        ],
        &["chk-a", "chk-b", "chk-c"],
    );
    (f, r)
}

#[trace("FR-043-AC-1", "FR-043-AC-5", "FR-043-AC-14", "FR-043-AC-16")]
#[test]
fn markdown_puts_a_design_column_between_statement_and_binders() {
    let (f, r) = lit_and_dark();
    let before = tree(&r.root);
    let out = f.matrix(&r, &[]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains(
            "| Criterion | Statement | Design | Binders | Status | Requirement Criticality |"
        ),
        "{text}"
    );
    // AC-5: LAN-002 names chk-a only; LAN-001 realizes the requirement whole.
    assert!(
        text.contains("| chk-a | First holds. | LAN-001 (Lantern), LAN-002 (Lantern) |"),
        "{text}"
    );
    assert!(
        text.contains("| chk-b | Second holds. | LAN-001 (Lantern) |"),
        "{text}"
    );
    assert!(
        text.contains("| chk-c | Third holds. | no design element (glyph-dark) |"),
        "{text}"
    );
    assert!(
        text.contains("## spec/GLY-002.md\n\nDesign gap: no design element (glyph-dark)\n\n|"),
        "{text}"
    );
    // AC-16: no format writes anything under the scope, and each succeeds.
    for format in ["human", "json", "tsv", "markdown"] {
        let out = f.matrix(&r, &["--format", format]);
        assert_eq!(code(&out), 0, "{format}: {}", stderr(&out));
    }
    assert_eq!(before, tree(&r.root));
}

#[trace("FR-043-AC-2", "FR-043-AC-25")]
#[test]
fn human_prints_the_design_line_the_gap_lines_and_one_line_per_criterion() {
    let (f, r) = lit_and_dark();
    let out = f.matrix(&r, &["--format", "human"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(
        stdout(&out),
        "spec/GLY-001.md\n  design: LAN-001 (Lantern), LAN-002 (Lantern)\n  \
         chk-a  tagged  design: LAN-001 (Lantern), LAN-002 (Lantern)  tests: src/lib.rs:3:1\n  \
         chk-b  tagged  design: LAN-001 (Lantern)  tests: src/lib.rs:7:1\n\
         \n\
         spec/GLY-002.md\nDesign gap: no design element (glyph-dark)\n  \
         design: no design element (glyph-dark)\n  \
         chk-c  tagged  design: no design element (glyph-dark)  tests: src/lib.rs:11:1\n"
    );
}

#[trace("FR-043-AC-3")]
#[test]
fn json_carries_the_engines_design_key_and_omits_it_out_of_scope() {
    let (f, r) = lit_and_dark();
    let first = f.matrix(&r, &["--format", "json"]);
    let second = f.matrix(&r, &["--format", "json"]);
    assert_eq!(first.stdout, second.stdout, "two runs are byte-identical");
    let value = json(&first);
    let entries = value["coverage_matrix"].as_array().expect("array");
    let covered = &entries[0];
    assert_eq!(covered["design"][0]["state"], "covered");
    assert_eq!(covered["design"][0]["elements"][0]["kind"], "Lantern");
    assert_eq!(entries[1]["design"][0]["state"], "uncovered");

    // Out of scope: the key is absent, not null.
    let f = Fixture::new(&manifest(
        &format!(
            "{}\n",
            LIT.replace(
                "from: Glyph",
                "from: Glyph\n    where: { field: tier, values: [high] }"
            )
        ),
        "  design_relations: [glyph-has-lantern]\n",
    ));
    let r = f.repo(
        "r",
        &[doc(
            "GLY-001",
            "Glyph",
            "tier: low\n",
            &[],
            &[("chk-a", "A.")],
        )],
        &["chk-a"],
    );
    let value = json(&f.matrix(&r, &["--format", "json"]));
    assert!(value["coverage_matrix"][0].get("design").is_none());
}

#[trace("FR-043-AC-4", "FR-043-AC-20")]
#[test]
fn tsv_inserts_design_and_design_state_after_criterion() {
    let (f, r) = lit_and_dark();
    let out = f.matrix(&r, &["--format", "tsv"]);
    assert_eq!(
        stdout(&out),
        "document\tcriterion\tdesign\tdesign_state\tstatus\tbinders\tstatement\t\
         requirement_criticality\n\
         spec/GLY-001.md\tchk-a\tLantern:LAN-001,Lantern:LAN-002\tcovered\ttagged\t\
         src/lib.rs:3:1\tFirst holds.\tunassigned\n\
         spec/GLY-001.md\tchk-b\tLantern:LAN-001\tcovered\ttagged\tsrc/lib.rs:7:1\t\
         Second holds.\tunassigned\n\
         spec/GLY-002.md\tchk-c\t\tuncovered\ttagged\tsrc/lib.rs:11:1\tThird holds.\tunassigned\n"
    );

    // AC-20 and the out-of-scope row: an untyped element prints its id alone,
    // an entry with no `design` key has empty design cells.
    let f = Fixture::new(&manifest(
        &LIT.replace(
            "from: Glyph",
            "from: Glyph\n    where: { field: tier, values: [high] }",
        )
        .replace("to: [Lantern]", "to: []"),
        "  design_relations: [glyph-has-lantern]\n",
    ));
    let r = f.repo(
        "r",
        &[
            doc("GLY-001", "Glyph", "tier: low\n", &[], &[("chk-a", "A.")]),
            doc("GLY-002", "Glyph", "tier: high\n", &[], &[("chk-b", "B.")]),
            // A document with no `type` is an untyped element.
            "---\nid: ELT-001\ntitle: untyped\nrelationships:\n  - target: \"GLY-002\"\n    type: \"kindles\"\n---\n\n# x\n".to_string(),
        ],
        &["chk-a", "chk-b"],
    );
    let tsv = stdout(&f.matrix(&r, &["--format", "tsv"]));
    assert!(tsv.contains("spec/GLY-001.md\tchk-a\t\t\ttagged"), "{tsv}");
    assert!(
        tsv.contains("spec/GLY-002.md\tchk-b\tELT-001\tcovered\ttagged"),
        "{tsv}"
    );
    let md = stdout(&f.matrix(&r, &["--format", "markdown"]));
    assert!(md.contains("| chk-a | A. | n/a |"), "{md}");
    assert!(md.contains("| chk-b | B. | ELT-001 |"), "{md}");
    let human = stdout(&f.matrix(&r, &["--format", "human"]));
    assert!(human.contains("  design: n/a\n"), "{human}");
    assert!(
        human.contains("  chk-b  tagged  design: ELT-001  tests:"),
        "{human}"
    );
}

#[trace("FR-043-AC-6", "FR-043-AC-19")]
#[test]
fn a_requirement_without_criteria_prints_its_gap_and_fails_strict() {
    let f = Fixture::new(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let r = f.repo(
        "r",
        &[
            glyph("GLY-001", &[], &[]),
            glyph("GLY-002", &[], &[]),
            lantern("LAN-001", &[("kindles", "GLY-002")]),
        ],
        &[],
    );
    let md = f.matrix(&r, &[]);
    assert_eq!(
        stdout(&md),
        "## spec/GLY-001.md\n\nDesign gap: no design element (glyph-dark)\n\n\
         ## spec/GLY-002.md\n"
    );
    let human = stdout(&f.matrix(&r, &["--format", "human"]));
    assert_eq!(
        human,
        "spec/GLY-001.md\nDesign gap: no design element (glyph-dark)\n  \
         design: no design element (glyph-dark)\n\nspec/GLY-002.md\n  design: LAN-001 (Lantern)\n"
    );
    let tsv = stdout(&f.matrix(&r, &["--format", "tsv"]));
    assert!(
        tsv.ends_with(
            "spec/GLY-001.md\t\t\tuncovered\t\t\t\t\nspec/GLY-002.md\t\tLantern:LAN-001\tcovered\t\t\t\t\n"
        ),
        "{tsv}"
    );
    assert!(!md.stdout.is_empty() && !stdout(&md).contains("No obligations"));
    let strict = f.matrix(&r, &["--strict"]);
    assert_eq!(code(&strict), 1);
    assert_eq!(
        design_lines(&stderr(&strict)),
        ["design gap: spec/GLY-001.md glyph-dark"]
    );
}

#[trace("FR-043-AC-7")]
#[test]
fn strict_names_the_requirement_and_check_and_non_strict_exits_zero() {
    let (f, r) = lit_and_dark();
    let strict = f.matrix(&r, &["--strict"]);
    assert_eq!(code(&strict), 1, "{}", stderr(&strict));
    assert_eq!(
        design_lines(&stderr(&strict)),
        ["design gap: spec/GLY-002.md glyph-dark"]
    );
    assert!(stdout(&strict).contains("Design gap: no design element (glyph-dark)"));
    let plain = f.matrix(&r, &[]);
    assert_eq!(code(&plain), 0);
    assert!(stdout(&plain).contains("Design gap:"));
}

#[trace("FR-043-AC-8")]
#[test]
fn the_gate_scope_is_module_data() {
    let docs = [
        doc("GLY-001", "Glyph", "tier: high\n", &[], &[("chk-a", "A.")]),
        doc("GLY-002", "Glyph", "tier: low\n", &[], &[("chk-b", "B.")]),
        lantern("LAN-001", &[("kindles", "GLY-001")]),
    ];
    let select = |value: &str| {
        manifest(
            &LIT.replace(
                "from: Glyph",
                &format!("from: Glyph\n    where: {{ field: tier, values: [{value}] }}"),
            ),
            "  design_relations: [glyph-has-lantern]\n",
        )
    };
    let f = Fixture::new(&select("high"));
    let r = f.repo("r", &docs, &["chk-a", "chk-b"]);
    let out = f.matrix(&r, &["--strict"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    // Same corpus, other module data: the gate fires with no CLI change.
    fs::write(f.dir.path().join("m/manifest.yaml"), select("low")).expect("rewrite");
    let out = f.matrix(&r, &["--strict"]);
    assert_eq!(code(&out), 1);
    assert_eq!(
        design_lines(&stderr(&out)),
        ["design gap: spec/GLY-002.md glyph-dark"]
    );
}

#[trace("FR-043-AC-10")]
#[test]
fn a_passing_corpus_exits_zero_and_a_failing_one_names_every_gap() {
    let f = Fixture::new(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let pass = f.repo(
        "pass",
        &[
            glyph("GLY-001", &[], &[("chk-a", "A.")]),
            lantern("LAN-001", &[("kindles", "GLY-001")]),
        ],
        &["chk-a"],
    );
    let out = f.matrix(&pass, &["--strict"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let fail = f.repo(
        "fail",
        &[
            glyph("GLY-001", &[], &[("chk-a", "A.")]),
            glyph("GLY-002", &[], &[("chk-b", "B.")]),
            lantern("LAN-001", &[("kindles", "GLY-001")]),
        ],
        &["chk-a"],
    );
    let out = f.matrix(&fail, &["--strict"]);
    assert_eq!(code(&out), 1);
    let err = stderr(&out);
    assert_eq!(
        design_lines(&err),
        ["design gap: spec/GLY-002.md glyph-dark"]
    );
    assert!(err.contains("1 untagged criterion(s)"), "{err}");
}

#[trace("FR-043-AC-11")]
#[test]
fn without_design_relations_the_output_is_the_matrix_without_design() {
    let f = Fixture::new(&manifest(LIT, ""));
    let r = f.repo(
        "r",
        &[
            glyph("GLY-001", &[], &[("chk-a", "A.")]),
            lantern("LAN-001", &[("kindles", "GLY-001")]),
        ],
        &["chk-a"],
    );
    let md = f.matrix(&r, &[]);
    assert_eq!(
        stdout(&md),
        "## spec/GLY-001.md\n\n| Criterion | Statement | Binders | Status | Requirement Criticality |\n\
         |---|---|---|---|---|\n| chk-a | A. | src/lib.rs:3:1 | tagged | unassigned |\n"
    );
    let tsv = stdout(&f.matrix(&r, &["--format", "tsv"]));
    assert!(
        tsv.starts_with("document\tcriterion\tstatus\tbinders\tstatement\t"),
        "{tsv}"
    );
    assert!(
        json(&f.matrix(&r, &["--format", "json"]))["coverage_matrix"][0]
            .get("design")
            .is_none()
    );
    assert_eq!(code(&f.matrix(&r, &["--strict"])), 0);

    // Under `--repo` the same entries gain a `repo` line and a TSV column.
    let out = f.matrix_repos(&[("o/a", &r)], &[]);
    assert!(
        stdout(&out).starts_with("## spec/GLY-001.md\n\nrepo: o/a\n\n| Criterion |"),
        "{}",
        stdout(&out)
    );
    let out = f.matrix_repos(&[("o/a", &r)], &["--format", "tsv"]);
    assert!(
        stdout(&out).starts_with("repo\tdocument\tcriterion\tstatus\t"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("\no/a\tspec/GLY-001.md\tchk-a\ttagged\t"));
}

#[trace("FR-043-AC-12")]
#[test]
fn an_undeclared_design_relation_exits_three_with_nothing_on_stdout() {
    let f = Fixture::new(&manifest(LIT, "  design_relations: [ghost-relation]\n"));
    let r = f.repo(
        "r",
        &[glyph("GLY-001", &[], &[("chk-a", "A.")])],
        &["chk-a"],
    );
    for format in ["markdown", "human", "json", "tsv"] {
        let out = f.matrix(&r, &["--format", format]);
        assert_eq!(code(&out), 3, "{format}: {}", stderr(&out));
        assert!(out.stdout.is_empty(), "{format}");
        assert!(stderr(&out).contains("ghost-relation"), "{}", stderr(&out));
    }
}

#[trace("FR-043-AC-13", "FR-043-AC-24")]
#[test]
fn a_sibling_repository_realizes_a_requirement_and_repos_stay_distinct() {
    let f = Fixture::new(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let a = f.repo(
        "a",
        &[
            glyph("GLY-001", &[], &[("chk-a", "A.")]),
            glyph("GLY-002", &[], &[("chk-b", "B.")]),
        ],
        &["chk-a", "chk-b"],
    );
    let b = f.repo(
        "b",
        &[lantern("LAN-001", &[("kindles", "ix://o/a/spec/GLY-001")])],
        &[],
    );
    let out = f.matrix_repos(&[("o/a", &a), ("o/b", &b)], &[]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("## spec/GLY-001.md\n\nrepo: o/a\n\n"),
        "{text}"
    );
    assert!(
        text.contains("| chk-a | A. | o/b/LAN-001 (Lantern) |"),
        "{text}"
    );
    assert!(
        text.contains("repo: o/a\nDesign gap: no design element (glyph-dark)\n"),
        "{text}"
    );
    let tsv = stdout(&f.matrix_repos(&[("o/a", &a), ("o/b", &b)], &["--format", "tsv"]));
    assert!(
        tsv.contains("o/a\tspec/GLY-001.md\tchk-a\tLantern:o/b/LAN-001\tcovered\ttagged"),
        "{tsv}"
    );
    let value = json(&f.matrix_repos(&[("o/a", &a), ("o/b", &b)], &["--format", "json"]));
    assert_eq!(value["coverage_matrix"][0]["repo"], "o/a");
    assert_eq!(
        value["coverage_matrix"][0]["design"][0]["elements"][0]["repo"],
        "o/b"
    );

    // Without the o/b flag the requirement is a gap.
    let alone = f.matrix_repos(&[("o/a", &a)], &[]);
    assert_eq!(
        stdout(&alone).matches("Design gap:").count(),
        2,
        "{}",
        stdout(&alone)
    );

    // Two members, one document path: distinguishable in the strict line.
    let c = f.repo(
        "c",
        &[glyph("GLY-001", &[], &[("chk-z", "Z.")])],
        &["chk-z"],
    );
    let strict = f.matrix_repos(&[("o/a", &a), ("o/c", &c)], &["--strict"]);
    assert_eq!(code(&strict), 1);
    let shown = stdout(&strict);
    assert!(
        shown.contains("## spec/GLY-001.md\n\nrepo: o/a\n"),
        "{shown}"
    );
    assert!(
        shown.contains("## spec/GLY-001.md\n\nrepo: o/c\n"),
        "{shown}"
    );
    let lines = design_lines(&stderr(&strict)).join("\n");
    assert!(
        lines.contains("design gap: o/a:spec/GLY-001.md glyph-dark"),
        "{lines}"
    );
    assert!(
        lines.contains("design gap: o/c:spec/GLY-001.md glyph-dark"),
        "{lines}"
    );
}

#[trace("FR-043-AC-17", "FR-043-AC-26")]
#[test]
fn covered_and_uncovered_blocks_share_one_cell_and_each_gap_is_a_line() {
    let f = Fixture::new(&manifest(
        &format!("{LIT}{WARDED}"),
        "  design_relations: [glyph-has-lantern, glyph-has-prism]\n",
    ));
    let r = f.repo(
        "r",
        &[
            glyph("GLY-001", &[], &[("chk-a", "A.")]),
            glyph("GLY-002", &[], &[("chk-b", "B.")]),
            lantern("LAN-001", &[("kindles", "GLY-001")]),
        ],
        &["chk-a", "chk-b"],
    );
    let md = stdout(&f.matrix(&r, &[]));
    assert!(
        md.contains("| chk-a | A. | LAN-001 (Lantern), no design element (glyph-unwarded) |"),
        "{md}"
    );
    let strict = f.matrix(&r, &["--strict"]);
    assert_eq!(
        design_lines(&stderr(&strict)),
        [
            "design gap: spec/GLY-001.md glyph-unwarded",
            "design gap: spec/GLY-002.md glyph-dark",
            "design gap: spec/GLY-002.md glyph-unwarded",
        ]
    );
}

#[trace("FR-043-AC-18", "FR-043-AC-22")]
#[test]
fn dangling_none_and_long_cells_are_never_truncated() {
    let open = "  - name: glyph-open
    from: Glyph
    edges: [kindles]
    to: []
    direction: incoming
    check: glyph-open-dark
";
    let f = Fixture::new(&manifest(open, "  design_relations: [glyph-open]\n"));
    let r = f.repo(
        "r",
        &[
            glyph(
                "GLY-001",
                &[("kindled_by", "ix://o/b/spec/LAN-404")],
                &[("chk-a", "A.")],
            ),
            glyph("GLY-002", &[], &[("chk-b", "B.")]),
            lantern("LAN-001", &[("kindles", "GLY-002")]),
        ],
        &["chk-a", "chk-b"],
    );
    let md = stdout(&f.matrix(&r, &[]));
    assert!(
        md.contains("| chk-a | A. | -> ix://o/b/spec/LAN-404 (dangling) |"),
        "{md}"
    );

    // `(none)`: a criterion beside an element that names another criterion.
    let g = Fixture::new(&manifest(LIT, "  design_relations: [glyph-has-lantern]\n"));
    let r = g.repo(
        "r",
        &[
            glyph("GLY-001", &[], &[("chk-a", "A."), ("chk-b", "B.")]),
            lantern("LAN-001", &[("kindles", "chk-a")]),
        ],
        &["chk-a", "chk-b"],
    );
    let md = stdout(&g.matrix(&r, &[]));
    assert!(md.contains("| chk-b | B. | (none) |"), "{md}");

    // A long statement is truncated; the gap marker beside long elements is not.
    let long = "S".repeat(120);
    let lanterns: Vec<String> = (1..=6)
        .map(|n| lantern(&format!("LAN-LONG-{n:03}"), &[("kindles", "GLY-001")]))
        .collect();
    let mut docs = vec![glyph("GLY-001", &[], &[("chk-a", &long)])];
    docs.extend(lanterns);
    let h = Fixture::new(&manifest(
        &format!("{LIT}{WARDED}"),
        "  design_relations: [glyph-has-lantern, glyph-has-prism]\n",
    ));
    let r = h.repo("r", &docs, &["chk-a"]);
    let md = stdout(&h.matrix(&r, &[]));
    assert!(md.contains(&format!("{}...", "S".repeat(77))), "{md}");
    assert!(
        md.contains("LAN-LONG-006 (Lantern), no design element (glyph-unwarded) |"),
        "{md}"
    );
}

#[trace("FR-043-AC-21")]
#[test]
fn scope_with_repo_exits_three_naming_both_flags() {
    let (f, r) = lit_and_dark();
    let out = common::quire()
        .args(["matrix", "--module", &f.module(), "--scope", &r.dir()])
        .args(["--repo", &format!("o/a={}", r.dir())])
        .output()
        .expect("run");
    assert_eq!(code(&out), 3, "{}", stderr(&out));
    assert!(out.stdout.is_empty());
    let err = stderr(&out);
    assert!(err.contains("--scope") && err.contains("--repo"), "{err}");
}

#[trace("FR-043-AC-23")]
#[test]
fn a_kind_mismatch_prints_beside_the_gap_marker() {
    let f = Fixture::new(&manifest(OUT, "  design_relations: [glyph-out]\n"));
    let r = f.repo(
        "r",
        &[
            glyph("GLY-001", &[("kindled_by", "BUL-001")], &[("chk-a", "A.")]),
            doc("BUL-001", "Bulb", "", &[], &[]),
        ],
        &["chk-a"],
    );
    let md = stdout(&f.matrix(&r, &[]));
    assert!(
        md.contains(
            "| chk-a | A. | no design element (glyph-dark), kind mismatch: BUL-001 (Bulb) |"
        ),
        "{md}"
    );
}

#[trace("FR-043-AC-25")]
#[test]
fn human_design_line_has_three_distinct_forms() {
    let f = Fixture::new(&manifest(
        &LIT.replace(
            "from: Glyph",
            "from: Glyph\n    where: { field: tier, values: [high] }",
        ),
        "  design_relations: [glyph-has-lantern]\n",
    ));
    let r = f.repo(
        "r",
        &[
            doc("GLY-001", "Glyph", "tier: low\n", &[], &[("chk-a", "A.")]),
            doc(
                "GLY-002",
                "Glyph",
                "tier: high\n",
                &[],
                &[("chk-b", "B."), ("chk-c", "C.")],
            ),
            doc("GLY-003", "Glyph", "tier: high\n", &[], &[("chk-d", "D.")]),
            lantern("LAN-001", &[("kindles", "chk-b")]),
        ],
        &["chk-a", "chk-b", "chk-c"],
    );
    let human = stdout(&f.matrix(&r, &["--format", "human"]));
    // Out of scope: n/a. Covered entry, criterion no element names: (none).
    assert!(
        human.contains("spec/GLY-001.md\n  design: n/a\n"),
        "{human}"
    );
    assert!(
        human.contains("  chk-c  tagged  design: (none)  tests:"),
        "{human}"
    );
    // Uncovered entry with no elements.
    assert!(
        human.contains("spec/GLY-003.md\nDesign gap: no design element (glyph-dark)\n  design: no design element (glyph-dark)\n"),
        "{human}"
    );
}

#[test]
fn a_malformed_repo_value_exits_three() {
    let (f, r) = lit_and_dark();
    for bad in ["o//a", "/o/a", "o/a/", "o", "o/a", "o/a="] {
        let value = if bad.contains('=') || bad == "o/a" {
            bad.to_string()
        } else {
            format!("{bad}={}", r.dir())
        };
        let out = common::quire()
            .args(["matrix", "--module", &f.module(), "--repo", &value])
            .output()
            .expect("run");
        assert_eq!(code(&out), 3, "{value}: {}", stderr(&out));
        assert!(out.stdout.is_empty(), "{value}");
    }
}

#[trace("FR-043-AC-18")]
#[test]
fn a_dangling_edge_beside_an_element_prints_no_dangling_marker() {
    let open = "  - name: glyph-open
    from: Glyph
    edges: [kindled_by]
    to: []
    direction: outgoing
    check: glyph-open-dark
";
    let f = Fixture::new(&manifest(open, "  design_relations: [glyph-open]\n"));
    let r = f.repo(
        "r",
        &[
            glyph(
                "GLY-001",
                &[
                    ("kindled_by", "LAN-001"),
                    ("kindled_by", "ix://o/b/spec/LAN-404"),
                ],
                &[("chk-a", "A.")],
            ),
            lantern("LAN-001", &[]),
        ],
        &["chk-a"],
    );
    let md = stdout(&f.matrix(&r, &[]));
    assert!(md.contains("| chk-a | A. | LAN-001 (Lantern) |"), "{md}");
    assert!(!md.contains("(dangling)"), "{md}");
}

/// A module whose retired vocabulary lets the engine class a row `retired`.
fn retired_fixture(rows: &str) -> (Fixture, Repo) {
    let mut text = manifest(LIT, "  design_relations: [glyph-has-lantern]\n");
    text = text.replace(
        "  obligations:",
        "  document_references:\n  - name: verification\n    archetype: Glyph\n\
         \x20   section: Acceptance Criteria\n    column: Verification\n\
         \x20   row_id_column: ID\n    pattern: '(chk-[a-z])'\n    targets: [glyph-check]\n\
         \x20 status:\n    column: Status\n    complete: [done]\n    pending: [planned]\n\
         \x20 vocabularies:\n    retired: [Withdrawn]\n  obligations:",
    );
    let f = Fixture::new(&text);
    let doc = format!(
        "---\nid: GLY-001\ntype: Glyph\ntitle: g\n---\n\n## Acceptance Criteria\n\n\
         | ID | Criteria | Verification | Status |\n|---|---|---|---|\n{rows}"
    );
    let r = f.repo("r", &[doc], &["chk-b"]);
    (f, r)
}

#[trace("FR-043-AC-9")]
#[test]
fn a_retired_requirement_keeps_its_gap_without_failing_strict() {
    let (f, r) = retired_fixture("| chk-a | Withdrawn behaviour. | Withdrawn | planned |\n");
    let out = f.matrix(&r, &["--strict"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("retired"), "{text}");
    assert!(
        text.contains("Design gap: no design element (glyph-dark)"),
        "{text}"
    );
    assert!(design_lines(&stderr(&out)).is_empty());

    let (f, r) = retired_fixture(
        "| chk-a | Withdrawn behaviour. | Withdrawn | planned |\n\
         | chk-b | Live behaviour. | Test | done |\n",
    );
    let out = f.matrix(&r, &["--strict"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert_eq!(
        design_lines(&stderr(&out)),
        ["design gap: spec/GLY-001.md glyph-dark"]
    );
}
