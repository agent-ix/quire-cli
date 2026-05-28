//! Happy + parametric validate ITs.
//! Covers IT-003, IT-014 (8-archetype valid + invalid), IT-021.

mod common;

use assert_cmd::prelude::*;
use predicates::prelude::*;

use common::{ctx_path, iso_module, quire, ISO_ARCHETYPES};

#[test]
fn it_003_validate_returns_0_on_valid_data() {
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .assert()
        .success();
}

#[test]
fn it_003_validate_returns_1_on_invalid_data() {
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR-invalid"))
        .assert()
        .failure()
        .code(1);
}

#[test]
fn it_014_validate_parametric_sweep_each_archetype() {
    for archetype in ISO_ARCHETYPES {
        quire()
            .arg("validate")
            .arg(archetype)
            .arg("--module")
            .arg(iso_module())
            .arg("--data")
            .arg(ctx_path(archetype))
            .assert()
            .success();
    }
}

#[test]
fn it_021_validate_writes_nothing_to_stdout_on_success() {
    quire()
        .arg("validate")
        .arg("FR")
        .arg("--module")
        .arg(iso_module())
        .arg("--data")
        .arg(ctx_path("FR"))
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
