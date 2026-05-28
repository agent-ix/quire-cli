//! Shared helpers for process-boundary integration tests.

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;

use assert_cmd::prelude::*;

pub fn quire() -> Command {
    Command::cargo_bin("quire").expect("quire binary built")
}

pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn iso_module() -> PathBuf {
    fixture_root().join("iso")
}

pub fn extract_module() -> PathBuf {
    fixture_root().join("extract-mod")
}

pub fn extract_sample_doc() -> PathBuf {
    fixture_root().join("extract-mod/sample.md")
}

pub fn ctx_path(name: &str) -> PathBuf {
    fixture_root().join(format!("contexts/{name}.json"))
}

pub const ISO_ARCHETYPES: &[&str] = &["FR", "NFR", "StR", "US", "IT", "TC", "AC", "CON"];
