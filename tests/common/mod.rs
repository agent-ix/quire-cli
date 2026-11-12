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

pub fn extract_module() -> PathBuf {
    fixture_root().join("extract-mod")
}

pub fn extract_sample_doc() -> PathBuf {
    fixture_root().join("extract-mod/sample.md")
}

/// Module exercising markdown `validate` (FR-004/FR-010): an `FR`
/// type with a `body_extraction` DSL carrying asserts.
pub fn validate_module() -> PathBuf {
    fixture_root().join("validate-mod")
}

/// A document fixture under `validate-mod/docs/`.
pub fn validate_doc(name: &str) -> PathBuf {
    fixture_root().join(format!("validate-mod/docs/{name}"))
}

/// Builds, under `dir`, a copy of the `validate-mod` module whose grammar is
/// named `iso-spec-core`, the name that enables the EARS grammar checks.
/// Returns the module root.
pub fn ears_module(dir: &std::path::Path) -> PathBuf {
    let src = validate_module();
    let root = dir.join("ears-module");
    std::fs::create_dir_all(root.join("schemas")).expect("mkdir schemas");
    let manifest = std::fs::read_to_string(src.join("manifest.yaml")).expect("read manifest");
    std::fs::write(
        root.join("manifest.yaml"),
        manifest.replace("validate-mod-core", "iso-spec-core"),
    )
    .expect("write manifest");
    std::fs::copy(
        src.join("schemas/fr-frontmatter.schema.json"),
        root.join("schemas/fr-frontmatter.schema.json"),
    )
    .expect("copy schema");
    root
}
