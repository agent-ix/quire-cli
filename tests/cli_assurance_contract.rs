//! IT-145: public documentation and the exact engine pin agree on FR-020.

mod common;

use std::fs;

use common::quire;

// Trace: IT-145, FR-020-AC-9
#[test]
fn it_145_help_docs_capability_and_dependency_pin_agree() {
    let help = quire().arg("--help").output().expect("help");
    assert!(help.status.success());
    let help = String::from_utf8(help.stdout).expect("UTF-8 help");
    assert!(help.contains("assurance   Emit source-grounded assurance facts"));

    let root = env!("CARGO_MANIFEST_DIR");
    let read = |relative: &str| fs::read_to_string(format!("{root}/{relative}")).expect(relative);
    let manifest = read("Cargo.toml");
    let lock = read("Cargo.lock");
    let readme = read("README.md");
    let changelog = read("CHANGELOG.md");

    // The engine identity lives in Cargo.toml alone; the lock must resolve the
    // revision the manifest pins. No prose restates the SHA.
    let pinned = manifest
        .lines()
        .find(|line| line.starts_with("quire-rs = "))
        .and_then(|line| line.split("rev = \"").nth(1))
        .and_then(|rest| rest.split('"').next())
        .expect("Cargo.toml pins quire-rs by rev");
    let resolved = quire_cli::lockfile::engine_source_revision(&lock).expect("engine revision");
    assert_eq!(resolved, pinned);
    let version = quire_cli::lockfile::engine_manifest_version(&lock).expect("engine version");
    assert!(manifest.contains(&format!("version = \"={version}\"")), "{version}");

    assert!(readme.contains("quire assurance"));
    assert!(readme.contains("--expect-schema <MODULE/ARCHETYPE@SHA256>"));
    assert!(changelog.contains("assurance_export.v1"));
    assert!(quire_cli::engine::CAPABILITIES.contains(&"assurance_export.v1"));
}
