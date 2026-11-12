//! `validate --okf` ITs — permissive OKF bundle posture.
//!
//! `type` is required in BOTH postures, but under `--okf` unknown types,
//! broken `ix://` links, and `index.md` completeness gaps degrade to
//! warnings (exit 0) instead of errors. Bundles are built in a tempdir and
//! validated against the existing `validate-mod` module.

mod common;

use ix_trace_rs::trace;
use std::fs;

use assert_cmd::prelude::*;
use predicates::prelude::*;
use tempfile::TempDir;

use common::{quire, validate_module};

fn bundle(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    for (rel, body) in files {
        let path = dir.path().join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }
    dir
}

// FR-014-AC-1, FR-014-AC-8, FR-003-AC-5: `type` stays required under
// OKF — an untyped document is a hard error carrying the shared
// `[frontmatter]` vocabulary.
#[trace("FR-014-AC-1", "FR-014-AC-8", "FR-003-AC-5")]
#[test]
fn okf_untyped_document_is_error() {
    let dir = bundle(&[("NOTE-001.md", "---\nid: NOTE-001\n---\n# note\nbody\n")]);
    quire()
        .arg("validate")
        .arg("--okf")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .code(3)
        .stderr(predicate::str::contains("type").and(predicate::str::contains("[frontmatter]")));
}

// FR-014-AC-2, FR-014-AC-3: an unknown type and a dangling `ix://`
// link are tolerated as warnings (exit 0).
#[trace("FR-014-AC-2", "FR-014-AC-3")]
#[test]
fn okf_tolerates_unknown_type_and_broken_link() {
    let dir = bundle(&[(
        "X-1.md",
        "---\nid: X-1\ntype: weird\n---\n# x\nsee [missing](ix://o/r/MISSING)\n",
    )]);
    quire()
        .arg("validate")
        .arg("--okf")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stderr(
            predicate::str::contains("[unknown-type]")
                .and(predicate::str::contains("[dangling-reference]")),
        );
}

// FR-014-AC-4, FR-014-AC-5: an `index.md` missing a sibling artifact
// warns under OKF (exit 0).
#[trace("FR-014-AC-4", "FR-014-AC-5")]
#[test]
fn okf_index_incompleteness_warns() {
    let dir = bundle(&[
        ("X-1.md", "---\nid: X-1\ntype: weird\n---\n# x\nbody\n"),
        ("X-2.md", "---\nid: X-2\ntype: weird\n---\n# x\nbody\n"),
        (
            "index.md",
            "---\ntype: index\n---\n# Root\n\n## Contents\n\n* [X-1](./X-1.md)\n",
        ),
    ]);
    quire()
        .arg("validate")
        .arg("--okf")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success()
        .stderr(
            predicate::str::contains("[index-incomplete]").and(predicate::str::contains("X-2")),
        );
}

// FR-014-AC-6: with no positional, `--okf` validates the document
// root `<scope>/spec` (CR-045) — never the scope itself, which was the
// repo-wide crawl.
#[trace("FR-014-AC-6")]
#[test]
fn okf_defaults_to_scope_spec_directory() {
    let dir = bundle(&[
        ("spec/X-1.md", "---\nid: X-1\ntype: weird\n---\n# x\nbody\n"),
        // An untyped stray at the repo root would be a hard error if it
        // were visited; bounded to spec/, it never is.
        ("NOTE-1.md", "---\nid: NOTE-1\n---\n# note\nbody\n"),
    ]);
    quire()
        .arg("validate")
        .arg("--okf")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .success();
}

// FR-014-AC-6: a scope with no `spec/` is a named error, never a
// silent fallback to walking the scope (CR-045).
#[trace("FR-014-AC-6")]
#[test]
fn okf_missing_spec_root_is_a_named_error() {
    let dir = bundle(&[("X-1.md", "---\nid: X-1\ntype: weird\n---\n# x\nbody\n")]);
    // The interpolated path, not `contains("spec")`: the module-discovery
    // refusal names `spec-artifacts-process` and would satisfy the weaker
    // assertion (agent-ix/quire-cli#31).
    let expected = dir.path().join("spec").display().to_string();
    quire()
        .arg("validate")
        .arg("--okf")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
}

