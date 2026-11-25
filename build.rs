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
    embed_cli_anomaly_documents(&manifest_dir);
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

/// Embed authored Quire CLI anomaly records under stable, source-qualified paths.
fn embed_cli_anomaly_documents(manifest_dir: &Path) {
    let anomaly_dir = manifest_dir.join("spec/anomalies");
    println!("cargo:rerun-if-changed={}", anomaly_dir.display());
    let mut paths = std::fs::read_dir(&anomaly_dir)
        .unwrap_or_else(|error| panic!("reading {}: {error}", anomaly_dir.display()))
        .map(|entry| entry.expect("reading anomaly directory entry").path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("AN-") && name.ends_with(".md"))
        })
        .collect::<Vec<_>>();
    paths.sort();

    let mut generated = String::from("const CLI_ANOMALY_DOCUMENTS: &[(&str, &str)] = &[\n");
    for path in paths {
        println!("cargo:rerun-if-changed={}", path.display());
        let relative = path
            .strip_prefix(manifest_dir)
            .expect("anomaly source is within the manifest directory")
            .to_string_lossy()
            .replace('\\', "/");
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()));
        generated.push_str(&format!("    ({relative:?}, {content:?}),\n"));
    }
    generated.push_str("];\n");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    std::fs::write(out_dir.join("anomaly_documents.rs"), generated)
        .expect("writing embedded anomaly documents");
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
