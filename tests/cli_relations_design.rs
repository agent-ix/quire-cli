//! FR-041 — `quire relations --design`.
//!
//! Every kind, verb, relation, check token and id below is authored by the
//! fixture module written here and is unrelated to requirements or design: a
//! `Sigil` is the obliged document, a `Beacon` is what lights it, and
//! `kindles` / `kindled_by` are the verb pair. The CLI knows none of them.
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use ix_trace_rs::trace;
use serde_json::Value;
use tempfile::TempDir;

/// The fixture module. `design` is the `design_relations` line.
fn manifest(design: Option<&str>) -> String {
    let design = design
        .map(|names| format!("  design_relations: {names}\n"))
        .unwrap_or_default();
    format!(
        "name: design-fixture\nmanifest_version: 1.0.0\nversion: 0.1.0\n\
         artifact_types:\n- name: Sigil\n- name: Beacon\n- name: Kiln\n\
         edge_types:\n  kindles: {{ description: a Beacon kindles a Sigil, category: traceability, inverse: kindled_by }}\n  stokes: {{ description: a Kiln stokes a Sigil, category: traceability }}\n\
         traceability:\n  trace_targets:\n  - name: sigil-check\n    archetype: Sigil\n    section: Acceptance Criteria\n    id_column: ID\n\
         \x20 obligations:\n  - name: sigil-check\n    target: sigil-check\n    statement_column: Criteria\n\
         \x20 required_relations:\n\
         \x20 - name: sigil-has-beacon\n    from: Sigil\n    edges: [kindles]\n    to: [Beacon]\n    direction: incoming\n    check: sigil-dark\n\
         \x20 - name: sigil-lit-by\n    from: Sigil\n    edges: [kindled_by]\n    to: [Beacon]\n    direction: outgoing\n    check: sigil-unlit\n\
         \x20 - name: sigil-has-kiln\n    from: Sigil\n    edges: [stokes]\n    to: [Kiln]\n    direction: incoming\n    check: sigil-cold\n\
         {design}"
    )
}

fn doc(id: &str, ty: &str, rels: &[(&str, &str)], criteria: &[&str]) -> String {
    let mut text = format!("---\nid: {id}\ntype: {ty}\ntitle: {id}\n");
    if !rels.is_empty() {
        text.push_str("relationships:\n");
        for (verb, target) in rels {
            text.push_str(&format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"));
        }
    }
    text.push_str(&format!("---\n\n# {id}\n"));
    if !criteria.is_empty() {
        text.push_str("\n## Acceptance Criteria\n\n| ID | Criteria |\n|----|----------|\n");
        for ac in criteria {
            text.push_str(&format!("| {ac} | The glow holds. |\n"));
        }
    }
    text
}

struct World {
    tmp: TempDir,
}

impl World {
    fn new(design: Option<&str>) -> Self {
        let world = Self {
            tmp: TempDir::new().unwrap(),
        };
        world.module(design);
        world
    }

    fn module(&self, design: Option<&str>) {
        let dir = self.tmp.path().join("m");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.yaml"), manifest(design)).unwrap();
    }

    fn module_dir(&self) -> PathBuf {
        self.tmp.path().join("m")
    }

    fn repo(&self, name: &str, docs: &[(&str, String)]) -> PathBuf {
        let root = self.tmp.path().join(name);
        fs::create_dir_all(root.join("spec")).unwrap();
        for (stem, text) in docs {
            fs::write(root.join("spec").join(format!("{stem}.md")), text).unwrap();
        }
        root
    }

    /// SIG-001 is lit by BEA-001 (as a whole and through one criterion);
    /// SIG-002 has no design element.
    fn worked_example(&self) -> PathBuf {
        self.repo(
            "repo",
            &[
                ("SIG-001", doc("SIG-001", "Sigil", &[], &["SIG-001-AC-1"])),
                ("SIG-002", doc("SIG-002", "Sigil", &[], &["SIG-002-AC-1"])),
                (
                    "BEA-001",
                    doc(
                        "BEA-001",
                        "Beacon",
                        &[("kindles", "SIG-001"), ("kindles", "SIG-001-AC-1")],
                        &[],
                    ),
                ),
            ],
        )
    }

    fn design(&self, scope: &Path, extra: &[&str]) -> Output {
        let mut command = common::quire();
        command
            .args(["relations", "--design", "--scope"])
            .arg(scope)
            .arg("--module")
            .arg(self.module_dir())
            .args(extra);
        command.output().unwrap()
    }

    fn design_repos(&self, repos: &[(&str, &Path)], extra: &[&str]) -> Output {
        let mut command = common::quire();
        command
            .args(["relations", "--design", "--module"])
            .arg(self.module_dir());
        for (identity, dir) in repos {
            command
                .arg("--repo")
                .arg(format!("{identity}={}", dir.display()));
        }
        command.args(extra);
        command.output().unwrap()
    }
}

const DESIGN: Option<&str> = Some("[sigil-has-beacon, sigil-lit-by]");

fn json_of(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "stdout is not one JSON document: {}\nstderr: {}",
            stdout(output),
            stderr(output)
        )
    })
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn section<'v>(report: &'v Value, relation: &str) -> &'v Value {
    report["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == relation)
        .unwrap_or_else(|| panic!("no relation {relation}"))
}