// IT-088 (agent-ix/quire-rs#110): a non-fatal bundle warning is emitted as a
// WARNING in the machine surface. Every `report.warnings` entry used to go
// through the error diagnostic path, which hardcoded `"severity": "error"` — so under
// `--diagnostics json` the severity contradicted the exit code, which was
// correctly 0. The frontmatter-less file is the CR-048 warning that made this
// visible, and the engine now distinguishes its two flavors by machine reason
// (quire-rs CR-051).
#[trace("FR-014-AC-6")]
#[test]
fn it088_bundle_warnings_are_emitted_as_warnings_in_json() {
    let dir = bundle(&[
        (
            "spec/FR-001.md",
            "---\nid: FR-001\ntype: FR\n---\n# FR-001\nbody\n",
        ),
        // No frontmatter block at all.
        ("spec/draft.md", "# draft\n\nno front block.\n"),
        // A complete fence block that is not a YAML mapping.
        ("spec/listy.md", "---\n- a\n- b\n---\n# listy\nbody\n"),
    ]);

    let out = quire()
        .arg("--diagnostics-format")
        .arg("json")
        .arg("validate")
        .arg("--okf")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .output()
        .expect("run");

    // Warnings never move the exit code.
    assert!(
        out.status.success(),
        "frontmatter-less files are warnings, not errors: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let err = String::from_utf8_lossy(&out.stderr);
    let lines: Vec<serde_json::Value> = err
        .lines()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();

    let warning_for = |needle: &str| {
        lines
            .iter()
            .find(|l| l["message"].as_str().is_some_and(|m| m.contains(needle)))
    };

    let absent = warning_for("draft.md").expect("the no-frontmatter warning");
    assert_eq!(
        absent["severity"], "warning",
        "a warning must not be emitted with error severity: {absent}"
    );
    assert_eq!(absent["kind"], "ValidationWarning");
    assert!(
        absent["message"]
            .as_str()
            .unwrap()
            .contains("[no-frontmatter]"),
        "carries the machine reason: {absent}"
    );

    // The two flavors are distinguishable in the machine surface
    // (quire-rs CR-051) — before it, both read `[no-frontmatter]`.
    let malformed = warning_for("listy.md").expect("the malformed-frontmatter warning");
    assert_eq!(malformed["severity"], "warning");
    assert!(
        malformed["message"]
            .as_str()
            .unwrap()
            .contains("[malformed-frontmatter]"),
        "the malformed flavor carries its own reason: {malformed}"
    );
}

// FR-014-AC-3, FR-014-AC-10 (PLAT-1300, quire-rs FR-076): `--okf` declares the repo's bundle
// package from the scope's `origin` remote, so a cross-repo
// `ix://<other-org>/<other-repo>/ID` target is not dangling while a miss
// under the repo's own authority still is.
#[trace("FR-014-AC-3", "FR-014-AC-10")]
#[test]
fn it178_okf_declares_repo_package_so_cross_repo_targets_are_not_dangling() {
    let dir = bundle(&[(
        "spec/X-1.md",
        "---\nid: X-1\ntype: weird\n---\n# x\n\
         cross [other](ix://other-org/other-repo/FR-9) \
         own [missing](ix://acme/widgets/MISSING)\n",
    )]);
    for args in [
        &["init", "-q"][..],
        &[
            "remote",
            "add",
            "origin",
            "https://github.com/acme/widgets.git",
        ][..],
    ] {
        std::process::Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(args)
            .status()
            .expect("git")
            .success()
            .then_some(())
            .expect("git setup");
    }

    let out = quire()
        .arg("validate")
        .arg("--okf")
        .arg("--scope")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .output()
        .expect("run");

    let err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "dangling is a warning: {err}");
    assert!(
        !err.contains("'FR-9'"),
        "cross-repo target must not be reported: {err}"
    );
    assert!(
        err.contains("dangling reference 'MISSING'"),
        "in-repo miss stays dangling: {err}"
    );
}

