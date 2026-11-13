//! FR-030 — `quire relations` and `quire validate --okf --repo`.
//!
//! Kinds, verbs and fields are authored by the fixture module written below;
//! the CLI knows none of them.
mod common;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

use ix_trace_rs::trace;
use serde_json::Value;
use tempfile::TempDir;

/// The fixture module: a forward relation (every `Concern` is reached by
/// `addresses` from a `Rule`) and a reverse one (every gold-tier `Rule`
/// reaches a `Concern`). `obligations` toggles the FR-050 matrix source.
fn manifest(obligations: bool, exclude: bool) -> String {
    let obligations = if obligations {
        "  obligations:\n  - name: acceptance-criterion\n    target: acceptance-criterion\n    statement_column: Criteria\n    method_column: Verification\n"
    } else {
        ""
    };
    let exclude = if exclude {
        "    exclude: ['spec/fixtures/**']\n"
    } else {
        ""
    };
    format!(
        "name: relations-fixture\nmanifest_version: 1.0.0\nversion: 0.1.0\nartifact_types:\n\
         - name: Concern\n- name: Rule\n\
         traceability:\n  trace_targets:\n  - name: acceptance-criterion\n    archetype: Rule\n    section: Acceptance Criteria\n    id_column: ID\n{exclude}\
         {obligations}\
         \x20 trace_tags:\n    markers:\n    - name: rust-trace-attribute\n      language: rust\n      pattern: '#\\[trace\\(([^)]*)\\)\\]'\n      template: '#[trace({{ids}})]'\n\
         \x20 required_relations:\n\
         \x20 - name: concern-is-addressed\n    from: Concern\n    edges: [addresses]\n    to: [Rule]\n    direction: incoming\n    check: unaddressed-concern\n\
         \x20 - name: gold-rule-addresses\n    from: Rule\n    where:\n      field: tier\n      values: [gold]\n    edges: [addresses]\n    to: [Concern]\n    direction: outgoing\n    check: gold-rule-idle\n"
    )
}

fn relationships(rels: &[(&str, &str)]) -> String {
    if rels.is_empty() {
        return String::new();
    }
    let mut out = String::from("relationships:\n");
    for (verb, target) in rels {
        out.push_str(&format!("  - target: \"{target}\"\n    type: \"{verb}\"\n"));
    }
    out
}

fn concern(id: &str) -> String {
    format!("---\nid: {id}\ntype: Concern\ntitle: {id}\n---\n\n# {id}\n")
}

fn rule(id: &str, tier: Option<&str>, rels: &[(&str, &str)], criteria: &[&str]) -> String {
    let tier = tier.map(|t| format!("tier: {t}\n")).unwrap_or_default();
    let mut text = format!(
        "---\nid: {id}\ntype: Rule\ntitle: {id}\n{tier}{}---\n\n# {id}\n",
        relationships(rels)
    );
    if !criteria.is_empty() {
        text.push_str(
            "\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|----|----------|--------------|\n",
        );
        for ac in criteria {
            text.push_str(&format!("| {ac} | The system shall hold. | Test |\n"));
        }
    }
    text
}

struct World {
    tmp: TempDir,
}

impl World {
    fn new() -> Self {
        let tmp = TempDir::new().unwrap();
        let world = Self { tmp };
        world.module(true, false);
        world
    }