fn row<'v>(report: &'v Value, relation: &str, id: &str) -> &'v Value {
    section(report, relation)["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["document"]["id"] == id)
        .unwrap_or_else(|| panic!("no row {id}"))
}

#[trace("FR-041-AC-1")]
#[trace("FR-041-AC-2")]
#[trace("FR-041-AC-10")]
#[test]
fn json_is_the_join_alone_and_is_byte_identical_across_runs() {
    let world = World::new(DESIGN);
    let scope = world.worked_example();
    let first = world.design(&scope, &["--json"]);
    assert!(first.status.success(), "{}", stderr(&first));
    let report = json_of(&first);

    let lit = row(&report, "sigil-has-beacon", "SIG-001");
    assert_eq!(lit["state"], "covered");
    assert_eq!(lit["linked"][0]["id"], "BEA-001");
    assert_eq!(lit["linked"][0]["archetype"], "Beacon");
    assert_eq!(lit["linked"][0]["realized_criteria"][0], "SIG-001-AC-1");

    let gap = row(&report, "sigil-has-beacon", "SIG-002");
    assert_eq!(gap["state"], "uncovered");
    assert_eq!(gap["linked"].as_array().unwrap().len(), 0);

    assert_eq!(first.stderr, b"" as &[u8]);
    let second = world.design(&scope, &["--json"]);
    assert_eq!(first.stdout, second.stdout);
}