// FR-014-AC-10 (PLAT-1300): the declared package is the bundle's own
// checkout, not the caller's cwd: a positional bundle with its own origin,
// validated from a different directory, still warns on its own-authority miss
// and skips a foreign one.
#[trace("FR-014-AC-10")]
#[test]
fn it179_positional_bundle_uses_its_own_checkout_identity() {
    let dir = bundle(&[(
        "X-1.md",
        "---\nid: X-1\ntype: weird\n---\n# x\n\
         cross [other](ix://other-org/other-repo/FR-9) \
         own [missing](ix://acme/widgets/MISSING)\n",
    )]);
    for args in [
        &["init", "-q"][..],
        &["remote", "add", "origin", "git@github.com:Acme/Widgets.git"][..],
    ] {
        std::process::Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(args)
            .status()
            .expect("git")
            .success()
            .then_some(())
            .expect("git setup");
    }
    let elsewhere = tempfile::tempdir().expect("tempdir");

    let out = quire()
        .current_dir(elsewhere.path())
        .arg("validate")
        .arg("--okf")
        .arg(dir.path())
        .arg("--module")
        .arg(validate_module())
        .output()
        .expect("run");

    let err = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "dangling is a warning: {err}");
    assert!(
        !err.contains("'FR-9'"),
        "foreign target must be skipped: {err}"
    );
    assert!(
        err.contains("dangling reference 'MISSING'"),
        "own-authority miss still warns: {err}"
    );
}

fn okf_edge_module() -> std::path::PathBuf {
    common::fixture_root().join("okf-edge-mod")
}

/// A note document with the given `object:` and `mitigates` targets.
fn note_doc(id: &str, object: &str, mitigates: &[&str]) -> String {
    let edges: String = mitigates
        .iter()
        .map(|t| format!("- target: {t}\n  type: mitigates\n"))
        .collect();
    let relationships = if edges.is_empty() {
        String::new()
    } else {
        format!("relationships:\n{edges}")
    };
    format!("---\nid: {id}\ntype: note\nobject: {object}\n{relationships}---\n# {id}\nbody\n")
}

/// A bundle that is otherwise Strict-conformant: a root index listing every
/// document, each a registered `note`.
fn strict_bundle(docs: &[(&str, String)]) -> TempDir {
    let listing: String = docs
        .iter()
        .map(|(name, _)| format!("* [{name}](./{name}.md)\n"))
        .collect();
    let mut files: Vec<(String, String)> = vec![(
        "index.md".into(),
        format!("---\ntype: index\nokf_version: \"0.1\"\n---\n# Root\n\n## Contents\n\n{listing}"),
    )];
    for (name, body) in docs {
        files.push((format!("{name}.md"), body.clone()));
    }
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    bundle(&refs)
}

fn set_origin(dir: &TempDir, url: &str) {
    for args in [&["init", "-q"][..], &["remote", "add", "origin", url][..]] {
        std::process::Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(args)
            .status()
            .expect("git")
            .success()
            .then_some(())
            .expect("git setup");
    }
}

fn okf(dir: &TempDir, strict: bool) -> assert_cmd::assert::Assert {
    let mut cmd = quire();
    cmd.arg("validate").arg("--okf");
    if strict {
        cmd.arg("--strict");
    }
    cmd.arg(dir.path())
        .arg("--module")
        .arg(okf_edge_module())
        .assert()
}

const ORIGIN: &str = "https://github.com/acme/widgets.git";

