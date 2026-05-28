//! Happy-path render ITs.
//! Covers IT-001, IT-017, IT-018 (8-archetype sweep), IT-009 (deterministic re-run).

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ctx_path, iso_module, quire, ISO_ARCHETYPES};

#[test]
fn it_001_render_fr_happy_path() {
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .assert()
        .success()
        .stdout(predicate::str::contains("FR-001"))
        .stdout(predicate::str::contains("# [FR-001] FR fixture"));
}

#[test]
fn it_017_out_flag_writes_file_and_empty_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("rendered.md");
    quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .arg("--out")
        .arg(&out)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
    let written = std::fs::read_to_string(&out).unwrap();
    assert!(written.contains("FR-001"));
}

#[test]
fn it_018_render_parity_sweep_all_8_iso_archetypes() {
    for archetype in ISO_ARCHETYPES {
        // StR's id pattern allows 2-4 uppercase letters, so its fixture
        // ships id="ST-001" rather than "StR-001". Every other archetype
        // uses the canonical `{ARCHETYPE}-001` id.
        let expected_id_marker = if *archetype == "StR" {
            "ST-001"
        } else {
            // Use a leak-free static lookup: just verify "{archetype}-"
            // prefix is present in stdout; the rest is the rendered body.
            // This keeps the per-iteration check simple.
            ""
        };
        let assertion = quire()
            .arg("render")
            .arg(archetype)
            .arg("--module")
            .arg(iso_module())
            .arg("--data")
            .arg(ctx_path(archetype))
            .assert()
            .success();
        if !expected_id_marker.is_empty() {
            assertion.stdout(predicate::str::contains(expected_id_marker));
        } else {
            assertion.stdout(predicate::str::contains(format!("{archetype}-001")));
        }
    }
}

#[test]
fn it_009_render_is_deterministic_across_reruns() {
    let first = quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .output()
        .unwrap();
    let second = quire()
        .arg("render")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .output()
        .unwrap();
    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout, "render not deterministic");
}