    fn module(&self, obligations: bool, exclude: bool) {
        let dir = self.tmp.path().join("m");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("manifest.yaml"), manifest(obligations, exclude)).unwrap();
    }

    fn module_dir(&self) -> PathBuf {
        self.tmp.path().join("m")
    }

    /// A repository `name` holding `docs` (`(file stem, text)`) and one test
    /// per tagged criterion.
    fn repo(&self, name: &str, docs: &[(&str, String)], tagged: &[&str]) -> PathBuf {
        let root = self.tmp.path().join(name);
        fs::create_dir_all(root.join("spec")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        for (stem, text) in docs {
            let path = root.join("spec").join(format!("{stem}.md"));
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        let mut src = String::new();
        for (n, ac) in tagged.iter().enumerate() {
            src.push_str(&format!("#[trace(\"{ac}\")]\n#[test]\nfn t{n}() {{}}\n"));
        }
        fs::write(root.join("src/lib.rs"), src).unwrap();
        root
    }
}

fn quire(args: &[&dyn AsRef<std::ffi::OsStr>]) -> Output {
    let mut command = common::quire();
    for arg in args {
        command.arg(arg.as_ref());
    }
    command.output().unwrap()
}

fn relations(world: &World, scope: &Path, extra: &[&str]) -> Output {
    let mut command = common::quire();
    command
        .arg("relations")
        .arg("--scope")
        .arg(scope)
        .arg("--module")
        .arg(world.module_dir())
        .args(extra);
    command.output().unwrap()
}

fn json_of(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
        panic!(
            "stdout is not one JSON document: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn row<'v>(report: &'v Value, relation: &str, id: &str) -> &'v Value {
    report["relations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == relation)
        .unwrap_or_else(|| panic!("no relation {relation}"))["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["document"]["id"] == id)
        .unwrap_or_else(|| panic!("no row {id}"))
}

/// C1 unmitigated, C2 addressed by an untested rule, C3 by a tested one; R3 is
/// a gold rule that addresses nothing.
fn worked_example(world: &World) -> PathBuf {
    world.repo(
        "repo",
        &[
            ("CON-001", concern("CON-001")),
            ("CON-002", concern("CON-002")),
            ("CON-003", concern("CON-003")),
            (
                "RULE-001",
                rule(
                    "RULE-001",
                    Some("gold"),
                    &[("addresses", "CON-003")],
                    &["RULE-001-AC-1"],
                ),
            ),
            (
                "RULE-002",
                rule(
                    "RULE-002",
                    None,
                    &[("addresses", "CON-002")],
                    &["RULE-002-AC-1"],
                ),
            ),
            ("RULE-003", rule("RULE-003", Some("gold"), &[], &[])),
        ],
        &["RULE-001-AC-1"],
    )
}

#[trace("FR-030-AC-1")]
#[trace("FR-030-AC-2")]
#[test]
fn json_rows_carry_state_verification_and_linked_documents() {
    let world = World::new();
    let scope = worked_example(&world);
    let first = relations(&world, &scope, &["--json"]);
    assert!(first.status.success(), "{}", stderr(&first));
    let report = json_of(&first);

    let uncovered = row(&report, "concern-is-addressed", "CON-001");
    assert_eq!(uncovered["state"], "uncovered");
    let untested = row(&report, "concern-is-addressed", "CON-002");
    assert_eq!(untested["state"], "covered");
    assert_eq!(untested["verification"], "unverified");
    assert_eq!(untested["linked"][0]["id"], "RULE-002");
    assert_eq!(untested["linked"][0]["verification"], "unverified");
    let tested = row(&report, "concern-is-addressed", "CON-003");
    assert_eq!(tested["verification"], "verified");
    assert_eq!(tested["linked"][0]["id"], "RULE-001");
    assert_eq!(tested["linked"][0]["verification"], "verified");

    let second = relations(&world, &scope, &["--json"]);
    assert_eq!(first.stdout, second.stdout, "byte-identical across runs");
}

#[trace("FR-030-AC-3")]
#[trace("FR-030-AC-4")]
#[trace("FR-030-AC-5")]
#[test]
fn human_output_marks_gaps_with_the_check_token_and_ends_with_the_summary() {
    let world = World::new();
    let scope = worked_example(&world);
    let human = relations(&world, &scope, &[]);
    assert!(human.status.success(), "{}", stderr(&human));
    let text = stdout(&human);
    let line_of = |needle: &str| {
        text.lines()
            .find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("no line with {needle}: {text}"))
            .to_string()
    };
    let gap = line_of("CON-001");
    assert!(
        gap.contains("uncovered") && gap.contains("GAP unaddressed-concern"),
        "{gap}"
    );
    let ok = line_of("CON-003");
    assert!(
        ok.contains("covered") && ok.contains("verified") && !ok.contains("GAP"),
        "{ok}"
    );
    assert!(ok.contains("RULE-001 (verified)"), "{ok}");
    // Reverse relation: a gold rule with no linked concern.
    let reverse = line_of("RULE-003");
    assert!(reverse.contains("GAP gold-rule-idle"), "{reverse}");

    // The summary of each block equals the `--json` summary.
    let report = json_of(&relations(&world, &scope, &["--json"]));
    for section in report["relations"].as_array().unwrap() {
        let s = &section["summary"];
        let expected = format!(
            "summary: {} row(s); uncovered {}, covered {}; verified {}, partial {}, unverified {}, not-tracked {}",
            s["rows"],
            s["state"]["uncovered"],
            s["state"]["covered"],
            s["verification"]["verified"],
            s["verification"]["partial"],
            s["verification"]["unverified"],
            s["verification"]["not-tracked"],
        );
        assert!(text.contains(&expected), "missing `{expected}` in {text}");
    }
}

#[trace("FR-030-AC-1")]
#[trace("FR-030-AC-3")]
#[test]
fn worked_example_matches_its_snapshots() {
    let world = World::new();
    let scope = worked_example(&world);
    assert_eq!(
        stdout(&relations(&world, &scope, &[])),
        include_str!("snapshots/relations-worked-example.txt")
    );
    assert_eq!(
        stdout(&relations(&world, &scope, &["--json", "--pretty"])),
        include_str!("snapshots/relations-worked-example.json")
    );
}

#[trace("FR-030-AC-6")]
#[test]
fn a_cross_repo_mitigation_covers_the_hazard_and_a_missing_member_dangles() {
    let world = World::new();
    let a = world.repo(
        "a",
        &[(
            "RULE-001",
            rule(
                "RULE-001",
                Some("gold"),
                &[("addresses", "ix://o/b/spec/CON-001")],
                &["RULE-001-AC-1"],
            ),
        )],
        &["RULE-001-AC-1"],
    );
    let b = world.repo("b", &[("CON-001", concern("CON-001"))], &[]);
    let repo_arg = |identity: &str, dir: &Path| format!("{identity}={}", dir.display());

    let mut command = common::quire();
    command
        .args(["relations", "--json", "--module"])
        .arg(world.module_dir())
        .arg("--repo")
        .arg(repo_arg("o/a", &a))
        .arg("--repo")
        .arg(repo_arg("o/b", &b));
    let joined = command.output().unwrap();
    assert!(joined.status.success(), "{}", stderr(&joined));
    let report = json_of(&joined);
    let hazard = row(&report, "concern-is-addressed", "CON-001");
    assert_eq!(hazard["state"], "covered");
    assert_eq!(hazard["document"]["repo"], "o/b");
    assert_eq!(hazard["linked"][0]["repo"], "o/a");

    let mut command = common::quire();
    command
        .args(["relations", "--json", "--module"])
        .arg(world.module_dir())
        .arg("--repo")
        .arg(repo_arg("o/a", &a));
    let alone = command.output().unwrap();
    let report = json_of(&alone);
    let rule_row = row(&report, "gold-rule-addresses", "RULE-001");
    assert_eq!(rule_row["state"], "uncovered");
    assert_eq!(
        rule_row["dangling"][0]["reference"],
        "ix://o/b/spec/CON-001"
    );
}

#[trace("FR-030-AC-7")]
#[test]
fn a_wrong_kind_target_prints_beneath_its_row_as_a_kind_mismatch() {
    let world = World::new();
    let scope = world.repo(
        "repo",
        &[
            (
                "RULE-001",
                rule("RULE-001", Some("gold"), &[("addresses", "RULE-002")], &[]),
            ),
            ("RULE-002", rule("RULE-002", None, &[], &[])),
        ],
        &[],
    );
    let json = json_of(&relations(&world, &scope, &["--json"]));
    let mismatch = &row(&json, "gold-rule-addresses", "RULE-001")["kind_mismatch"][0];
    assert_eq!(mismatch["id"], "RULE-002");
    assert_eq!(mismatch["archetype"], "Rule");
    let text = stdout(&relations(&world, &scope, &[]));
    assert!(
        text.contains("kind mismatch: RULE-002 is Rule"),
        "human output: {text}"
    );
}

#[trace("FR-030-AC-7")]
#[test]
fn an_untyped_kind_mismatch_target_prints_the_engines_untyped_wording() {
    let world = World::new();
    let untyped = "---\nid: UNTYPED-1\ntitle: no type\n---\n\n# UNTYPED-1\n";
    let scope = world.repo(
        "repo",
        &[
            (
                "RULE-001",
                rule("RULE-001", Some("gold"), &[("addresses", "UNTYPED-1")], &[]),
            ),
            ("UNTYPED-1", untyped.to_string()),
        ],
        &[],
    );
    let json = json_of(&relations(&world, &scope, &["--json"]));
    let mismatch = &row(&json, "gold-rule-addresses", "RULE-001")["kind_mismatch"][0];
    assert_eq!(mismatch["id"], "UNTYPED-1");
    assert!(mismatch.get("archetype").is_none(), "{mismatch}");
    let text = stdout(&relations(&world, &scope, &[]));
    assert!(
        text.contains("kind mismatch: UNTYPED-1 is (untyped) (via addresses)"),
        "human output: {text}"
    );
}

#[trace("FR-030-AC-8")]
#[trace("FR-030-AC-9")]
#[test]
fn relation_restricts_the_report_and_an_unknown_name_is_refused() {
    let world = World::new();
    let scope = worked_example(&world);
    let one = json_of(&relations(
        &world,
        &scope,
        &["--json", "--relation", "gold-rule-addresses"],
    ));
    assert_eq!(one["relations"].as_array().unwrap().len(), 1);
    assert_eq!(one["relations"][0]["name"], "gold-rule-addresses");

    let unknown = relations(
        &world,
        &scope,
        &["--json", "--relation", "no-such-relation"],
    );
    assert_eq!(unknown.status.code(), Some(3));
    assert!(stderr(&unknown).contains("no-such-relation"));
    assert!(unknown.stdout.is_empty(), "no report on stdout");
}

#[trace("FR-030-AC-9")]
#[test]
fn an_unknown_relation_name_is_refused_before_any_coverage_is_computed() {
    let world = World::new();
    // A tagged symbol in an unparseable source file makes the coverage walk
    // emit a stderr diagnostic; a refusal before the walk emits none.
    let scope = worked_example(&world);
    fs::write(
        scope.join("src/lib.rs"),
        "#[trace(\"FR-1\")]\n#[test]\nfn broken( {\n",
    )
    .unwrap();
    let known = relations(&world, &scope, &["--relation", "gold-rule-addresses"]);
    let unknown = relations(&world, &scope, &["--relation", "no-such-relation"]);
    assert_eq!(unknown.status.code(), Some(3));
    assert_eq!(
        stderr(&unknown).lines().count(),
        1,
        "only the refusal, no coverage diagnostics (a computed run prints: {}): {}",
        stderr(&known),
        stderr(&unknown)
    );
    assert!(stderr(&unknown).contains("unknown --relation 'no-such-relation'"));
}

#[trace("FR-030-AC-9")]
#[test]
fn module_discovery_without_module_is_bounded_by_the_first_repo_member() {
    let world = World::new();
    let a = world.repo("a", &[("CON-001", concern("CON-001"))], &[]);
    let b = world.repo("b", &[("CON-002", concern("CON-002"))], &[]);
    // Only member `a` carries a module at its root.
    fs::copy(
        world.module_dir().join("manifest.yaml"),
        a.join("manifest.yaml"),
    )
    .unwrap();
    let (arg_a, arg_b) = (
        format!("o/a={}", a.display()),
        format!("o/b={}", b.display()),
    );
    let a_first = quire(&[&"relations", &"--repo", &arg_a, &"--repo", &arg_b]);
    assert!(a_first.status.success(), "{}", stderr(&a_first));
    assert!(stdout(&a_first).contains("concern-is-addressed"));
    let b_first = quire(&[&"relations", &"--repo", &arg_b, &"--repo", &arg_a]);
    assert!(b_first.status.success(), "{}", stderr(&b_first));
    assert!(
        !stdout(&b_first).contains("concern-is-addressed"),
        "member a's module is ignored when it is not first: {}",
        stdout(&b_first)
    );
}

/// One concern per rule, each addressed by its rule; `criteria`/`tagged`
/// describe the rule's table and tests.
fn verdict_repo(world: &World, rules: &[(&str, &[&str], &[&str])]) -> PathBuf {
    let mut docs = Vec::new();
    let mut tagged: Vec<&str> = Vec::new();
    for (n, (id, criteria, tags)) in rules.iter().enumerate() {
        let concern_id = format!("CON-00{}", n + 1);
        docs.push((concern_id.clone(), concern(&concern_id)));
        docs.push((
            id.to_string(),
            rule(id, None, &[("addresses", &concern_id)], criteria),
        ));
        tagged.extend(tags.iter().copied());
    }
    let docs: Vec<(&str, String)> = docs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
    world.repo("repo", &docs, &tagged)
}

#[trace("FR-030-AC-10")]
#[test]
fn strict_emits_the_whole_report_then_fails_on_an_uncovered_row() {
    let world = World::new();
    let scope = world.repo("repo", &[("CON-001", concern("CON-001"))], &[]);
    let args = ["--json", "--relation", "concern-is-addressed"];
    let plain = relations(&world, &scope, &args);
    assert_eq!(plain.status.code(), Some(0));
    let mut strict_args = args.to_vec();
    strict_args.push("--strict");
    let strict = relations(&world, &scope, &strict_args);
    assert_eq!(strict.status.code(), Some(1));
    assert_eq!(
        plain.stdout, strict.stdout,
        "the complete report is emitted"
    );
}

#[trace("FR-030-AC-11")]
#[trace("FR-030-AC-12")]
#[test]
fn strict_fails_on_unverified_and_partial_rows_and_passes_on_verified() {
    let world = World::new();
    let args = ["--relation", "concern-is-addressed"];
    let with_strict = |strict: bool| {
        let mut a = args.to_vec();
        if strict {
            a.push("--strict");
        }
        a
    };
    let code =
        |scope: &Path, strict: bool| relations(&world, scope, &with_strict(strict)).status.code();

    let unverified = verdict_repo(&world, &[("RULE-001", &["RULE-001-AC-1"], &[])]);
    assert_eq!(
        (code(&unverified, true), code(&unverified, false)),
        (Some(1), Some(0))
    );
    fs::remove_dir_all(&unverified).unwrap();

    let partial = verdict_repo(
        &world,
        &[(
            "RULE-001",
            &["RULE-001-AC-1", "RULE-001-AC-2"],
            &["RULE-001-AC-1"],
        )],
    );
    assert_eq!(
        (code(&partial, true), code(&partial, false)),
        (Some(1), Some(0))
    );
    fs::remove_dir_all(&partial).unwrap();

    let verified = verdict_repo(
        &world,
        &[("RULE-001", &["RULE-001-AC-1"], &["RULE-001-AC-1"])],
    );
    assert_eq!(
        (code(&verified, true), code(&verified, false)),
        (Some(0), Some(0))
    );
}

#[trace("FR-030-AC-13")]
#[test]
fn strict_refuses_an_empty_population_and_a_measurement_of_nothing() {
    let world = World::new();
    // Zero rows: the reverse relation selects no gold rule.
    let empty = world.repo(
        "empty",
        &[("RULE-001", rule("RULE-001", None, &[], &[]))],
        &[],
    );
    let args = ["--relation", "gold-rule-addresses"];
    let strict = relations(&world, &empty, &[args[0], args[1], "--strict"]);
    assert_eq!(strict.status.code(), Some(1));
    assert!(
        stderr(&strict).contains("empty-population"),
        "{}",
        stderr(&strict)
    );
    assert_eq!(relations(&world, &empty, &args).status.code(), Some(0));

    // Every row not-tracked: a module declaring no `obligations:` source.
    world.module(false, false);
    let untracked = verdict_repo(
        &world,
        &[("RULE-001", &["RULE-001-AC-1"], &["RULE-001-AC-1"])],
    );
    let args = ["--relation", "concern-is-addressed"];
    let strict = relations(&world, &untracked, &[args[0], args[1], "--strict"]);
    assert_eq!(strict.status.code(), Some(1));
    assert!(
        stderr(&strict).contains("nothing-measured"),
        "{}",
        stderr(&strict)
    );
    assert!(!stderr(&strict).contains("empty-population"));
    assert_eq!(relations(&world, &untracked, &args).status.code(), Some(0));
}

#[trace("FR-030-AC-14")]
#[trace("FR-030-AC-15")]
#[trace("FR-030-AC-17")]
#[test]
fn missing_inputs_are_named_and_exit_3() {
    let world = World::new();
    let bare = world.tmp.path().join("bare");
    fs::create_dir_all(&bare).unwrap();
    let out = quire(&[
        &"--diagnostics-format",
        &"json",
        &"relations",
        &"--scope",
        &bare,
        &"--module",
        &world.module_dir(),
    ]);
    assert_eq!(out.status.code(), Some(3));
    let err = stderr(&out);
    assert!(err.contains("MissingDocumentRoot"), "{err}");
    assert!(
        err.contains(&bare.join("spec").display().to_string()),
        "{err}"
    );

    let out = quire(&[
        &"relations",
        &"--repo",
        &format!("o/a={}", bare.display()),
        &"--module",
        &world.module_dir(),
    ]);
    assert_eq!(out.status.code(), Some(3));
    let err = stderr(&out);
    assert!(err.contains("member 'o/a'"), "{err}");
    assert!(err.contains("no document root"), "{err}");
    assert!(
        err.contains(&bare.join("spec").display().to_string()),
        "{err}"
    );
    assert!(!err.contains("point --scope"), "{err}");
    let json = quire(&[
        &"--diagnostics-format",
        &"json",
        &"relations",
        &"--repo",
        &format!("o/a={}", bare.display()),
        &"--module",
        &world.module_dir(),
    ]);
    assert_eq!(json.status.code(), Some(3));
    assert!(
        stderr(&json).contains("MissingDocumentRoot"),
        "{}",
        stderr(&json)
    );

    // A module set that declares no `required_relations`.
    let plain = world.tmp.path().join("plain");
    fs::create_dir_all(&plain).unwrap();
    fs::write(plain.join("manifest.yaml"), "name: plain\nmanifest_version: 1.0.0\nversion: 0.1.0\nartifact_types:\n- name: Rule\ntraceability:\n  trace_targets:\n  - name: t\n    archetype: Rule\n    section: S\n    id_column: ID\n").unwrap();
    let scope = worked_example(&world);
    let out = quire(&[&"relations", &"--scope", &scope, &"--module", &plain]);
    assert_eq!(out.status.code(), Some(3));
    assert!(
        stderr(&out).contains("required_relations"),
        "{}",
        stderr(&out)
    );
    assert!(out.stdout.is_empty());
}

#[trace("FR-030-AC-16")]
#[trace("FR-030-AC-18")]
#[trace("FR-030-AC-19")]
#[test]
fn repo_flag_misuse_is_refused() {
    let world = World::new();
    let a = worked_example(&world);
    let module = world.module_dir();
    let arg = format!("o/a={}", a.display());

    let both = quire(&[
        &"relations",
        &"--scope",
        &a,
        &"--repo",
        &arg,
        &"--module",
        &module,
    ]);
    assert_eq!(both.status.code(), Some(3));
    let err = stderr(&both);
    assert!(err.contains("--scope") && err.contains("--repo"), "{err}");

    let traversal = quire(&[
        &"relations",
        &"--repo",
        &"o/a=../escape",
        &"--module",
        &module,
    ]);
    assert_eq!(traversal.status.code(), Some(2), "{}", stderr(&traversal));
    assert!(traversal.stdout.is_empty());

    let twice = quire(&[
        &"relations",
        &"--repo",
        &arg,
        &"--repo",
        &arg,
        &"--module",
        &module,
    ]);
    assert_eq!(twice.status.code(), Some(3));
    assert!(stderr(&twice).contains("o/a"), "{}", stderr(&twice));

    let malformed = quire(&[
        &"relations",
        &"--repo",
        &"not-an-identity",
        &"--module",
        &module,
    ]);
    assert_eq!(malformed.status.code(), Some(3));
    assert!(stderr(&malformed).contains("not-an-identity"));
}

fn validate_okf(world: &World, repos: &[(&str, &Path)], extra: &[&str]) -> Output {
    let mut command = common::quire();
    command.args(["validate", "--okf", "--module"]);
    command.arg(world.module_dir());
    for (identity, dir) in repos {
        command
            .arg("--repo")
            .arg(format!("{identity}={}", dir.display()));
    }
    command.args(extra);
    command.output().unwrap()
}

#[trace("FR-030-AC-20")]
#[test]
fn validate_resolves_an_edge_into_a_sibling_member_and_names_the_member() {
    let world = World::new();
    let a = world.repo(
        "a",
        &[(
            "RULE-001",
            rule(
                "RULE-001",
                Some("gold"),
                &[("addresses", "ix://o/b/spec/CON-001")],
                &[],
            ),
        )],
        &[],
    );
    let b = world.repo("b", &[("CON-001", concern("CON-001"))], &[]);

    let both = validate_okf(&world, &[("o/a", &a), ("o/b", &b)], &[]);
    assert!(both.status.success(), "{}", stderr(&both));
    assert!(
        !stderr(&both).contains("gold-rule-idle"),
        "the edge resolves: {}",
        stderr(&both)
    );

    let alone = validate_okf(&world, &[("o/a", &a)], &[]);
    let err = stderr(&alone);
    let finding = err
        .lines()
        .find(|l| l.contains("gold-rule-idle"))
        .unwrap_or_else(|| panic!("no finding: {err}"));
    assert!(finding.contains("[o/a] "), "{finding}");
}

#[trace("FR-030-AC-21")]
#[trace("FR-030-AC-23")]
#[test]
fn validate_repo_requires_okf_and_excludes_scope_and_positional_bundle() {
    let world = World::new();
    let a = worked_example(&world);
    let arg = format!("o/a={}", a.display());

    let no_okf = quire(&[
        &"validate",
        &"--repo",
        &arg,
        &"--module",
        &world.module_dir(),
    ]);
    assert_eq!(no_okf.status.code(), Some(3));
    assert!(stderr(&no_okf).contains("--okf"), "{}", stderr(&no_okf));

    let scoped = quire(&[&"validate", &"--okf", &"--repo", &arg, &"--scope", &a]);
    assert_eq!(scoped.status.code(), Some(3));
    let err = stderr(&scoped);
    assert!(err.contains("--scope") && err.contains("--repo"), "{err}");

    let positional = quire(&[&"validate", &a, &"--okf", &"--repo", &arg]);
    assert_eq!(positional.status.code(), Some(3));
    let err = stderr(&positional);
    assert!(
        err.contains("--repo") && err.contains("DOC_OR_GLOB"),
        "{err}"
    );
}

#[trace("FR-030-AC-24")]
#[test]
fn each_members_declarations_resolve_against_its_own_repository_root() {
    let world = World::new();
    let broken = "---\nid: FR-900\ntype: Rule\ntitle: fixture\n---\n\n## Acceptance Criteria\n\n| ID | Criteria | Verification |\n|----|----------|--------------|\n| FR-900-AC-1 | A broken fixture. | Test (TC-404) |\n";
    let docs = [("fixtures/FR-900", broken.to_string())];
    let a = world.repo("a", &docs, &[]);
    let b = world.repo("b", &docs, &[]);
    // The fixture subtree is excluded by a glob authored against the repository
    // root: `<DIR>`, not `<DIR>/spec`.
    let write_module = |exclude: bool| {
        let mut text = manifest(true, exclude);
        text = text.replace(
            "  trace_tags:",
            &format!("  document_references:\n  - name: verification\n    archetype: Rule\n    section: Acceptance Criteria\n    column: Verification\n    row_id_column: ID\n    pattern: '\\((TC-\\d+)\\)'\n    targets: [acceptance-criterion]\n{}  trace_tags:", if exclude { "    exclude: ['spec/fixtures/**']\n" } else { "" }),
        );
        fs::write(world.module_dir().join("manifest.yaml"), text).unwrap();
    };
    let dangling = |out: &Output| {
        stderr(out)
            .lines()
            .filter(|l| l.contains("TC-404"))
            .map(str::to_string)
            .collect::<Vec<_>>()
    };

    write_module(true);
    let excluded = validate_okf(&world, &[("o/a", &a), ("o/b", &b)], &[]);
    assert!(excluded.status.success(), "{}", stderr(&excluded));
    assert!(dangling(&excluded).is_empty(), "{}", stderr(&excluded));

    write_module(false);
    let included = validate_okf(&world, &[("o/a", &a), ("o/b", &b)], &[]);
    let lines = dangling(&included);
    assert!(
        lines.iter().any(|l| l.contains("[o/a] ")) && lines.iter().any(|l| l.contains("[o/b] ")),
        "each member reports its own: {}",
        stderr(&included)
    );
}