// FR-014-AC-11: under `--okf --strict` a dangling `mitigates` bare-id
// target exits 3 and stderr carries the source path, edge type and id. The
// bundle is otherwise conformant: with the target present it exits 0, so the
// dangling edge alone is what fails.
#[trace("FR-014-AC-11")]
#[test]
fn okf_strict_dangling_mitigates_exits_3_with_located_message() {
    let ok = strict_bundle(&[
        ("a", note_doc("A-1", "control", &["HZ-1"])),
        ("hz", note_doc("HZ-1", "hazard", &[])),
    ]);
    okf(&ok, true).success();

    let dir = strict_bundle(&[("a", note_doc("A-1", "control", &["HZ-9"]))]);
    okf(&dir, true).failure().code(3).stderr(predicate::str::contains(
        "a.md: dangling reference 'HZ-9' (edge 'mitigates') has no target in the bundle [dangling-reference]",
    ));
    okf(&dir, false).success();
}

// FR-014-AC-11: a miss under the bundle's own `ix://` authority exits 3.
#[trace("FR-014-AC-11")]
#[test]
fn okf_strict_own_authority_miss_exits_3() {
    let dir = strict_bundle(&[("a", note_doc("A-1", "control", &["ix://acme/widgets/HZ-9"]))]);
    set_origin(&dir, ORIGIN);
    okf(&dir, true)
        .failure()
        .code(3)
        .stderr(predicate::str::contains(
            "dangling reference 'HZ-9' (edge 'mitigates')",
        ));
}

// FR-014-AC-11: a foreign-authority target is skipped, so an
// otherwise conformant bundle exits 0.
#[trace("FR-014-AC-11")]
#[test]
fn okf_strict_foreign_authority_target_exits_0() {
    let dir = strict_bundle(&[("a", note_doc("A-1", "control", &["ix://other/repo/HZ-9"]))]);
    set_origin(&dir, ORIGIN);
    okf(&dir, true).success();
}

// FR-014-AC-11: with no package declared every miss is dangling.
#[trace("FR-014-AC-11")]
#[test]
fn okf_strict_miss_with_no_package_declared_exits_3() {
    let dir = strict_bundle(&[("a", note_doc("A-1", "control", &["ix://other/repo/HZ-9"]))]);
    okf(&dir, true)
        .failure()
        .code(3)
        .stderr(predicate::str::contains("dangling reference 'HZ-9'"));
}

// FR-014-AC-11: a two-segment `ix://<org>/<id>` link to an absent id
// is dangling even when a package is declared.
#[trace("FR-014-AC-11")]
#[test]
fn okf_strict_two_segment_link_to_absent_id_exits_3() {
    let dir = strict_bundle(&[("a", note_doc("A-1", "control", &["ix://acme/HZ-9"]))]);
    set_origin(&dir, ORIGIN);
    okf(&dir, true)
        .failure()
        .code(3)
        .stderr(predicate::str::contains("dangling reference 'HZ-9'"));
}

// FR-014-AC-11: a bundle with no findings exits 0 under `--strict`.
#[trace("FR-014-AC-11")]
#[test]
fn okf_strict_finding_free_bundle_exits_0() {
    let dir = strict_bundle(&[
        ("a", note_doc("A-1", "control", &["HZ-1"])),
        ("hz", note_doc("HZ-1", "hazard", &[])),
    ]);
    okf(&dir, true).success().stderr(predicate::str::is_empty());
}

// FR-014-AC-11: a finding that stays a warning under Strict (the
// FR-040-AC-10 disallowed-edge-target check) exits 3 under `--strict` and is
// a plain warning, exit 0, without it.
#[trace("FR-014-AC-11")]
#[test]
fn okf_strict_fails_on_warning_only_finding() {
    let dir = strict_bundle(&[
        ("a", note_doc("A-1", "control", &["AS-1"])),
        ("as", note_doc("AS-1", "asset", &[])),
    ]);
    let pred = predicate::str::contains("[disallowed-edge-target]");
    okf(&dir, false).success().stderr(pred.clone());
    okf(&dir, true).failure().code(3).stderr(pred);
}
