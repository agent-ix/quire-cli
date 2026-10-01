//! Report the `quire-rs` version this binary links.
//!
//! Read from the `quire-rs` stanza of `Cargo.lock`: the version of the tool
//! that produced an output is the provenance, and the lockfile records what
//! Cargo actually linked.

use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let version = find_lockfile(&manifest_dir)
        .and_then(|lock| {
            println!("cargo:rerun-if-changed={}", lock.display());
            std::fs::read_to_string(lock).ok()
        })
        .and_then(|text| engine_version(&text))
        // `unknown` rather than a fallback to the crate version: an
        // unresolvable engine must be visible, never replaced by a plausible
        // number.
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=QUIRE_ENGINE_VERSION={version}");
}

/// The `version` of the last `[[package]]` named `quire-rs`.
fn engine_version(lock: &str) -> Option<String> {
    let mut in_engine = false;
    let mut found = None;
    for line in lock.lines().map(str::trim) {
        if line.starts_with('[') {
            in_engine = false;
        } else if let Some(name) = line.strip_prefix("name = ") {
            in_engine = name.trim_matches('"') == "quire-rs";
        } else if let (true, Some(version)) = (in_engine, line.strip_prefix("version = ")) {
            found = Some(version.trim_matches('"').to_string());
        }
    }
    found
}

/// The nearest `Cargo.lock` at or above the manifest directory.
fn find_lockfile(from: &Path) -> Option<PathBuf> {
    from.ancestors()
        .map(|dir| dir.join("Cargo.lock"))
        .find(|candidate| candidate.is_file())
}