#[trace("FR-041-AC-3")]
#[test]
fn a_relation_outside_design_relations_produces_no_block() {
    let world = World::new(DESIGN);
    let scope = world.worked_example();
    let report = json_of(&world.design(&scope, &["--json"]));
    let names: Vec<&str> = report["relations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["sigil-has-beacon", "sigil-lit-by"]);
    let human = stdout(&world.design(&scope, &[]));
    assert!(!human.contains("sigil-has-kiln"), "{human}");
}

#[trace("FR-041-AC-4")]
#[trace("FR-041-AC-5")]
#[trace("FR-041-AC-10")]
#[test]
fn human_output_prints_elements_gaps_and_a_summary_equal_to_the_json() {
    let world = World::new(DESIGN);
    let scope = world.worked_example();
    let human = stdout(&world.design(&scope, &[]));
    assert!(
        human.contains(
            "SIG-001  covered  unverified  linked: BEA-001 (Beacon) realizes SIG-001-AC-1"
        ),
        "{human}"
    );
    assert!(
        human.contains("SIG-002  uncovered  unverified  no design element sigil-dark"),
        "{human}"
    );

    let report = json_of(&world.design(&scope, &["--json"]));
    for section in report["relations"].as_array().unwrap() {
        let s = &section["summary"];
        let expected = format!(
            "  summary: {} row(s); uncovered {}, covered {}; verified {}, partial {}, \
             unverified {}, not-tracked {}\n",
            s["rows"],
            s["state"]["uncovered"],
            s["state"]["covered"],
            s["verification"]["verified"],
            s["verification"]["partial"],
            s["verification"]["unverified"],
            s["verification"]["not-tracked"],
        );
        assert!(human.contains(&expected), "missing `{expected}` in {human}");
    }
}

#[trace("FR-041-AC-1")]
#[trace("FR-041-AC-4")]
#[trace("FR-041-AC-10")]
#[test]
fn worked_example_matches_its_snapshots() {
    let world = World::new(DESIGN);
    let scope = world.worked_example();
    assert_eq!(
        stdout(&world.design(&scope, &[])),
        include_str!("snapshots/relations-design-worked-example.txt")
    );
    assert_eq!(
        stdout(&world.design(&scope, &["--json", "--pretty"])),
        include_str!("snapshots/relations-design-worked-example.json")
    );
}

#[trace("FR-041-AC-6")]
#[test]
fn an_element_in_a_sibling_repo_resolves_and_a_missing_member_dangles() {
    let world = World::new(DESIGN);
    let a = world.repo(
        "a",
        &[(
            "SIG-001",
            doc(
                "SIG-001",
                "Sigil",
                &[("kindled_by", "ix://o/b/spec/BEA-001")],
                &[],
            ),
        )],
    );
    let b = world.repo("b", &[("BEA-001", doc("BEA-001", "Beacon", &[], &[]))]);

    let joined = world.design_repos(&[("o/a", &a), ("o/b", &b)], &["--json"]);
    assert!(joined.status.success(), "{}", stderr(&joined));
    let report = json_of(&joined);
    let lit = row(&report, "sigil-lit-by", "SIG-001");
    assert_eq!(lit["state"], "covered");
    assert_eq!(lit["linked"][0]["id"], "BEA-001");
    assert_eq!(lit["linked"][0]["repo"], "o/b");
    let human = stdout(&world.design_repos(&[("o/a", &a), ("o/b", &b)], &[]));
    assert!(human.contains("o/b:BEA-001 (Beacon)"), "{human}");

    let alone = json_of(&world.design_repos(&[("o/a", &a)], &["--json"]));
    let gap = row(&alone, "sigil-lit-by", "SIG-001");
    assert_eq!(gap["state"], "uncovered");
    assert_eq!(gap["dangling"][0]["reference"], "ix://o/b/spec/BEA-001");
    let human = stdout(&world.design_repos(&[("o/a", &a)], &[]));
    assert!(human.contains("dangling: ix://o/b/spec/BEA-001"), "{human}");
}

#[trace("FR-041-AC-4")]
#[test]
fn an_element_linked_as_a_whole_prints_without_a_realizes_list() {
    let world = World::new(DESIGN);
    let scope = world.repo(
        "repo",
        &[
            ("SIG-003", doc("SIG-003", "Sigil", &[], &["SIG-003-AC-1"])),
            (
                "BEA-002",
                doc("BEA-002", "Beacon", &[("kindles", "SIG-003")], &[]),
            ),
        ],
    );
    let human = stdout(&world.design(&scope, &[]));
    let line = human
        .lines()
        .find(|l| l.contains("BEA-002"))
        .unwrap_or_else(|| panic!("no element line in {human}"));
    assert!(line.ends_with("linked: BEA-002 (Beacon)"), "{line}");
    assert!(!human.contains("realizes"), "{human}");
}

#[trace("FR-030-AC-25")]
#[trace("FR-041-AC-6")]
#[test]
fn design_module_discovery_without_module_is_bounded_by_the_first_repo_member() {
    let world = World::new(DESIGN);
    let a = world.repo("a", &[("SIG-001", doc("SIG-001", "Sigil", &[], &[]))]);
    let b = world.repo("b", &[("SIG-002", doc("SIG-002", "Sigil", &[], &[]))]);
    // Each member carries its own module at its root, with different relation
    // names, so the run never falls through to host-installed modules.
    let manifest_a = fs::read_to_string(world.module_dir().join("manifest.yaml")).unwrap();
    fs::write(a.join("manifest.yaml"), &manifest_a).unwrap();
    fs::write(
        b.join("manifest.yaml"),
        manifest_a
            .replace("sigil-has-beacon", "sigil-has-lamp")
            .replace("sigil-lit-by", "sigil-lamped-by"),
    )
    .unwrap();
    let (arg_a, arg_b) = (
        format!("o/a={}", a.display()),
        format!("o/b={}", b.display()),
    );
    let run = |first: &str, second: &str| {
        let mut command = common::quire();
        command.args(["relations", "--design", "--repo", first, "--repo", second]);
        command.output().unwrap()
    };
    let a_first = run(&arg_a, &arg_b);
    assert!(a_first.status.success(), "{}", stderr(&a_first));
    let text = stdout(&a_first);
    assert!(
        text.contains("sigil-has-beacon") && !text.contains("sigil-has-lamp"),
        "{text}"
    );
    let b_first = run(&arg_b, &arg_a);
    assert!(b_first.status.success(), "{}", stderr(&b_first));
    let text = stdout(&b_first);
    assert!(
        text.contains("sigil-has-lamp") && !text.contains("sigil-has-beacon"),
        "{text}"
    );
}

fn assert_invalid(output: &Output, needles: &[&str]) {
    assert_eq!(output.status.code(), Some(3), "{}", stderr(output));
    assert_eq!(stdout(output), "", "no report on stdout");
    let err = stderr(output);
    for needle in needles {
        assert!(err.contains(needle), "missing `{needle}` in {err}");
    }
}

#[trace("FR-041-AC-7")]
#[test]
fn a_module_set_without_design_relations_exits_3_naming_the_declaration() {
    let world = World::new(None);
    let scope = world.worked_example();
    assert_invalid(&world.design(&scope, &["--json"]), &["design_relations"]);
    assert_invalid(&world.design(&scope, &[]), &["design_relations"]);
}

#[trace("FR-041-AC-8")]
#[test]
fn an_entry_naming_no_declared_relation_exits_3_naming_the_entry() {
    let world = World::new(Some("[sigil-has-beacon, sigil-phantom]"));
    let scope = world.worked_example();
    assert_invalid(&world.design(&scope, &["--json"]), &["sigil-phantom"]);
}

#[trace("FR-041-AC-9")]
#[test]
fn design_conflicts_with_relation_and_with_strict() {
    let world = World::new(DESIGN);
    let scope = world.worked_example();
    assert_invalid(
        &world.design(&scope, &["--relation", "sigil-has-beacon"]),
        &["--design", "--relation"],
    );
    assert_invalid(
        &world.design(&scope, &["--strict"]),
        &["--design", "--strict"],
    );
}
