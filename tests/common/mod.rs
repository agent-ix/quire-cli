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

/// Module exercising markdown `validate` (FR-004/FR-010): an `FR`
/// artifact_type with a `body_extraction` DSL carrying asserts.
pub fn validate_module() -> PathBuf {
    fixture_root().join("validate-mod")
}

/// A document fixture under `validate-mod/docs/`.
pub fn validate_doc(name: &str) -> PathBuf {
    fixture_root().join(format!("validate-mod/docs/{name}"))
}

/// A direct-markdown document fixture under `iso-docs/` (used by the
/// render-free validate sweep, IT-014, and FR-004 failure-path ITs).
pub fn iso_doc(name: &str) -> PathBuf {
    fixture_root().join(format!("iso-docs/{name}"))
}

pub const ISO_ARCHETYPES: &[&str] = &["FR", "NFR", "StR", "US", "IT", "TC", "AC", "CON"];
